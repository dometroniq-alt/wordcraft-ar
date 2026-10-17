//! Service tab thumbnails (كتابات الخدمة): the numbering cycle, the classification stamp, page
//! numbers, kashida levels and the ServiceWord-AR engine. Hand-drawn scenes on the shared kit.

use egui::{Align2, Color32, Rect, Stroke, pos2, vec2};

use super::Entry;
use super::draw::Draw;

pub const ENTRIES: &[Entry] = &[
    Entry::new(
        "service.renumber",
        "Reads the markers you typed (١. أ. أولاً. (١)) and renumbers every item by the service cycle, level by level, with hanging indents.",
        |d| d.renumbered(false),
    ),
    Entry::new(
        "service.renumber:selection",
        "Renumbers only the selected paragraphs by the service cycle; the rest of the document is untouched.",
        |d| d.renumbered(true),
    ),
    Entry::new("service.applyList", "Turns the typed markers into a real Word multilevel list that keeps numbering itself as you add items.", |d| {
        d.service_list()
    }),
    Entry::new("service.finalDot", "Puts a full stop at the end of every item that lacks one.", |d| d.final_dot()),
    Entry::new("service.classification", "Stamps the classification word (محدود، سري…) in the header and footer of every page.", |d| {
        d.classification()
    }),
    Entry::new("service.pageNumbers", "Writes ( page - total ) in Arabic-Indic digits at the bottom of every page.", |d| d.page_numbers()),
    Entry::new("service.kashida:off", "Justifies by widening the spaces only; the last line of a paragraph stays short.", |d| d.kashida(0.0, false)),
    Entry::new("service.kashida:low", "Justifies by stretching one letter joint per word a little, then the spaces.", |d| d.kashida(0.35, false)),
    Entry::new("service.kashida:medium", "Justifies by stretching the letter joints more, before the spaces.", |d| d.kashida(0.6, false)),
    Entry::new("service.kashida:high", "Justifies mostly by stretching the letter joints; spaces barely widen.", |d| d.kashida(0.9, false)),
    Entry::new("service.kashida:distribute", "Stretches every line to both margins, the last line of each paragraph too.", |d| d.kashida(0.6, true)),
    Entry::new(
        "service.external",
        "Sends the document to the ServiceWord-AR engine, which measures the real fonts and writes the kashida into the text, and opens the result.",
        |d| d.engine(false),
    ),
    Entry::new("service.external:pdf", "Formats with ServiceWord-AR and also makes the stamped print PDF beside the document.", |d| d.engine(true)),
];

impl Draw<'_> {
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
}
