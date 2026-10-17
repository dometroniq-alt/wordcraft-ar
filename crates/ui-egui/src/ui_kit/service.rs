//! Service tab thumbnails (كتابات الخدمة): the numbering cycle, the classification stamp, page
//! numbers, kashida levels and the ServiceWord-AR engine. Hand-drawn scenes on the shared kit,
//! each a little film: the typed markers turn into the cycle one line at a time, the dots land
//! one by one, the stamp comes down, the joints stretch, the gear turns.

use egui::{Align2, Color32, Rect, Stroke, pos2, vec2};

use super::Entry;
use super::draw::Draw;

pub const ENTRIES: &[Entry] = &[
    Entry::new(
        "service.renumber",
        "Renumbers every item by the service cycle (١. then أ. then أولاً. then (١)…), level by level, with hanging indents. Example: «1- المهمة» and «a- الواجبات» typed any way become «١.» and «أ.», each level stepped in.",
        |d| d.renumbered(false),
    ),
    Entry::new(
        "service.renumber:selection",
        "Renumbers only the paragraphs you selected by the service cycle; the rest of the document stays as it is. Example: select one section and fix its numbering alone.",
        |d| d.renumbered(true),
    ),
    Entry::new(
        "service.applyList",
        "Turns the typed markers into a real Word multilevel list that numbers itself: add or delete an item and the rest renumber. Example: insert a new «ب.» and «جـ.» becomes «د.» on its own.",
        |d| d.service_list(),
    ),
    Entry::new(
        "service.finalDot",
        "Puts a full stop at the end of every item that lacks one, as service writing requires. Example: «فحص المرضى» becomes «فحص المرضى.».",
        |d| d.final_dot(),
    ),
    Entry::new(
        "service.classification",
        "Stamps the classification word in the header and footer of every page, centred. Example: pick «محدود» and every page is marked محدود top and bottom.",
        |d| d.classification(),
    ),
    Entry::new(
        "service.pageNumbers",
        "Writes ( page - total ) in Arabic-Indic digits at the bottom of every page, and it updates itself. Example: the third of seven pages reads ( ٣ - ٧ ).",
        |d| d.page_numbers(),
    ),
    Entry::new(
        "service.kashida:off",
        "Justifies by widening the spaces only; letters keep their shape and the last line of a paragraph stays short.",
        |d| d.kashida(0.0, false),
    ),
    Entry::new(
        "service.kashida:low",
        "Justifies by stretching one letter joint per word a little (ـ), then widening the spaces. Example: «فحص» becomes «فحـص».",
        |d| d.kashida(0.35, false),
    ),
    Entry::new(
        "service.kashida:medium",
        "Justifies by stretching the letter joints more before the spaces widen: the usual look of a typed service letter.",
        |d| d.kashida(0.6, false),
    ),
    Entry::new(
        "service.kashida:high",
        "Justifies mostly by stretching the letter joints; the spaces barely widen. Example: «فحص» becomes «فحــــص».",
        |d| d.kashida(0.9, false),
    ),
    Entry::new(
        "service.kashida:distribute",
        "Stretches every line to both margins, the last line of each paragraph too, like a signature block or a title.",
        |d| d.kashida(0.6, true),
    ),
    Entry::new(
        "service.external",
        "Sends the document to the ServiceWord-AR engine, which measures the real fonts and writes the kashida into the text itself, then opens the result beside the original.",
        |d| d.engine(false),
    ),
    Entry::new(
        "service.external:pdf",
        "Formats with ServiceWord-AR and also makes the stamped print PDF beside the document, ready for the printer.",
        |d| d.engine(true),
    ),
];

impl Draw<'_> {
    /// Typed markers on the left page become the cycle's markers, each level stepped in from
    /// the right, on the right page — one line at a time.
    fn renumbered(&self, selection: bool) {
        let before = self.page(6.0, 8.0, 70.0, 88.0);
        let after = self.page(100.0, 8.0, 70.0, 88.0);
        let arrow = self.reveal(0.0, 0.2);
        if arrow > 0.0 {
            self.arrow_right(self.at(80.0, 52.0), self.at(80.0 + 16.0 * arrow, 52.0));
        }
        let grey = self.t.text_dim;
        // Typed any way: dashes, Latin digits, no indents.
        let typed = ["1-", "a-", "اولا-", "ثانيا-", "b-", "اولا-"];
        for (i, m) in typed.iter().enumerate() {
            let y = before.min.y + 6.0 + i as f32 * 12.0;
            self.label(pos2(before.max.x, y), m, 7.0, grey, Align2::RIGHT_CENTER);
            self.line(pos2(before.min.x, y), pos2(before.max.x - 22.0, y), grey, 1.5);
        }
        let levels = [0.0, 1.0, 2.0, 2.0, 1.0, 2.0];
        let marks = ["١.", "أ.", "أولاً.", "ثانياً.", "ب.", "أولاً."];
        for i in 0..6 {
            let y = after.min.y + 6.0 + i as f32 * 12.0;
            let selected = !selection || (2..=3).contains(&i);
            // Each line lands after the previous one; the untouched lines are there from the start.
            let land = if selected { self.reveal(0.2 + i as f32 * 0.1, 0.3 + i as f32 * 0.1) } else { 1.0 };
            if selection && selected {
                self.p.rect_filled(Rect::from_min_max(pos2(after.min.x - 2.0, y - 5.0), pos2(after.max.x + 2.0, y + 5.0)), 1.0, self.t.selection);
            }
            if land <= 0.0 {
                // Still typed.
                self.label(pos2(after.max.x, y), typed[i], 7.0, grey, Align2::RIGHT_CENTER);
                self.line(pos2(after.min.x, y), pos2(after.max.x - 22.0, y), grey, 1.5);
                continue;
            }
            let indent = levels[i] * 8.0 * land;
            let c = if selected { self.t.accent } else { grey };
            self.label(pos2(after.max.x - indent, y), marks[i], 7.0, c, Align2::RIGHT_CENTER);
            self.line(pos2(after.min.x, y), pos2(after.max.x - indent - 20.0, y), grey, 1.5);
        }
    }

    /// Word list markers (boxed) that renumber themselves: a new item slides in and the ones
    /// below it change their letters.
    fn service_list(&self) {
        let page = self.page(50.0, 8.0, 76.0, 88.0);
        let grey = self.t.text_dim;
        // The new item (index 2) slides in at mid-film; the two below it renumber.
        let slide = self.reveal(0.35, 0.6);
        let marks_before = ["١.", "أ.", "", "ب.", "جـ."];
        let marks_after = ["١.", "أ.", "ب.", "جـ.", "د."];
        let levels = [0.0, 1.0, 1.0, 1.0, 1.0];
        let mut row = 0.0;
        for i in 0..5 {
            let is_new = i == 2;
            if is_new && slide <= 0.0 {
                continue;
            }
            let y = page.min.y + 6.0 + row * 14.0;
            row += if is_new { slide } else { 1.0 };
            let indent = levels[i] * 9.0;
            let x = page.max.x - indent;
            let mark = if slide >= 1.0 || i < 2 {
                marks_after[i]
            } else if is_new {
                ""
            } else {
                marks_before[i]
            };
            let c = if is_new || (slide > 0.0 && i > 2) { self.t.accent } else { grey };
            let alpha = if is_new { slide } else { 1.0 };
            self.p.rect_stroke(
                Rect::from_min_max(pos2(x - 16.0, y - 5.5), pos2(x + 1.0, y + 5.5)),
                1.0,
                Stroke::new(1.0, c.linear_multiply(alpha)),
                egui::StrokeKind::Inside,
            );
            if !mark.is_empty() {
                self.label(pos2(x - 1.0, y), mark, 7.0, c.linear_multiply(alpha), Align2::RIGHT_CENTER);
            }
            self.line(pos2(page.min.x, y), pos2(x - 20.0, y), grey.linear_multiply(alpha), 1.5);
        }
        if slide >= 1.0 {
            self.label(self.at(88.0, 100.0), "↻", 9.0, self.t.accent, Align2::CENTER_CENTER);
        }
    }

    /// A dot lands at the end of each item, one after the other.
    fn final_dot(&self) {
        let page = self.page(50.0, 8.0, 76.0, 88.0);
        let grey = self.t.text_dim;
        for i in 0..4 {
            let y = page.min.y + 8.0 + i as f32 * 18.0;
            let w = [44.0, 52.0, 40.0, 48.0][i];
            self.line(pos2(page.max.x - w, y), pos2(page.max.x, y), grey, 1.5);
            let land = self.reveal(0.15 + i as f32 * 0.15, 0.3 + i as f32 * 0.15);
            if land > 0.0 {
                // Drops in from above and settles.
                let drop = (1.0 - land) * 10.0;
                self.p.circle_filled(
                    pos2(page.max.x - w - 4.0, y + 1.0 - drop),
                    2.0 + (1.0 - land) * 2.0,
                    self.t.accent.linear_multiply(land.max(0.3)),
                );
            }
        }
    }

    /// The stamp comes down on the header, then on the footer.
    fn classification(&self) {
        let page = self.page(50.0, 6.0, 76.0, 92.0);
        self.text_lines(page.shrink2(vec2(0.0, 14.0)), true, &[56.0, 50.0, 54.0, 36.0], 10.0, self.t.text_dim);
        for (i, y) in [page.min.y + 2.0, page.max.y - 2.0].into_iter().enumerate() {
            let land = self.reveal(0.1 + i as f32 * 0.35, 0.4 + i as f32 * 0.35);
            if land <= 0.0 {
                continue;
            }
            // Big and faint while it comes down, small and firm once stamped.
            let size = 8.0 + (1.0 - land) * 10.0;
            self.label(pos2(page.center().x, y), "محدود", size, self.t.accent.linear_multiply(0.3 + 0.7 * land), Align2::CENTER_CENTER);
            if land >= 1.0 {
                let r = Rect::from_center_size(pos2(page.center().x, y), vec2(30.0, 11.0));
                self.p.rect_stroke(r, 2.0, Stroke::new(1.0, self.t.accent), egui::StrokeKind::Outside);
            }
        }
    }

    /// The page number counts up to the page's own number.
    fn page_numbers(&self) {
        let page = self.page(50.0, 6.0, 76.0, 92.0);
        self.text_lines(page, true, &[56.0, 50.0, 54.0, 36.0, 48.0], 11.0, self.t.text_dim);
        let count = self.reveal(0.1, 0.7);
        let n = ["( ١ - ٧ )", "( ٢ - ٧ )", "( ٣ - ٧ )"];
        let i = ((count * 2.999) as usize).min(2);
        if count > 0.0 {
            self.label(pos2(page.center().x, page.max.y - 2.0), n[i], 8.0, self.t.accent, Align2::CENTER_CENTER);
        }
    }

    /// Three right-to-left lines of "words" (grey blocks); `stretch` of the slack goes into the
    /// joints (accent strokes) and the rest into the spaces. The last line is short unless
    /// `last_too` (Distribute). In motion the lines start ragged and stretch out to the margin.
    fn kashida(&self, stretch: f32, last_too: bool) {
        let page = self.page(28.0, 10.0, 120.0, 84.0);
        let grey = self.t.text_dim;
        let grow = self.reveal(0.15, 0.7);
        let words: [&[f32]; 3] = [&[18.0, 12.0, 22.0, 14.0], &[14.0, 20.0, 10.0, 16.0], &[16.0, 12.0]];
        for (row, ws) in words.iter().enumerate() {
            let y = page.min.y + 10.0 + row as f32 * 22.0;
            let natural: f32 = ws.iter().sum::<f32>() + 4.0 * (ws.len() as f32 - 1.0);
            let full = page.width();
            let last = row == words.len() - 1;
            let slack = if last && !last_too { 0.0 } else { (full - natural) * grow };
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
        // The margin the lines reach for.
        self.line(pos2(page.min.x, page.min.y), pos2(page.min.x, page.max.y), self.t.accent.linear_multiply(0.4), 1.0);
    }

    /// The page goes to the gear (the engine), which turns, and comes back justified.
    fn engine(&self, pdf: bool) {
        let before = self.page(6.0, 8.0, 56.0, 76.0);
        self.text_lines(before, true, &[38.0, 30.0, 36.0, 20.0], 12.0, self.t.text_dim);
        // The gear turns while the film plays.
        let c = self.at(88.0, 44.0);
        let turn = if self.phase >= 1.0 { 0.0 } else { self.phase * std::f32::consts::TAU };
        self.p.circle_stroke(c, 7.0, Stroke::new(2.0, self.t.accent));
        for i in 0..8 {
            let ang = i as f32 * std::f32::consts::FRAC_PI_4 + turn;
            let dir = vec2(ang.cos(), ang.sin());
            self.line(c + dir * 8.5, c + dir * 12.5, self.t.accent, 2.0);
        }
        let go = self.reveal(0.0, 0.25);
        if go > 0.0 {
            self.arrow_right(self.at(64.0, 44.0), self.at(64.0 + 10.0 * go, 44.0));
        }
        let back = self.reveal(0.45, 0.65);
        if back > 0.0 {
            self.arrow_right(self.at(102.0, 44.0), self.at(102.0 + 10.0 * back, 44.0));
        }
        let after = self.page(114.0, 8.0, 56.0, 76.0);
        // Fully justified lines, joints in accent, filling in one by one.
        for i in 0..4 {
            let fill = self.reveal(0.6 + i as f32 * 0.08, 0.7 + i as f32 * 0.08);
            let y = after.min.y + 4.0 + i as f32 * 12.0;
            if fill <= 0.0 {
                self.line(pos2(after.max.x - 30.0, y), pos2(after.max.x, y), self.t.text_dim, 1.5);
                continue;
            }
            let x0 = after.max.x - 30.0 - (after.width() - 30.0) * fill;
            self.line(pos2(x0, y), pos2(after.max.x, y), self.t.text_dim, 1.5);
            if fill >= 1.0 {
                self.line(pos2(after.min.x + 10.0, y), pos2(after.min.x + 18.0, y), self.t.accent, 2.5);
                self.line(pos2(after.max.x - 14.0, y), pos2(after.max.x - 8.0, y), self.t.accent, 2.5);
            }
        }
        if pdf {
            let pop = self.reveal(0.85, 1.0);
            if pop > 0.0 {
                let badge = Rect::from_center_size(self.at(150.0, 87.0), vec2(36.0, 18.0) * (0.6 + 0.4 * pop));
                self.p.rect_filled(badge, 3.0, Color32::from_rgb(190, 40, 40));
                self.label(badge.center(), "PDF", 9.0, Color32::WHITE, Align2::CENTER_CENTER);
            }
        }
    }
}
