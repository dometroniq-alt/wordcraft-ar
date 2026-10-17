//! Mailings tab thumbnails: envelopes, labels and the mail-merge steps.

use egui::{Align2, Color32, Rect, Shape, Stroke, pos2, vec2};

use super::Entry;
use super::draw::Draw;

pub const ENTRIES: &[Entry] = &[
    Entry::new("mailings.envelopes", "Makes an envelope page with the delivery and return addresses, ready to print.", |d| d.envelope()),
    Entry::new("mailings.labels", "Makes a sheet of address labels, the same address on each or one per recipient.", |d| d.labels()),
    Entry::new("mailings.start", "Starts a mail merge: one letter, envelope or label design that is filled in for every person in a list.", |d| {
        d.mail_merge()
    }),
    Entry::new("mailings.recipients", "Picks the list of people to merge: a table you type, or a spreadsheet or CSV file.", |d| d.recipients(false)),
    Entry::new("mailings.editRecipients", "Shows the recipient list so you can tick who is included, sort it or filter it.", |d| d.recipients(true)),
    Entry::new("mailings.highlightFields", "Shades every merge field in the document so you can see where the names and addresses will go.", |d| {
        d.letter_fields(true, false)
    }),
    Entry::new("mailings.addressBlock", "Inserts a complete address (name, street, city) as one field, laid out like on an envelope.", |d| {
        d.address_block()
    }),
    Entry::new("mailings.greetingLine", "Inserts the opening line (Dear …,) as a field that takes each recipient's name.", |d| d.greeting_line()),
    Entry::new("mailings.insertField", "Inserts one column of the list (first name, rank, unit…) at the caret as a «field».", |d| {
        d.letter_fields(false, true)
    }),
    Entry::new("mailings.rules", "Adds conditions: if a field has a certain value, write one text, otherwise another.", |d| d.rules()),
    Entry::new("mailings.matchFields", "Pairs the columns of your list with the parts of an address or greeting when the names differ.", |d| {
        d.match_fields()
    }),
    Entry::new("mailings.preview", "Replaces the «fields» with one recipient's real data so you see the finished letter.", |d| d.preview()),
    Entry::new("mailings.previous", "Shows the previous recipient's letter in the preview.", |d| d.record(false)),
    Entry::new("mailings.next", "Shows the next recipient's letter in the preview.", |d| d.record(true)),
    Entry::new("mailings.findRecipient", "Finds a recipient by name or any value and previews that letter.", |d| d.find_recipient()),
    Entry::new("mailings.checkErrors", "Checks every merged letter for missing fields or data before you print.", |d| d.check_errors()),
    Entry::new("mailings.finish", "Produces the merged letters: one document with every letter, or straight to the printer.", |d| d.finish()),
];

impl Draw<'_> {
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
