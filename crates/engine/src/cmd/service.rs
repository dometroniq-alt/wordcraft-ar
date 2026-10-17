//! «كتابات الخدمة» — military service writing (WordCraft AR): hierarchical numbering by the
//! service cycle, the classification word in the header and footer, the «( page - last )» page
//! line in Arabic-Indic digits, kashida justification, and a hybrid hand-off to the external
//! ServiceWord-AR engine (Python) that measures the real fonts and returns the document finished.
//!
//! The numbering cycle (each level starts under its parent's dot):
//! `١.` → `أ.` (abjad: أ ب جـ د هـ و ز ح ط ي) → `أولاً.` → `(١).` → `(أ).` → `(أولاً).` → again.
//! The first level present in a document is level 1, so a chapter cut out of a larger file
//! renumbers itself from `١.`.

use serde_json::{Value, json};
use wordcraft_doc::arabic_numbers as ar;
use wordcraft_doc::numbering::Level;
use wordcraft_doc::props::{Align, CharProps, NumRef, TabAlign, TabStop};
use wordcraft_doc::{InlineObject, Paragraph, Pos, StoryRef, para_block};

use super::sel_result;
use crate::{CmdError, CmdResult, CommandSpec, Session, p};

/// Ribbon tab of every command here.
pub const TAB: &str = "Service";

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("service.renumber", "Renumber by the Service Cycle", "Service › Numbering", renumber).params(
            r#"{"scope"?: "document|selection" (default document), "firstLevelIsTop"?: bool (default true: the shallowest level present becomes ١.)}"#,
        ),
        CommandSpec::new("service.applyList", "Service Multilevel List", "Service › Numbering", apply_list)
            .params(r#"{"scope"?: "document|selection"} — typed markers become real list levels (١. أ. أولاً. …)"#),
        CommandSpec::new("service.classification", "Classification", "Service › Page", classification)
            .params(r#"{"value": "عام|محدود|سري|سري وشخصي|سري للغاية", "pageNumbers"?: bool (default true), "start"?: n, "end"?: n}"#),
        CommandSpec::new("service.pageNumbers", "Page Numbers ( n - N )", "Service › Page", page_numbers)
            .params(r#"{"start"?: n (default 1), "end"?: n (default: the section's page count)}"#),
        CommandSpec::new("service.kashida", "Kashida Justification", "Service › Text", kashida)
            .params(r#"{"value"?: "distribute|low|medium|high|off" (default distribute)}"#),
        CommandSpec::new("service.finalDot", "End Items with a Dot", "Service › Text", final_dot),
        CommandSpec::new("service.external", "Format with ServiceWord-AR", "Service › Engine", external).params(
            r#"{"engine"?: path of ServiceWord-AR's cli.py (default: $WORDCRAFT_AR_ENGINE, then servicewordar/cli.py beside the program or in the settings folder), "python"?: interpreter (default: py -3 on Windows, python3 elsewhere), "pdf"?: bool (also make the stamped print PDF)}"#,
        ),
    ]
}

// ------------------------------------------------------------------ markers

/// Marker kinds of the cycle, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Digit = 0,
    Abjad = 1,
    Ordinal = 2,
    DigitParen = 3,
    AbjadParen = 4,
    OrdinalParen = 5,
}

impl Kind {
    fn from_depth(d: usize) -> Kind {
        match d % 6 {
            0 => Kind::Digit,
            1 => Kind::Abjad,
            2 => Kind::Ordinal,
            3 => Kind::DigitParen,
            4 => Kind::AbjadParen,
            _ => Kind::OrdinalParen,
        }
    }
    fn index(self) -> usize {
        self as usize
    }
}

fn to_ascii_digits(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '٠'..='٩' => char::from_u32('0' as u32 + (c as u32 - '٠' as u32)).unwrap_or(c),
            '۰'..='۹' => char::from_u32('0' as u32 + (c as u32 - '۰' as u32)).unwrap_or(c),
            c => c,
        })
        .collect()
}

/// Arabic text with tanween, hamza forms and tatweel normalised, for matching ordinal words.
fn norm(s: &str) -> String {
    s.chars()
        .filter(|c| !matches!(c, '\u{064B}'..='\u{0652}' | '\u{0670}' | '\u{0640}'))
        .map(|c| match c {
            'أ' | 'إ' | 'آ' => 'ا',
            'ة' => 'ه',
            'ى' => 'ي',
            c => c,
        })
        .collect()
}

const ORDINAL_WORDS: [&str; 30] = [
    "اولا",
    "ثانيا",
    "ثالثا",
    "رابعا",
    "خامسا",
    "سادسا",
    "سابعا",
    "ثامنا",
    "تاسعا",
    "عاشرا",
    "حادي عشر",
    "ثاني عشر",
    "ثالث عشر",
    "رابع عشر",
    "خامس عشر",
    "سادس عشر",
    "سابع عشر",
    "ثامن عشر",
    "تاسع عشر",
    "عشرون",
    "حادي وعشرون",
    "ثاني وعشرون",
    "ثالث وعشرون",
    "رابع وعشرون",
    "خامس وعشرون",
    "سادس وعشرون",
    "سابع وعشرون",
    "ثامن وعشرون",
    "تاسع وعشرون",
    "ثلاثون",
];
const ABJAD_LETTERS: &str = "أابجدهوزحطيكلمنسعفصقرشتثخذضظغ";

/// A typed marker at the start of `text`: (kind, number, byte length including the space after).
fn detect(text: &str) -> Option<(Kind, u32, usize)> {
    let t = text.trim_start_matches([' ', '\u{00A0}', '\t']);
    let lead = text.len() - t.len();
    let (paren, body) = match t.strip_prefix(['(', '（']) {
        Some(rest) => (true, rest.trim_start()),
        None => (false, t),
    };
    // the marker token: up to the closing paren / dot
    let mut tok_end = 0;
    for (i, c) in body.char_indices() {
        if matches!(c, '.' | '．' | ')' | '）' | '-' | '–' | ':' | '\t') {
            tok_end = i;
            break;
        }
        if c == ' ' && paren {
            continue;
        }
        if c == ' ' {
            // ordinal words have one inner space ("حادي عشر"): keep scanning once
            let rest = body.get(i + 1..).unwrap_or("");
            if rest.starts_with("عشر") || rest.starts_with("وعشرون") {
                continue;
            }
            return None;
        }
        if i > 40 {
            return None;
        }
    }
    if tok_end == 0 {
        return None;
    }
    let token = body.get(..tok_end)?.trim();
    let mut rest = body.get(tok_end..)?;
    if paren {
        rest = rest.trim_start_matches([' ']).strip_prefix([')', '）'])?;
    }
    let rest_trim = rest.trim_start();
    let has_dot = rest_trim.starts_with(['.', '．', '-', '–', ':']);
    if !paren && !has_dot {
        return None;
    }
    let after = if has_dot { rest_trim.get(1..).unwrap_or("") } else { rest_trim };
    let consumed = text.len() - after.len();
    let consumed = consumed + (after.len() - after.trim_start_matches([' ', '\u{00A0}', '\t']).len());
    let _ = lead;
    // digits
    let ascii = to_ascii_digits(token);
    if !ascii.is_empty() && ascii.chars().all(|c| c.is_ascii_digit()) {
        let n: u32 = ascii.parse().unwrap_or(1);
        return Some((if paren { Kind::DigitParen } else { Kind::Digit }, n.max(1), consumed));
    }
    // ordinal words
    let nt = norm(token);
    if let Some(i) = ORDINAL_WORDS.iter().position(|w| *w == nt) {
        return Some((if paren { Kind::OrdinalParen } else { Kind::Ordinal }, i as u32 + 1, consumed));
    }
    // abjad: one letter, maybe with a tatweel
    let letters: Vec<char> = token.chars().filter(|c| *c != '\u{0640}').collect();
    if letters.len() == 1 && ABJAD_LETTERS.contains(letters[0]) {
        let c = if letters[0] == 'ا' { 'أ' } else { letters[0] };
        let n = "أبجدهوزحطيكلمنسعفصقرشتثخذضظغ".chars().position(|x| x == c).map(|i| i as u32 + 1).unwrap_or(1);
        return Some((if paren { Kind::AbjadParen } else { Kind::Abjad }, n, consumed));
    }
    None
}

/// Marker text for a level (0-based depth) and number.
fn marker_text(depth: usize, n: u32) -> String {
    let core = match Kind::from_depth(depth) {
        Kind::Digit | Kind::DigitParen => ar::arabic_indic(n),
        Kind::Abjad | Kind::AbjadParen => ar::abjad(n),
        Kind::Ordinal | Kind::OrdinalParen => ar::ordinal(n),
    };
    if depth % 6 >= 3 { format!("({core}).") } else { format!("{core}.") }
}

/// Hanging width of a level's marker at `size` points (the engine's rough estimate; the external
/// engine measures the real fonts).
fn hang_for(depth: usize, size: f32) -> f32 {
    let em = size.max(8.0);
    match Kind::from_depth(depth) {
        Kind::Digit => 1.4 * em,
        Kind::Abjad => 1.5 * em,
        Kind::Ordinal => 2.9 * em,
        Kind::DigitParen => 2.0 * em,
        Kind::AbjadParen => 2.1 * em,
        Kind::OrdinalParen => 3.5 * em,
    }
}

/// Paragraphs of the scope with their detected markers and resolved depths (`None` depth: an
/// unnumbered paragraph under the previous marker).
struct Item {
    path: wordcraft_doc::Path,
    marker: Option<(Kind, u32, usize)>,
    depth: usize,
    numbered: bool,
}

fn analyse(s: &Session, paths: &[wordcraft_doc::Path], first_is_top: bool) -> Vec<Item> {
    let mut items = Vec::new();
    let mut prev_depth: i32 = -1;
    let mut typed_depth: i32 = -1;
    let mut clamp: Vec<(usize, usize)> = Vec::new(); // typed kind -> depth
    for path in paths {
        let Some(para) = s.doc.para(StoryRef::Body, path) else { continue };
        let det = detect(&para.text);
        // A real list item (Word numbering, its marker not in the text) nests under the last typed
        // marker: list level 0 is one level below it.
        let listed = para.props.numbering.as_ref().filter(|n| n.num > 0).map(|n| n.level as usize);
        match det {
            None if listed.is_some() => {
                let level = listed.unwrap_or(0);
                let base = (typed_depth + 1).max(0) as usize;
                let depth = (base + level).min(50);
                items.push(Item { path: path.clone(), marker: None, depth, numbered: true });
                prev_depth = depth as i32;
            }
            None => {
                if para.text.trim().is_empty() || prev_depth < 0 {
                    continue;
                }
                items.push(Item { path: path.clone(), marker: None, depth: (prev_depth + 1) as usize, numbered: false });
            }
            Some((kind, n, len)) => {
                let typed = kind.index();
                let depth = if let Some((_, d)) = clamp.iter().find(|(k, _)| *k == typed) {
                    *d
                } else {
                    let mut d = None;
                    for k in 0..4usize {
                        let c = typed + 6 * k;
                        if c as i32 <= prev_depth + 1 {
                            d = Some(c);
                        }
                    }
                    match d {
                        Some(d) => d,
                        None => {
                            let d = (prev_depth + 1).max(0) as usize;
                            clamp.push((typed, d));
                            d
                        }
                    }
                };
                clamp.retain(|(k, d)| *d <= depth || *k == typed);
                items.push(Item { path: path.clone(), marker: Some((kind, n, len)), depth, numbered: true });
                prev_depth = depth as i32;
                typed_depth = depth as i32;
            }
        }
    }
    if first_is_top {
        let shift = items.iter().filter(|i| i.numbered).map(|i| i.depth).min().unwrap_or(0);
        if shift > 0 {
            for it in &mut items {
                it.depth = it.depth.saturating_sub(shift);
            }
        }
    }
    items
}

fn scope_paths(s: &Session, v: &Value) -> Vec<wordcraft_doc::Path> {
    if p::str(v, "scope") == Some("selection") { super::selected_paths(s) } else { s.doc.para_paths(StoryRef::Body) }
}

fn para_size(s: &Session, path: &wordcraft_doc::Path) -> f32 {
    s.doc.para(StoryRef::Body, path).and_then(|p| p.runs.first().and_then(|r| r.props.size_cs.or(r.props.size))).unwrap_or(14.0)
}

/// `service.renumber`: typed markers rewritten by the cycle, as text, with hanging indents so
/// each level starts under its parent's marker.
fn renumber(s: &mut Session, v: &Value) -> CmdResult {
    let paths = scope_paths(s, v);
    let items = analyse(s, &paths, p::bool(v, "firstLevelIsTop").unwrap_or(true));
    let mut counters: Vec<u32> = vec![0; 64];
    let mut last_depth: usize = 0;
    let mut changed = 0usize;
    for it in &items {
        let size = para_size(s, &it.path);
        if it.numbered {
            if it.depth < last_depth {
                for c in counters.iter_mut().skip(it.depth + 1) {
                    *c = 0;
                }
            }
            if let Some(c) = counters.get_mut(it.depth) {
                *c += 1;
            }
            last_depth = it.depth;
            let n = counters.get(it.depth).copied().unwrap_or(1);
            let marker = marker_text(it.depth, n);
            let len = it.marker.map(|m| m.2).unwrap_or(0);
            let a = Pos::new(StoryRef::Body, it.path.clone(), 0);
            let b = Pos::new(StoryRef::Body, it.path.clone(), len);
            let props = s.doc.para(StoryRef::Body, &it.path).and_then(|p| p.runs.first().map(|r| r.props.clone())).unwrap_or_default();
            if len > 0 {
                s.doc.delete_range(&a, &b)?;
            }
            s.doc.insert_text(&a, &format!("{marker}\t"), &props)?;
            let hang = hang_for(it.depth, size);
            let start: f32 = (0..it.depth).map(|d| hang_for(d, size) * 0.9).sum();
            let para = s.doc.para_mut(StoryRef::Body, &it.path)?;
            para.props.bidi = Some(true);
            para.props.indent_left = Some(start + hang);
            para.props.indent_first = Some(-hang);
            para.props.tabs = Some(vec![TabStop { pos: start + hang, align: TabAlign::Left, leader: Default::default() }]);
            // a Word list item becomes a typed marker: the list numbering goes
            para.props.numbering = Some(NumRef { num: 0, level: 0 });
            para.touch();
            changed += 1;
        } else {
            // body text under a marker: starts where the parent's text starts
            let start: f32 = (0..it.depth).map(|d| hang_for(d, size) * 0.9).sum();
            let para = s.doc.para_mut(StoryRef::Body, &it.path)?;
            para.props.bidi = Some(true);
            para.props.indent_left = Some(start);
            para.props.indent_first = Some(0.0);
            para.touch();
        }
    }
    let mut r = sel_result(s)?;
    r["renumbered"] = json!(changed);
    Ok(r)
}

/// The service cycle as a nine-level list definition (see `wordcraft_doc::numbering::service_level`).
pub fn service_levels(_size: f32) -> Vec<Level> {
    wordcraft_doc::numbering::levels_for(wordcraft_doc::numbering::ListKind::Service)
}

/// `service.applyList`: typed markers become real list levels of one service-cycle list.
fn apply_list(s: &mut Session, v: &Value) -> CmdResult {
    let paths = scope_paths(s, v);
    let items = analyse(s, &paths, p::bool(v, "firstLevelIsTop").unwrap_or(true));
    let size = items.first().map(|it| para_size(s, &it.path)).unwrap_or(14.0);
    let num = s.doc.numbering.try_add_levels(Some("كتابات الخدمة".into()), service_levels(size))?;
    let mut changed = 0usize;
    for it in &items {
        if !it.numbered {
            continue;
        }
        let len = it.marker.map(|m| m.2).unwrap_or(0);
        if len > 0 {
            let a = Pos::new(StoryRef::Body, it.path.clone(), 0);
            let b = Pos::new(StoryRef::Body, it.path.clone(), len);
            s.doc.delete_range(&a, &b)?;
        }
        let para = s.doc.para_mut(StoryRef::Body, &it.path)?;
        para.props.bidi = Some(true);
        para.props.numbering = Some(NumRef { num, level: it.depth.min(8) as u8 });
        para.props.indent_left = None;
        para.props.indent_first = None;
        para.props.tabs = None;
        para.touch();
        changed += 1;
    }
    let mut r = sel_result(s)?;
    r["num"] = json!(num);
    r["numbered"] = json!(changed);
    Ok(r)
}

// ------------------------------------------------------------- page furniture

pub const CLASSIFICATIONS: [&str; 5] = ["عام", "محدود", "سري", "سري وشخصي", "سري للغاية"];

fn hf_part(s: &mut Session, header: bool) -> u32 {
    let sect = s.doc.section_mut(0).clone();
    let existing = if header { sect.headers.default } else { sect.footers.default };
    if let Some(id) = existing.filter(|id| s.doc.parts.contains_key(id)) {
        return id;
    }
    let style = if header { "Header" } else { "Footer" };
    let id = s.doc.add_part(
        if header { wordcraft_doc::PartKind::Header } else { wordcraft_doc::PartKind::Footer },
        vec![para_block(Paragraph::new().styled(style))],
    );
    let sect = s.doc.section_mut(0);
    if header {
        sect.headers.default = Some(id);
    } else {
        sect.footers.default = Some(id);
    }
    id
}

fn centred(style: &str) -> Paragraph {
    let mut para = Paragraph::new().styled(style);
    para.props.align = Some(Align::Center);
    para.props.bidi = Some(true);
    para.props.tabs = Some(vec![TabStop { pos: 0.0, align: TabAlign::Clear, leader: Default::default() }]);
    para
}

fn field(instr: &str, result: &str) -> InlineObject {
    InlineObject::Field { instr: instr.into(), result: result.into(), locked: false, dirty: false, code: false, form: String::new() }
}

/// The footer's page line «( n - N )»: page number on the right, last page on the left, in
/// Arabic-Indic digits whatever the viewer's numeral setting (the field switch does it).
fn page_line(start: u32, end: Option<u32>) -> Result<Paragraph, CmdError> {
    let mut para = centred("Footer");
    let bold = CharProps { bold: Some(true), bold_cs: Some(true), ..CharProps::default() };
    para.insert_text(0, "( ", &bold)?;
    para.insert_object(para.len(), field("PAGE \\* HindiArabic", &ar::arabic_indic(start)), &bold)?;
    para.insert_text(para.len(), " - ", &bold)?;
    let off = para.len();
    match end {
        Some(e) => {
            para.insert_text(off, &ar::arabic_indic(e), &bold)?;
        }
        None => para.insert_object(off, field("SECTIONPAGES \\* HindiArabic", &ar::arabic_indic(1)), &bold)?,
    }
    let off = para.len();
    para.insert_text(off, " )", &bold)?;
    Ok(para)
}

/// `service.classification`: the classification word centred in the header, and in the footer
/// under the page line.
fn classification(s: &mut Session, v: &Value) -> CmdResult {
    let value = p::req_str(v, "value")?.trim().to_string();
    if !CLASSIFICATIONS.contains(&value.as_str()) {
        return Err(CmdError::Params(format!("unknown classification `{value}` (عام | محدود | سري | سري وشخصي | سري للغاية)")));
    }
    let bold = CharProps { bold: Some(true), bold_cs: Some(true), underline: Some(wordcraft_doc::props::Underline::Single), ..CharProps::default() };
    let header = hf_part(s, true);
    let mut hp = centred("Header");
    hp.insert_text(0, &value, &bold)?;
    s.doc.set_story(StoryRef::Part(header), vec![para_block(hp)])?;
    let footer = hf_part(s, false);
    let mut blocks = Vec::new();
    if p::bool(v, "pageNumbers").unwrap_or(true) {
        let start = p::u64(v, "start").map(|x| x.min(9999) as u32).unwrap_or(1);
        let end = p::u64(v, "end").map(|x| x.min(9999) as u32);
        blocks.push(para_block(page_line(start, end)?));
        if start > 1 {
            s.doc.section_mut(0).page_num_start = Some(start);
        }
    }
    let mut fp = centred("Footer");
    fp.insert_text(0, &value, &bold)?;
    blocks.push(para_block(fp));
    s.doc.set_story(StoryRef::Part(footer), blocks)?;
    Ok(json!({"header": header, "footer": footer, "classification": value}))
}

fn page_numbers(s: &mut Session, v: &Value) -> CmdResult {
    let start = p::u64(v, "start").map(|x| x.min(9999) as u32).unwrap_or(1);
    let end = p::u64(v, "end").map(|x| x.min(9999) as u32);
    let footer = hf_part(s, false);
    // keep a classification paragraph if the footer has one
    let keep: Vec<_> = s
        .doc
        .parts
        .get(&footer)
        .map(|part| part.blocks.iter().filter(|b| b.as_para().is_some_and(|p| CLASSIFICATIONS.contains(&p.text.trim()))).cloned().collect())
        .unwrap_or_default();
    let mut blocks = vec![para_block(page_line(start, end)?)];
    blocks.extend(keep);
    s.doc.set_story(StoryRef::Part(footer), blocks)?;
    if start > 1 {
        s.doc.section_mut(0).page_num_start = Some(start);
    }
    Ok(json!({"footer": footer}))
}

// ------------------------------------------------------------------- text

fn kashida(s: &mut Session, v: &Value) -> CmdResult {
    let a = match p::str(v, "value").unwrap_or("distribute") {
        "off" | "justify" => Align::Justify,
        "low" => Align::LowKashida,
        "medium" => Align::MediumKashida,
        "high" => Align::HighKashida,
        _ => Align::Distribute,
    };
    super::para::fmt(s, &|pp| {
        pp.align = Some(a);
        pp.bidi = Some(true);
    })
}

/// `service.finalDot`: every selected (or all) numbered item ends with a dot.
fn final_dot(s: &mut Session, v: &Value) -> CmdResult {
    let paths = scope_paths(s, v);
    let mut added = 0usize;
    for path in paths {
        let Some(para) = s.doc.para(StoryRef::Body, &path) else { continue };
        let t = para.text.trim_end();
        let listed = para.props.numbering.as_ref().is_some_and(|n| n.num > 0);
        if t.is_empty() || (!listed && detect(&para.text).is_none()) {
            continue;
        }
        let last = t.chars().last().unwrap_or(' ');
        if matches!(last, '.' | ':' | '؛' | '؟' | '!' | '،' | '…') {
            continue;
        }
        let props = para.runs.last().map(|r| r.props.clone()).unwrap_or_default();
        let at = Pos::new(StoryRef::Body, path.clone(), t.len());
        // trailing spaces are dropped
        if t.len() < para.text.len() {
            let b = Pos::new(StoryRef::Body, path.clone(), para.text.len());
            s.doc.delete_range(&at, &b)?;
        }
        s.doc.insert_text(&at, ".", &props)?;
        added += 1;
    }
    let mut r = sel_result(s)?;
    r["dots"] = json!(added);
    Ok(r)
}

// ----------------------------------------------------------- external engine

fn find_engine(given: Option<&str>) -> Option<std::path::PathBuf> {
    if let Some(g) = given {
        let p = std::path::PathBuf::from(g);
        return p.exists().then_some(p);
    }
    if let Some(e) = std::env::var_os("WORDCRAFT_AR_ENGINE") {
        let p = std::path::PathBuf::from(e);
        if p.exists() {
            return Some(p);
        }
    }
    let mut cands = Vec::new();
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        cands.push(dir.join("servicewordar").join("cli.py"));
        if let Some(up) = dir.parent() {
            cands.push(up.join("servicewordar").join("cli.py"));
        }
    }
    if let Some(d) = wordcraft_control_key::settings_dir() {
        cands.push(d.join("servicewordar").join("cli.py"));
    }
    cands.into_iter().find(|p| p.exists())
}

/// Interpreters to try for the engine, in order: the one given, then on Windows `py -3`, the
/// per-user installs under `%LOCALAPPDATA%\\Programs\\Python` (a program started before Python
/// was installed doesn't have them on its PATH) and `python`; `python3` then `python` elsewhere.
fn python_candidates(given: Option<&str>) -> Vec<(String, Vec<String>)> {
    let mut c: Vec<(String, Vec<String>)> = Vec::new();
    if let Some(py) = given {
        c.push((py.to_string(), Vec::new()));
        return c;
    }
    if cfg!(windows) {
        c.push(("py".into(), vec!["-3".into()]));
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let programs = std::path::PathBuf::from(local).join("Programs").join("Python");
            let launcher = programs.join("Launcher").join("py.exe");
            if launcher.exists() {
                c.push((launcher.to_string_lossy().into_owned(), vec!["-3".into()]));
            }
            // Newest first; the installer names the folder after the version.
            for v in ["Python314", "Python313", "Python312", "Python311", "Python310", "Python39"] {
                let exe = programs.join(v).join("python.exe");
                if exe.exists() {
                    c.push((exe.to_string_lossy().into_owned(), Vec::new()));
                }
            }
        }
        // Last: a bare `python`, which on a fresh Windows is the Store's stub (it exits with an
        // error that then reaches the user as "ServiceWord-AR failed").
        c.push(("python".into(), Vec::new()));
    } else {
        c.push(("python3".into(), Vec::new()));
        c.push(("python".into(), Vec::new()));
    }
    c
}

/// Run the engine with the first interpreter that starts; a Python that exists but fails is
/// reported as such (its last stderr line), a missing one is skipped.
fn run_python(given: Option<&str>, tail: &[String]) -> Result<std::process::Output, CmdError> {
    let cands = python_candidates(given);
    let mut last_err = String::from("no Python interpreter found (install Python 3 from python.org)");
    for (prog, args) in &cands {
        // The engine prints Arabic (file names, progress); a Windows console defaults to a legacy
        // code page that cannot encode it, which would make Python fail instead of formatting.
        match std::process::Command::new(prog).args(args).args(tail).env("PYTHONUTF8", "1").env("PYTHONIOENCODING", "utf-8").output() {
            Ok(out) => return Ok(out),
            Err(e) => last_err = format!("cannot run {prog}: {e}"),
        }
    }
    Err(CmdError::Failed(last_err))
}

/// `service.external`: the document goes to ServiceWord-AR (Python; real-font measurement,
/// kashida written into the text, equalised markers, embedded digit fonts) and comes back
/// formatted. Needs Python with the engine's wheels installed.
fn external(s: &mut Session, v: &Value) -> CmdResult {
    let engine = find_engine(p::str(v, "engine"))
        .ok_or_else(|| CmdError::Failed("ServiceWord-AR engine not found (put the servicewordar folder beside the program)".into()))?;
    let bytes = crate::io::save_bytes("document.docx", &s.doc).map_err(CmdError::Failed)?;
    let dir = std::env::temp_dir();
    let stem = format!("wordcraft-ar-{}", std::process::id());
    let src = dir.join(format!("{stem}.docx"));
    crate::io::write_file(&src.to_string_lossy(), &bytes).map_err(CmdError::Failed)?;
    let mut tail: Vec<String> = vec![engine.to_string_lossy().into_owned(), src.to_string_lossy().into_owned()];
    if p::bool(v, "pdf").unwrap_or(false) {
        tail.push("--pdf".into());
    }
    let out = run_python(p::str(v, "python"), &tail)?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(CmdError::Failed(format!("ServiceWord-AR failed: {}", err.lines().last().unwrap_or("").trim())));
    }
    let formatted = dir.join(format!("{stem}_منسق.docx"));
    let doc = crate::io::open_path(&formatted).map_err(CmdError::Failed)?;
    let name = s.file_name.clone();
    s.set_document(doc);
    s.file_name = name;
    s.dirty = true;
    let pdf = dir.join(format!("{stem}_منسق_طباعة.pdf"));
    Ok(json!({"formatted": formatted.to_string_lossy(), "pdf": pdf.exists().then(|| pdf.to_string_lossy().into_owned())}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wordcraft_doc::section::NumFormat;

    #[test]
    fn python_candidates_prefer_the_given_interpreter_and_always_offer_a_fallback() {
        assert_eq!(python_candidates(Some("/opt/py/bin/python3")), vec![("/opt/py/bin/python3".to_string(), Vec::new())]);
        let c = python_candidates(None);
        assert!(!c.is_empty());
        if cfg!(windows) {
            assert_eq!(c[0], ("py".to_string(), vec!["-3".to_string()]));
            assert_eq!(c.last().map(|x| x.0.as_str()), Some("python"));
        } else {
            assert_eq!(c[0], ("python3".to_string(), Vec::new()));
        }
    }

    #[test]
    fn missing_interpreter_is_reported_not_panicked() {
        let err = run_python(Some("wordcraft-no-such-python-xyz"), &["x".to_string()]).err().map(|e| e.to_string()).unwrap_or_default();
        assert!(err.contains("wordcraft-no-such-python-xyz"), "{err}");
    }

    #[test]
    fn detects_typed_markers() {
        assert_eq!(detect("١. مهام"), Some((Kind::Digit, 1, "١. ".len())));
        assert_eq!(detect("4. مهام"), Some((Kind::Digit, 4, 3)));
        assert_eq!(detect("جـ. واجبات"), Some((Kind::Abjad, 3, "جـ. ".len())));
        assert_eq!(detect("أولا. المهمة"), Some((Kind::Ordinal, 1, "أولا. ".len())));
        assert_eq!(detect("ثانياً. الواجبات"), Some((Kind::Ordinal, 2, "ثانياً. ".len())));
        assert_eq!(detect("حادي عشر. بند"), Some((Kind::Ordinal, 11, "حادي عشر. ".len())));
        assert_eq!(detect("(٣) بند"), Some((Kind::DigitParen, 3, "(٣) ".len())));
        assert_eq!(detect("(أ). بند"), Some((Kind::AbjadParen, 1, "(أ). ".len())));
        assert_eq!(detect("نص عادي بلا علامة"), None);
        assert_eq!(detect(""), None);
    }

    #[test]
    fn markers_follow_the_cycle() {
        assert_eq!(marker_text(0, 2), "٢.");
        assert_eq!(marker_text(1, 3), "جـ.");
        assert_eq!(marker_text(2, 1), "أولاً.");
        assert_eq!(marker_text(3, 10), "(١٠).");
        assert_eq!(marker_text(4, 5), "(هـ).");
        assert_eq!(marker_text(5, 2), "(ثانياً).");
        assert_eq!(marker_text(6, 1), "١.", "the cycle repeats");
    }

    #[test]
    fn renumbers_a_cut_out_chapter_from_one() {
        let mut s = Session::new(wordcraft_doc::Document::from_text(
            "أولا. المهمة\nتوفر رعاية.\nثانياً. الواجبات\n(١) تقديم الخدمات\n(٢) تصنيف المرضى\nثالثا. واجبات المدير",
        ));
        let r = s.run("service.renumber", &json!({})).unwrap();
        assert_eq!(r["renumbered"], 5);
        let texts: Vec<String> = s.doc.para_paths(StoryRef::Body).iter().map(|p| s.doc.para(StoryRef::Body, p).unwrap().text.clone()).collect();
        assert!(texts[0].starts_with("١.\t"), "{texts:?}");
        assert!(texts[2].starts_with("٢.\t"), "{texts:?}");
        assert!(texts[3].starts_with("أ.\t"), "the items under ٢. are one level down: {texts:?}");
        assert!(texts[4].starts_with("ب.\t"), "{texts:?}");
        assert!(texts[5].starts_with("٣.\t"), "{texts:?}");
        // the body line keeps its text and sits under its heading
        assert_eq!(texts[1], "توفر رعاية.");
    }

    /// Word list items (markers not in the text) nest under the last typed marker and come out as
    /// typed markers of the next level.
    #[test]
    fn word_list_items_nest_under_the_typed_marker() {
        let mut doc = wordcraft_doc::Document::from_text("١. شعبة الأسنان\nثانياً. الواجبات\nفحص المرضى\nإدامة الأجهزة");
        let num = doc.numbering.add_list(wordcraft_doc::numbering::ListKind::NumberedParen);
        for path in doc.para_paths(StoryRef::Body).into_iter().skip(2) {
            doc.para_mut(StoryRef::Body, &path).unwrap().props.numbering = Some(NumRef { num, level: 0 });
        }
        let mut s = Session::new(doc);
        let r = s.run("service.renumber", &json!({})).unwrap();
        assert_eq!(r["renumbered"], 4);
        let texts: Vec<String> = s.doc.para_paths(StoryRef::Body).iter().map(|p| s.doc.para(StoryRef::Body, p).unwrap().text.clone()).collect();
        assert!(texts[1].starts_with("أ.\t"), "{texts:?}");
        assert!(texts[2].starts_with("أولاً.\t"), "{texts:?}");
        assert!(texts[3].starts_with("ثانياً.\t"), "{texts:?}");
        let p3 = s.doc.para(StoryRef::Body, &s.doc.para_paths(StoryRef::Body)[3]).unwrap();
        assert_eq!(p3.props.numbering.as_ref().map(|n| n.num), Some(0), "the Word list numbering is gone");
    }

    #[test]
    fn list_levels_and_classification() {
        let mut s = Session::new(wordcraft_doc::Document::from_text("١. مهام\nأ. شعبة\nأولاً. المهمة\n(١) بند"));
        let r = s.run("service.applyList", &json!({})).unwrap();
        assert_eq!(r["numbered"], 4);
        let num = r["num"].as_u64().unwrap() as u32;
        let lv = s.doc.numbering.level(num, 1).unwrap();
        assert_eq!(lv.format, NumFormat::ArabicAbjad);
        assert_eq!(lv.format.format_lang(3, lv.chr.lang_bidi.as_deref()), "جـ");
        let r = s.run("service.classification", &json!({"value": "سري"})).unwrap();
        let footer = r["footer"].as_u64().unwrap() as u32;
        let part = s.doc.parts.get(&footer).unwrap();
        assert_eq!(part.blocks.len(), 2, "page line + classification");
        assert_eq!(part.blocks[1].as_para().unwrap().text, "سري");
        assert!(s.run("service.classification", &json!({"value": "x"})).is_err());
        let r = s.run("service.finalDot", &json!({})).unwrap();
        assert_eq!(r["dots"], 4);
        s.run("service.kashida", &json!({"value": "medium"})).unwrap();
    }
}
