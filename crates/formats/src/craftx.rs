//! Craft Exchange (`.craftx`): the JSON hand-off between WordCraft AR, DeckCraft AR and
//! GridCraft AR (`docs/ar-exchange.md`). One file carries a document's content *and its Arabic
//! typography* — paragraph direction, alignment, kashida level, lists, tables — without going
//! through an Office format.
//!
//! [`write`] walks the body in order: headings, paragraphs (runs), lists (consecutive items
//! grouped), tables (rows × cells, `rtl` from the table's `bidiVisual`), page breaks and PNG
//! pictures. [`read`] builds a document from the blocks; everything unknown is ignored, every
//! number is clamped, and a `slide` block (from DeckCraft) becomes a Heading 1 with the slide's
//! body after it and its notes as an italic paragraph.
//!
//! The mapping that keeps kashida and order (spec table): `bidi` ↔ `w:bidi`; `align: justify` +
//! `kashida: low|medium|high` ↔ `w:jc lowKashida|mediumKashida|highKashida` ([`Align::LowKashida`]…);
//! `align: distribute` ↔ `w:jc distribute`; `start`/`end` are the logical [`Align::Left`] /
//! [`Align::Right`] (right/left on the page in a bidi paragraph).

use std::sync::Arc;

use serde_json::{Map, Value, json};
use wordcraft_doc::props::{Align, LineSpacing};
use wordcraft_doc::{Block, Blocks, Document, Paragraph, Table};

use crate::model::{self, Cell, FBlock, FTable, Fmt, Inline, Kind, ListInfo, Para, Spacing};

/// What went wrong reading or writing a `.craftx` file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatError(pub String);

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for FormatError {}

impl From<serde_json::Error> for FormatError {
    fn from(e: serde_json::Error) -> Self {
        FormatError(format!("craftx: {e}"))
    }
}

/// The format version this module writes and reads.
pub const VERSION: u64 = 1;
/// The `from` field this app writes.
pub const FROM: &str = "wordcraft-ar";
/// Most blocks a file may hold; the rest are ignored (hostile input).
const MAX_BLOCKS: usize = 200_000;
/// Most list items in one `list` block.
const MAX_ITEMS: usize = 50_000;
/// Nesting of tables inside cells the writer flattens into text.
const MAX_DEPTH: usize = model::MAX_DEPTH;
/// Largest paragraph spacing or indent (points).
const MAX_PT: f32 = 1584.0;

// ---------------------------------------------------------------------------------------------
// Document → craftx

/// Serialise `doc`'s body as a `.craftx` file (pretty JSON) titled `title`.
pub fn write(doc: &Document, title: &str) -> Result<String, FormatError> {
    let rtl = document_rtl(doc);
    let mut w = Writer { doc, blocks: Vec::new(), list: Vec::new(), list_ordered: false };
    for b in &doc.body {
        w.block(b);
    }
    w.flush_list();
    // The empty paragraph a document must end with after a table (Word's rule) is not content.
    if w.blocks.len() >= 2
        && w.blocks.last().is_some_and(|b| b.get("kind") == Some(&json!("paragraph")) && b.get("text") == Some(&json!("")))
        && w.blocks.get(w.blocks.len() - 2).is_some_and(|b| b.get("kind") == Some(&json!("table")))
    {
        w.blocks.pop();
    }
    let mut top = Map::new();
    top.insert("craftx".into(), json!(VERSION));
    top.insert("from".into(), json!(FROM));
    top.insert("title".into(), json!(title));
    top.insert("rtl".into(), json!(rtl));
    if let Some(font) = body_font(doc, rtl) {
        top.insert("font".into(), json!(font));
    }
    top.insert("blocks".into(), Value::Array(w.blocks));
    Ok(serde_json::to_string_pretty(&Value::Object(top))?)
}

/// The document's main direction: what most body paragraphs resolve to, the Normal style's
/// direction when they tie (or there are none).
fn document_rtl(doc: &Document) -> bool {
    let (mut rtl, mut ltr) = (0usize, 0usize);
    for b in &doc.body {
        if let Block::Para(p) = &**b {
            if doc.styles.resolve_para(&p.props).bidi {
                rtl += 1;
            } else {
                ltr += 1;
            }
        }
    }
    match rtl.cmp(&ltr) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => doc.styles.resolve_para(&Default::default()).bidi,
    }
}

/// The preferred body font: the complex-script default for a right-to-left document, else the
/// Normal style's font.
fn body_font(doc: &Document, rtl: bool) -> Option<String> {
    let chr = &doc.styles.default_chr;
    let cs = chr.font_cs.clone().filter(|f| !f.is_empty());
    let normal = doc.styles.resolve_char(None, &Default::default()).font;
    let normal = Some(normal).filter(|f| !f.is_empty());
    if rtl { cs.or(normal) } else { normal.or(cs) }
}

struct Writer<'a> {
    doc: &'a Document,
    blocks: Vec<Value>,
    /// Consecutive list items waiting to be written as one `list` block.
    list: Vec<Value>,
    list_ordered: bool,
}

impl Writer<'_> {
    fn push(&mut self, v: Value) {
        if self.blocks.len() < MAX_BLOCKS {
            self.blocks.push(v);
        }
    }

    fn flush_list(&mut self) {
        if self.list.is_empty() {
            return;
        }
        let items = std::mem::take(&mut self.list);
        let ordered = self.list_ordered;
        self.push(json!({"kind": "list", "ordered": ordered, "items": items}));
    }

    fn block(&mut self, b: &Block) {
        match b {
            Block::Para(p) => self.para(p),
            Block::Table(t) => {
                self.flush_list();
                let v = self.table(t);
                self.push(v);
            }
        }
    }

    fn para(&mut self, p: &Paragraph) {
        let doc = self.doc;
        let rp = doc.styles.resolve_para(&p.props);
        let (kind, list) = model::para_kind(doc, p);
        let para = para_json(&rp);
        for (i, fp) in model::flow_paras(doc, p).into_iter().enumerate() {
            // A page break before the paragraph, or one inside its text, is its own block.
            if fp.page_break && (i > 0 || rp.page_break_before) {
                self.flush_list();
                self.push(json!({"kind": "page_break"}));
            }
            let (runs, text, images) = inlines_json(&fp.inlines);
            let has_text = !text.is_empty() || images.is_empty();
            // The Title style has no block of its own in the spec: it is the first heading.
            let heading = match kind {
                Kind::Heading(n) => Some(n.clamp(1, 6)),
                Kind::Title => Some(1),
                _ => None,
            };
            match (heading, list) {
                (Some(n), _) if has_text => {
                    self.flush_list();
                    self.push(json!({"kind": "heading", "level": n, "runs": runs, "text": text, "para": para}));
                }
                (_, Some(li)) if has_text => {
                    if !self.list.is_empty() && self.list_ordered != li.ordered {
                        self.flush_list();
                    }
                    self.list_ordered = li.ordered;
                    if self.list.len() < MAX_ITEMS {
                        self.list.push(json!({"runs": runs, "text": text, "para": para, "level": li.level.min(8)}));
                    }
                }
                _ if has_text => {
                    self.flush_list();
                    self.push(json!({"kind": "paragraph", "runs": runs, "text": text, "para": para}));
                }
                _ => {}
            }
            for img in images {
                self.flush_list();
                self.push(img);
            }
        }
    }

    fn table(&self, t: &Table) -> Value {
        let doc = self.doc;
        let mut rows = Vec::with_capacity(t.rows.len());
        let mut first_bold = true;
        let mut first_shaded = true;
        let mut first_text = false;
        for (ri, r) in t.rows.iter().enumerate() {
            let mut cells = Vec::with_capacity(r.cells.len());
            for c in &r.cells {
                let mut runs = Vec::new();
                let mut first_para: Option<Paragraph> = None;
                cell_runs(doc, &c.blocks, 0, &mut runs, &mut first_para);
                let (runs, text, _) = inlines_json(&runs);
                let rp = doc.styles.resolve_para(&first_para.map(|p| p.props).unwrap_or_default());
                if ri == 0 {
                    first_shaded &= c.props.shading.is_some();
                    if !text.trim().is_empty() {
                        first_text = true;
                        first_bold &= runs.iter().all(|r| r.get("bold") == Some(&Value::Bool(true)));
                    }
                }
                let (align, _) = align_json(rp.align);
                cells.push(json!({"runs": runs, "text": text, "para": para_json(&rp), "align": align}));
            }
            rows.push(Value::Array(cells));
        }
        let header = t.rows.first().is_some_and(|r| r.props.header)
            || (t.props.look.header_row && t.rows.len() > 1)
            || (!t.rows.is_empty() && first_shaded)
            || (first_text && first_bold);
        let sum: f32 = t.grid.iter().filter(|w| w.is_finite() && **w > 0.0).sum();
        let widths: Vec<Value> = if sum > 0.0 && !t.grid.is_empty() {
            t.grid.iter().map(|w| json!(((if w.is_finite() && *w > 0.0 { *w } else { 0.0 }) / sum * 10_000.0).round() / 10_000.0)).collect()
        } else {
            Vec::new()
        };
        json!({"kind": "table", "rtl": t.props.bidi_visual, "header": header, "widths": widths, "rows": rows})
    }
}

/// The text of a cell's blocks as one list of inlines, paragraphs separated by a newline; a
/// table inside the cell contributes its cells' text. `first` receives the first paragraph (for
/// the cell's paragraph properties).
fn cell_runs(doc: &Document, blocks: &Blocks, depth: usize, out: &mut Vec<Inline>, first: &mut Option<Paragraph>) {
    for b in blocks {
        match &**b {
            Block::Para(p) => {
                if first.is_none() {
                    *first = Some(p.clone());
                }
                for fp in model::flow_paras(doc, p) {
                    if !out.is_empty() {
                        out.push(Inline::Text("\n".into(), Fmt::default()));
                    }
                    out.extend(fp.inlines.into_iter().filter(|i| matches!(i, Inline::Text(..) | Inline::Equation { .. })));
                }
            }
            Block::Table(t) if depth < MAX_DEPTH => {
                for r in &t.rows {
                    for c in &r.cells {
                        cell_runs(doc, &c.blocks, depth + 1, out, first);
                    }
                }
            }
            Block::Table(_) => {}
        }
    }
}

/// Runs, plain text and picture blocks of a flow paragraph's inlines.
fn inlines_json(inlines: &[Inline]) -> (Vec<Value>, String, Vec<Value>) {
    let mut runs = Vec::new();
    let mut text = String::new();
    let mut images = Vec::new();
    for i in inlines {
        match i {
            Inline::Text(t, f) => {
                if t.is_empty() {
                    continue;
                }
                text.push_str(t);
                runs.push(run_json(t, f));
            }
            Inline::Equation { linear, .. } => {
                if !linear.is_empty() {
                    text.push_str(linear);
                    runs.push(run_json(linear, &Fmt::default()));
                }
            }
            // Only PNG pictures travel; other formats are left out (the spec has `png_base64`).
            Inline::Image(img) if img.ext == "png" => {
                let name = if img.alt.trim().is_empty() { "picture.png".to_string() } else { img.alt.clone() };
                images.push(json!({
                    "kind": "image", "name": name, "png_base64": model::base64_encode(&img.data),
                    "width_pt": round1(img.w), "height_pt": round1(img.h),
                }));
            }
            Inline::Image(_) | Inline::Anchor(_) | Inline::Figure(_) => {}
        }
    }
    (runs, text, images)
}

fn run_json(text: &str, f: &Fmt) -> Value {
    let mut m = Map::new();
    m.insert("text".into(), json!(text));
    if f.bold {
        m.insert("bold".into(), json!(true));
    }
    if f.italic {
        m.insert("italic".into(), json!(true));
    }
    if f.underline {
        m.insert("underline".into(), json!(true));
    }
    if let Some(size) = f.size.filter(|s| s.is_finite() && *s > 0.0) {
        m.insert("size_pt".into(), json!(round1(size)));
    }
    if let Some(c) = f.color {
        m.insert("color".into(), json!(format!("#{}", c.hex())));
    }
    if let Some(font) = f.font.as_deref().filter(|s| !s.is_empty()) {
        m.insert("font".into(), json!(font));
    }
    Value::Object(m)
}

/// The spec's `align` and `kashida` words for an alignment.
fn align_json(a: Align) -> (&'static str, &'static str) {
    match a {
        Align::Left => ("start", "none"),
        Align::Center => ("center", "none"),
        Align::Right => ("end", "none"),
        Align::Justify => ("justify", "none"),
        Align::Distribute => ("distribute", "none"),
        Align::LowKashida => ("justify", "low"),
        Align::MediumKashida => ("justify", "medium"),
        Align::HighKashida => ("justify", "high"),
    }
}

/// An alignment from the spec's words; `None` for words it doesn't know.
fn align_from(align: Option<&str>, kashida: Option<&str>) -> Option<Align> {
    Some(match (align?, kashida.unwrap_or("none")) {
        ("start" | "left", _) => Align::Left,
        ("center" | "centre", _) => Align::Center,
        ("end" | "right", _) => Align::Right,
        ("distribute" | "distributed", _) => Align::Distribute,
        ("justify" | "justified" | "both", "low") => Align::LowKashida,
        ("justify" | "justified" | "both", "medium") => Align::MediumKashida,
        ("justify" | "justified" | "both", "high") => Align::HighKashida,
        ("justify" | "justified" | "both", _) => Align::Justify,
        _ => return None,
    })
}

fn para_json(rp: &wordcraft_doc::resolve::ResolvedPara) -> Value {
    let (align, kashida) = align_json(rp.align);
    let mut m = Map::new();
    m.insert("bidi".into(), json!(rp.bidi));
    m.insert("align".into(), json!(align));
    m.insert("kashida".into(), json!(kashida));
    m.insert("space_before_pt".into(), json!(round1(rp.space_before.clamp(0.0, MAX_PT))));
    m.insert("space_after_pt".into(), json!(round1(rp.space_after.clamp(0.0, MAX_PT))));
    if let LineSpacing::Multiple(x) = rp.line_spacing
        && x.is_finite()
        && x > 0.0
    {
        m.insert("line_spacing".into(), json!(round2(x)));
    }
    // The start indent (measured from the right in a bidi paragraph, like Word's `w:start`).
    m.insert("indent_pt".into(), json!(round1(rp.indent_left.clamp(-MAX_PT, MAX_PT))));
    Value::Object(m)
}

fn round1(x: f32) -> f32 {
    if x.is_finite() { (x * 10.0).round() / 10.0 } else { 0.0 }
}

fn round2(x: f32) -> f32 {
    if x.is_finite() { (x * 100.0).round() / 100.0 } else { 0.0 }
}

// ---------------------------------------------------------------------------------------------
// craftx → Document

/// Build a document from a `.craftx` file. Unknown fields and block kinds are ignored; hostile
/// numbers are clamped; only JSON that doesn't parse, or isn't an object, is an error.
pub fn read(json: &str) -> Result<Document, FormatError> {
    let v: Value = serde_json::from_str(json)?;
    let Some(top) = v.as_object() else { return Err(FormatError("craftx: the file is not a JSON object".into())) };
    let rtl = top.get("rtl").and_then(Value::as_bool).unwrap_or(false);
    let mut doc = Document::new();
    doc.numbering = Default::default();
    if rtl {
        doc.styles.default_para.bidi = Some(true);
    }
    if let Some(font) = top.get("font").and_then(Value::as_str).map(str::trim).filter(|f| !f.is_empty() && f.chars().count() <= 64) {
        let chr = &mut doc.styles.default_chr;
        chr.font = Some(font.to_string());
        chr.font_cs = Some(font.to_string());
    }
    if let Some(title) = top.get("title").and_then(Value::as_str) {
        doc.core.title = title.chars().take(256).collect();
    }
    let blocks = top.get("blocks").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
    let mut r = Reader { doc: &mut doc, rtl, out: Blocks::new(), pending_break: false, notes: None, list: Vec::new() };
    for b in blocks.iter().take(MAX_BLOCKS) {
        r.block(b);
    }
    r.finish();
    doc.body = r.out;
    doc.ensure_nonempty();
    Ok(doc)
}

struct Reader<'a> {
    doc: &'a mut Document,
    rtl: bool,
    out: Blocks,
    /// A `page_break` block waits for the next paragraph to carry it.
    pending_break: bool,
    /// The current slide's notes, written as an italic paragraph when its body ends.
    notes: Option<String>,
    /// Consecutive list items (one `list` block, or several in a row) become one list.
    list: Vec<(Para, bool)>,
}

impl Reader<'_> {
    fn block(&mut self, b: &Value) {
        let Some(kind) = b.get("kind").and_then(Value::as_str) else { return };
        match kind {
            "slide" => {
                self.end_slide();
                let title = b.get("title").and_then(Value::as_str).unwrap_or("");
                let mut p = Para::new(Kind::Heading(1));
                p.push_text(&model::clean_text(title), &Fmt::default());
                let bidi = self.para_bidi(b.get("para"));
                self.emit_para(p, bidi);
                self.notes = b.get("notes").and_then(Value::as_str).map(str::trim).filter(|n| !n.is_empty()).map(str::to_string);
            }
            "heading" => {
                let level = b.get("level").and_then(Value::as_f64).unwrap_or(1.0);
                let level = if level.is_finite() { level.clamp(1.0, 6.0) as u8 } else { 1 };
                let (p, bidi) = self.text_para(b, Kind::Heading(level));
                self.emit_para(p, bidi);
            }
            "paragraph" => {
                let (p, bidi) = self.text_para(b, Kind::Normal);
                self.emit_para(p, bidi);
            }
            "list" => {
                let ordered = b.get("ordered").and_then(Value::as_bool).unwrap_or(false);
                let items = b.get("items").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
                for item in items.iter().take(MAX_ITEMS) {
                    let (mut p, bidi) = self.text_para(item, Kind::Normal);
                    let level = item.get("level").and_then(Value::as_f64).unwrap_or(0.0);
                    let level = if level.is_finite() { level.clamp(0.0, 8.0) as u8 } else { 0 };
                    p.list = Some(ListInfo { ordered, level });
                    p.page_break = std::mem::take(&mut self.pending_break);
                    self.list.push((p, bidi));
                }
            }
            "table" => {
                self.flush_list();
                self.table(b);
            }
            "image" => {
                let data = b.get("png_base64").and_then(Value::as_str).and_then(model::base64_decode);
                let name = b.get("name").and_then(Value::as_str).unwrap_or("");
                let img = data.and_then(|d| model::make_img(d, pt(b, "width_pt"), pt(b, "height_pt"), name));
                if let Some(img) = img {
                    let mut p = Para::new(Kind::Normal);
                    p.inlines.push(Inline::Image(img));
                    p.align = Some(Align::Center);
                    let bidi = self.rtl;
                    self.emit_para(p, bidi);
                }
            }
            "page_break" => self.pending_break = true,
            _ => {}
        }
    }

    /// A text block's paragraph (runs, or `text`) with its properties, and its direction.
    fn text_para(&self, b: &Value, kind: Kind) -> (Para, bool) {
        let mut p = Para::new(kind);
        match b.get("runs").and_then(Value::as_array) {
            Some(runs) if !runs.is_empty() => {
                for r in runs {
                    let Some(text) = r.get("text").and_then(Value::as_str) else { continue };
                    p.push_text(&model::clean_text(text), &fmt_from(r));
                }
            }
            _ => {
                if let Some(text) = b.get("text").and_then(Value::as_str) {
                    p.push_text(&model::clean_text(text), &Fmt::default());
                }
            }
        }
        let pp = b.get("para");
        p.align = align_from(pp.and_then(|p| p.get("align")).and_then(Value::as_str), pp.and_then(|p| p.get("kashida")).and_then(Value::as_str))
            // A table cell's own `align`.
            .or_else(|| align_from(b.get("align").and_then(Value::as_str), None));
        if let Some(pp) = pp {
            p.spacing = Spacing {
                before: pt(pp, "space_before_pt").map(|v| v.clamp(0.0, MAX_PT)),
                after: pt(pp, "space_after_pt").map(|v| v.clamp(0.0, MAX_PT)),
                line: pp
                    .get("line_spacing")
                    .and_then(Value::as_f64)
                    .map(|x| x as f32)
                    .filter(|x| x.is_finite() && *x > 0.0)
                    .map(|x| LineSpacing::Multiple(x.clamp(0.06, 132.0))),
                left: pt(pp, "indent_pt").map(|v| v.clamp(-MAX_PT, MAX_PT)),
                right: None,
                first: None,
            };
        }
        (p, self.para_bidi(pp))
    }

    fn para_bidi(&self, pp: Option<&Value>) -> bool {
        pp.and_then(|p| p.get("bidi")).and_then(Value::as_bool).unwrap_or(self.rtl)
    }

    /// Write one paragraph into the body (after any list waiting), carrying a pending page break.
    fn emit_para(&mut self, mut p: Para, bidi: bool) {
        self.flush_list();
        p.page_break = std::mem::take(&mut self.pending_break);
        let blocks = model::part_blocks(self.doc, &[FBlock::Para(p)]);
        self.push_with_bidi(blocks, &[bidi]);
    }

    /// Append `blocks`, setting each paragraph's direction from `bidi` in order (the last value
    /// serves any paragraph beyond the list). Only a direction that differs from the document's
    /// is written on the paragraph.
    fn push_with_bidi(&mut self, blocks: Blocks, bidi: &[bool]) {
        let rtl = self.rtl;
        for (i, mut b) in blocks.into_iter().enumerate() {
            let want = bidi.get(i).or(bidi.last()).copied().unwrap_or(rtl);
            if want != rtl
                && let Block::Para(p) = Arc::make_mut(&mut b)
            {
                p.props.bidi = Some(want);
                p.touch();
            }
            self.out.push(b);
        }
    }

    fn flush_list(&mut self) {
        if self.list.is_empty() {
            return;
        }
        let items = std::mem::take(&mut self.list);
        let bidi: Vec<bool> = items.iter().map(|(_, b)| *b).collect();
        let flow: Vec<FBlock> = items.into_iter().map(|(p, _)| FBlock::Para(p)).collect();
        let blocks = model::part_blocks(self.doc, &flow);
        self.push_with_bidi(blocks, &bidi);
    }

    fn table(&mut self, b: &Value) {
        let rtl = b.get("rtl").and_then(Value::as_bool).unwrap_or(self.rtl);
        let header = b.get("header").and_then(Value::as_bool).unwrap_or(false);
        let rows_in = b.get("rows").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
        let mut rows: Vec<Vec<Cell>> = Vec::new();
        // Cell directions in the order the builder makes paragraphs: row by row, cell by cell.
        let mut bidi = Vec::new();
        for (ri, r) in rows_in.iter().take(wordcraft_doc::table::MAX_ROWS).enumerate() {
            let Some(cells_in) = r.as_array() else { continue };
            let mut cells = Vec::new();
            for c in cells_in.iter().take(wordcraft_doc::table::MAX_COLS) {
                let (p, b) = self.text_para(c, Kind::Normal);
                // Inside a table the direction defaults to the table's.
                let b = if c.get("para").and_then(|p| p.get("bidi")).is_some() { b } else { rtl };
                bidi.push(b);
                cells.push(Cell { blocks: vec![FBlock::Para(p)], header: header && ri == 0, ..Default::default() });
            }
            if !cells.is_empty() {
                rows.push(cells);
            }
        }
        if rows.is_empty() {
            return;
        }
        let cols = rows.iter().map(Vec::len).max().unwrap_or(0);
        let width = self.doc.last_section.text_width().max(72.0);
        let widths: Vec<f32> = b
            .get("widths")
            .and_then(Value::as_array)
            .map(|w| w.iter().map(|x| x.as_f64().map(|x| x as f32).filter(|x| x.is_finite() && *x > 0.0).unwrap_or(0.0)).collect())
            .unwrap_or_default();
        let sum: f32 = widths.iter().sum();
        let widths = if widths.len() == cols && sum > 0.0 { widths.iter().map(|w| (w / sum * width).max(6.0)).collect() } else { Vec::new() };
        let flow = FBlock::Table(FTable { rows, widths, borderless: false });
        let mut blocks = model::part_blocks(self.doc, &[flow]);
        for blk in &mut blocks {
            if let Block::Table(t) = Arc::make_mut(blk) {
                t.props.bidi_visual = rtl;
                let mut k = 0usize;
                for row in &mut t.rows {
                    for cell in &mut row.cells {
                        let want = bidi.get(k).copied().unwrap_or(rtl);
                        k += 1;
                        if want == self.rtl {
                            continue;
                        }
                        for cb in &mut cell.blocks {
                            if let Block::Para(p) = Arc::make_mut(cb) {
                                p.props.bidi = Some(want);
                                p.touch();
                            }
                        }
                    }
                }
            }
        }
        self.pending_break = false;
        self.out.extend(blocks);
    }

    /// The notes of the slide whose body just ended, as an italic paragraph.
    fn end_slide(&mut self) {
        if let Some(notes) = self.notes.take() {
            let mut p = Para::new(Kind::Normal);
            p.push_text(&model::clean_text(&notes), &Fmt { italic: true, ..Default::default() });
            let bidi = self.rtl;
            self.emit_para(p, bidi);
        }
    }

    fn finish(&mut self) {
        self.flush_list();
        self.end_slide();
        if self.pending_break {
            let mut p = Para::new(Kind::Normal);
            p.page_break = true;
            self.pending_break = false;
            let blocks = model::part_blocks(self.doc, &[FBlock::Para(p)]);
            self.out.extend(blocks);
        }
    }
}

/// A finite, positive point value from `k`.
fn pt(v: &Value, k: &str) -> Option<f32> {
    v.get(k).and_then(Value::as_f64).map(|x| x as f32).filter(|x| x.is_finite())
}

fn fmt_from(r: &Value) -> Fmt {
    let flag = |k: &str| r.get(k).and_then(Value::as_bool).unwrap_or(false);
    Fmt {
        bold: flag("bold"),
        italic: flag("italic"),
        underline: flag("underline"),
        size: pt(r, "size_pt").filter(|s| *s >= 1.0 && *s <= 1638.0),
        color: r.get("color").and_then(Value::as_str).and_then(model::parse_color),
        font: r.get("font").and_then(Value::as_str).map(str::trim).filter(|f| !f.is_empty() && f.chars().count() <= 64).map(str::to_string),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wordcraft_doc::StoryRef;
    use wordcraft_doc::numbering::ListKind;
    use wordcraft_doc::props::{CharProps, NumRef};

    fn arabic_para(text: &str, align: Align) -> Paragraph {
        let mut p = Paragraph::with_text(text, CharProps::default());
        p.props.bidi = Some(true);
        p.props.align = Some(align);
        p
    }

    /// An Arabic service document: a heading, two justified bidi paragraphs with kashida (low,
    /// high), a bulleted list, a 2 × 3 right-to-left table.
    fn arabic_document() -> Document {
        let mut doc = Document::new();
        doc.styles.default_para.bidi = Some(true);
        let mut body = Blocks::new();
        let mut h = arabic_para("واجبات المستوصف", Align::Left);
        h.props.style = Some("Heading1".into());
        body.push(wordcraft_doc::para_block(h));
        body.push(wordcraft_doc::para_block(arabic_para("تقوم شعبة الأسنان بتقديم الخدمات الطبية لمنتسبي الوزارة.", Align::LowKashida)));
        body.push(wordcraft_doc::para_block(arabic_para("يشرف المدير على إدامة الأجهزة وتدريب الملاك.", Align::HighKashida)));
        let num = doc.numbering.add_list(ListKind::Bullet);
        for item in ["فحص المرضى", "إدامة الأجهزة", "تدريب الملاك"] {
            let mut p = arabic_para(item, Align::Left);
            p.props.numbering = Some(NumRef { num, level: 0 });
            body.push(wordcraft_doc::para_block(p));
        }
        let mut t = Table::new(2, 3, 400.0);
        t.props.bidi_visual = true;
        let texts = [["الاسم", "الرتبة", "الواجب"], ["أحمد", "نقيب", "المدير"]];
        for (ri, row) in t.rows.iter_mut().enumerate() {
            for (ci, cell) in row.cells.iter_mut().enumerate() {
                let mut p = Paragraph::with_text(texts[ri][ci], CharProps { bold: Some(ri == 0), ..Default::default() });
                p.props.bidi = Some(true);
                cell.blocks = vec![wordcraft_doc::para_block(p)];
            }
        }
        body.push(Arc::new(Block::Table(t)));
        doc.body = body;
        doc
    }

    #[test]
    fn arabic_document_round_trips_text_order_bidi_alignment_and_kashida() {
        let doc = arabic_document();
        let json = write(&doc, "واجبات").unwrap();
        let v: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["craftx"], 1);
        assert_eq!(v["from"], "wordcraft-ar");
        assert_eq!(v["title"], "واجبات");
        assert_eq!(v["rtl"], true);
        let blocks = v["blocks"].as_array().unwrap();
        let kinds: Vec<&str> = blocks.iter().map(|b| b["kind"].as_str().unwrap()).collect();
        assert_eq!(kinds, ["heading", "paragraph", "paragraph", "list", "table"]);
        assert_eq!(blocks[0]["level"], 1);
        assert_eq!(blocks[1]["para"]["align"], "justify");
        assert_eq!(blocks[1]["para"]["kashida"], "low");
        assert_eq!(blocks[2]["para"]["kashida"], "high");
        assert_eq!(blocks[1]["para"]["bidi"], true);
        assert_eq!(blocks[3]["ordered"], false);
        assert_eq!(blocks[3]["items"].as_array().unwrap().len(), 3);
        assert_eq!(blocks[4]["rtl"], true);
        assert_eq!(blocks[4]["header"], true, "a bold first row is the header");
        assert_eq!(blocks[4]["widths"].as_array().unwrap().len(), 3);

        let back = read(&json).unwrap();
        let paths = back.para_paths(StoryRef::Body);
        let paras: Vec<&Paragraph> = paths.iter().filter_map(|p| back.para(StoryRef::Body, p)).collect();
        // Heading, two paragraphs, three items, then the six cells, in order, and the empty
        // paragraph every document ends with after a table.
        assert_eq!(paras.len(), 13, "{paths:?}");
        let texts: Vec<&str> = paras.iter().map(|p| p.text.as_str()).collect();
        assert_eq!(texts[0], "واجبات المستوصف");
        assert_eq!(texts[1], "تقوم شعبة الأسنان بتقديم الخدمات الطبية لمنتسبي الوزارة.");
        assert_eq!(texts[2], "يشرف المدير على إدامة الأجهزة وتدريب الملاك.");
        assert_eq!(&texts[3..6], ["فحص المرضى", "إدامة الأجهزة", "تدريب الملاك"]);
        assert_eq!(&texts[6..12], ["الاسم", "الرتبة", "الواجب", "أحمد", "نقيب", "المدير"], "cell order");
        let rp = |i: usize| back.styles.resolve_para(&paras[i].props);
        assert_eq!(rp(0).outline_level, Some(0), "Heading 1");
        assert!(rp(1).bidi && rp(2).bidi && rp(3).bidi);
        assert_eq!(rp(1).align, Align::LowKashida);
        assert_eq!(rp(2).align, Align::HighKashida);
        assert!(rp(3).numbering.is_some() && rp(5).numbering.is_some(), "list items are list paragraphs");
        let table = back.body.iter().find_map(|b| match &**b {
            Block::Table(t) => Some(t.clone()),
            _ => None,
        });
        let table = table.expect("a table");
        assert!(table.props.bidi_visual);
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0].cells.len(), 3);
        assert!(table.rows[0].props.header);
        assert!(back.styles.resolve_para(&table.rows[1].cells[0].blocks[0].as_para().unwrap().props).bidi);
        // Writing again gives the same blocks.
        let again: Value = serde_json::from_str(&write(&back, "واجبات").unwrap()).unwrap();
        assert_eq!(again["blocks"], v["blocks"]);
    }

    #[test]
    fn hostile_json_never_panics() {
        let hostile = r#"{"craftx":1,"blocks":[{"kind":"table","rows":[[]]},{"kind":"heading","level":99},{"kind":"image","png_base64":"@@"}]}"#;
        let doc = read(hostile).unwrap();
        assert!(!doc.body.is_empty());
        let paras = doc.para_paths(StoryRef::Body);
        let p = doc.para(StoryRef::Body, &paras[0]).unwrap();
        assert_eq!(doc.styles.resolve_para(&p.props).outline_level, Some(5), "level 99 is clamped to Heading 6");
        for junk in [
            "",
            "[]",
            "null",
            "{",
            r#"{"craftx":"x","rtl":"yes","font":5,"blocks":5}"#,
            r##"{"blocks":[{"kind":"list","items":[{"level":-1e308,"text":"a"},{"level":1e308},5,null]},{"kind":"paragraph","runs":[{"text":"x","size_pt":1e30,"color":"#zz","font":""}],"para":{"align":"weird","kashida":"max","space_before_pt":-1e308,"line_spacing":1e308,"indent_pt":1e308}}]}"##,
            r#"{"blocks":[{"kind":"slide","title":"t","notes":"n"},{"kind":"page_break"},{"kind":"page_break"},{"kind":"slide"},{"kind":"table","rtl":true,"widths":[1,-1,"x"],"rows":[[{"text":"a"},{"text":"b"}],[{"text":"c"}],"x",[]]}]}"#,
            r#"{"blocks":[{"kind":"image","png_base64":"iVBORw0KGgo=","width_pt":-5,"height_pt":1e30},{"kind":"heading","level":"x","text":"h"}]}"#,
        ] {
            let r = read(junk);
            if let Ok(doc) = &r {
                assert!(!doc.body.is_empty());
                let _ = write(doc, "x").unwrap();
            }
        }
        assert!(read("").is_err() && read("[]").is_err() && read("{").is_err());
    }

    #[test]
    fn slides_become_headings_with_notes_after_the_body() {
        let json = r#"{"craftx":1,"from":"deckcraft-ar","rtl":true,"blocks":[
            {"kind":"slide","title":"المهمة","notes":"اقرأ ببطء"},
            {"kind":"paragraph","text":"نص الشريحة","para":{"align":"justify","kashida":"medium"}},
            {"kind":"slide","title":"الواجبات"},
            {"kind":"list","ordered":true,"items":[{"text":"أولاً","level":0},{"text":"ثانياً","level":0}]}
        ]}"#;
        let doc = read(json).unwrap();
        let paths = doc.para_paths(StoryRef::Body);
        let texts: Vec<String> = paths.iter().map(|p| doc.para(StoryRef::Body, p).unwrap().text.clone()).collect();
        assert_eq!(texts, ["المهمة", "نص الشريحة", "اقرأ ببطء", "الواجبات", "أولاً", "ثانياً"]);
        let p = |i: usize| doc.para(StoryRef::Body, &paths[i]).unwrap();
        assert_eq!(doc.styles.resolve_para(&p(0).props).outline_level, Some(0));
        assert_eq!(doc.styles.resolve_para(&p(1).props).align, Align::MediumKashida);
        assert_eq!(p(2).runs.first().and_then(|r| r.props.italic), Some(true), "notes are italic");
        assert!(doc.styles.resolve_para(&p(4).props).numbering.is_some());
        // Direction: everything right-to-left, from the document flag.
        assert!(paths.iter().all(|path| doc.styles.resolve_para(&doc.para(StoryRef::Body, path).unwrap().props).bidi));
        // An LTR paragraph in an RTL file keeps its own direction, and vice versa.
        let mixed = read(r#"{"craftx":1,"rtl":true,"blocks":[{"kind":"paragraph","text":"Left","para":{"bidi":false,"align":"end"}}]}"#).unwrap();
        let rp = mixed.styles.resolve_para(&mixed.para(StoryRef::Body, &mixed.para_paths(StoryRef::Body)[0]).unwrap().props);
        assert!(!rp.bidi);
        assert_eq!(rp.align, Align::Right);
    }

    #[test]
    fn pictures_and_page_breaks_travel() {
        // A 1×1 PNG.
        let png = model::base64_decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==").unwrap();
        let mut doc = Document::new();
        let key = doc.add_media(png, "png");
        let mut p = Paragraph::with_text("Before", CharProps::default());
        p.props.page_break_before = Some(true);
        let mut pic = Paragraph::new();
        pic.objects.push(wordcraft_doc::para::InlineObject::Image {
            media: key,
            w: 30.0,
            h: 20.0,
            alt: "dot.png".into(),
            float: Default::default(),
            crop: [0.0; 4],
            ole: None,
        });
        pic.text.push(wordcraft_doc::para::OBJ);
        pic.runs.push(wordcraft_doc::para::Run { len: wordcraft_doc::para::OBJ.len_utf8(), props: CharProps::default() });
        doc.body = vec![wordcraft_doc::para_block(p), wordcraft_doc::para_block(pic)];
        let json = write(&doc, "pics").unwrap();
        let v: Value = serde_json::from_str(&json).unwrap();
        let kinds: Vec<&str> = v["blocks"].as_array().unwrap().iter().map(|b| b["kind"].as_str().unwrap()).collect();
        assert_eq!(kinds, ["page_break", "paragraph", "image"]);
        assert_eq!(v["blocks"][2]["name"], "dot.png");
        assert_eq!(v["blocks"][2]["width_pt"], 30.0);
        let back = read(&json).unwrap();
        assert_eq!(back.media.len(), 1, "the picture came back");
        let first = back.para(StoryRef::Body, &back.para_paths(StoryRef::Body)[0]).unwrap();
        assert_eq!(first.props.page_break_before, Some(true));
    }
}
