//! Insert tab thumbnails: pages and breaks, tables and pictures, links and comments, headers
//! and footers, text objects, equations and symbols. Mostly one page with the inserted thing
//! drawn in the accent colour.

use egui::{Align2, Color32, Rect, Shape, Stroke, pos2, vec2};

use super::Entry;
use super::draw::{Dir, Draw};

pub const ENTRIES: &[Entry] = &[
    Entry::new("insert.coverPage", "Adds a first page with a big title, a subtitle and the author's name.", |d| d.in_cover_page()),
    Entry::new("insert.blankPage", "Adds an empty page right after the page the caret is on.", |d| d.in_blank_page()),
    Entry::new("insert.pageBreak", "Ends the page at the caret; the text after it starts on a new page.", |d| d.in_page_break()),
    Entry::new("insert.table", "Inserts a grid of rows and columns to hold text in cells.", |d| {
        let a = d.page_rl();
        d.table(Rect::from_min_size(pos2(a.min.x, a.min.y + 22.0), vec2(a.width(), 34.0)), 3, 3);
    }),
    Entry::new("insert.picture", "Inserts an image file from your computer into the page.", |d| {
        let a = d.page_rl();
        d.picture(Rect::from_min_size(pos2(a.min.x + 8.0, a.min.y + 20.0), vec2(a.width() - 16.0, 34.0)));
    }),
    Entry::new("insert.shape", "Draws a shape (a rectangle, a circle, an arrow, a star) you can move, size and colour.", |d| d.in_shapes()),
    Entry::new("insert.icon", "Inserts a simple symbol picture (a star, a heart, a tool) from the icon library.", |d| d.in_icon()),
    Entry::new("insert.smartArt", "Inserts a diagram of boxes and arrows: a list, a process, a cycle or a hierarchy.", |d| d.in_smart_art()),
    Entry::new("insert.chart", "Inserts a chart (bars, lines, a pie) drawn from numbers you type.", |d| {
        let a = d.page_rl();
        d.chart(Rect::from_min_size(pos2(a.min.x + 4.0, a.min.y + 20.0), vec2(a.width() - 8.0, 36.0)));
    }),
    Entry::new("ui.dialog:link", "Turns the selected text into a link that opens a web page, a file or a place in this document.", |d| d.in_link()),
    Entry::new("ui.dialog:bookmark", "Marks the selected place in the document with a name, so you can jump or refer to it later.", |d| {
        d.in_bookmark()
    }),
    Entry::new("insert.crossReference", "Inserts a pointer to a heading, table or figure (see page 3) that updates when it moves.", |d| {
        d.in_cross_reference()
    }),
    Entry::new("review.newComment", "Attaches a note to the selected text; it shows in the margin, not in the text.", |d| d.in_comment()),
    Entry::new("insert.header", "Adds text that repeats at the top of every page, such as a title or a stamp.", |d| d.in_header_footer(true)),
    Entry::new("insert.footer", "Adds text that repeats at the bottom of every page, such as a note or a date.", |d| d.in_header_footer(false)),
    Entry::new("insert.header:blank", "A header with one empty line to type into, at the top of every page.", |d| d.in_hf_preset(true, 0)),
    Entry::new("insert.header:blankThree", "A header with three slots: left, centre and right, for a date, a title and a number.", |d| {
        d.in_hf_preset(true, 1)
    }),
    Entry::new("insert.header:title", "A header that shows the document's title on every page.", |d| d.in_hf_preset(true, 2)),
    Entry::new("insert.footer:blank", "A footer with one empty line to type into, at the bottom of every page.", |d| d.in_hf_preset(false, 0)),
    Entry::new("insert.footer:blankThree", "A footer with three slots: left, centre and right.", |d| d.in_hf_preset(false, 1)),
    Entry::new("insert.footer:pageNumber", "A footer with the page number centred at the bottom of every page.", |d| d.in_hf_preset(false, 3)),
    Entry::new("insert.pageNumber", "Numbers every page automatically, at the top or the bottom.", |d| d.in_page_number()),
    Entry::new("insert.textBox", "Inserts a framed box of text you can place anywhere on the page.", |d| {
        let a = d.page_rl();
        d.text_box(Rect::from_min_size(pos2(a.min.x + 6.0, a.min.y + 26.0), vec2(a.width() - 12.0, 30.0)));
    }),
    Entry::new("insert.quickParts", "Saves a piece of text to reuse, and inserts a saved piece with one click.", |d| d.in_quick_parts()),
    Entry::new("insert.wordArt", "Inserts decorative text with a colour fill, an outline and a shadow.", |d| d.in_word_art()),
    Entry::new("insert.dropCap", "Makes the first letter of the paragraph large, dropped over several lines.", |d| d.in_drop_cap()),
    Entry::new("insert.signatureLine", "Inserts a line to sign on, with the signer's name and title beneath it.", |d| d.in_signature_line()),
    Entry::new("insert.dateTime", "Inserts today's date or the current time, in the format you choose.", |d| d.in_date_time()),
    Entry::new("insert.object", "Inserts the contents of another file into the document at the caret.", |d| d.in_object()),
    Entry::new("insert.equation", "Inserts a mathematical formula, built with fractions, roots and symbols.", |d| d.in_equation()),
    Entry::new("insert.symbol", "Inserts a character the keyboard lacks, such as © or ±.", |d| d.in_symbol()),
];

impl Draw<'_> {
    fn in_cover_page(&self) {
        let a = self.page_center();
        self.p.rect_filled(Rect::from_min_size(pos2(a.min.x - 8.0, a.min.y - 8.0), vec2(a.width() + 16.0, 30.0)), 0.0, self.accent());
        let y = a.min.y + 34.0;
        self.line(pos2(a.min.x + 6.0, y), pos2(a.max.x - 6.0, y), self.ink(), 4.0);
        self.line(pos2(a.min.x + 14.0, y + 9.0), pos2(a.max.x - 14.0, y + 9.0), self.ink(), 2.0);
        self.line(pos2(a.min.x + 20.0, y + 32.0), pos2(a.max.x - 20.0, y + 32.0), self.grey(), 1.5);
    }

    fn in_blank_page(&self) {
        let p1 = self.page(16.0, 8.0, 62.0, 88.0);
        self.paragraph(p1, Dir::Rtl, 4, self.grey());
        self.caret(pos2(p1.max.x - 20.0, p1.min.y + 32.0), 10.0);
        self.arrow_right(self.at(80.0, 52.0), self.at(94.0, 52.0));
        let p2 = self.page(98.0, 8.0, 62.0, 88.0);
        self.p.rect_stroke(p2.expand(8.0), 2.0, Stroke::new(1.5, self.accent()), egui::StrokeKind::Inside);
    }

    fn in_page_break(&self) {
        let p1 = self.page(16.0, 8.0, 62.0, 88.0);
        self.text_lines(p1, true, &[46.0, 42.0, 44.0], 10.0, self.grey());
        let y = p1.min.y + 34.0;
        let mut x = p1.min.x - 6.0;
        while x < p1.max.x + 6.0 {
            self.line(pos2(x, y), pos2((x + 3.0).min(p1.max.x + 6.0), y), self.accent(), 1.5);
            x += 5.0;
        }
        self.arrow_right(self.at(80.0, 52.0), self.at(94.0, 52.0));
        let p2 = self.page(98.0, 8.0, 62.0, 88.0);
        self.text_lines(p2, true, &[44.0, 40.0, 46.0, 24.0], 10.0, self.accent());
    }

    fn in_shapes(&self) {
        let a = self.page_rl();
        self.shape(Rect::from_min_size(pos2(a.min.x + 2.0, a.min.y + 24.0), vec2(24.0, 18.0)));
        let c = pos2(a.max.x - 12.0, a.min.y + 33.0);
        self.p.circle_filled(c, 9.0, self.accent().linear_multiply(0.35));
        self.p.circle_stroke(c, 9.0, Stroke::new(1.5, self.accent()));
        let s = pos2(a.center().x, a.min.y + 58.0);
        let tri = vec![s + vec2(0.0, -8.0), s + vec2(9.0, 7.0), s + vec2(-9.0, 7.0)];
        self.p.add(Shape::convex_polygon(tri, self.accent().linear_multiply(0.35), Stroke::new(1.5, self.accent())));
    }

    /// A simple five-point star, the kind an icon library holds.
    fn in_icon(&self) {
        let a = self.page_rl();
        let c = pos2(a.center().x, a.min.y + 36.0);
        let pts: Vec<egui::Pos2> = (0..10)
            .map(|i| {
                let r = if i % 2 == 0 { 16.0 } else { 7.0 };
                let ang = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
                c + vec2(ang.cos() * r, ang.sin() * r)
            })
            .collect();
        self.p.rect_filled(Rect::from_center_size(c, vec2(38.0, 38.0)), 0.0, Color32::WHITE);
        self.p.add(Shape::closed_line(pts, Stroke::new(2.0, self.accent())));
    }

    /// Three boxes joined by arrows (a process diagram).
    fn in_smart_art(&self) {
        let a = self.page_wide();
        self.text_lines(a, true, &[90.0], 10.0, self.grey());
        for i in 0..3 {
            let x = a.min.x + 4.0 + i as f32 * 36.0;
            let r = Rect::from_min_size(pos2(x, a.min.y + 22.0), vec2(26.0, 22.0));
            self.shape(r);
            if i < 2 {
                self.arrow_right(pos2(r.max.x + 1.0, r.center().y), pos2(r.max.x + 9.0, r.center().y));
            }
        }
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 52.0), a.max), true, &[96.0, 60.0], 10.0, self.grey());
    }

    /// Blue underlined text on the page, and the dialog asking for the address.
    fn in_link(&self) {
        let a = self.page_rl();
        let y = a.min.y + 24.0;
        let blue = Color32::from_rgb(30, 90, 200);
        self.p.rect_filled(Rect::from_min_max(pos2(a.max.x - 40.0, y - 4.0), pos2(a.max.x - 10.0, y + 4.0)), 0.0, Color32::WHITE);
        self.line(pos2(a.max.x - 40.0, y), pos2(a.max.x - 10.0, y), blue, 1.5);
        self.line(pos2(a.max.x - 40.0, y + 3.0), pos2(a.max.x - 10.0, y + 3.0), blue, 1.0);
        let inner = self.dialog("Link", 1);
        self.label(pos2(inner.min.x + 38.0, inner.min.y + 5.0), "https://…", 6.5, blue, Align2::LEFT_CENTER);
        self.button(pos2(inner.center().x, inner.max.y - 12.0), "OK", false);
    }

    /// A ribbon-like bookmark flag in the margin next to the marked line.
    fn in_bookmark(&self) {
        let a = self.page_rl();
        let y = a.min.y + 24.0;
        self.selection(Rect::from_min_max(pos2(a.max.x - 36.0, y - 5.0), pos2(a.max.x, y + 5.0)));
        self.line(pos2(a.max.x - 34.0, y), pos2(a.max.x, y), self.ink(), 1.5);
        let x = a.max.x + 6.0;
        let flag = vec![pos2(x - 4.0, y - 10.0), pos2(x + 4.0, y - 10.0), pos2(x + 4.0, y + 8.0), pos2(x, y + 3.0), pos2(x - 4.0, y + 8.0)];
        self.p.add(Shape::convex_polygon(flag, self.accent(), Stroke::NONE));
        self.label(pos2(a.center().x, a.max.y - 2.0), "اسم", 7.0, self.accent(), Align2::CENTER_CENTER);
    }

    /// A «see …» chip low on the page pointing up to a heading.
    fn in_cross_reference(&self) {
        let a = self.page_center();
        self.heading_then_text(a, Dir::Rtl, 2, self.ink());
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 34.0), a.max), true, &[56.0, 52.0, 30.0], 10.0, self.grey());
        let y = a.min.y + 66.0;
        self.field(pos2(a.min.x + 10.0, y - 3.5), 30.0);
        self.arrow_up(pos2(a.min.x + 6.0, y), pos2(a.min.x + 6.0, a.min.y + 8.0));
    }

    /// Selected words with a note balloon beside the page, joined by a thin line.
    fn in_comment(&self) {
        let page = self.page(28.0, 6.0, 76.0, 92.0);
        self.paragraph(page, Dir::Rtl, 6, self.grey());
        let y = page.min.y + 24.0;
        self.highlight(Rect::from_min_max(pos2(page.max.x - 38.0, y - 4.0), pos2(page.max.x - 10.0, y + 4.0)));
        self.line(pos2(page.max.x - 38.0, y), pos2(page.max.x - 10.0, y), self.ink(), 1.5);
        let note = Rect::from_min_size(self.at(116.0, 12.0), vec2(52.0, 30.0));
        self.p.rect_filled(note, 3.0, self.yellow().linear_multiply(0.5));
        self.p.rect_stroke(note, 3.0, self.stroke(self.accent()), egui::StrokeKind::Inside);
        self.text_lines(note.shrink(5.0), true, &[36.0, 28.0], 8.0, self.ink());
        self.line(pos2(page.max.x - 10.0, y), note.left_center(), self.accent(), 1.0);
    }

    /// An accent band at the top (`header`) or bottom of the page, repeated on a second page.
    fn in_header_footer(&self, header: bool) {
        for x in [26.0, 94.0] {
            let a = self.page(x, 8.0, 56.0, 88.0);
            let body = if header { Rect::from_min_max(pos2(a.min.x, a.min.y + 14.0), a.max) } else { a };
            self.paragraph(body, Dir::Rtl, 5, self.grey());
            let y = if header { a.min.y - 2.0 } else { a.max.y + 2.0 };
            self.line(pos2(a.min.x, y), pos2(a.max.x, y), self.accent(), 2.5);
            let sep = if header { y + 5.0 } else { y - 5.0 };
            self.line(pos2(a.min.x, sep), pos2(a.max.x, sep), self.t.border, 1.0);
        }
    }

    /// One page with the preset drawn in its header or footer band: 0 blank line, 1 three
    /// slots, 2 the title, 3 the page number.
    fn in_hf_preset(&self, header: bool, preset: u8) {
        let a = self.page(50.0, 6.0, 76.0, 92.0);
        let body =
            if header { Rect::from_min_max(pos2(a.min.x, a.min.y + 16.0), a.max) } else { Rect::from_min_max(a.min, pos2(a.max.x, a.max.y - 16.0)) };
        self.paragraph(body, Dir::Rtl, 6, self.grey());
        let y = if header { a.min.y + 2.0 } else { a.max.y - 4.0 };
        let band = Rect::from_min_max(pos2(a.min.x - 4.0, y - 6.0), pos2(a.max.x + 4.0, y + 6.0));
        self.p.rect_filled(band, 1.0, self.accent().linear_multiply(0.12));
        let show = self.reveal(0.2, 0.6);
        match preset {
            0 => self.caret(pos2(a.max.x - 2.0, y - 4.0), 8.0),
            1 => {
                for x in [a.min.x + 10.0, a.center().x, a.max.x - 10.0] {
                    self.line(pos2(x - 8.0 * show, y), pos2(x + 8.0 * show, y), self.accent(), 2.0);
                }
            }
            2 => self.label(pos2(a.center().x, y), "عنوان المستند", 7.0, self.accent().linear_multiply(show.max(0.2)), Align2::CENTER_CENTER),
            _ => self.label(pos2(a.center().x, y), "٣", 8.0, self.accent().linear_multiply(show.max(0.2)), Align2::CENTER_CENTER),
        }
    }

    fn in_page_number(&self) {
        for (i, x) in [20.0, 64.0, 108.0].iter().enumerate() {
            let a = self.page(*x, 10.0, 48.0, 80.0);
            self.text_lines(a, true, &[32.0, 28.0, 30.0, 18.0], 10.0, self.grey());
            self.badge(pos2(a.center().x, a.max.y - 2.0), ["١", "٢", "٣"][i], self.accent());
        }
    }

    /// A shelf of saved pieces; one goes onto the page.
    fn in_quick_parts(&self) {
        for i in 0..3 {
            let y = 22.0 + i as f32 * 20.0;
            let r = Rect::from_min_size(self.at(8.0, y), vec2(48.0, 14.0));
            self.p.rect_filled(r, 2.0, Color32::WHITE);
            self.p.rect_stroke(r, 2.0, self.stroke(if i == 1 { self.accent() } else { self.t.border_strong }), egui::StrokeKind::Inside);
            self.text_lines(r.shrink2(vec2(4.0, 3.0)), true, &[30.0], 6.0, self.grey());
        }
        self.arrow_right(self.at(62.0, 49.0), self.at(82.0, 49.0));
        let a = self.page(90.0, 8.0, 70.0, 88.0);
        self.text_lines(a, true, &[50.0, 44.0], 10.0, self.grey());
        let r = Rect::from_min_size(pos2(a.min.x + 2.0, a.min.y + 24.0), vec2(50.0, 20.0));
        self.p.rect_stroke(r, 2.0, self.stroke(self.accent()), egui::StrokeKind::Outside);
        self.text_lines(r.shrink2(vec2(2.0, 2.0)), true, &[40.0, 30.0], 8.0, self.accent());
    }

    fn in_word_art(&self) {
        let a = self.page_center();
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 44.0), a.max), true, &[56.0, 50.0, 30.0], 10.0, self.grey());
        let c = pos2(a.center().x, a.min.y + 20.0);
        self.glyph(c + vec2(3.0, 3.0), "A", 34.0, self.grey());
        for off in [vec2(-1.5, 0.0), vec2(1.5, 0.0), vec2(0.0, -1.5), vec2(0.0, 1.5)] {
            self.glyph(c + off, "A", 34.0, self.ink());
        }
        self.glyph(c, "A", 34.0, self.accent());
    }

    fn in_drop_cap(&self) {
        let a = self.page_center();
        let big = Rect::from_min_size(pos2(a.max.x - 22.0, a.min.y), vec2(22.0, 28.0));
        self.glyph(big.center(), "ك", 26.0, self.accent());
        self.text_lines(Rect::from_min_max(a.min, pos2(a.max.x - 24.0, a.max.y)), true, &[36.0, 34.0, 36.0], 10.0, self.grey());
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 30.0), a.max), true, &[58.0, 54.0, 56.0, 30.0], 10.0, self.grey());
    }

    fn in_signature_line(&self) {
        let a = self.page_center();
        self.paragraph(a, Dir::Rtl, 3, self.grey());
        let y = a.min.y + 44.0;
        self.label(pos2(a.max.x, y), "X", 9.0, self.accent(), Align2::RIGHT_CENTER);
        self.line(pos2(a.min.x + 4.0, y + 4.0), pos2(a.max.x - 10.0, y + 4.0), self.ink(), 1.2);
        self.text_lines(Rect::from_min_max(pos2(a.min.x, y + 8.0), a.max), true, &[30.0, 22.0], 8.0, self.grey());
    }

    fn in_date_time(&self) {
        let a = self.page_rl();
        let y = a.min.y + 24.0;
        self.p.rect_filled(Rect::from_min_max(pos2(a.max.x - 46.0, y - 5.0), pos2(a.max.x + 2.0, y + 5.0)), 1.0, Color32::WHITE);
        self.label(pos2(a.max.x, y), "٢٠٢٦/١٠/١٧", 7.5, self.accent(), Align2::RIGHT_CENTER);
        // A small clock.
        let c = self.at(26.0, 72.0);
        self.p.circle_stroke(c, 11.0, Stroke::new(2.0, self.accent()));
        self.line(c, c + vec2(0.0, -7.0), self.accent(), 1.5);
        self.line(c, c + vec2(5.0, 2.0), self.accent(), 1.5);
    }

    /// A file (a page with a folded corner) whose contents land on the page.
    fn in_object(&self) {
        let f = Rect::from_min_size(self.at(14.0, 24.0), vec2(40.0, 50.0));
        self.p.rect_filled(f, 1.0, Color32::WHITE);
        self.p.rect_stroke(f, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        let fold = vec![pos2(f.max.x - 12.0, f.min.y), pos2(f.max.x, f.min.y + 12.0), pos2(f.max.x - 12.0, f.min.y + 12.0)];
        self.p.add(Shape::convex_polygon(fold, self.t.hover, Stroke::new(1.0, self.t.border_strong)));
        self.text_lines(f.shrink2(vec2(5.0, 16.0)), true, &[28.0, 24.0, 26.0], 8.0, self.accent());
        self.arrow_right(self.at(60.0, 50.0), self.at(82.0, 50.0));
        let a = self.page(90.0, 8.0, 70.0, 88.0);
        self.text_lines(a, true, &[50.0, 44.0], 10.0, self.grey());
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 22.0), a.max), true, &[48.0, 52.0, 40.0], 10.0, self.accent());
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 54.0), a.max), true, &[46.0, 24.0], 10.0, self.grey());
    }

    fn in_equation(&self) {
        let a = self.page_rl();
        let box_ = Rect::from_min_max(pos2(a.min.x + 2.0, a.min.y + 18.0), pos2(a.max.x - 2.0, a.min.y + 48.0));
        self.p.rect_filled(box_, 2.0, Color32::WHITE);
        self.p.rect_stroke(box_, 2.0, self.stroke(self.t.border), egui::StrokeKind::Inside);
        let c = box_.center();
        self.glyph(c + vec2(-16.0, 0.0), "∑", 20.0, self.accent());
        self.glyph(c + vec2(6.0, -6.0), "x²", 9.0, self.ink());
        self.line(c + vec2(-1.0, 0.0), c + vec2(14.0, 0.0), self.ink(), 1.2);
        self.glyph(c + vec2(6.0, 7.0), "n", 9.0, self.ink());
    }

    fn in_symbol(&self) {
        let a = self.page_rl();
        let y = a.min.y + 24.0;
        self.p.rect_filled(Rect::from_min_max(pos2(a.max.x - 30.0, y - 6.0), pos2(a.max.x - 14.0, y + 6.0)), 0.0, Color32::WHITE);
        self.glyph(pos2(a.max.x - 22.0, y), "Ω", 12.0, self.accent());
        // A small palette of symbols to pick from.
        let g = Rect::from_min_size(self.at(10.0, 56.0), vec2(48.0, 32.0));
        self.p.rect_filled(g, 2.0, self.t.menu);
        self.grid(g, 2, 3, self.t.border);
        for (i, s) in ["©", "±", "€", "→", "√", "∞"].iter().enumerate() {
            let cell = self.cell(g, 2, 3, i / 3, i % 3);
            self.label(cell.center(), s, 8.0, if i == 1 { self.accent() } else { self.ink() }, Align2::CENTER_CENTER);
        }
    }
}
