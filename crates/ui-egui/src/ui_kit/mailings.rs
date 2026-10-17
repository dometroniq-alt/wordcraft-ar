//! Mailings tab thumbnails: envelopes, labels and the mail-merge steps. Each one a little film:
//! the addresses are typed onto the envelope, the letters pile up one per recipient, the ticks
//! land in the list, the «field» becomes a real name, the finished letters go to the printer.

use egui::{Align2, Color32, Rect, Stroke, pos2, vec2};

use super::Entry;
use super::draw::Draw;

pub const ENTRIES: &[Entry] = &[
    Entry::new(
        "mailings.envelopes",
        "Makes an envelope page with the address of the person you write to in the middle and your own address in the corner, ready to print. Example: «إلى: آمر الوحدة الطبية» in the middle, your unit top left.",
        |d| d.ml_envelope(),
    ),
    Entry::new(
        "mailings.labels",
        "Makes a page of sticker labels: the same address on every label, or one person per label from a list. Example: 8 labels on one sheet, one for each officer.",
        |d| d.ml_labels(),
    ),
    Entry::new(
        "mailings.start",
        "Starts a mail merge: you write one letter, envelope or label once, and it is filled in for every person in a list. Example: one letter with «name» becomes thirty letters, each with its own name.",
        |d| d.ml_mail_merge(),
    ),
    Entry::new(
        "mailings.start:letters",
        "Makes the merge produce letters: one page per person in the list, each with that person's name and details in place of the fields. Example: thirty letters from one draft.",
        |d| d.ml_mail_merge(),
    ),
    Entry::new(
        "mailings.start:emails",
        "Makes the merge produce e-mail messages: one message per person in the list, sent to the address in the list. Example: the same notice e-mailed to every member, each with his name.",
        |d| d.ml_merge_emails(),
    ),
    Entry::new(
        "mailings.start:envelopes",
        "Makes the merge produce envelopes: one envelope per person in the list, with that person's address on it. Example: thirty addressed envelopes from one design.",
        |d| d.ml_merge_envelopes(),
    ),
    Entry::new(
        "mailings.start:labels",
        "Makes the merge produce a sheet of labels: one label per person in the list, filled from the list. Example: a sheet of eight labels, each with a different name.",
        |d| d.ml_merge_labels(),
    ),
    Entry::new(
        "mailings.start:directory",
        "Makes the merge produce one list page: every person's details one after another on the same page, like a roster or a phone list. Example: all thirty names and ranks in one list.",
        |d| d.ml_merge_directory(),
    ),
    Entry::new(
        "mailings.start:normal",
        "Turns the mail merge off: the document becomes an ordinary document again and stops using the list. Example: the «name» fields stay as plain text and the list is no longer attached.",
        |d| d.ml_merge_normal(),
    ),
    Entry::new(
        "mailings.recipients",
        "Picks the list of people the letter is made for: a table you type here, or a spreadsheet or CSV file you already have. Example: a file with the columns name, rank and unit.",
        |d| d.ml_recipients(false),
    ),
    Entry::new(
        "ui.dialog:newRecipientList",
        "Opens a small table where you type the people yourself: one row per person, one column per detail. Example: type «أحمد / نقيب / المستشفى» as the first row.",
        |d| d.ml_new_list(),
    ),
    Entry::new(
        "ui.openRecipientList",
        "Opens a list you already have on the computer (a spreadsheet or a CSV file) and uses its rows as the people to write to. Example: choose «الضباط.xlsx» and its 30 rows become 30 letters.",
        |d| d.ml_open_list(),
    ),
    Entry::new(
        "mailings.editRecipients",
        "Shows the list of people so you can tick who gets a letter and who does not, sort them or filter them. Example: untick one officer who is on leave.",
        |d| d.ml_recipients(true),
    ),
    Entry::new(
        "mailings.highlightFields",
        "Colours every «field» in the letter yellow so you can see at a glance where the names and addresses will be filled in. Example: «name» and «rank» light up in the first line.",
        |d| d.ml_letter_fields(true, false),
    ),
    Entry::new(
        "mailings.addressBlock",
        "Inserts a whole address as one field, laid out like on an envelope: name, street, city, each on its own line. Example: «أحمد علي / شارع الرشيد / بغداد» in three lines.",
        |d| d.ml_address_block(),
    ),
    Entry::new(
        "mailings.greetingLine",
        "Inserts the opening line of the letter as a field that takes each person's name. Example: «Dear «name»,» becomes «Dear Ahmed,» in one letter and «Dear Sara,» in the next.",
        |d| d.ml_greeting_line(),
    ),
    Entry::new(
        "mailings.insertField",
        "Inserts one column of the list (first name, rank, unit…) where the caret is, as a «field» that is filled in for each person. Example: put «rank» before the name so the letter says «نقيب أحمد».",
        |d| d.ml_letter_fields(false, true),
    ),
    Entry::new(
        "mailings.rules",
        "Adds a condition to the letter: if a field has a certain value, write one text, otherwise another. Example: if rank is «ضابط» write «سيدي», otherwise write «عزيزي».",
        |d| d.ml_rules(),
    ),
    Entry::new(
        "mailings.matchFields",
        "Pairs the columns of your list with the parts of the address or greeting when their names differ, so the right value goes in the right place. Example: tell it that your column «الاسم» is the «Name».",
        |d| d.ml_match_fields(),
    ),
    Entry::new(
        "mailings.preview",
        "Replaces the «fields» with one real person's details so you can see the finished letter before making them all. Example: «name» becomes «Ali Hasan» on the page.",
        |d| d.ml_preview(),
    ),
    Entry::new(
        "mailings.previous",
        "Shows the letter of the person before this one in the preview. Example: from the third person's letter back to the second.",
        |d| d.ml_record(false),
    ),
    Entry::new(
        "mailings.next",
        "Shows the letter of the next person in the preview. Example: from the second person's letter on to the third.",
        |d| d.ml_record(true),
    ),
    Entry::new(
        "mailings.findRecipient",
        "Finds a person in the list by name or any other detail and shows that person's letter. Example: type «أحمد» and his letter appears.",
        |d| d.ml_find_recipient(),
    ),
    Entry::new(
        "mailings.checkErrors",
        "Checks every merged letter for fields that have no value or details that are missing, before you print. Example: it warns that one person has no address.",
        |d| d.ml_check_errors(),
    ),
    Entry::new(
        "mailings.finish",
        "Makes the finished letters: one document with all of them one after another, or sends them straight to the printer. Example: thirty letters printed in one go.",
        |d| d.ml_finish(),
    ),
];

impl Draw<'_> {
    /// How far item `i` of `n` has landed in the film: 0 before its turn, 1 once it is there
    /// (always 1 in a still drawing).
    fn ml_step(&self, i: usize, n: usize) -> f32 {
        let n = n.max(1) as f32;
        let from = 0.1 + 0.6 * i as f32 / n;
        self.reveal(from, from + 0.15)
    }

    /// An envelope in `r` with its flap; returns it.
    fn ml_envelope_shape(&self, r: Rect) -> Rect {
        self.p.rect_filled(r, 2.0, Color32::WHITE);
        self.p.rect_stroke(r, 2.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        let flap = r.height() * 0.43;
        self.line(r.min, pos2(r.center().x, r.min.y + flap), self.t.border_strong, 1.0);
        self.line(pos2(r.max.x, r.min.y), pos2(r.center().x, r.min.y + flap), self.t.border_strong, 1.0);
        r
    }

    /// The addresses are typed onto the envelope: the return address first, then the delivery
    /// address, then the stamp.
    fn ml_envelope(&self) {
        let r = self.ml_envelope_shape(Rect::from_min_size(self.at(18.0, 18.0), vec2(140.0, 70.0)));
        let k1 = self.reveal(0.05, 0.3);
        self.text_lines(Rect::from_min_size(r.min + vec2(6.0, 2.0), vec2(40.0, 20.0)), false, &[26.0 * k1, 20.0 * k1], 5.0, self.t.text_dim);
        let k2 = self.reveal(0.3, 0.7);
        self.text_lines(Rect::from_min_size(r.min + vec2(58.0, 32.0), vec2(70.0, 30.0)), false, &[44.0 * k2, 56.0 * k2, 36.0 * k2], 8.0, self.t.text);
        if self.reveal(0.75, 0.9) > 0.0 {
            self.p.rect_stroke(
                Rect::from_min_size(pos2(r.max.x - 22.0, r.min.y + 4.0), vec2(16.0, 18.0)),
                1.0,
                self.stroke(self.t.accent),
                egui::StrokeKind::Inside,
            );
        }
    }

    /// A sheet of labels in `page`; `names` puts a different first line on each label (a
    /// merge), otherwise every label repeats the same address. The labels fill one by one.
    fn ml_label_sheet(&self, page: Rect, names: bool) {
        for row in 0..4 {
            for col in 0..2 {
                let i = row * 2 + col;
                let cell = Rect::from_min_size(pos2(page.min.x + col as f32 * 38.0, page.min.y + row as f32 * 20.0), vec2(34.0, 17.0));
                self.p.rect_stroke(cell, 1.0, self.stroke(self.t.border), egui::StrokeKind::Inside);
                let k = self.ml_step(i, 8);
                if k <= 0.0 {
                    continue;
                }
                // The first line is the name (its own length on each label of a merge); the
                // second is the rest of the address.
                let c = if names || i == 0 { self.t.accent } else { self.t.text_dim };
                let first = if names { [14.0, 20.0, 12.0, 18.0, 16.0, 22.0, 10.0, 19.0][i % 8] } else { 20.0 };
                self.text_lines(cell.shrink(3.0), false, &[first * k], 6.0, c);
                self.line(pos2(cell.min.x + 3.0, cell.min.y + 13.0), pos2(cell.min.x + 3.0 + 26.0 * k, cell.min.y + 13.0), self.t.text_dim, 1.5);
            }
        }
    }

    fn ml_labels(&self) {
        let page = self.page(44.0, 6.0, 88.0, 92.0);
        self.ml_label_sheet(page, false);
    }

    /// The list (a table) and one letter with a field, on the left, and the arrow to the
    /// result; returns nothing — the caller draws the result on the right.
    fn ml_merge_source(&self) {
        let table = Rect::from_min_size(self.at(6.0, 10.0), vec2(44.0, 40.0));
        self.table(table, 4, 2);
        let letter = self.page(6.0, 56.0, 44.0, 42.0);
        self.field(pos2(letter.min.x, letter.min.y + 2.0), 18.0);
        self.text_lines(Rect::from_min_size(letter.min + vec2(0.0, 10.0), letter.size()), false, &[28.0, 24.0, 20.0], 6.0, self.t.text_dim);
        let arrow = self.reveal(0.0, 0.2);
        if arrow > 0.0 {
            self.arrow_right(self.at(58.0, 52.0), self.at(58.0 + 20.0 * arrow, 52.0));
        }
    }

    /// A list plus one letter with fields → a pile of filled letters, one per person.
    fn ml_mail_merge(&self) {
        self.ml_merge_source();
        for i in 0..3 {
            let k = self.ml_step(i, 3);
            if k <= 0.0 {
                continue;
            }
            let off = i as f32 * 14.0;
            let p = self.page(84.0 + off, 10.0 + off, 60.0, 60.0);
            self.label(pos2(p.min.x, p.min.y + 3.0), ["Ali", "Sara", "Omar"][i], 7.0, self.t.accent, Align2::LEFT_CENTER);
            self.text_lines(Rect::from_min_size(p.min + vec2(0.0, 10.0), p.size()), false, &[40.0 * k, 34.0 * k, 28.0 * k], 7.0, self.t.text_dim);
        }
    }

    /// A list plus one message → a pile of e-mails, each with its «@» address.
    fn ml_merge_emails(&self) {
        self.ml_merge_source();
        for i in 0..3 {
            let k = self.ml_step(i, 3);
            if k <= 0.0 {
                continue;
            }
            let off = i as f32 * 14.0;
            let r = Rect::from_min_size(self.at(84.0 + off, 14.0 + off), vec2(62.0, 50.0));
            self.p.rect_filled(r, 2.0, Color32::WHITE);
            self.p.rect_stroke(r, 2.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
            // The address line: «@» and the name.
            self.p.rect_filled(Rect::from_min_size(r.min, vec2(r.width(), 12.0)), 2.0, self.t.hover);
            self.label(pos2(r.min.x + 4.0, r.min.y + 6.0), "@", 8.0, self.t.accent, Align2::LEFT_CENTER);
            self.label(pos2(r.min.x + 14.0, r.min.y + 6.0), ["ali", "sara", "omar"][i], 6.5, self.t.text, Align2::LEFT_CENTER);
            self.text_lines(
                Rect::from_min_size(r.min + vec2(6.0, 14.0), vec2(50.0, 30.0)),
                false,
                &[40.0 * k, 34.0 * k, 24.0 * k],
                7.0,
                self.t.text_dim,
            );
        }
    }

    /// A list plus one envelope → a pile of addressed envelopes.
    fn ml_merge_envelopes(&self) {
        self.ml_merge_source();
        for i in 0..3 {
            let k = self.ml_step(i, 3);
            if k <= 0.0 {
                continue;
            }
            let off = i as f32 * 12.0;
            let r = self.ml_envelope_shape(Rect::from_min_size(self.at(84.0 + off, 20.0 + off), vec2(72.0, 38.0)));
            self.label(pos2(r.min.x + 26.0, r.min.y + 20.0), ["Ali", "Sara", "Omar"][i], 6.5, self.t.accent, Align2::LEFT_CENTER);
            self.text_lines(Rect::from_min_size(r.min + vec2(26.0, 23.0), vec2(40.0, 14.0)), false, &[34.0 * k, 26.0 * k], 5.0, self.t.text_dim);
        }
    }

    /// A list plus one label → a sheet of labels, a different name on each.
    fn ml_merge_labels(&self) {
        self.ml_merge_source();
        let page = self.page(86.0, 6.0, 86.0, 92.0);
        self.ml_label_sheet(page, true);
    }

    /// A list plus one row design → one page listing everyone, row after row.
    fn ml_merge_directory(&self) {
        self.ml_merge_source();
        let p = self.page(90.0, 8.0, 76.0, 88.0);
        for (i, name) in ["Ali", "Sara", "Omar", "Noor", "Hadi"].iter().enumerate() {
            let k = self.ml_step(i, 5);
            if k <= 0.0 {
                continue;
            }
            let y = p.min.y + 5.0 + i as f32 * 14.0;
            self.label(pos2(p.min.x, y), name, 6.5, self.t.accent, Align2::LEFT_CENTER);
            self.line(pos2(p.min.x + 22.0, y), pos2(p.min.x + 22.0 + 36.0 * k, y), self.t.text_dim, 1.5);
            self.line(pos2(p.min.x, y + 6.0), pos2(p.min.x + 44.0 * k, y + 6.0), self.t.text_dim, 1.5);
        }
    }

    /// The list is let go (crossed out, faded) and the letter is an ordinary page again.
    fn ml_merge_normal(&self) {
        let k = self.reveal(0.1, 0.5);
        let table = Rect::from_min_size(self.at(6.0, 10.0), vec2(44.0, 40.0));
        self.table(table, 4, 2);
        if k > 0.0 {
            self.p.rect_filled(table, 1.0, Color32::WHITE.linear_multiply(0.7 * k));
            self.line(table.left_top(), table.left_top() + (table.right_bottom() - table.left_top()) * k, self.red(), 2.0);
            self.line(table.left_bottom(), table.left_bottom() + (table.right_top() - table.left_bottom()) * k, self.red(), 2.0);
        }
        let letter = self.page(6.0, 56.0, 44.0, 42.0);
        self.field(pos2(letter.min.x, letter.min.y + 2.0), 18.0);
        self.text_lines(Rect::from_min_size(letter.min + vec2(0.0, 10.0), letter.size()), false, &[28.0, 24.0, 20.0], 6.0, self.t.text_dim);
        let arrow = self.reveal(0.5, 0.7);
        if arrow > 0.0 {
            self.arrow_right(self.at(58.0, 52.0), self.at(58.0 + 20.0 * arrow, 52.0));
        }
        let p = self.page(96.0, 10.0, 66.0, 84.0);
        // The field is plain text now, and the page is an ordinary one.
        let plain = self.reveal(0.7, 0.9);
        if plain < 1.0 {
            self.field(pos2(p.min.x, p.min.y + 2.0), 24.0);
        }
        if plain > 0.0 {
            self.p.rect_filled(Rect::from_min_size(pos2(p.min.x - 1.0, p.min.y + 1.0), vec2(26.0, 9.0)), 0.0, Color32::WHITE.linear_multiply(plain));
            self.line(pos2(p.min.x, p.min.y + 5.5), pos2(p.min.x + 24.0, p.min.y + 5.5), self.t.text.linear_multiply(plain), 1.5);
        }
        self.text_lines(Rect::from_min_size(p.min + vec2(0.0, 12.0), p.size()), false, &[44.0, 40.0, 46.0, 30.0], 9.0, self.t.text_dim);
    }

    /// The recipients table; the ticks land one by one. With `edit`, a pencil comes down and
    /// one tick is cleared.
    fn ml_recipients(&self, edit: bool) {
        let table = Rect::from_min_size(self.at(24.0, 10.0), vec2(128.0, 84.0));
        self.table(table, 6, 3);
        let rh = table.height() / 6.0;
        let untick = self.reveal(0.6, 0.8);
        for i in 1..6 {
            let y = table.min.y + (i as f32 + 0.5) * rh;
            let b = Rect::from_center_size(pos2(table.min.x + 8.0, y), vec2(7.0, 7.0));
            self.p.rect_stroke(b, 1.0, self.stroke(self.t.text_dim), egui::StrokeKind::Inside);
            // Every tick is there when editing (the fourth is cleared); otherwise they land
            // one by one, the fourth row never ticked.
            let k = if edit { 1.0 } else { self.ml_step(i - 1, 5) };
            let shown = match (edit, i == 4) {
                (true, true) => 1.0 - untick,
                (false, true) => 0.0,
                _ => k,
            };
            if shown > 0.0 {
                let c = self.t.accent.linear_multiply(shown);
                self.line(pos2(b.min.x + 1.5, b.center().y), pos2(b.center().x, b.max.y - 1.5), c, 1.5);
                self.line(pos2(b.center().x, b.max.y - 1.5), pos2(b.max.x - 1.0, b.min.y + 1.5), c, 1.5);
            }
        }
        if edit {
            // A pencil over the table, coming down to the row it unticks.
            let k = self.reveal(0.3, 0.6);
            let drop = (1.0 - k) * 14.0;
            let a = self.at(136.0, 84.0 - drop);
            let b = self.at(156.0, 64.0 - drop);
            self.line(a, b, self.t.accent, 4.0);
            self.line(a, a + vec2(-3.0, 3.0), self.t.accent, 2.0);
        }
    }

    /// The New List box: a small table with a row being typed and an add-row button.
    fn ml_new_list(&self) {
        let inner = self.dialog("New List", 0);
        let r = Rect::from_min_size(inner.min, vec2(inner.width(), 36.0));
        self.table(r, 3, 3);
        // The first row is typed in, cell by cell.
        for j in 0..3 {
            let k = self.ml_step(j, 3);
            if k <= 0.0 {
                continue;
            }
            let cell = self.cell(r, 3, 3, 1, j);
            self.p.rect_filled(cell.shrink(0.5), 0.0, Color32::WHITE);
            self.line(pos2(cell.min.x + 3.0, cell.center().y), pos2(cell.min.x + 3.0 + cell.width() * 0.6 * k, cell.center().y), self.t.text, 1.5);
        }
        let cell = self.cell(r, 3, 3, 2, 0);
        self.p.rect_filled(Rect::from_min_size(cell.min, vec2(r.width(), cell.height())).shrink(0.5), 0.0, Color32::WHITE);
        self.button(pos2(inner.max.x - 22.0, inner.max.y - 8.0), "+ Row", false);
    }

    /// A spreadsheet file opens into the recipients table.
    fn ml_open_list(&self) {
        // The file: a page with a grid and an «xlsx» badge.
        let f = self.page(10.0, 18.0, 52.0, 66.0);
        self.grid(Rect::from_min_size(f.min, vec2(f.width(), 40.0)), 5, 3, self.t.border);
        self.badge(pos2(f.center().x, f.max.y - 4.0), "xlsx", self.green());
        let arrow = self.reveal(0.0, 0.25);
        if arrow > 0.0 {
            self.arrow_right(self.at(68.0, 52.0), self.at(68.0 + 18.0 * arrow, 52.0));
        }
        let table = Rect::from_min_size(self.at(92.0, 10.0), vec2(78.0, 84.0));
        let k = self.reveal(0.25, 0.7);
        if k <= 0.0 {
            self.p.rect_stroke(table, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
            return;
        }
        let clip = Rect::from_min_max(table.min, pos2(table.max.x, table.min.y + table.height() * k));
        let clipped = self.p.with_clip_rect(clip.intersect(self.p.clip_rect()));
        self.with_painter(&clipped).table(table, 6, 3);
        if k < 1.0 {
            self.p.rect_stroke(table, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        }
    }

    /// The letter with its fields; with `highlight` they light up one by one; with `caret` a
    /// new field appears at the caret.
    fn ml_letter_fields(&self, highlight: bool, caret: bool) {
        let page = self.page(44.0, 6.0, 88.0, 92.0);
        self.text_lines(Rect::from_min_size(page.min, page.size()), false, &[40.0, 48.0], 7.0, self.t.text_dim);
        let fields =
            [(page.min.x, page.min.y + 18.0, 28.0), (page.min.x + 36.0, page.min.y + 18.0, 22.0), (page.min.x + 14.0, page.min.y + 36.0, 30.0)];
        for (i, (x, y, w)) in fields.into_iter().enumerate() {
            if highlight {
                let k = self.ml_step(i, 3);
                if k > 0.0 {
                    self.p.rect_filled(Rect::from_min_size(pos2(x, y - 1.0), vec2(w, 9.0)), 1.0, self.t.find_highlight.linear_multiply(k));
                }
            }
            self.field(pos2(x, y), w);
        }
        self.text_lines(Rect::from_min_size(pos2(page.min.x, page.min.y + 46.0), page.size()), false, &[60.0, 54.0, 30.0], 8.0, self.t.text_dim);
        if caret {
            let x = page.min.x + 50.0;
            self.line(pos2(x, page.min.y + 33.0), pos2(x, page.min.y + 46.0), self.t.text, 1.5);
            let k = self.reveal(0.3, 0.6);
            if k > 0.0 {
                let w = 26.0 * k;
                self.field(pos2(x + 2.0, page.min.y + 36.0), w);
                if k >= 1.0 {
                    self.label(pos2(x + 15.0, page.min.y + 39.5), "«…»", 6.0, self.t.accent, Align2::CENTER_CENTER);
                }
            }
        }
    }

    /// The address block's three lines appear inside their frame.
    fn ml_address_block(&self) {
        let page = self.page(44.0, 6.0, 88.0, 92.0);
        let block = Rect::from_min_size(page.min + vec2(0.0, 2.0), vec2(48.0, 30.0));
        let k = self.reveal(0.1, 0.3);
        if k > 0.0 {
            self.p.rect_stroke(block.expand(3.0), 2.0, self.stroke(self.t.accent.linear_multiply(k)), egui::StrokeKind::Outside);
        }
        let ws = [30.0, 44.0, 36.0];
        let widths: Vec<f32> = ws.iter().enumerate().map(|(i, w)| w * self.ml_step(i, 3)).collect();
        self.text_lines(block, false, &widths, 9.0, self.t.text);
        self.text_lines(Rect::from_min_size(pos2(page.min.x, page.min.y + 44.0), page.size()), false, &[60.0, 56.0, 40.0], 8.0, self.t.text_dim);
    }

    /// «Dear» then the name field slides in, then the comma.
    fn ml_greeting_line(&self) {
        let page = self.page(44.0, 6.0, 88.0, 92.0);
        self.text_lines(page, false, &[30.0, 44.0], 7.0, self.t.text_dim);
        let y = page.min.y + 24.0;
        self.label(pos2(page.min.x, y), "Dear", 7.5, self.t.text, Align2::LEFT_CENTER);
        let k = self.reveal(0.15, 0.45);
        if k > 0.0 {
            self.field(pos2(page.min.x + 24.0, y - 3.5), 30.0 * k);
        }
        if self.reveal(0.45, 0.55) > 0.0 {
            self.label(pos2(page.min.x + 56.0, y), ",", 7.5, self.t.text, Align2::LEFT_CENTER);
        }
        let frame = self.reveal(0.55, 0.75);
        if frame > 0.0 {
            self.p.rect_stroke(
                Rect::from_min_size(pos2(page.min.x - 3.0, y - 7.0), vec2(66.0, 14.0)),
                2.0,
                self.stroke(self.t.accent.linear_multiply(frame)),
                egui::StrokeKind::Outside,
            );
        }
        self.text_lines(
            Rect::from_min_size(pos2(page.min.x, page.min.y + 36.0), page.size()),
            false,
            &[64.0, 58.0, 50.0, 30.0],
            8.0,
            self.t.text_dim,
        );
    }

    /// If «field» = x then A else B: the two branches grow from the question.
    fn ml_rules(&self) {
        let top = self.at(88.0, 14.0);
        self.field(top - vec2(14.0, 3.5), 28.0);
        self.label(top + vec2(0.0, 12.0), "= ?", 8.0, self.t.text, Align2::CENTER_CENTER);
        let a = self.at(44.0, 60.0);
        let b = self.at(132.0, 60.0);
        let k = self.reveal(0.15, 0.5);
        let from = top + vec2(0.0, 18.0);
        for to in [a - vec2(0.0, 12.0), b - vec2(0.0, 12.0)] {
            if k > 0.0 {
                self.line(from, from + (to - from) * k, self.t.accent, 1.5);
            }
        }
        self.label(self.at(58.0, 38.0), "yes", 7.0, self.t.text_dim, Align2::CENTER_CENTER);
        self.label(self.at(118.0, 38.0), "no", 7.0, self.t.text_dim, Align2::CENTER_CENTER);
        let pop = self.reveal(0.5, 0.75);
        for (c, text) in [(a, "text A"), (b, "text B")] {
            if pop <= 0.0 {
                continue;
            }
            let r = Rect::from_center_size(c, vec2(48.0, 20.0) * (0.7 + 0.3 * pop));
            self.p.rect_filled(r, 2.0, Color32::WHITE);
            self.p.rect_stroke(r, 2.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
            self.label(c, text, 7.5, self.t.text.linear_multiply(pop), Align2::CENTER_CENTER);
        }
        self.text_lines(Rect::from_min_size(self.at(20.0, 80.0), vec2(136.0, 20.0)), false, &[136.0, 100.0], 8.0, self.t.text_dim);
    }

    /// The list's column names on the left, the address parts on the right, joined one by one.
    fn ml_match_fields(&self) {
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
            let k = self.ml_step(i, 3);
            if k > 0.0 {
                let (a, b) = (l.right_center(), r.left_center());
                self.line(a, a + (b - a) * k, self.t.accent, 1.5);
            }
        }
    }

    /// The field on the left becomes a real name on the right.
    fn ml_preview(&self) {
        let before = self.page(6.0, 10.0, 64.0, 80.0);
        self.field(pos2(before.min.x, before.min.y + 2.0), 30.0);
        self.text_lines(Rect::from_min_size(before.min + vec2(0.0, 14.0), before.size()), false, &[44.0, 40.0, 28.0], 9.0, self.t.text_dim);
        let arrow = self.reveal(0.0, 0.25);
        if arrow > 0.0 {
            self.arrow_right(self.at(76.0, 50.0), self.at(76.0 + 24.0 * arrow, 50.0));
        }
        let after = self.page(106.0, 10.0, 64.0, 80.0);
        let k = self.reveal(0.3, 0.6);
        if k < 1.0 {
            self.field(pos2(after.min.x, after.min.y + 2.0), 30.0);
        }
        if k > 0.0 {
            self.p.rect_filled(Rect::from_min_size(pos2(after.min.x - 1.0, after.min.y), vec2(42.0, 11.0)), 0.0, Color32::WHITE.linear_multiply(k));
            self.label(pos2(after.min.x, after.min.y + 5.0), "Ali Hasan", 7.5, self.t.accent.linear_multiply(k), Align2::LEFT_CENTER);
        }
        self.text_lines(Rect::from_min_size(after.min + vec2(0.0, 14.0), after.size()), false, &[44.0, 40.0, 28.0], 9.0, self.t.text_dim);
    }

    /// The pile of letters; the counter steps to the previous or the next person and the
    /// arrow grows that way.
    fn ml_record(&self, next: bool) {
        let k = self.reveal(0.2, 0.5);
        for i in 0..3 {
            let off = i as f32 * 10.0;
            let p = self.page(50.0 + off, 8.0 + off, 60.0, 64.0);
            if i == 2 {
                // From the second person to the third (next) or the first (previous).
                let n = if k < 1.0 {
                    "2 / 5"
                } else if next {
                    "3 / 5"
                } else {
                    "1 / 5"
                };
                self.label(pos2(p.min.x, p.min.y + 5.0), n, 8.0, self.t.accent, Align2::LEFT_CENTER);
                self.text_lines(Rect::from_min_size(p.min + vec2(0.0, 14.0), p.size()), false, &[40.0, 36.0, 24.0], 9.0, self.t.text_dim);
            }
        }
        let grow = self.reveal(0.0, 0.3);
        if grow <= 0.0 {
            return;
        }
        if next {
            self.arrow_right(self.at(130.0, 92.0), self.at(130.0 + 30.0 * grow, 92.0));
        } else {
            self.arrow_left(self.at(46.0, 92.0), self.at(46.0 - 30.0 * grow, 92.0));
        }
    }

    /// The magnifier searches down the list and the found row lights up.
    fn ml_find_recipient(&self) {
        let table = Rect::from_min_size(self.at(16.0, 10.0), vec2(110.0, 84.0));
        self.table(table, 6, 2);
        let rh = table.height() / 6.0;
        let k = self.reveal(0.1, 0.6);
        let y = table.min.y + (1.0 + 2.0 * k) * rh;
        self.p.rect_filled(
            Rect::from_min_size(pos2(table.min.x, y), vec2(table.width(), rh)),
            0.0,
            self.t.find_highlight.linear_multiply(0.4 + 0.6 * k),
        );
        let c = self.at(146.0, 24.0 + 16.0 * k);
        self.p.circle_stroke(c, 10.0, Stroke::new(2.5, self.t.accent));
        self.line(c + vec2(7.0, 7.0), c + vec2(16.0, 16.0), self.t.accent, 3.0);
    }

    /// The warning pops onto the empty field, then the tick says the rest is fine.
    fn ml_check_errors(&self) {
        let page = self.page(50.0, 6.0, 76.0, 92.0);
        self.text_lines(page, false, &[50.0, 56.0, 40.0, 54.0, 30.0], 11.0, self.t.text_dim);
        self.field(pos2(page.min.x + 20.0, page.min.y + 34.0), 24.0);
        if self.reveal(0.2, 0.35) > 0.0 {
            self.warning(pos2(page.min.x + 44.0, page.min.y + 30.0));
        }
        let k = self.reveal(0.5, 0.75);
        if k > 0.0 {
            let c = self.at(146.0, 84.0);
            self.p.circle_filled(c, 11.0 * (0.6 + 0.4 * k), self.t.accent);
            self.line(c + vec2(-5.0, 0.0), c + vec2(-1.5, 4.0), Color32::WHITE, 2.5);
            self.line(c + vec2(-1.5, 4.0), c + vec2(6.0, -5.0), Color32::WHITE, 2.5);
        }
    }

    /// The letters pile up, then go to the printer.
    fn ml_finish(&self) {
        for i in 0..4 {
            let k = self.ml_step(i, 4);
            if k <= 0.0 {
                continue;
            }
            let off = i as f32 * 12.0;
            let p = self.page(14.0 + off, 6.0 + off, 56.0, 56.0);
            if i == 3 {
                self.label(pos2(p.min.x, p.min.y + 4.0), "Omar", 7.0, self.t.accent, Align2::LEFT_CENTER);
                self.text_lines(Rect::from_min_size(p.min + vec2(0.0, 12.0), p.size()), false, &[36.0 * k, 30.0 * k, 20.0 * k], 8.0, self.t.text_dim);
            }
        }
        let arrow = self.reveal(0.7, 0.9);
        if arrow > 0.0 {
            self.arrow_right(self.at(112.0, 52.0), self.at(112.0 + 18.0 * arrow, 52.0));
        }
        self.printer(self.at(151.0, 53.0));
    }
}
