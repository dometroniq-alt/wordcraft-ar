//! Help, Developer and Zotero tabs thumbnails, and the Service tab's classification choices:
//! the help dialogs, the form controls a reader fills in (content controls), the Zotero
//! citation tools and the stamp of each classification word. Each one a little film: the chat
//! bubbles pop up, the search finds its commands, the form box is drawn, the stamp comes down.

use egui::{Align2, Color32, Rect, Stroke, pos2, vec2};

use super::Entry;
use super::draw::{Dir, Draw};

pub const ENTRIES: &[Entry] = &[
    // Help.
    Entry::new(
        "ui.dialog:about",
        "Shows which version of WordCraft AR this is and where to get help. Example: «WordCraft AR 1.2» and a link to the guide.",
        |d| d.mz_about(),
    ),
    Entry::new(
        "ui.discord",
        "Opens the community chat in your web browser, where you can ask a question or report a problem. Example: ask how to number a letter and get an answer.",
        |d| d.mz_discord(),
    ),
    Entry::new(
        "ui.dialog:commands",
        "Opens a searchable list of every command: type part of its name, pick it and run it, without looking for its button. Example: type «ترقيم» and run Renumber.",
        |d| d.mz_commands(),
    ),
    // Developer › Controls.
    Entry::new(
        "developer.richText",
        "Puts a box in the form where the reader can type text with formatting: bold, lists or pictures. Example: a «ملاحظات» box the reader fills in with a bulleted list.",
        |d| d.mz_control_text(true),
    ),
    Entry::new(
        "developer.plainText",
        "Puts a box in the form where the reader can type plain text only, all in one style. Example: a box for the name that stays in the form's font.",
        |d| d.mz_control_text(false),
    ),
    Entry::new("developer.picture", "Puts a box in the form where the reader can place a picture. Example: a box for the applicant's photo.", |d| {
        d.mz_control_picture()
    }),
    Entry::new(
        "developer.buildingBlock",
        "Puts a box in the form where the reader picks one ready-made block of text from a gallery. Example: choose one of three standard closing paragraphs.",
        |d| d.mz_control_gallery(),
    ),
    Entry::new(
        "developer.checkBox",
        "Puts a small box in the form that the reader can tick or untick. Example: «☑ تم الفحص» ticked when the check is done.",
        |d| d.mz_control_check_box(),
    ),
    Entry::new(
        "developer.comboBox",
        "Puts a list in the form that the reader can choose from, or type their own entry into. Example: choose «نقيب» from the list or type another rank.",
        |d| d.mz_control_list(true),
    ),
    Entry::new(
        "developer.dropDown",
        "Puts a list in the form that the reader must choose one entry from; typing is not allowed. Example: choose the unit from a fixed list of units.",
        |d| d.mz_control_list(false),
    ),
    Entry::new(
        "developer.datePicker",
        "Puts a box in the form where the reader picks a date from a small calendar. Example: click the box and choose ١٧/١٠/٢٠٢٦.",
        |d| d.mz_control_date(),
    ),
    Entry::new(
        "developer.repeatingSection",
        "Puts a section in the form that the reader can copy as many times as needed, like adding rows. Example: one «person» block that the reader duplicates for each member.",
        |d| d.mz_control_repeating(),
    ),
    Entry::new(
        "developer.designMode",
        "Shows the edges and titles of every form box so you can change the boxes themselves rather than fill them in. Example: the «الاسم» box gets its frame and tag and can be moved.",
        |d| d.mz_design_mode(),
    ),
    Entry::new(
        "developer.properties",
        "Opens a box to set a form box's title, its tag, the entries of its list and whether the reader may delete it. Example: give the rank list its entries: ملازم، نقيب، رائد.",
        |d| d.mz_control_properties(),
    ),
    // Zotero.
    Entry::new(
        "ui.zotero.addEditCitation",
        "Inserts a citation from your Zotero library where the caret is, or edits the one under it. Example: pick a book in Zotero and «(علي، ٢٠٢٠)» appears in the text.",
        |d| d.mz_citation(),
    ),
    Entry::new(
        "ui.zotero.addNote",
        "Inserts one of your Zotero notes as text where the caret is. Example: the note you wrote on a book becomes a paragraph in the document.",
        |d| d.mz_zotero_note(),
    ),
    Entry::new(
        "caret.pastCitation",
        "Moves the caret to just after the citation it is in, so you can keep typing without changing the citation. Example: from inside «(علي، ٢٠٢٠)» to the space after it.",
        |d| d.mz_past_citation(),
    ),
    Entry::new(
        "ui.zotero.addEditBibliography",
        "Inserts the list of every source you cited where the caret is, or updates the list already there. Example: a «المصادر» list at the end naming every cited book.",
        |d| d.mz_bibliography(),
    ),
    Entry::new(
        "ui.zotero.refresh",
        "Updates every citation and the bibliography after you changed something in your Zotero library. Example: fix a year in Zotero and every citation shows the new year.",
        |d| d.mz_zotero_refresh(),
    ),
    Entry::new(
        "ui.zotero.setDocPrefs",
        "Opens Zotero's settings for this document: the citation style and the language of the citations. Example: switch the document to APA in Arabic.",
        |d| d.mz_zotero_prefs(),
    ),
    Entry::new(
        "ui.zotero.removeCodes",
        "Turns the citations and the bibliography into plain text that Zotero no longer updates, for sending the final copy. Example: «(علي، ٢٠٢٠)» stays but is ordinary text now.",
        |d| d.mz_unlink_citations(),
    ),
    // Service › Classification: one choice per classification word.
    Entry::new(
        "service.classification:عام",
        "Stamps «عام» at the top and bottom of every page: the letter is public and anyone may read it. Example: a general notice to all units.",
        |d| d.mz_classification("عام"),
    ),
    Entry::new(
        "service.classification:محدود",
        "Stamps «محدود» at the top and bottom of every page: the letter is for the people concerned only, the usual mark of service writing. Example: an ordinary internal letter.",
        |d| d.mz_classification("محدود"),
    ),
    Entry::new(
        "service.classification:سري",
        "Stamps «سري» at the top and bottom of every page: the letter is secret and goes only to those cleared to read it. Example: a report on a security matter.",
        |d| d.mz_classification("سري"),
    ),
    Entry::new(
        "service.classification:سري وشخصي",
        "Stamps «سري وشخصي» at the top and bottom of every page: secret, and to be opened by the named person alone. Example: a letter about one officer's own affairs.",
        |d| d.mz_classification("سري وشخصي"),
    ),
    Entry::new(
        "service.classification:سري للغاية",
        "Stamps «سري للغاية» at the top and bottom of every page: the highest secrecy, for the fewest eyes. Example: a letter on a matter of the greatest sensitivity.",
        |d| d.mz_classification("سري للغاية"),
    ),
];

impl Draw<'_> {
    /// How far item `i` of `n` has landed in the film: 0 before its turn, 1 once it is there
    /// (always 1 in a still drawing).
    fn mz_step(&self, i: usize, n: usize) -> f32 {
        let n = n.max(1) as f32;
        let from = 0.1 + 0.6 * i as f32 / n;
        self.reveal(from, from + 0.15)
    }

    // ---- Help ----

    /// The question mark grows in, then the text beside it.
    fn mz_about(&self) {
        let c = self.at(50.0, 52.0);
        let k = self.reveal(0.05, 0.35);
        self.p.circle_filled(c, 24.0 * (0.5 + 0.5 * k), self.accent());
        self.glyph(c, "?", 30.0 * (0.5 + 0.5 * k), Color32::WHITE);
        let a = Rect::from_min_size(self.at(90.0, 30.0), vec2(76.0, 50.0));
        let k2 = self.reveal(0.35, 0.6);
        self.line(pos2(a.min.x, a.min.y + 4.0), pos2(a.min.x + 56.0 * k2, a.min.y + 4.0), self.ink(), 3.0);
        let widths: Vec<f32> = [40.0, 60.0, 48.0].iter().enumerate().map(|(i, w)| w * self.mz_step(i + 2, 5)).collect();
        self.text_lines(Rect::from_min_size(a.min + vec2(0.0, 10.0), a.size()), false, &widths, 10.0, self.grey());
    }

    /// A chat bubble with a few lines of talk pops up, then a smaller reply bubble.
    fn mz_discord(&self) {
        let k = self.reveal(0.05, 0.35);
        let b = Rect::from_center_size(self.at(70.0, 39.0), vec2(96.0, 50.0) * (0.6 + 0.4 * k));
        self.p.rect_filled(b, 8.0, self.accent());
        self.p.add(egui::Shape::convex_polygon(
            vec![pos2(b.min.x + 16.0, b.max.y - 1.0), pos2(b.min.x + 10.0, b.max.y + 12.0), pos2(b.min.x + 30.0, b.max.y - 1.0)],
            self.accent(),
            Stroke::NONE,
        ));
        if k >= 1.0 {
            self.text_lines(b.shrink(10.0), false, &[60.0, 72.0, 44.0], 11.0, Color32::WHITE);
        }
        let k2 = self.reveal(0.5, 0.8);
        if k2 > 0.0 {
            let r = Rect::from_center_size(self.at(120.0, 81.0), vec2(72.0, 30.0) * (0.6 + 0.4 * k2));
            self.p.rect_filled(r, 8.0, self.t.hover);
            self.p.rect_stroke(r, 8.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
            if k2 >= 1.0 {
                self.text_lines(r.shrink(8.0), false, &[48.0, 30.0], 10.0, self.grey());
            }
        }
    }

    /// The search box is typed into and the matching commands appear under it.
    fn mz_commands(&self) {
        let inner = self.dialog("Commands", 0);
        let f = Rect::from_min_size(inner.min, vec2(inner.width(), 11.0));
        self.p.rect_filled(f, 2.0, self.t.input);
        self.p.rect_stroke(f, 2.0, self.stroke(self.t.input_border), egui::StrokeKind::Inside);
        self.p.circle_stroke(pos2(f.min.x + 6.0, f.center().y), 3.0, Stroke::new(1.5, self.accent()));
        self.line(pos2(f.min.x + 8.0, f.center().y + 2.0), pos2(f.min.x + 11.0, f.center().y + 5.0), self.accent(), 1.5);
        let typed = self.reveal(0.05, 0.4);
        let end = f.min.x + 16.0 + 24.0 * typed;
        self.line(pos2(f.min.x + 16.0, f.center().y), pos2(end, f.center().y), self.ink(), 1.5);
        self.caret(pos2(end + 2.0, f.min.y + 2.0), 7.0);
        for i in 0..3 {
            let k = self.mz_step(i + 2, 5);
            if k <= 0.0 {
                continue;
            }
            let y = inner.min.y + 20.0 + i as f32 * 11.0;
            let row = Rect::from_min_size(pos2(inner.min.x, y - 4.0), vec2(inner.width(), 9.0));
            if i == 0 {
                self.selection(row);
            }
            self.line(pos2(row.min.x + 3.0, y), pos2(row.min.x + 3.0 + (31.0 + i as f32 * 10.0) * k, y), self.grey(), 1.5);
        }
    }

    // ---- Developer: content controls ----

    /// A page with ordinary text and one control box of height `h`; its dashed frame fades in.
    /// Returns the box.
    fn mz_control_box(&self, h: f32) -> Rect {
        let a = self.page_center();
        self.text_lines(a, false, &[52.0, 44.0], 10.0, self.grey());
        let b = Rect::from_min_size(pos2(a.min.x, a.min.y + 22.0), vec2(a.width(), h));
        self.p.rect_filled(b, 1.0, Color32::WHITE);
        let k = self.reveal(0.05, 0.3);
        if k > 0.0 {
            self.dashed_rect(b, self.accent().linear_multiply(k));
            // The control's little handle (its title tab) at the top-left corner.
            self.p.rect_filled(Rect::from_min_size(pos2(b.min.x - 1.0, b.min.y - 5.0), vec2(10.0, 5.0)), 1.0, self.accent().linear_multiply(k));
        }
        let below = Rect::from_min_size(pos2(a.min.x, b.max.y + 4.0), vec2(a.width(), a.max.y - b.max.y - 4.0));
        if below.height() > 8.0 {
            self.text_lines(below, false, &[48.0, 30.0], 10.0, self.grey());
        }
        b
    }

    /// The text box; the reader's text is typed into it.
    fn mz_control_text(&self, rich: bool) {
        let b = self.mz_control_box(28.0);
        let inner = b.shrink(4.0);
        let k = self.reveal(0.35, 0.8);
        if k <= 0.0 {
            return;
        }
        if rich {
            self.line(pos2(inner.min.x, inner.min.y + 3.0), pos2(inner.min.x + 30.0 * k.min(0.5) * 2.0, inner.min.y + 3.0), self.ink(), 3.0);
            if k > 0.5 {
                let k2 = (k - 0.5) * 2.0;
                self.label(pos2(inner.min.x + 2.0, inner.min.y + 14.0), "•", 8.0, self.accent(), Align2::CENTER_CENTER);
                self.line(
                    pos2(inner.min.x + 8.0, inner.min.y + 14.0),
                    pos2(inner.min.x + 8.0 + (inner.width() - 8.0) * k2, inner.min.y + 14.0),
                    self.grey(),
                    1.5,
                );
            }
        } else {
            self.text_lines(inner, false, &[inner.width() * k.min(0.5) * 2.0, inner.width() * 0.6 * ((k - 0.5) * 2.0).max(0.0)], 10.0, self.grey());
        }
    }

    /// The picture box; the picture fades in.
    fn mz_control_picture(&self) {
        let b = self.mz_control_box(34.0);
        let k = self.reveal(0.35, 0.7);
        if k > 0.0 {
            self.picture(b.shrink(4.0 + (1.0 - k) * 8.0));
        }
    }

    /// Four tiles to pick from appear, and the first is chosen.
    fn mz_control_gallery(&self) {
        let b = self.mz_control_box(30.0);
        let tw = (b.width() - 14.0) / 2.0;
        for (i, (dx, dy)) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)].iter().enumerate() {
            let k = self.mz_step(i + 1, 5);
            if k <= 0.0 {
                continue;
            }
            let t = Rect::from_min_size(pos2(b.min.x + 4.0 + dx * (tw + 6.0), b.min.y + 3.0 + dy * 13.0), vec2(tw, 11.0));
            let chosen = i == 0 && self.reveal(0.8, 0.95) > 0.0;
            self.p.rect_filled(t, 1.0, if chosen { self.accent().linear_multiply(0.3) } else { self.t.hover });
            self.line(pos2(t.min.x + 2.0, t.center().y), pos2(t.max.x - 4.0, t.center().y), self.grey(), 1.5);
        }
    }

    /// The check box; the tick is drawn into it.
    fn mz_control_check_box(&self) {
        let b = self.mz_control_box(16.0);
        let c = pos2(b.min.x + 10.0, b.center().y);
        let box_r = Rect::from_center_size(c, vec2(10.0, 10.0));
        self.p.rect_filled(box_r, 1.0, Color32::WHITE);
        self.p.rect_stroke(box_r, 1.0, Stroke::new(1.5, self.accent()), egui::StrokeKind::Inside);
        let k = self.reveal(0.4, 0.7);
        if k > 0.0 {
            let k1 = (k * 2.0).min(1.0);
            self.line(c + vec2(-3.5, 0.0), c + vec2(-3.5 + 2.5 * k1, 3.0 * k1), self.accent(), 2.0);
            if k > 0.5 {
                let k2 = (k - 0.5) * 2.0;
                self.line(c + vec2(-1.0, 3.0), c + vec2(-1.0 + 5.0 * k2, 3.0 - 6.5 * k2), self.accent(), 2.0);
            }
        }
        self.line(pos2(b.min.x + 20.0, c.y), pos2(b.max.x - 4.0, c.y), self.grey(), 1.5);
    }

    /// A small "open the list" triangle centred at `c`.
    fn mz_drop_arrow(&self, c: egui::Pos2) {
        self.p.add(egui::Shape::convex_polygon(vec![c + vec2(-3.5, -2.0), c + vec2(3.5, -2.0), c + vec2(0.0, 2.5)], self.accent(), Stroke::NONE));
    }

    /// A list field with its open-list triangle; the list drops open below it. A combo box
    /// also has a caret in the field (you can type), a drop-down has one entry chosen.
    fn mz_control_list(&self, typable: bool) {
        let b = self.mz_control_box(14.0);
        let field = b.shrink(2.0);
        self.p.rect_filled(field, 1.0, self.t.input);
        self.mz_drop_arrow(pos2(field.max.x - 6.0, field.center().y));
        if typable {
            let typed = self.reveal(0.6, 0.9);
            let end = field.min.x + 3.0 + 19.0 * typed;
            self.line(pos2(field.min.x + 3.0, field.center().y), pos2(end, field.center().y), self.ink(), 1.5);
            self.caret(pos2(end + 2.0, field.min.y + 1.5), 7.0);
        } else {
            self.line(pos2(field.min.x + 3.0, field.center().y), pos2(field.min.x + 30.0, field.center().y), self.grey(), 1.5);
        }
        // The open list, dropping over the text below.
        let k = self.reveal(0.3, 0.6);
        if k <= 0.0 {
            return;
        }
        let list = Rect::from_min_size(pos2(b.min.x, b.max.y + 1.0), vec2(b.width(), 30.0 * k));
        self.p.rect_filled(list, 1.0, self.t.menu);
        self.p.rect_stroke(list, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        let clipped = self.p.with_clip_rect(list.intersect(self.p.clip_rect()));
        let d = self.with_painter(&clipped);
        for i in 0..3 {
            let y = list.min.y + 5.0 + i as f32 * 10.0;
            let row = Rect::from_min_size(pos2(list.min.x + 1.0, y - 4.0), vec2(list.width() - 2.0, 9.0));
            if !typable && i == 1 && self.reveal(0.65, 0.8) > 0.0 {
                d.selection(row);
            }
            d.line(pos2(row.min.x + 3.0, y), pos2(row.min.x + 20.0 + i as f32 * 8.0, y), d.grey(), 1.5);
        }
    }

    /// The date field; the calendar drops open and a day is picked.
    fn mz_control_date(&self) {
        let b = self.mz_control_box(14.0);
        let field = b.shrink(2.0);
        self.p.rect_filled(field, 1.0, self.t.input);
        let picked = self.reveal(0.75, 0.9);
        if picked > 0.0 {
            self.label(pos2(field.min.x + 3.0, field.center().y), "١٧/١٠/٢٠٢٦", 7.0, self.ink(), Align2::LEFT_CENTER);
        }
        self.mz_drop_arrow(pos2(field.max.x - 6.0, field.center().y));
        let k = self.reveal(0.3, 0.55);
        if k <= 0.0 {
            return;
        }
        // A little calendar below: a 4 × 5 grid with one day picked.
        let cal = Rect::from_min_size(pos2(b.min.x, b.max.y + 1.0), vec2(b.width(), 32.0 * k));
        self.p.rect_filled(cal, 1.0, self.t.menu);
        self.p.rect_stroke(cal, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        self.p.rect_filled(Rect::from_min_size(cal.min, vec2(cal.width(), 7.0 * k)), 1.0, self.t.hover);
        if k < 1.0 {
            return;
        }
        let days = Rect::from_min_max(pos2(cal.min.x + 2.0, cal.min.y + 9.0), pos2(cal.max.x - 2.0, cal.max.y - 2.0));
        for row in 0..4 {
            for col in 0..5 {
                let c = self.cell(days, 4, 5, row, col).center();
                if row == 2 && col == 2 && picked > 0.0 {
                    self.p.circle_filled(c, 3.0 * (0.5 + 0.5 * picked), self.accent());
                } else {
                    self.p.circle_filled(c, 1.2, self.grey());
                }
            }
        }
    }

    /// One section, then its copy appears under it, with the "add another" handle.
    fn mz_control_repeating(&self) {
        let a = self.page_center();
        self.text_lines(a, false, &[52.0], 10.0, self.grey());
        for i in 0..2 {
            let k = if i == 0 { 1.0 } else { self.reveal(0.35, 0.65) };
            if k <= 0.0 {
                continue;
            }
            let b = Rect::from_min_size(pos2(a.min.x, a.min.y + 12.0 + i as f32 * 28.0), vec2(a.width(), 22.0 * k));
            self.p.rect_filled(b, 1.0, Color32::WHITE);
            self.dashed_rect(b, self.accent());
            if k >= 1.0 {
                self.text_lines(b.shrink(4.0), false, &[36.0, 26.0], 8.0, self.grey());
            }
        }
        // The "add another" handle below the last copy.
        let c = pos2(a.max.x - 6.0, a.min.y + 12.0 + 2.0 * 28.0 + 2.0);
        self.p.circle_filled(c, 6.0, self.accent());
        self.glyph(c, "+", 10.0, Color32::WHITE);
    }

    /// The boxes get their frames and title tabs, and the pencil comes in to edit them.
    fn mz_design_mode(&self) {
        let a = self.page_center();
        self.text_lines(a, false, &[52.0], 10.0, self.grey());
        let k = self.reveal(0.1, 0.45);
        for (i, h) in [14.0, 22.0].iter().enumerate() {
            let b = Rect::from_min_size(pos2(a.min.x, a.min.y + 14.0 + i as f32 * 30.0), vec2(a.width(), *h));
            self.p.rect_filled(b, 1.0, Color32::WHITE);
            if k > 0.0 {
                self.p.rect_stroke(b, 1.0, Stroke::new(1.0, self.accent().linear_multiply(k)), egui::StrokeKind::Inside);
                // The title tab at each end, as design mode shows them.
                self.p.rect_filled(Rect::from_min_size(pos2(b.min.x, b.min.y - 5.0), vec2(16.0, 5.0)), 1.0, self.accent().linear_multiply(k));
                self.p.rect_filled(
                    Rect::from_min_size(pos2(b.max.x - 6.0, b.min.y), vec2(6.0, b.height())),
                    0.0,
                    self.accent().linear_multiply(0.4 * k),
                );
            }
            self.text_lines(b.shrink(3.0), false, &[30.0, 20.0], 8.0, self.grey());
        }
        let k2 = self.reveal(0.5, 0.8);
        let slide = (1.0 - k2) * 16.0;
        self.pencil(pos2(a.max.x - 4.0 + slide, a.min.y + 18.0 - slide), pos2(a.max.x + 14.0 + slide, a.min.y - 4.0 - slide));
    }

    fn mz_control_properties(&self) {
        self.dialog("Properties", 3);
    }

    // ---- Zotero ----

    /// A page of text with a citation chip after the second line's words; the chip pops in by
    /// `k`.
    fn mz_cited_page(&self, k: f32) -> (Rect, Rect) {
        let a = self.page_lr();
        let y = a.min.y + 14.0;
        let chip = Rect::from_min_size(pos2(a.min.x + 30.0, y - 4.0), vec2(26.0, 8.0));
        self.p.rect_filled(Rect::from_min_size(pos2(chip.min.x - 1.0, y - 1.0), vec2(a.max.x - chip.min.x + 1.0, 3.0)), 0.0, Color32::WHITE);
        if k > 0.0 {
            self.p.rect_filled(chip, 1.0, self.accent().linear_multiply(0.25 * k));
            self.p.rect_stroke(chip, 1.0, self.stroke(self.accent().linear_multiply(k)), egui::StrokeKind::Inside);
            self.label(chip.center(), "(٢٠٢٠)", 6.0, self.accent().linear_multiply(k), Align2::CENTER_CENTER);
        }
        (a, chip)
    }

    /// The arrow comes down and the citation lands.
    fn mz_citation(&self) {
        let (_, chip) = self.mz_cited_page(self.reveal(0.35, 0.6));
        let k = self.reveal(0.05, 0.35);
        if k > 0.0 {
            self.arrow_down(pos2(chip.center().x, chip.min.y - 14.0), pos2(chip.center().x, chip.min.y - 14.0 + 12.0 * k));
        }
    }

    /// The yellow note slides into the page.
    fn mz_zotero_note(&self) {
        let a = self.page_center();
        self.text_lines(a, false, &[52.0, 44.0], 10.0, self.grey());
        let k = self.reveal(0.2, 0.6);
        let n = Rect::from_min_size(pos2(a.min.x, a.min.y + 22.0), vec2(a.width(), 30.0 * k.max(0.05)));
        if k > 0.0 {
            self.p.rect_filled(n, 1.0, self.yellow().linear_multiply(0.6));
            self.p.rect_stroke(n, 1.0, self.stroke(self.accent()), egui::StrokeKind::Inside);
            if k >= 1.0 {
                self.text_lines(n.shrink(4.0), false, &[48.0, 40.0], 10.0, self.ink());
            }
        }
        self.text_lines(Rect::from_min_size(pos2(a.min.x, a.min.y + 56.0), vec2(a.width(), 20.0)), false, &[48.0], 10.0, self.grey());
    }

    /// The caret jumps from inside the citation to just after it.
    fn mz_past_citation(&self) {
        let (_, chip) = self.mz_cited_page(1.0);
        let k = self.reveal(0.15, 0.5);
        if k < 1.0 {
            self.caret(pos2(chip.center().x, chip.min.y - 1.0), 10.0);
        }
        if k > 0.0 {
            let x0 = chip.center().x + 2.0;
            self.arrow_right(pos2(x0, chip.max.y + 6.0), pos2(x0 + (chip.max.x + 6.0 - x0) * k, chip.max.y + 6.0));
        }
        if k >= 1.0 {
            self.caret(pos2(chip.max.x + 2.0, chip.min.y - 1.0), 10.0);
        }
    }

    /// The bibliography heading, then its hanging-indent entries one by one.
    fn mz_bibliography(&self) {
        let a = self.page_center();
        self.text_lines(a, false, &[52.0, 40.0], 10.0, self.grey());
        let y0 = a.min.y + 26.0;
        let k0 = self.reveal(0.05, 0.25);
        self.line(pos2(a.min.x, y0), pos2(a.min.x + 34.0 * k0, y0), self.accent(), 3.0);
        for i in 0..3 {
            let k = self.mz_step(i + 1, 4);
            if k <= 0.0 {
                continue;
            }
            let y = y0 + 10.0 + i as f32 * 16.0;
            self.line(pos2(a.min.x, y), pos2(a.min.x + a.width() * k, y), self.accent(), 1.5);
            let w = [30.0, 40.0, 24.0].get(i).copied().unwrap_or(30.0);
            self.line(pos2(a.min.x + 8.0, y + 6.0), pos2(a.min.x + 8.0 + w * k, y + 6.0), self.accent(), 1.5);
        }
    }

    /// The refresh arrow turns beside the citation.
    fn mz_zotero_refresh(&self) {
        let (_, chip) = self.mz_cited_page(1.0);
        let c = pos2(chip.max.x + 18.0, chip.center().y - 6.0);
        let turn = if self.phase >= 1.0 { 0.0 } else { self.phase * std::f32::consts::TAU };
        // A ring with a gap and an arrow head, turned by `turn`.
        let r = 7.0;
        let n = 14;
        let (a0, a1) = (-1.2_f32 + turn, 3.6_f32 + turn);
        let pts: Vec<_> = (0..=n).map(|i| a0 + (a1 - a0) * i as f32 / n as f32).map(|ang| c + vec2(ang.cos(), ang.sin()) * r).collect();
        self.p.add(egui::Shape::line(pts, Stroke::new(2.0, self.accent())));
        let tip = c + vec2(a0.cos(), a0.sin()) * r;
        let rot = |v: egui::Vec2| vec2(v.x * turn.cos() - v.y * turn.sin(), v.x * turn.sin() + v.y * turn.cos());
        self.p.add(egui::Shape::convex_polygon(
            vec![tip + rot(vec2(0.0, -4.0)), tip + rot(vec2(4.0, 1.0)), tip + rot(vec2(-3.5, 2.5))],
            self.accent(),
            Stroke::NONE,
        ));
    }

    fn mz_zotero_prefs(&self) {
        let inner = self.dialog("Zotero", 2);
        self.label(pos2(inner.min.x, inner.max.y - 6.0), "APA · ar", 7.0, self.accent(), Align2::LEFT_CENTER);
    }

    fn mz_unlink_citations(&self) {
        self.before_after(
            |d, a| {
                d.paragraph(a, Dir::Ltr, 5, d.grey());
                let y = a.min.y + 14.0;
                let chip = Rect::from_min_size(pos2(a.min.x + 24.0, y - 4.0), vec2(24.0, 8.0));
                d.p.rect_filled(chip, 1.0, Color32::WHITE);
                d.p.rect_filled(chip, 1.0, d.accent().linear_multiply(0.25));
                d.p.rect_stroke(chip, 1.0, d.stroke(d.accent()), egui::StrokeKind::Inside);
            },
            |d, a| {
                d.paragraph(a, Dir::Ltr, 5, d.grey());
                let y = a.min.y + 14.0;
                d.line(pos2(a.min.x + 24.0, y), pos2(a.min.x + 48.0, y), d.ink(), 1.5);
            },
        );
    }

    // ---- Service: classification ----

    /// A page stamped with `word` at its head and its foot: the stamp comes down on the
    /// header first, then on the footer, big and faint on the way, small and firm once landed.
    fn mz_classification(&self, word: &str) {
        let page = self.page(50.0, 6.0, 76.0, 92.0);
        self.text_lines(page.shrink2(vec2(0.0, 14.0)), true, &[56.0, 50.0, 54.0, 36.0], 10.0, self.t.text_dim);
        let w = (word.chars().count() as f32 * 4.6 + 10.0).max(30.0);
        for (i, y) in [page.min.y + 2.0, page.max.y - 2.0].into_iter().enumerate() {
            let land = self.reveal(0.1 + i as f32 * 0.35, 0.4 + i as f32 * 0.35);
            if land <= 0.0 {
                continue;
            }
            let size = 8.0 + (1.0 - land) * 10.0;
            self.label(pos2(page.center().x, y), word, size, self.t.accent.linear_multiply(0.3 + 0.7 * land), Align2::CENTER_CENTER);
            if land >= 1.0 {
                let r = Rect::from_center_size(pos2(page.center().x, y), vec2(w, 11.0));
                self.p.rect_stroke(r, 2.0, Stroke::new(1.0, self.t.accent), egui::StrokeKind::Outside);
            }
        }
    }
}
