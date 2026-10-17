//! Help, Developer and Zotero tabs thumbnails: the help dialogs, the form controls a reader
//! fills in (content controls) and the Zotero citation tools.

use egui::{Align2, Color32, Rect, Stroke, pos2, vec2};

use super::Entry;
use super::draw::{Dir, Draw};

pub const ENTRIES: &[Entry] = &[
    // Help.
    Entry::new("ui.dialog:about", "Shows which version of WordCraft AR this is and where to get help.", |d| d.mz_about()),
    Entry::new("ui.discord", "Opens the community chat in your browser, where you can ask questions and report problems.", |d| d.mz_discord()),
    Entry::new("ui.dialog:commands", "Opens a searchable list of every command, so you can find one by name and run it.", |d| d.mz_commands()),
    // Developer › Controls.
    Entry::new("developer.richText", "Inserts a box where the reader can type formatted text, with bold, lists or pictures.", |d| {
        d.mz_control_text(true)
    }),
    Entry::new("developer.plainText", "Inserts a box where the reader can type plain text only, all in one style.", |d| d.mz_control_text(false)),
    Entry::new("developer.picture", "Inserts a box where the reader can place a picture.", |d| d.mz_control_picture()),
    Entry::new("developer.buildingBlock", "Inserts a box where the reader picks a ready-made block of text from a gallery.", |d| {
        d.mz_control_gallery()
    }),
    Entry::new("developer.checkBox", "Inserts a small box the reader can tick or untick.", |d| d.mz_control_check_box()),
    Entry::new("developer.comboBox", "Inserts a list the reader can choose from or type their own entry into.", |d| d.mz_control_list(true)),
    Entry::new("developer.dropDown", "Inserts a list the reader must choose one entry from, without typing.", |d| d.mz_control_list(false)),
    Entry::new("developer.datePicker", "Inserts a box where the reader picks a date from a small calendar.", |d| d.mz_control_date()),
    Entry::new("developer.repeatingSection", "Inserts a section the reader can copy as many times as needed, like extra rows.", |d| {
        d.mz_control_repeating()
    }),
    Entry::new("developer.designMode", "Shows the edges and titles of every form box so you can edit the boxes themselves.", |d| d.mz_design_mode()),
    Entry::new("developer.properties", "Opens a box to set a form box's title, tag, list entries and locks.", |d| d.mz_control_properties()),
    // Zotero.
    Entry::new("ui.zotero.addEditCitation", "Inserts a citation from your Zotero library at the caret, or edits the one under it.", |d| {
        d.mz_citation()
    }),
    Entry::new("ui.zotero.addNote", "Inserts one of your Zotero notes as text at the caret.", |d| d.mz_zotero_note()),
    Entry::new("caret.pastCitation", "Moves the caret to just after the citation it is in, so you can keep typing.", |d| d.mz_past_citation()),
    Entry::new("ui.zotero.addEditBibliography", "Inserts the list of every cited source at the caret, or updates the existing list.", |d| {
        d.mz_bibliography()
    }),
    Entry::new("ui.zotero.refresh", "Updates every citation and the bibliography after changes in your Zotero library.", |d| d.mz_zotero_refresh()),
    Entry::new("ui.zotero.setDocPrefs", "Opens Zotero's settings for this document: the citation style and its language.", |d| d.mz_zotero_prefs()),
    Entry::new("ui.zotero.removeCodes", "Turns the citations and bibliography into plain text that Zotero no longer updates.", |d| {
        d.mz_unlink_citations()
    }),
];

impl Draw<'_> {
    // ---- Help ----

    fn mz_about(&self) {
        let c = self.at(50.0, 52.0);
        self.p.circle_filled(c, 24.0, self.accent());
        self.glyph(c, "?", 30.0, Color32::WHITE);
        let a = Rect::from_min_size(self.at(90.0, 30.0), vec2(76.0, 50.0));
        self.line(pos2(a.min.x, a.min.y + 4.0), pos2(a.min.x + 56.0, a.min.y + 4.0), self.ink(), 3.0);
        self.text_lines(Rect::from_min_size(a.min + vec2(0.0, 10.0), a.size()), false, &[40.0, 60.0, 48.0], 10.0, self.grey());
    }

    fn mz_discord(&self) {
        // A chat bubble with a few lines of talk, and a smaller reply bubble.
        let b = Rect::from_min_size(self.at(22.0, 14.0), vec2(96.0, 50.0));
        self.p.rect_filled(b, 8.0, self.accent());
        self.p.add(egui::Shape::convex_polygon(
            vec![pos2(b.min.x + 16.0, b.max.y - 1.0), pos2(b.min.x + 10.0, b.max.y + 12.0), pos2(b.min.x + 30.0, b.max.y - 1.0)],
            self.accent(),
            Stroke::NONE,
        ));
        self.text_lines(b.shrink(10.0), false, &[60.0, 72.0, 44.0], 11.0, Color32::WHITE);
        let r = Rect::from_min_size(self.at(84.0, 66.0), vec2(72.0, 30.0));
        self.p.rect_filled(r, 8.0, self.t.hover);
        self.p.rect_stroke(r, 8.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        self.text_lines(r.shrink(8.0), false, &[48.0, 30.0], 10.0, self.grey());
    }

    fn mz_commands(&self) {
        let inner = self.dialog("Commands", 0);
        // A search box with a magnifier, then the matching commands.
        let f = Rect::from_min_size(inner.min, vec2(inner.width(), 11.0));
        self.p.rect_filled(f, 2.0, self.t.input);
        self.p.rect_stroke(f, 2.0, self.stroke(self.t.input_border), egui::StrokeKind::Inside);
        self.p.circle_stroke(pos2(f.min.x + 6.0, f.center().y), 3.0, Stroke::new(1.5, self.accent()));
        self.line(pos2(f.min.x + 8.0, f.center().y + 2.0), pos2(f.min.x + 11.0, f.center().y + 5.0), self.accent(), 1.5);
        self.line(pos2(f.min.x + 16.0, f.center().y), pos2(f.min.x + 40.0, f.center().y), self.ink(), 1.5);
        self.caret(pos2(f.min.x + 42.0, f.min.y + 2.0), 7.0);
        for i in 0..3 {
            let y = inner.min.y + 20.0 + i as f32 * 11.0;
            let row = Rect::from_min_size(pos2(inner.min.x, y - 4.0), vec2(inner.width(), 9.0));
            if i == 0 {
                self.selection(row);
            }
            self.line(pos2(row.min.x + 3.0, y), pos2(row.min.x + 34.0 + i as f32 * 10.0, y), self.grey(), 1.5);
        }
    }

    // ---- Developer: content controls ----

    /// A page with ordinary text and one control box at `y` of height `h`; returns the box.
    fn mz_control_box(&self, h: f32) -> Rect {
        let a = self.page_center();
        self.text_lines(a, false, &[52.0, 44.0], 10.0, self.grey());
        let b = Rect::from_min_size(pos2(a.min.x, a.min.y + 22.0), vec2(a.width(), h));
        self.p.rect_filled(b, 1.0, Color32::WHITE);
        self.dashed_rect(b, self.accent());
        // The control's little handle (its title tab) at the top-left corner.
        self.p.rect_filled(Rect::from_min_size(pos2(b.min.x - 1.0, b.min.y - 5.0), vec2(10.0, 5.0)), 1.0, self.accent());
        let below = Rect::from_min_size(pos2(a.min.x, b.max.y + 4.0), vec2(a.width(), a.max.y - b.max.y - 4.0));
        if below.height() > 8.0 {
            self.text_lines(below, false, &[48.0, 30.0], 10.0, self.grey());
        }
        b
    }

    fn mz_control_text(&self, rich: bool) {
        let b = self.mz_control_box(28.0);
        let inner = b.shrink(4.0);
        if rich {
            self.line(pos2(inner.min.x, inner.min.y + 3.0), pos2(inner.min.x + 30.0, inner.min.y + 3.0), self.ink(), 3.0);
            self.label(pos2(inner.min.x + 2.0, inner.min.y + 14.0), "•", 8.0, self.accent(), Align2::CENTER_CENTER);
            self.line(pos2(inner.min.x + 8.0, inner.min.y + 14.0), pos2(inner.max.x, inner.min.y + 14.0), self.grey(), 1.5);
        } else {
            self.text_lines(inner, false, &[inner.width(), inner.width() * 0.6], 10.0, self.grey());
        }
    }

    fn mz_control_picture(&self) {
        let b = self.mz_control_box(34.0);
        self.picture(b.shrink(4.0));
    }

    fn mz_control_gallery(&self) {
        let b = self.mz_control_box(30.0);
        // Four tiles to pick from: little pages with a line or two each.
        let tw = (b.width() - 14.0) / 2.0;
        for (i, (dx, dy)) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)].iter().enumerate() {
            let t = Rect::from_min_size(pos2(b.min.x + 4.0 + dx * (tw + 6.0), b.min.y + 3.0 + dy * 13.0), vec2(tw, 11.0));
            self.p.rect_filled(t, 1.0, if i == 0 { self.accent().linear_multiply(0.3) } else { self.t.hover });
            self.line(pos2(t.min.x + 2.0, t.center().y), pos2(t.max.x - 4.0, t.center().y), self.grey(), 1.5);
        }
    }

    fn mz_control_check_box(&self) {
        let b = self.mz_control_box(16.0);
        let c = pos2(b.min.x + 10.0, b.center().y);
        let box_r = Rect::from_center_size(c, vec2(10.0, 10.0));
        self.p.rect_filled(box_r, 1.0, Color32::WHITE);
        self.p.rect_stroke(box_r, 1.0, Stroke::new(1.5, self.accent()), egui::StrokeKind::Inside);
        self.line(c + vec2(-3.5, 0.0), c + vec2(-1.0, 3.0), self.accent(), 2.0);
        self.line(c + vec2(-1.0, 3.0), c + vec2(4.0, -3.5), self.accent(), 2.0);
        self.line(pos2(b.min.x + 20.0, c.y), pos2(b.max.x - 4.0, c.y), self.grey(), 1.5);
    }

    /// A small "open the list" triangle centred at `c`.
    fn mz_drop_arrow(&self, c: egui::Pos2) {
        self.p.add(egui::Shape::convex_polygon(vec![c + vec2(-3.5, -2.0), c + vec2(3.5, -2.0), c + vec2(0.0, 2.5)], self.accent(), Stroke::NONE));
    }

    /// A list field with its open-list triangle and the open list below; a combo box also has a caret in the
    /// field (you can type), a drop-down has one entry chosen.
    fn mz_control_list(&self, typable: bool) {
        let b = self.mz_control_box(14.0);
        let field = b.shrink(2.0);
        self.p.rect_filled(field, 1.0, self.t.input);
        self.mz_drop_arrow(pos2(field.max.x - 6.0, field.center().y));
        if typable {
            self.line(pos2(field.min.x + 3.0, field.center().y), pos2(field.min.x + 22.0, field.center().y), self.ink(), 1.5);
            self.caret(pos2(field.min.x + 24.0, field.min.y + 1.5), 7.0);
        } else {
            self.line(pos2(field.min.x + 3.0, field.center().y), pos2(field.min.x + 30.0, field.center().y), self.grey(), 1.5);
        }
        // The open list, overlapping the text below.
        let list = Rect::from_min_size(pos2(b.min.x, b.max.y + 1.0), vec2(b.width(), 30.0));
        self.p.rect_filled(list, 1.0, self.t.menu);
        self.p.rect_stroke(list, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        for i in 0..3 {
            let y = list.min.y + 5.0 + i as f32 * 10.0;
            let row = Rect::from_min_size(pos2(list.min.x + 1.0, y - 4.0), vec2(list.width() - 2.0, 9.0));
            if !typable && i == 1 {
                self.selection(row);
            }
            self.line(pos2(row.min.x + 3.0, y), pos2(row.min.x + 20.0 + i as f32 * 8.0, y), self.grey(), 1.5);
        }
    }

    fn mz_control_date(&self) {
        let b = self.mz_control_box(14.0);
        let field = b.shrink(2.0);
        self.p.rect_filled(field, 1.0, self.t.input);
        self.label(pos2(field.min.x + 3.0, field.center().y), "١٧/١٠/٢٠٢٦", 7.0, self.ink(), Align2::LEFT_CENTER);
        self.mz_drop_arrow(pos2(field.max.x - 6.0, field.center().y));
        // A little calendar below: a 4 × 5 grid with one day picked.
        let cal = Rect::from_min_size(pos2(b.min.x, b.max.y + 1.0), vec2(b.width(), 32.0));
        self.p.rect_filled(cal, 1.0, self.t.menu);
        self.p.rect_stroke(cal, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        self.p.rect_filled(Rect::from_min_size(cal.min, vec2(cal.width(), 7.0)), 1.0, self.t.hover);
        let days = Rect::from_min_max(pos2(cal.min.x + 2.0, cal.min.y + 9.0), pos2(cal.max.x - 2.0, cal.max.y - 2.0));
        for row in 0..4 {
            for col in 0..5 {
                let c = self.cell(days, 4, 5, row, col).center();
                if row == 2 && col == 2 {
                    self.p.circle_filled(c, 3.0, self.accent());
                } else {
                    self.p.circle_filled(c, 1.2, self.grey());
                }
            }
        }
    }

    fn mz_control_repeating(&self) {
        let a = self.page_center();
        self.text_lines(a, false, &[52.0], 10.0, self.grey());
        for i in 0..2 {
            let b = Rect::from_min_size(pos2(a.min.x, a.min.y + 12.0 + i as f32 * 28.0), vec2(a.width(), 22.0));
            self.p.rect_filled(b, 1.0, Color32::WHITE);
            self.dashed_rect(b, self.accent());
            self.text_lines(b.shrink(4.0), false, &[36.0, 26.0], 8.0, self.grey());
        }
        // The "add another" handle below the last copy.
        let c = pos2(a.max.x - 6.0, a.min.y + 12.0 + 2.0 * 28.0 + 2.0);
        self.p.circle_filled(c, 6.0, self.accent());
        self.glyph(c, "+", 10.0, Color32::WHITE);
    }

    fn mz_design_mode(&self) {
        let a = self.page_center();
        self.text_lines(a, false, &[52.0], 10.0, self.grey());
        for (i, h) in [14.0, 22.0].iter().enumerate() {
            let b = Rect::from_min_size(pos2(a.min.x, a.min.y + 14.0 + i as f32 * 30.0), vec2(a.width(), *h));
            self.p.rect_filled(b, 1.0, Color32::WHITE);
            self.p.rect_stroke(b, 1.0, Stroke::new(1.0, self.accent()), egui::StrokeKind::Inside);
            // The title tab at each end, as design mode shows them.
            self.p.rect_filled(Rect::from_min_size(pos2(b.min.x, b.min.y - 5.0), vec2(16.0, 5.0)), 1.0, self.accent());
            self.p.rect_filled(Rect::from_min_size(pos2(b.max.x - 6.0, b.min.y), vec2(6.0, b.height())), 0.0, self.accent().linear_multiply(0.4));
            self.text_lines(b.shrink(3.0), false, &[30.0, 20.0], 8.0, self.grey());
        }
        self.pencil(pos2(a.max.x - 4.0, a.min.y + 18.0), pos2(a.max.x + 14.0, a.min.y - 4.0));
    }

    fn mz_control_properties(&self) {
        self.dialog("Properties", 3);
    }

    // ---- Zotero ----

    /// A page of text with a citation chip after the second line's words.
    fn mz_cited_page(&self) -> (Rect, Rect) {
        let a = self.page_lr();
        let y = a.min.y + 14.0;
        let chip = Rect::from_min_size(pos2(a.min.x + 30.0, y - 4.0), vec2(26.0, 8.0));
        self.p.rect_filled(Rect::from_min_size(pos2(chip.min.x - 1.0, y - 1.0), vec2(a.max.x - chip.min.x + 1.0, 3.0)), 0.0, Color32::WHITE);
        self.p.rect_filled(chip, 1.0, self.accent().linear_multiply(0.25));
        self.p.rect_stroke(chip, 1.0, self.stroke(self.accent()), egui::StrokeKind::Inside);
        self.label(chip.center(), "(٢٠٢٠)", 6.0, self.accent(), Align2::CENTER_CENTER);
        (a, chip)
    }

    fn mz_citation(&self) {
        let (_, chip) = self.mz_cited_page();
        self.arrow_down(pos2(chip.center().x, chip.min.y - 14.0), pos2(chip.center().x, chip.min.y - 2.0));
    }

    fn mz_zotero_note(&self) {
        let a = self.page_center();
        self.text_lines(a, false, &[52.0, 44.0], 10.0, self.grey());
        let n = Rect::from_min_size(pos2(a.min.x, a.min.y + 22.0), vec2(a.width(), 30.0));
        self.p.rect_filled(n, 1.0, self.yellow().linear_multiply(0.6));
        self.p.rect_stroke(n, 1.0, self.stroke(self.accent()), egui::StrokeKind::Inside);
        self.text_lines(n.shrink(4.0), false, &[48.0, 40.0], 10.0, self.ink());
        self.text_lines(Rect::from_min_size(pos2(a.min.x, n.max.y + 4.0), vec2(a.width(), 20.0)), false, &[48.0], 10.0, self.grey());
    }

    fn mz_past_citation(&self) {
        let (_, chip) = self.mz_cited_page();
        self.caret(pos2(chip.center().x, chip.min.y - 1.0), 10.0);
        self.arrow_right(pos2(chip.center().x + 2.0, chip.max.y + 6.0), pos2(chip.max.x + 6.0, chip.max.y + 6.0));
        self.caret(pos2(chip.max.x + 2.0, chip.min.y - 1.0), 10.0);
    }

    fn mz_bibliography(&self) {
        let a = self.page_center();
        self.text_lines(a, false, &[52.0, 40.0], 10.0, self.grey());
        let y0 = a.min.y + 26.0;
        self.line(pos2(a.min.x, y0), pos2(a.min.x + 34.0, y0), self.accent(), 3.0);
        // Hanging-indent entries: a full first line, a shorter indented second.
        for i in 0..3 {
            let y = y0 + 10.0 + i as f32 * 16.0;
            self.line(pos2(a.min.x, y), pos2(a.max.x, y), self.accent(), 1.5);
            self.line(
                pos2(a.min.x + 8.0, y + 6.0),
                pos2(a.min.x + 8.0 + [30.0, 40.0, 24.0].get(i).copied().unwrap_or(30.0), y + 6.0),
                self.accent(),
                1.5,
            );
        }
    }

    fn mz_zotero_refresh(&self) {
        let (_, chip) = self.mz_cited_page();
        let c = pos2(chip.max.x + 18.0, chip.center().y - 6.0);
        self.glyph(c, "↻", 18.0, self.accent());
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
}
