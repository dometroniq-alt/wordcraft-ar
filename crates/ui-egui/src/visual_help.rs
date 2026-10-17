//! WordCraft AR — visual help for ribbon buttons: a thumbnail of what a command does and a
//! one-sentence explanation, shown in the button's tooltip ([`crate::widgets`]). The drawings
//! are original, made of lines and boxes on a mini page (no screenshots, no product artwork).
//!
//! Each entry is keyed by the command id plus the parameter that picks its variant (the
//! kashida level, "selection only", "with PDF"), see [`key`]. [`EXPLAINED`] lists every key, so
//! a test can hold the ribbon to it; [`thumbnail`] says whether it drew one.

use egui::{Align2, Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};
use serde_json::Value;

use crate::theme::Tokens;

/// Thumbnail size, in points.
pub const SIZE: egui::Vec2 = egui::Vec2::new(176.0, 104.0);

/// The visual-help key of a command and its parameters: the id, plus `:value` for the kashida
/// level, `:selection` for Selection Only and `:pdf` for the engine's PDF variant.
pub fn key(id: &str, params: &Value) -> String {
    let variant = match id {
        "service.kashida" => params.get("value").and_then(Value::as_str),
        "service.renumber" => params.get("scope").and_then(Value::as_str).filter(|s| *s == "selection"),
        "service.external" => params.get("pdf").and_then(Value::as_bool).filter(|b| *b).map(|_| "pdf"),
        _ => None,
    };
    match variant {
        Some(v) => format!("{id}:{v}"),
        None => id.to_string(),
    }
}

/// What a command does, in one plain sentence (English source; the catalogs translate it).
pub fn explanation(key: &str) -> Option<&'static str> {
    EXPLAINED.iter().find(|(k, _)| *k == key).map(|(_, e)| *e)
}

/// Every command with visual help: (key, explanation).
pub const EXPLAINED: &[(&str, &str)] = &[
    // Service tab
    (
        "service.renumber",
        "Reads the markers you typed (١. أ. أولاً. (١)) and renumbers every item by the service cycle, level by level, with hanging indents.",
    ),
    ("service.renumber:selection", "Renumbers only the selected paragraphs by the service cycle; the rest of the document is untouched."),
    ("service.applyList", "Turns the typed markers into a real Word multilevel list that keeps numbering itself as you add items."),
    ("service.finalDot", "Puts a full stop at the end of every item that lacks one."),
    ("service.classification", "Stamps the classification word (محدود، سري…) in the header and footer of every page."),
    ("service.pageNumbers", "Writes ( page - total ) in Arabic-Indic digits at the bottom of every page."),
    ("service.kashida:off", "Justifies by widening the spaces only; the last line of a paragraph stays short."),
    ("service.kashida:low", "Justifies by stretching one letter joint per word a little, then the spaces."),
    ("service.kashida:medium", "Justifies by stretching the letter joints more, before the spaces."),
    ("service.kashida:high", "Justifies mostly by stretching the letter joints; spaces barely widen."),
    ("service.kashida:distribute", "Stretches every line to both margins, the last line of each paragraph too."),
    (
        "service.external",
        "Sends the document to the ServiceWord-AR engine, which measures the real fonts and writes the kashida into the text, and opens the result.",
    ),
    ("service.external:pdf", "Formats with ServiceWord-AR and also makes the stamped print PDF beside the document."),
    // Mailings tab
    ("mailings.envelopes", "Makes an envelope page with the delivery and return addresses, ready to print."),
    ("mailings.labels", "Makes a sheet of address labels, the same address on each or one per recipient."),
    ("mailings.start", "Starts a mail merge: one letter, envelope or label design that is filled in for every person in a list."),
    ("mailings.recipients", "Picks the list of people to merge: a table you type, or a spreadsheet or CSV file."),
    ("mailings.editRecipients", "Shows the recipient list so you can tick who is included, sort it or filter it."),
    ("mailings.highlightFields", "Shades every merge field in the document so you can see where the names and addresses will go."),
    ("mailings.addressBlock", "Inserts a complete address (name, street, city) as one field, laid out like on an envelope."),
    ("mailings.greetingLine", "Inserts the opening line (Dear …,) as a field that takes each recipient's name."),
    ("mailings.insertField", "Inserts one column of the list (first name, rank, unit…) at the caret as a «field»."),
    ("mailings.rules", "Adds conditions: if a field has a certain value, write one text, otherwise another."),
    ("mailings.matchFields", "Pairs the columns of your list with the parts of an address or greeting when the names differ."),
    ("mailings.preview", "Replaces the «fields» with one recipient's real data so you see the finished letter."),
    ("mailings.previous", "Shows the previous recipient's letter in the preview."),
    ("mailings.next", "Shows the next recipient's letter in the preview."),
    ("mailings.findRecipient", "Finds a recipient by name or any value and previews that letter."),
    ("mailings.checkErrors", "Checks every merged letter for missing fields or data before you print."),
    ("mailings.finish", "Produces the merged letters: one document with every letter, or straight to the printer."),
];

/// Draw the thumbnail for `key` into `r`; false when there is none (the tooltip then shows text
/// only).
pub fn thumbnail(p: &Painter, r: Rect, key: &str, t: &Tokens) -> bool {
    let d = Draw { p, r, t };
    match key {
        "service.renumber" => d.renumbered(false),
        "service.renumber:selection" => d.renumbered(true),
        "service.applyList" => d.service_list(),
        "service.finalDot" => d.final_dot(),
        "service.classification" => d.classification(),
        "service.pageNumbers" => d.page_numbers(),
        "service.kashida:off" => d.kashida(0.0, false),
        "service.kashida:low" => d.kashida(0.35, false),
        "service.kashida:medium" => d.kashida(0.6, false),
        "service.kashida:high" => d.kashida(0.9, false),
        "service.kashida:distribute" => d.kashida(0.6, true),
        "service.external" => d.engine(false),
        "service.external:pdf" => d.engine(true),
        "mailings.envelopes" => d.envelope(),
        "mailings.labels" => d.labels(),
        "mailings.start" => d.mail_merge(),
        "mailings.recipients" => d.recipients(false),
        "mailings.editRecipients" => d.recipients(true),
        "mailings.highlightFields" => d.letter_fields(true, false),
        "mailings.addressBlock" => d.address_block(),
        "mailings.greetingLine" => d.greeting_line(),
        "mailings.insertField" => d.letter_fields(false, true),
        "mailings.rules" => d.rules(),
        "mailings.matchFields" => d.match_fields(),
        "mailings.preview" => d.preview(),
        "mailings.previous" => d.record(false),
        "mailings.next" => d.record(true),
        "mailings.findRecipient" => d.find_recipient(),
        "mailings.checkErrors" => d.check_errors(),
        "mailings.finish" => d.finish(),
        _ => return false,
    }
    true
}

struct Draw<'a> {
    p: &'a Painter,
    r: Rect,
    t: &'a Tokens,
}

impl Draw<'_> {
    fn at(&self, x: f32, y: f32) -> Pos2 {
        pos2(self.r.min.x + x, self.r.min.y + y)
    }
    fn stroke(&self, c: Color32) -> Stroke {
        Stroke::new(1.0, c)
    }
    /// A page outline (white, grey border) with its text area returned.
    fn page(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        let r = Rect::from_min_size(self.at(x, y), vec2(w, h));
        self.p.rect_filled(r, 2.0, Color32::WHITE);
        self.p.rect_stroke(r, 2.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        r.shrink2(vec2(8.0, 8.0))
    }
    fn line(&self, a: Pos2, b: Pos2, c: Color32, w: f32) {
        self.p.add(Shape::line_segment([a, b], Stroke::new(w, c)));
    }
    /// Grey "text" lines of the given widths in `area`, right-aligned when `rtl`, `pitch` apart.
    fn text_lines(&self, area: Rect, rtl: bool, widths: &[f32], pitch: f32, c: Color32) {
        for (i, w) in widths.iter().enumerate() {
            let y = area.min.y + 4.0 + i as f32 * pitch;
            let (x0, x1) = if rtl { (area.max.x - w, area.max.x) } else { (area.min.x, area.min.x + w) };
            self.line(pos2(x0, y), pos2(x1, y), c, 1.5);
        }
    }
    fn label(&self, pos: Pos2, s: &str, size: f32, c: Color32, anchor: Align2) {
        self.p.text(pos, anchor, s, crate::theme::semibold(size), c);
    }
    fn arrow_right(&self, a: Pos2, b: Pos2) {
        let c = self.t.accent;
        self.line(a, b, c, 1.5);
        self.line(b, pos2(b.x - 5.0, b.y - 4.0), c, 1.5);
        self.line(b, pos2(b.x - 5.0, b.y + 4.0), c, 1.5);
    }
    fn arrow_left(&self, a: Pos2, b: Pos2) {
        let c = self.t.accent;
        self.line(a, b, c, 1.5);
        self.line(b, pos2(b.x + 5.0, b.y - 4.0), c, 1.5);
        self.line(b, pos2(b.x + 5.0, b.y + 4.0), c, 1.5);
    }

    // ---- Service tab ----

    /// Typed markers on the left page become the cycle's markers, each level stepped in from
    /// the right, on the right page.
    fn renumbered(&self, selection: bool) {
        let before = self.page(6.0, 8.0, 70.0, 88.0);
        let after = self.page(100.0, 8.0, 70.0, 88.0);
        self.arrow_right(self.at(80.0, 52.0), self.at(96.0, 52.0));
        let grey = self.t.text_dim;
        let rows = ["١.", "أ.", "أولاً.", "ثانياً.", "ب.", "أولاً."];
        for (i, m) in rows.iter().enumerate() {
            let y = before.min.y + 6.0 + i as f32 * 12.0;
            self.label(pos2(before.max.x, y), m, 7.0, grey, Align2::RIGHT_CENTER);
            self.line(pos2(before.min.x, y), pos2(before.max.x - 18.0, y), grey, 1.5);
        }
        let levels = [0.0, 1.0, 2.0, 2.0, 1.0, 2.0];
        let marks = ["١.", "أ.", "أولاً.", "ثانياً.", "ب.", "أولاً."];
        for i in 0..6 {
            let y = after.min.y + 6.0 + i as f32 * 12.0;
            let indent = levels[i] * 8.0;
            let selected = !selection || (2..=3).contains(&i);
            if selection && selected {
                self.p.rect_filled(Rect::from_min_max(pos2(after.min.x - 2.0, y - 5.0), pos2(after.max.x + 2.0, y + 5.0)), 1.0, self.t.selection);
            }
            let c = if selected { self.t.accent } else { grey };
            self.label(pos2(after.max.x - indent, y), marks[i], 7.0, c, Align2::RIGHT_CENTER);
            self.line(pos2(after.min.x, y), pos2(after.max.x - indent - 20.0, y), grey, 1.5);
        }
    }

    /// Word list markers (boxed) that renumber themselves: a new item slides in.
    fn service_list(&self) {
        let page = self.page(50.0, 8.0, 76.0, 88.0);
        let grey = self.t.text_dim;
        let marks = ["١.", "أ.", "ب.", "جـ.", "٢."];
        let levels = [0.0, 1.0, 1.0, 1.0, 0.0];
        for i in 0..5 {
            let y = page.min.y + 6.0 + i as f32 * 14.0;
            let indent = levels[i] * 9.0;
            let x = page.max.x - indent;
            self.p.rect_stroke(
                Rect::from_min_max(pos2(x - 16.0, y - 5.5), pos2(x + 1.0, y + 5.5)),
                1.0,
                self.stroke(self.t.accent),
                egui::StrokeKind::Inside,
            );
            self.label(pos2(x - 1.0, y), marks[i], 7.0, self.t.accent, Align2::RIGHT_CENTER);
            self.line(pos2(page.min.x, y), pos2(x - 20.0, y), grey, 1.5);
        }
        self.label(self.at(88.0, 100.0), "1 2 3 ↻", 7.0, self.t.accent, Align2::CENTER_CENTER);
    }

    fn final_dot(&self) {
        let page = self.page(50.0, 8.0, 76.0, 88.0);
        let grey = self.t.text_dim;
        for i in 0..4 {
            let y = page.min.y + 8.0 + i as f32 * 18.0;
            let w = [44.0, 52.0, 40.0, 48.0][i];
            self.line(pos2(page.max.x - w, y), pos2(page.max.x, y), grey, 1.5);
            self.p.circle_filled(pos2(page.max.x - w - 4.0, y + 1.0), 2.0, self.t.accent);
        }
    }

    fn classification(&self) {
        let page = self.page(50.0, 6.0, 76.0, 92.0);
        self.label(pos2(page.center().x, page.min.y + 2.0), "محدود", 8.0, self.t.accent, Align2::CENTER_CENTER);
        self.text_lines(page.shrink2(vec2(0.0, 14.0)), true, &[56.0, 50.0, 54.0, 36.0], 10.0, self.t.text_dim);
        self.label(pos2(page.center().x, page.max.y - 2.0), "محدود", 8.0, self.t.accent, Align2::CENTER_CENTER);
    }

    fn page_numbers(&self) {
        let page = self.page(50.0, 6.0, 76.0, 92.0);
        self.text_lines(page, true, &[56.0, 50.0, 54.0, 36.0, 48.0], 11.0, self.t.text_dim);
        self.label(pos2(page.center().x, page.max.y - 2.0), "( ٣ - ٧ )", 8.0, self.t.accent, Align2::CENTER_CENTER);
    }

    /// Three right-to-left lines of "words" (grey blocks); `stretch` of the slack goes into the
    /// joints (accent strokes) and the rest into the spaces. The last line is short unless
    /// `last_too` (Distribute).
    fn kashida(&self, stretch: f32, last_too: bool) {
        let page = self.page(28.0, 10.0, 120.0, 84.0);
        let grey = self.t.text_dim;
        let words: [&[f32]; 3] = [&[18.0, 12.0, 22.0, 14.0], &[14.0, 20.0, 10.0, 16.0], &[16.0, 12.0]];
        for (row, ws) in words.iter().enumerate() {
            let y = page.min.y + 10.0 + row as f32 * 22.0;
            let natural: f32 = ws.iter().sum::<f32>() + 4.0 * (ws.len() as f32 - 1.0);
            let full = page.width();
            let last = row == words.len() - 1;
            let slack = if last && !last_too { 0.0 } else { full - natural };
            let n = ws.len() as f32;
            let joint = if n > 0.0 { slack * stretch / n } else { 0.0 };
            let gap = 4.0 + if n > 1.0 { slack * (1.0 - stretch) / (n - 1.0) } else { 0.0 };
            let mut x = page.max.x;
            for w in ws.iter() {
                // The word: a block, with its joint (accent) lengthened by `joint`.
                let x0 = x - w - joint;
                self.p.rect_filled(Rect::from_min_max(pos2(x - w * 0.55, y - 3.0), pos2(x, y + 3.0)), 1.0, grey);
                if joint > 0.5 {
                    self.line(pos2(x - w * 0.55, y + 2.0), pos2(x - w * 0.55 - joint, y + 2.0), self.t.accent, 2.5);
                }
                self.p.rect_filled(Rect::from_min_max(pos2(x0, y - 3.0), pos2(x - w * 0.55 - joint, y + 3.0)), 1.0, grey);
                x = x0 - gap;
            }
        }
    }

    fn engine(&self, pdf: bool) {
        let before = self.page(6.0, 8.0, 56.0, 76.0);
        self.text_lines(before, true, &[38.0, 30.0, 36.0, 20.0], 12.0, self.t.text_dim);
        // The gear.
        let c = self.at(88.0, 44.0);
        self.p.circle_stroke(c, 7.0, Stroke::new(2.0, self.t.accent));
        for i in 0..8 {
            let ang = i as f32 * std::f32::consts::FRAC_PI_4;
            let dir = vec2(ang.cos(), ang.sin());
            self.line(c + dir * 8.5, c + dir * 12.5, self.t.accent, 2.0);
        }
        self.arrow_right(self.at(64.0, 44.0), self.at(74.0, 44.0));
        self.arrow_right(self.at(102.0, 44.0), self.at(112.0, 44.0));
        let after = self.page(114.0, 8.0, 56.0, 76.0);
        // Fully justified lines, joints in accent.
        for i in 0..4 {
            let y = after.min.y + 4.0 + i as f32 * 12.0;
            self.line(pos2(after.min.x, y), pos2(after.max.x, y), self.t.text_dim, 1.5);
            self.line(pos2(after.min.x + 10.0, y), pos2(after.min.x + 18.0, y), self.t.accent, 2.5);
            self.line(pos2(after.max.x - 14.0, y), pos2(after.max.x - 8.0, y), self.t.accent, 2.5);
        }
        if pdf {
            let badge = Rect::from_min_size(self.at(132.0, 78.0), vec2(36.0, 18.0));
            self.p.rect_filled(badge, 3.0, Color32::from_rgb(190, 40, 40));
            self.label(badge.center(), "PDF", 9.0, Color32::WHITE, Align2::CENTER_CENTER);
        }
    }

    // ---- Mailings tab ----

    fn envelope(&self) {
        let r = Rect::from_min_size(self.at(18.0, 18.0), vec2(140.0, 70.0));
        self.p.rect_filled(r, 2.0, Color32::WHITE);
        self.p.rect_stroke(r, 2.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        // Flap.
        self.line(r.min, pos2(r.center().x, r.min.y + 30.0), self.t.border_strong, 1.0);
        self.line(pos2(r.max.x, r.min.y), pos2(r.center().x, r.min.y + 30.0), self.t.border_strong, 1.0);
        // Return address (small, top left) and delivery address (centre).
        self.text_lines(Rect::from_min_size(r.min + vec2(6.0, 2.0), vec2(40.0, 20.0)), false, &[26.0, 20.0], 5.0, self.t.text_dim);
        self.text_lines(Rect::from_min_size(r.min + vec2(58.0, 32.0), vec2(70.0, 30.0)), false, &[44.0, 56.0, 36.0], 8.0, self.t.text);
        // Stamp.
        self.p.rect_stroke(
            Rect::from_min_size(pos2(r.max.x - 22.0, r.min.y + 4.0), vec2(16.0, 18.0)),
            1.0,
            self.stroke(self.t.accent),
            egui::StrokeKind::Inside,
        );
    }

    fn labels(&self) {
        let page = self.page(44.0, 6.0, 88.0, 92.0);
        for row in 0..4 {
            for col in 0..2 {
                let cell = Rect::from_min_size(pos2(page.min.x + col as f32 * 38.0, page.min.y + row as f32 * 20.0), vec2(34.0, 17.0));
                self.p.rect_stroke(cell, 1.0, self.stroke(self.t.border), egui::StrokeKind::Inside);
                self.text_lines(cell.shrink(3.0), false, &[20.0, 26.0], 6.0, if row == 0 && col == 0 { self.t.accent } else { self.t.text_dim });
            }
        }
    }

    /// A list (table) plus one letter with fields → a stack of filled letters.
    fn mail_merge(&self) {
        let table = Rect::from_min_size(self.at(6.0, 10.0), vec2(44.0, 40.0));
        self.table(table, 4, 2);
        let letter = self.page(6.0, 56.0, 44.0, 42.0);
        self.field(pos2(letter.min.x, letter.min.y + 2.0), 18.0);
        self.text_lines(Rect::from_min_size(letter.min + vec2(0.0, 10.0), letter.size()), false, &[28.0, 24.0, 20.0], 6.0, self.t.text_dim);
        self.arrow_right(self.at(58.0, 52.0), self.at(78.0, 52.0));
        for i in 0..3 {
            let off = i as f32 * 14.0;
            let p = self.page(84.0 + off, 10.0 + off, 60.0, 60.0);
            self.label(pos2(p.min.x, p.min.y + 3.0), ["Ali", "Sara", "Omar"][i], 7.0, self.t.accent, Align2::LEFT_CENTER);
            self.text_lines(Rect::from_min_size(p.min + vec2(0.0, 10.0), p.size()), false, &[40.0, 34.0, 28.0], 7.0, self.t.text_dim);
        }
    }

    fn table(&self, r: Rect, rows: usize, cols: usize) {
        self.p.rect_filled(r, 1.0, Color32::WHITE);
        self.p.rect_stroke(r, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        let rh = r.height() / rows as f32;
        let cw = r.width() / cols as f32;
        self.p.rect_filled(Rect::from_min_size(r.min, vec2(r.width(), rh)), 0.0, self.t.hover);
        for i in 1..rows {
            let y = r.min.y + i as f32 * rh;
            self.line(pos2(r.min.x, y), pos2(r.max.x, y), self.t.border, 1.0);
        }
        for j in 1..cols {
            let x = r.min.x + j as f32 * cw;
            self.line(pos2(x, r.min.y), pos2(x, r.max.y), self.t.border, 1.0);
        }
        for i in 0..rows {
            for j in 0..cols {
                let y = r.min.y + (i as f32 + 0.5) * rh;
                let x = r.min.x + j as f32 * cw + 3.0;
                self.line(pos2(x, y), pos2(x + cw * 0.6, y), if i == 0 { self.t.text } else { self.t.text_dim }, 1.5);
            }
        }
    }

    /// A «field» chip.
    fn field(&self, at: Pos2, w: f32) {
        let r = Rect::from_min_size(at, vec2(w, 7.0));
        self.p.rect_filled(r, 1.0, self.t.accent.linear_multiply(0.25));
        self.p.rect_stroke(r, 1.0, self.stroke(self.t.accent), egui::StrokeKind::Inside);
    }

    fn recipients(&self, edit: bool) {
        let table = Rect::from_min_size(self.at(24.0, 10.0), vec2(128.0, 84.0));
        self.table(table, 6, 3);
        // Tick column.
        let rh = table.height() / 6.0;
        for i in 1..6 {
            let y = table.min.y + (i as f32 + 0.5) * rh;
            let b = Rect::from_center_size(pos2(table.min.x + 8.0, y), vec2(7.0, 7.0));
            self.p.rect_stroke(b, 1.0, self.stroke(self.t.text_dim), egui::StrokeKind::Inside);
            if i != 4 {
                self.line(pos2(b.min.x + 1.5, b.center().y), pos2(b.center().x, b.max.y - 1.5), self.t.accent, 1.5);
                self.line(pos2(b.center().x, b.max.y - 1.5), pos2(b.max.x - 1.0, b.min.y + 1.5), self.t.accent, 1.5);
            }
        }
        if edit {
            // A pencil over the table.
            let a = self.at(136.0, 84.0);
            let b = self.at(156.0, 64.0);
            self.line(a, b, self.t.accent, 4.0);
            self.line(a, a + vec2(-3.0, 3.0), self.t.accent, 2.0);
        }
    }

    fn letter_fields(&self, highlight: bool, caret: bool) {
        let page = self.page(44.0, 6.0, 88.0, 92.0);
        self.text_lines(Rect::from_min_size(page.min, page.size()), false, &[40.0, 48.0], 7.0, self.t.text_dim);
        let fields =
            [(page.min.x, page.min.y + 18.0, 28.0), (page.min.x + 36.0, page.min.y + 18.0, 22.0), (page.min.x + 14.0, page.min.y + 36.0, 30.0)];
        for (x, y, w) in fields {
            if highlight {
                self.p.rect_filled(Rect::from_min_size(pos2(x, y - 1.0), vec2(w, 9.0)), 1.0, self.t.find_highlight);
            }
            self.field(pos2(x, y), w);
        }
        self.text_lines(Rect::from_min_size(pos2(page.min.x, page.min.y + 46.0), page.size()), false, &[60.0, 54.0, 30.0], 8.0, self.t.text_dim);
        if caret {
            let x = page.min.x + 50.0;
            self.line(pos2(x, page.min.y + 33.0), pos2(x, page.min.y + 46.0), self.t.text, 1.5);
            self.field(pos2(x + 2.0, page.min.y + 36.0), 26.0);
            self.label(pos2(x + 15.0, page.min.y + 39.5), "«…»", 6.0, self.t.accent, Align2::CENTER_CENTER);
        }
    }

    fn address_block(&self) {
        let page = self.page(44.0, 6.0, 88.0, 92.0);
        let block = Rect::from_min_size(page.min + vec2(0.0, 2.0), vec2(48.0, 30.0));
        self.p.rect_stroke(block.expand(3.0), 2.0, self.stroke(self.t.accent), egui::StrokeKind::Outside);
        self.text_lines(block, false, &[30.0, 44.0, 36.0], 9.0, self.t.text);
        self.text_lines(Rect::from_min_size(pos2(page.min.x, page.min.y + 44.0), page.size()), false, &[60.0, 56.0, 40.0], 8.0, self.t.text_dim);
    }

    fn greeting_line(&self) {
        let page = self.page(44.0, 6.0, 88.0, 92.0);
        self.text_lines(page, false, &[30.0, 44.0], 7.0, self.t.text_dim);
        let y = page.min.y + 24.0;
        self.label(pos2(page.min.x, y), "Dear", 7.5, self.t.text, Align2::LEFT_CENTER);
        self.field(pos2(page.min.x + 24.0, y - 3.5), 30.0);
        self.label(pos2(page.min.x + 56.0, y), ",", 7.5, self.t.text, Align2::LEFT_CENTER);
        self.p.rect_stroke(
            Rect::from_min_size(pos2(page.min.x - 3.0, y - 7.0), vec2(66.0, 14.0)),
            2.0,
            self.stroke(self.t.accent),
            egui::StrokeKind::Outside,
        );
        self.text_lines(
            Rect::from_min_size(pos2(page.min.x, page.min.y + 36.0), page.size()),
            false,
            &[64.0, 58.0, 50.0, 30.0],
            8.0,
            self.t.text_dim,
        );
    }

    fn rules(&self) {
        // If «field» = x then A else B.
        let top = self.at(88.0, 14.0);
        self.field(top - vec2(14.0, 3.5), 28.0);
        self.label(top + vec2(0.0, 12.0), "= ?", 8.0, self.t.text, Align2::CENTER_CENTER);
        let a = self.at(44.0, 60.0);
        let b = self.at(132.0, 60.0);
        self.line(top + vec2(0.0, 18.0), a - vec2(0.0, 12.0), self.t.accent, 1.5);
        self.line(top + vec2(0.0, 18.0), b - vec2(0.0, 12.0), self.t.accent, 1.5);
        self.label(self.at(58.0, 38.0), "yes", 7.0, self.t.text_dim, Align2::CENTER_CENTER);
        self.label(self.at(118.0, 38.0), "no", 7.0, self.t.text_dim, Align2::CENTER_CENTER);
        for (c, text) in [(a, "text A"), (b, "text B")] {
            let r = Rect::from_center_size(c, vec2(48.0, 20.0));
            self.p.rect_filled(r, 2.0, Color32::WHITE);
            self.p.rect_stroke(r, 2.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
            self.label(c, text, 7.5, self.t.text, Align2::CENTER_CENTER);
        }
        self.text_lines(Rect::from_min_size(self.at(20.0, 80.0), vec2(136.0, 20.0)), false, &[136.0, 100.0], 8.0, self.t.text_dim);
    }

    fn match_fields(&self) {
        let left = ["Name", "Rank", "Unit"];
        let right = ["الاسم", "الرتبة", "الوحدة"];
        for i in 0..3 {
            let y = 22.0 + i as f32 * 26.0;
            let l = Rect::from_min_size(self.at(10.0, y - 7.0), vec2(50.0, 14.0));
            let r = Rect::from_min_size(self.at(116.0, y - 7.0), vec2(50.0, 14.0));
            for (rect, text) in [(l, left[i]), (r, right[i])] {
                self.p.rect_filled(rect, 2.0, Color32::WHITE);
                self.p.rect_stroke(rect, 2.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
                self.label(rect.center(), text, 7.5, self.t.text, Align2::CENTER_CENTER);
            }
            self.line(l.right_center(), r.left_center(), self.t.accent, 1.5);
        }
    }

    fn preview(&self) {
        let before = self.page(6.0, 10.0, 64.0, 80.0);
        self.field(pos2(before.min.x, before.min.y + 2.0), 30.0);
        self.text_lines(Rect::from_min_size(before.min + vec2(0.0, 14.0), before.size()), false, &[44.0, 40.0, 28.0], 9.0, self.t.text_dim);
        self.arrow_right(self.at(76.0, 50.0), self.at(100.0, 50.0));
        let after = self.page(106.0, 10.0, 64.0, 80.0);
        self.label(pos2(after.min.x, after.min.y + 5.0), "Ali Hasan", 7.5, self.t.accent, Align2::LEFT_CENTER);
        self.text_lines(Rect::from_min_size(after.min + vec2(0.0, 14.0), after.size()), false, &[44.0, 40.0, 28.0], 9.0, self.t.text_dim);
    }

    fn record(&self, next: bool) {
        for i in 0..3 {
            let off = i as f32 * 10.0;
            let p = self.page(50.0 + off, 8.0 + off, 60.0, 64.0);
            if i == 2 {
                self.label(pos2(p.min.x, p.min.y + 5.0), if next { "3 / 5" } else { "1 / 5" }, 8.0, self.t.accent, Align2::LEFT_CENTER);
                self.text_lines(Rect::from_min_size(p.min + vec2(0.0, 14.0), p.size()), false, &[40.0, 36.0, 24.0], 9.0, self.t.text_dim);
            }
        }
        if next {
            self.arrow_right(self.at(130.0, 92.0), self.at(160.0, 92.0));
        } else {
            self.arrow_left(self.at(46.0, 92.0), self.at(16.0, 92.0));
        }
    }

    fn find_recipient(&self) {
        let table = Rect::from_min_size(self.at(16.0, 10.0), vec2(110.0, 84.0));
        self.table(table, 6, 2);
        let rh = table.height() / 6.0;
        let y = table.min.y + 3.0 * rh;
        self.p.rect_filled(Rect::from_min_size(pos2(table.min.x, y), vec2(table.width(), rh)), 0.0, self.t.find_highlight);
        let c = self.at(146.0, 40.0);
        self.p.circle_stroke(c, 10.0, Stroke::new(2.5, self.t.accent));
        self.line(c + vec2(7.0, 7.0), c + vec2(16.0, 16.0), self.t.accent, 3.0);
    }

    fn check_errors(&self) {
        let page = self.page(50.0, 6.0, 76.0, 92.0);
        self.text_lines(page, false, &[50.0, 56.0, 40.0, 54.0, 30.0], 11.0, self.t.text_dim);
        self.field(pos2(page.min.x + 20.0, page.min.y + 34.0), 24.0);
        // A warning triangle on the empty field, a tick for the page.
        let w = pos2(page.min.x + 44.0, page.min.y + 30.0);
        self.p.add(Shape::convex_polygon(
            vec![w + vec2(0.0, -8.0), w + vec2(8.0, 6.0), w + vec2(-8.0, 6.0)],
            Color32::from_rgb(220, 150, 20),
            Stroke::NONE,
        ));
        self.label(w + vec2(0.0, 1.0), "!", 8.0, Color32::WHITE, Align2::CENTER_CENTER);
        let c = self.at(146.0, 84.0);
        self.p.circle_filled(c, 11.0, self.t.accent);
        self.line(c + vec2(-5.0, 0.0), c + vec2(-1.5, 4.0), Color32::WHITE, 2.5);
        self.line(c + vec2(-1.5, 4.0), c + vec2(6.0, -5.0), Color32::WHITE, 2.5);
    }

    fn finish(&self) {
        for i in 0..4 {
            let off = i as f32 * 12.0;
            let p = self.page(14.0 + off, 6.0 + off, 56.0, 56.0);
            if i == 3 {
                self.label(pos2(p.min.x, p.min.y + 4.0), "Omar", 7.0, self.t.accent, Align2::LEFT_CENTER);
                self.text_lines(Rect::from_min_size(p.min + vec2(0.0, 12.0), p.size()), false, &[36.0, 30.0, 20.0], 8.0, self.t.text_dim);
            }
        }
        self.arrow_right(self.at(112.0, 52.0), self.at(130.0, 52.0));
        // A printer.
        let body = Rect::from_min_size(self.at(134.0, 44.0), vec2(34.0, 18.0));
        self.p.rect_filled(body, 2.0, self.t.hover);
        self.p.rect_stroke(body, 2.0, self.stroke(self.t.text), egui::StrokeKind::Inside);
        self.p.rect_stroke(Rect::from_min_size(self.at(141.0, 34.0), vec2(20.0, 10.0)), 1.0, self.stroke(self.t.text), egui::StrokeKind::Inside);
        self.p.rect_filled(Rect::from_min_size(self.at(141.0, 62.0), vec2(20.0, 12.0)), 1.0, Color32::WHITE);
        self.p.rect_stroke(Rect::from_min_size(self.at(141.0, 62.0), vec2(20.0, 12.0)), 1.0, self.stroke(self.t.text), egui::StrokeKind::Inside);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_explained_command_has_a_thumbnail_and_is_registered() {
        let reg = wordcraft_engine::cmd::registry();
        let ctx = egui::Context::default();
        ctx.set_fonts(crate::theme::font_definitions(false, false));
        let mut out = ctx.run_ui(Default::default(), |ui| {
            let t = Tokens::get(ui.ctx());
            for (key, text) in EXPLAINED {
                let id = key.split(':').next().unwrap_or("");
                assert!(reg.get(id).is_some(), "{key}: unknown command");
                assert!(!text.trim().is_empty() && text.ends_with('.'), "{key}: one full sentence");
                let r = Rect::from_min_size(Pos2::ZERO, SIZE);
                assert!(thumbnail(ui.painter(), r, key, &t), "{key}: no thumbnail");
                assert_eq!(explanation(key), Some(*text));
            }
            assert!(!thumbnail(ui.painter(), Rect::from_min_size(Pos2::ZERO, SIZE), "no.such", &Tokens::get(ui.ctx())));
        });
        out.textures_delta.clear();
    }

    #[test]
    fn keys_pick_the_variant() {
        assert_eq!(key("service.kashida", &serde_json::json!({"value": "low"})), "service.kashida:low");
        assert_eq!(key("service.renumber", &serde_json::json!({"scope": "selection"})), "service.renumber:selection");
        assert_eq!(key("service.renumber", &serde_json::json!({})), "service.renumber");
        assert_eq!(key("service.external", &serde_json::json!({"pdf": true})), "service.external:pdf");
        assert_eq!(key("service.external", &serde_json::json!({"pdf": false})), "service.external");
        assert_eq!(key("mailings.next", &serde_json::json!({})), "mailings.next");
    }

    /// "Check every button": every control of every ribbon tab runs on an Arabic service
    /// document without a panic (an error is a valid answer: a dialog needs input, a feature
    /// needs a selection), the document still lays out, and the Service buttons do their job.
    #[test]
    fn every_ribbon_button_runs_on_an_arabic_document() {
        let mut app = crate::WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        app.run("file.new", serde_json::json!({})).unwrap();
        for line in [
            "١. شعبة الأسنان",
            "أ. المهمة",
            "تقوم شعبة الاسنان بتقديم الخدمات الطبية لمنتسبي وزارة الدفاع.",
            "ب. الواجبات",
            "أولاً. فحص ومعالجة مرضى الفم والأسنان",
            "ثانياً. الإشراف على إدامة الأجهزة",
        ] {
            app.run("text.insert", serde_json::json!({"text": line})).unwrap();
            app.run("text.newParagraph", serde_json::json!({})).unwrap();
        }
        let mut ran = 0;
        for tab in crate::ribbon::TABS.iter().chain(&["Table Design", "Table Layout"]) {
            for (id, params) in crate::keytips::controls_public_with_params(tab) {
                if id == "ui.dialog" || id == "ui.tab" || id.starts_with("file.") || id == "ui.zotero.addEditCitation" {
                    continue;
                }
                let _ = app.run(id, params);
                app.session.clamp_selection();
                let _ = app.session.layout();
                ran += 1;
            }
        }
        assert!(ran > 150, "{ran} controls ran");
        // The Service tab's work is in the document: the cycle, a dot on every item, the stamp.
        app.run("select.all", serde_json::json!({})).unwrap();
        app.run("service.renumber", serde_json::json!({})).unwrap();
        app.run("service.finalDot", serde_json::json!({})).unwrap();
        app.run("service.classification", serde_json::json!({"value": "محدود"})).unwrap();
        app.run("service.pageNumbers", serde_json::json!({})).unwrap();
        let text = app.session.doc.plain_text(wordcraft_doc::StoryRef::Body);
        assert!(text.contains("أولاً.") && text.contains("فحص ومعالجة مرضى الفم والأسنان."), "{text}");
        assert!(!app.session.layout().pages.is_empty());
    }

    /// Every Service and Mailings ribbon control has visual help.
    #[test]
    fn service_and_mailings_buttons_are_all_explained() {
        for tab in ["Service", "Mailings"] {
            for c in crate::keytips::controls_public_with_params(tab) {
                if c.0 == "ui.dialog" || c.0 == "ui.tab" {
                    continue;
                }
                let k = key(c.0, &c.1);
                assert!(explanation(&k).is_some(), "{tab} › {}: no visual help ({k})", c.0);
            }
        }
    }
}
