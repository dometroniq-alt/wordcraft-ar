//! Insert tab thumbnails: pages and breaks, tables and pictures, links and comments, headers
//! and footers, text objects, equations and symbols — plus the items of the tab's menus (quick
//! tables, the shapes, the header, footer and page-number presets, the dialogs). Mostly one
//! page with the inserted thing drawn in the accent colour, and each a little film: the thing
//! grows, slides or lands on the page.

use egui::{Align2, Color32, Pos2, Rect, Shape, Stroke, pos2, vec2};

use super::Entry;
use super::draw::{Dir, Draw};

pub const ENTRIES: &[Entry] = &[
    Entry::new(
        "insert.coverPage",
        "Adds a front page before everything else, with a big title, a line under it and a place for your name. Example: a report gets a cover that says «التقرير السنوي» with your name beneath it.",
        |d| d.in_cover_page(),
    ),
    Entry::new(
        "insert.blankPage",
        "Puts a whole empty page right after the page you are on; the rest of your text is not changed. Example: you need a free page for a chart, so an empty page appears after page ٢.",
        |d| d.in_blank_page(),
    ),
    Entry::new(
        "insert.pageBreak",
        "Ends the page where the cursor is; whatever comes after it starts at the top of the next page. Example: the cursor is after «الخلاصة» and the next heading jumps to a new page.",
        |d| d.in_page_break(),
    ),
    Entry::new(
        "insert.table",
        "Puts a grid of rows and columns on the page, and you type into its cells. Example: ٣ rows by ٣ columns for a list of names, ranks and dates.",
        |d| {
            let a = d.page_rl();
            let r = Rect::from_min_size(pos2(a.min.x, a.min.y + 22.0), vec2(a.width(), 34.0));
            let k = d.reveal(0.2, 0.7);
            d.in_clipped(Rect::from_min_max(r.min - vec2(2.0, 2.0), pos2(r.max.x + 2.0, r.min.y + r.height() * k + 2.0)), |d| d.table(r, 3, 3));
        },
    ),
    Entry::new(
        "insert.picture",
        "Puts a photo or picture file from your computer onto the page at the cursor. Example: choose the unit's logo file and it appears in the text.",
        |d| {
            let a = d.page_rl();
            let r = Rect::from_min_size(pos2(a.min.x + 8.0, a.min.y + 20.0), vec2(a.width() - 16.0, 34.0));
            d.picture(Draw::in_scaled(r, 0.3 + 0.7 * d.reveal(0.2, 0.6)));
        },
    ),
    Entry::new(
        "insert.shape",
        "Draws a shape on the page (a rectangle, a circle, an arrow, a star) that you can drag, resize and colour. Example: a red arrow pointing at a line in a chart.",
        |d| d.in_shapes(),
    ),
    Entry::new(
        "insert.icon",
        "Puts a small simple picture (a star, a heart, a tool, a flag) from the built-in icon library onto the page. Example: a star icon next to a heading.",
        |d| d.in_icon(),
    ),
    Entry::new(
        "insert.smartArt",
        "Puts a ready-made diagram of boxes and arrows on the page: a list, a process, a cycle or an organisation chart. Example: three boxes joined by arrows for the steps of a procedure.",
        |d| d.in_smart_art(),
    ),
    Entry::new(
        "insert.chart",
        "Draws a chart (bars, lines or a pie) from numbers you type into a small table. Example: five bars showing the number of patients each month.",
        |d| {
            let a = d.page_rl();
            d.in_chart_bars(Rect::from_min_size(pos2(a.min.x + 4.0, a.min.y + 20.0), vec2(a.width() - 8.0, 36.0)), d.reveal(0.2, 0.8), false);
        },
    ),
    Entry::new(
        "ui.dialog:link",
        "Turns the selected text into a link you can click to open a web page, a file or another place in this document. Example: the words «موقع الوزارة» open the ministry's website.",
        |d| d.in_link(),
    ),
    Entry::new(
        "ui.dialog:bookmark",
        "Gives the selected place a name so you can jump back to it or point to it later. Example: name the section «الملحق» and jump to it from anywhere.",
        |d| d.in_bookmark(),
    ),
    Entry::new(
        "insert.crossReference",
        "Puts a note that points to a heading, table or figure elsewhere («see page ٣»); the number updates itself when things move. Example: «انظر الجدول ٢» stays right even if the table becomes number ٣.",
        |d| d.in_cross_reference(),
    ),
    Entry::new(
        "review.newComment",
        "Attaches a note to the selected words; it shows in the margin and is not part of the text. Example: mark a sentence and write «راجع هذا الرقم» for a colleague.",
        |d| d.in_comment(),
    ),
    Entry::new(
        "insert.header",
        "Adds a strip at the top of every page for text that repeats, such as the title or a stamp. Example: «وزارة الدفاع» appears at the top of every page.",
        |d| d.in_header_footer(true),
    ),
    Entry::new(
        "insert.footer",
        "Adds a strip at the bottom of every page for text that repeats, such as a note or a date. Example: the date appears at the bottom of every page.",
        |d| d.in_header_footer(false),
    ),
    Entry::new(
        "insert.header:blank",
        "A header with one empty line at the top of every page; type into it once and it repeats. Example: type the unit's name and it shows on every page.",
        |d| d.in_hf_preset(true, 0),
    ),
    Entry::new(
        "insert.header:blankThree",
        "A header with three places at the top of every page: left, centre and right. Example: the date on the left, the title in the centre and the page number on the right.",
        |d| d.in_hf_preset(true, 1),
    ),
    Entry::new(
        "insert.header:title",
        "A header that shows the document's title at the top of every page. Example: the title «خطة التدريب» appears above the text on every page.",
        |d| d.in_hf_preset(true, 2),
    ),
    Entry::new(
        "insert.footer:blank",
        "A footer with one empty line at the bottom of every page; type into it once and it repeats. Example: type a note and it shows under the text on every page.",
        |d| d.in_hf_preset(false, 0),
    ),
    Entry::new(
        "insert.footer:blankThree",
        "A footer with three places at the bottom of every page: left, centre and right. Example: the author on the left, a note in the centre and the date on the right.",
        |d| d.in_hf_preset(false, 1),
    ),
    Entry::new(
        "insert.footer:pageNumber",
        "A footer with the page number in the middle at the bottom of every page. Example: ١ on the first page, ٢ on the second, and so on.",
        |d| d.in_hf_preset(false, 3),
    ),
    Entry::new(
        "insert.editHeader",
        "Opens the top strip of the page so you can type or change what repeats there; double-clicking the top of a page does the same. Example: change the word in the header from «مسودة» to «نهائي».",
        |d| d.in_edit_hf(true),
    ),
    Entry::new(
        "insert.removeHeader",
        "Clears the top strip of every page; the text that repeated there is gone. Example: the stamp at the top of every page disappears.",
        |d| d.in_remove_hf(true),
    ),
    Entry::new(
        "insert.editFooter",
        "Opens the bottom strip of the page so you can type or change what repeats there. Example: add the date under the text on every page.",
        |d| d.in_edit_hf(false),
    ),
    Entry::new(
        "insert.removeFooter",
        "Clears the bottom strip of every page; the text that repeated there is gone. Example: the note under every page disappears.",
        |d| d.in_remove_hf(false),
    ),
    Entry::new(
        "insert.pageNumber",
        "Numbers every page automatically, at the top or the bottom; add or remove pages and the numbers fix themselves. Example: pages ١، ٢، ٣ get their numbers without you typing them.",
        |d| d.in_page_number(),
    ),
    Entry::new(
        "insert.pageNumber:top",
        "Puts the page number at the top of every page, in the outer corner. Example: ١ at the top of the first page, ٢ at the top of the second.",
        |d| d.in_page_number_at(0),
    ),
    Entry::new(
        "insert.pageNumber:bottom",
        "Puts the page number at the bottom of every page, in the middle; the «Page X of Y» form adds the total. Example: ٣ at the bottom of the third page, or «٣ من ٧».",
        |d| d.in_page_number_at(1),
    ),
    Entry::new(
        "insert.pageNumber:current",
        "Puts the page number right where the cursor is, inside the text, not at the edge of the page. Example: the sentence «هذه هي الصفحة ٣» writes its own number.",
        |d| d.in_page_number_at(2),
    ),
    Entry::new(
        "insert.textBox",
        "Puts a framed box of text on the page that you can drag anywhere, even over a picture. Example: a caption box placed beside a photo.",
        |d| {
            let a = d.page_rl();
            let r = Rect::from_min_size(pos2(a.min.x + 6.0, a.min.y + 26.0), vec2(a.width() - 12.0, 30.0));
            d.text_box(Draw::in_scaled(r, 0.3 + 0.7 * d.reveal(0.2, 0.6)));
        },
    ),
    Entry::new(
        "insert.quickParts",
        "Keeps pieces of text you use often, and puts one on the page with one click. Example: save the address block once and insert it in every letter.",
        |d| d.in_quick_parts(),
    ),
    Entry::new(
        "insert.wordArt",
        "Puts decorated text on the page: a coloured fill, an outline and a shadow, for titles and banners. Example: the word «تهنئة» in big blue letters with a shadow.",
        |d| d.in_word_art(),
    ),
    Entry::new(
        "insert.dropCap",
        "Makes the first letter of the paragraph big, so it drops down beside the first lines. Example: the «ك» of «كان» grows to the height of three lines.",
        |d| d.in_drop_cap(),
    ),
    Entry::new(
        "insert.signatureLine",
        "Puts a line to sign on, with the signer's name and job title written under it. Example: a line with «العقيد أحمد» and «آمر الوحدة» beneath it.",
        |d| d.in_signature_line(),
    ),
    Entry::new(
        "insert.dateTime",
        "Puts today's date or the current time at the cursor, in the form you pick. Example: «٢٠٢٦/١٠/١٧» or «١٧ تشرين الأول ٢٠٢٦».",
        |d| d.in_date_time(),
    ),
    Entry::new(
        "insert.object",
        "Puts the contents of another file (a sheet, a drawing, another document) into the page at the cursor. Example: a table from a spreadsheet file appears inside the letter.",
        |d| d.in_object(),
    ),
    Entry::new(
        "insert.equation",
        "Puts a mathematics formula on the page, built with fractions, roots and symbols. Example: a fraction with x² over n.",
        |d| d.in_equation(),
    ),
    Entry::new(
        "insert.symbol",
        "Puts a character that is not on the keyboard into the text, such as © or ±. Example: «±» after a number, or «→» between two words.",
        |d| d.in_symbol(),
    ),
    // ---- Tables menu ----
    Entry::new(
        "ui.dialog:insertTable",
        "Opens a box where you type how many rows and columns you want, then puts that table on the page. Example: ٥ rows and ٣ columns for a list of names, ranks and dates.",
        |d| d.in_table_dialog(),
    ),
    Entry::new(
        "table.fromText",
        "Turns lines of text you already typed into a table: each line becomes a row, and each comma or tab starts a new column. Example: «أحمد، نقيب، ٢٠٢٦» becomes a row with three cells.",
        |d| d.in_from_text(),
    ),
    Entry::new(
        "table.quick:tabular",
        "Puts a ready-made simple table with a heading row and plain rows below it. Example: a list of items with a bold title row on top.",
        |d| d.in_quick_table(0),
    ),
    Entry::new(
        "table.quick:matrix",
        "Puts a ready-made table with headings across the top and down the side, for comparing things. Example: units down the side, months across the top, numbers in the middle.",
        |d| d.in_quick_table(1),
    ),
    Entry::new(
        "table.quick:calendar",
        "Puts a ready-made calendar of one month: the days of the week across the top and the dates in the cells. Example: a month's duty roster with a cell for every day.",
        |d| d.in_quick_table(2),
    ),
    // ---- Shapes menu ----
    Entry::new(
        "insert.shape:rectangle",
        "Draws a rectangle with square corners that you can resize, colour and put text in. Example: a box around an important note.",
        |d| d.in_shape_kind(0),
    ),
    Entry::new(
        "insert.shape:roundedRectangle",
        "Draws a rectangle with rounded corners, softer than a plain box. Example: a rounded box for a label or a button.",
        |d| d.in_shape_kind(1),
    ),
    Entry::new(
        "insert.shape:ellipse",
        "Draws an oval; hold Shift while dragging to make a perfect circle. Example: a circle around a number on a diagram.",
        |d| d.in_shape_kind(2),
    ),
    Entry::new("insert.shape:triangle", "Draws a triangle with its point at the top. Example: a warning sign or an arrowhead on a diagram.", |d| {
        d.in_shape_kind(3)
    }),
    Entry::new(
        "insert.shape:diamond",
        "Draws a diamond (a square standing on its corner), the usual sign for a decision in a flowchart. Example: a box that asks «هل الطلب مكتمل؟».",
        |d| d.in_shape_kind(4),
    ),
    Entry::new(
        "insert.shape:line",
        "Draws a straight line between two points; drag its ends to any angle. Example: a line separating two parts of a form.",
        |d| d.in_shape_kind(5),
    ),
    Entry::new(
        "insert.shape:arrow",
        "Draws a straight line with an arrowhead, to point at something. Example: an arrow from a label to the part of a picture it names.",
        |d| d.in_shape_kind(6),
    ),
    Entry::new("insert.shape:star", "Draws a five-pointed star. Example: a star next to the best result in a list.", |d| d.in_shape_kind(7)),
    Entry::new("insert.shape:heart", "Draws a heart shape. Example: a heart on a greeting card.", |d| d.in_shape_kind(8)),
    Entry::new(
        "insert.shape:straightConnector",
        "Draws a straight line that glues its ends to two shapes and follows them when they move; with an arrowhead if you pick the arrow kind. Example: a line from one box of a diagram to the next.",
        |d| d.in_connector(0),
    ),
    Entry::new(
        "insert.shape:elbowConnector",
        "Draws a line with a right-angle bend that glues its ends to two shapes and follows them when they move; with an arrowhead if you pick the arrow kind. Example: a bent line from a box to another box lower down.",
        |d| d.in_connector(1),
    ),
    Entry::new(
        "insert.shape:curvedConnector",
        "Draws a curved line that glues its ends to two shapes and follows them when they move; with an arrowhead if you pick the arrow kind. Example: a smooth curve from one box of a diagram to another.",
        |d| d.in_connector(2),
    ),
    // ---- Field and symbol dialogs ----
    Entry::new(
        "ui.dialog:field",
        "Opens a list of fields: pieces of text that fill themselves in, such as the date, the page count or the file name. Example: a field that always shows the number of pages, even after you add some.",
        |d| d.in_field_dialog(),
    ),
    Entry::new(
        "ui.dialog:symbol",
        "Opens the full table of characters and signs you can put into the text, from every font. Example: find «☑» or a letter with an unusual mark.",
        |d| d.in_symbol_dialog(),
    ),
];

impl Draw<'_> {
    /// The same drawing through a painter clipped to `clip` (a wipe).
    fn in_clipped(&self, clip: Rect, f: impl FnOnce(&Draw<'_>)) {
        let p = self.p.with_clip_rect(clip.intersect(self.p.clip_rect()));
        f(&self.with_painter(&p));
    }

    /// `r` scaled about its centre by `k` (a thing popping in).
    fn in_scaled(r: Rect, k: f32) -> Rect {
        Rect::from_center_size(r.center(), r.size() * k.max(0.02))
    }

    /// The accent band comes down, then the title lines grow out from the middle.
    fn in_cover_page(&self) {
        let a = self.page_center();
        let band = self.reveal(0.1, 0.45);
        self.p.rect_filled(
            Rect::from_min_size(pos2(a.min.x - 8.0, a.min.y - 8.0), vec2(a.width() + 16.0, 30.0 * band.max(0.05))),
            0.0,
            self.accent(),
        );
        let y = a.min.y + 34.0;
        let grow = self.reveal(0.4, 0.8);
        let cx = a.center().x;
        let half = (a.width() / 2.0 - 6.0) * grow;
        self.line(pos2(cx - half, y), pos2(cx + half, y), self.ink(), 4.0);
        let half = (a.width() / 2.0 - 14.0) * grow;
        self.line(pos2(cx - half, y + 9.0), pos2(cx + half, y + 9.0), self.ink(), 2.0);
        self.line(pos2(a.min.x + 20.0, y + 32.0), pos2(a.max.x - 20.0, y + 32.0), self.grey(), 1.5);
    }

    /// The arrow grows, then the new page wipes in beside the old one.
    fn in_blank_page(&self) {
        let p1 = self.page(16.0, 8.0, 62.0, 88.0);
        self.paragraph(p1, Dir::Rtl, 4, self.grey());
        self.caret(pos2(p1.max.x - 20.0, p1.min.y + 32.0), 10.0);
        let arrow = self.reveal(0.05, 0.3);
        if arrow > 0.0 {
            self.arrow_right(self.at(80.0, 52.0), self.at(80.0 + 14.0 * arrow, 52.0));
        }
        let wipe = self.reveal(0.3, 0.7);
        if wipe > 0.0 {
            let outer = Rect::from_min_size(self.at(98.0, 8.0), vec2(62.0, 88.0));
            let clip = Rect::from_min_max(outer.min - vec2(2.0, 2.0), pos2(outer.max.x + 2.0, outer.min.y + outer.height() * wipe + 2.0));
            self.in_clipped(clip, |d| {
                let p2 = d.page(98.0, 8.0, 62.0, 88.0);
                d.p.rect_stroke(p2.expand(8.0), 2.0, Stroke::new(1.5, d.accent()), egui::StrokeKind::Inside);
            });
        }
    }

    /// The dashed cut draws across the page, then the text after it wipes onto the next page.
    fn in_page_break(&self) {
        let p1 = self.page(16.0, 8.0, 62.0, 88.0);
        self.text_lines(p1, true, &[46.0, 42.0, 44.0], 10.0, self.grey());
        let y = p1.min.y + 34.0;
        let cut = self.reveal(0.05, 0.3);
        let end = p1.min.x - 6.0 + (p1.width() + 12.0) * cut;
        let mut x = p1.min.x - 6.0;
        while x < end {
            self.line(pos2(x, y), pos2((x + 3.0).min(end), y), self.accent(), 1.5);
            x += 5.0;
        }
        let arrow = self.reveal(0.3, 0.5);
        if arrow > 0.0 {
            self.arrow_right(self.at(80.0, 52.0), self.at(80.0 + 14.0 * arrow, 52.0));
        }
        let p2 = self.page(98.0, 8.0, 62.0, 88.0);
        let wipe = self.reveal(0.5, 0.85);
        if wipe > 0.0 {
            self.in_clipped(Rect::from_min_max(p2.min - vec2(2.0, 6.0), pos2(p2.max.x + 2.0, p2.min.y + 44.0 * wipe)), |d| {
                d.text_lines(p2, true, &[44.0, 40.0, 46.0, 24.0], 10.0, d.accent());
            });
        }
    }

    /// Three shapes pop onto the page one after the other.
    fn in_shapes(&self) {
        let a = self.page_rl();
        let k1 = self.reveal(0.1, 0.35);
        self.shape(Draw::in_scaled(Rect::from_min_size(pos2(a.min.x + 2.0, a.min.y + 24.0), vec2(24.0, 18.0)), k1));
        let k2 = self.reveal(0.35, 0.6);
        let c = pos2(a.max.x - 12.0, a.min.y + 33.0);
        if k2 > 0.0 {
            self.p.circle_filled(c, 9.0 * k2, self.accent().linear_multiply(0.35));
            self.p.circle_stroke(c, 9.0 * k2, Stroke::new(1.5, self.accent()));
        }
        let k3 = self.reveal(0.6, 0.85);
        if k3 > 0.0 {
            let s = pos2(a.center().x, a.min.y + 58.0);
            let tri = vec![s + vec2(0.0, -8.0) * k3, s + vec2(9.0, 7.0) * k3, s + vec2(-9.0, 7.0) * k3];
            self.p.add(Shape::convex_polygon(tri, self.accent().linear_multiply(0.35), Stroke::new(1.5, self.accent())));
        }
    }

    /// The points of a star centred at `c`.
    fn in_star(c: Pos2, outer: f32, inner: f32) -> Vec<Pos2> {
        (0..10)
            .map(|i| {
                let r = if i % 2 == 0 { outer } else { inner };
                let ang = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
                c + vec2(ang.cos() * r, ang.sin() * r)
            })
            .collect()
    }

    /// A simple five-point star, the kind an icon library holds; it grows and darkens in.
    fn in_icon(&self) {
        let a = self.page_rl();
        let c = pos2(a.center().x, a.min.y + 36.0);
        let k = self.reveal(0.2, 0.7);
        self.p.rect_filled(Rect::from_center_size(c, vec2(38.0, 38.0)), 0.0, Color32::WHITE);
        if k > 0.0 {
            let s = 0.4 + 0.6 * k;
            self.p.add(Shape::closed_line(Draw::in_star(c, 16.0 * s, 7.0 * s), Stroke::new(2.0, self.accent().linear_multiply(k.max(0.3)))));
        }
    }

    /// Three boxes joined by arrows (a process diagram), appearing one by one.
    fn in_smart_art(&self) {
        let a = self.page_wide();
        self.text_lines(a, true, &[90.0], 10.0, self.grey());
        for i in 0..3 {
            let k = self.reveal(0.1 + i as f32 * 0.25, 0.3 + i as f32 * 0.25);
            if k <= 0.0 {
                continue;
            }
            let x = a.min.x + 4.0 + i as f32 * 36.0;
            let r = Rect::from_min_size(pos2(x, a.min.y + 22.0), vec2(26.0, 22.0));
            self.shape(Draw::in_scaled(r, k));
            if i < 2 && k >= 1.0 {
                let to = self.reveal(0.3 + i as f32 * 0.25, 0.4 + i as f32 * 0.25);
                self.arrow_right(pos2(r.max.x + 1.0, r.center().y), pos2(r.max.x + 1.0 + 8.0 * to.max(0.1), r.center().y));
            }
        }
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 52.0), a.max), true, &[96.0, 60.0], 10.0, self.grey());
    }

    /// A bar chart in `r` whose bars have grown to `k` of their height; `labels` writes each
    /// bar's value above it (Data Labels).
    pub(super) fn in_chart_bars(&self, r: Rect, k: f32, labels: bool) {
        self.p.rect_filled(r, 1.0, Color32::WHITE);
        self.p.rect_stroke(r, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        let n = 5;
        let bw = (r.width() - 8.0) / n as f32;
        let values = ["٢٥", "٤٠", "١٨", "٤٨", "٣٠"];
        for (i, (h, v)) in [0.5, 0.8, 0.35, 0.95, 0.6].iter().zip(values).enumerate() {
            let x = r.min.x + 4.0 + i as f32 * bw + 2.0;
            let top = r.max.y - 4.0 - (r.height() - 8.0) * h * k;
            self.p.rect_filled(
                Rect::from_min_max(pos2(x, top), pos2(x + bw - 4.0, r.max.y - 4.0)),
                1.0,
                if i % 2 == 0 { self.accent() } else { self.accent().linear_multiply(0.5) },
            );
            if labels {
                let show = self.reveal(0.3 + i as f32 * 0.1, 0.4 + i as f32 * 0.1);
                if show > 0.0 {
                    self.label(pos2(x + (bw - 4.0) / 2.0, top - 4.0), v, 5.5, self.ink().linear_multiply(show), Align2::CENTER_CENTER);
                }
            }
        }
    }

    /// Blue underlined text on the page (the underline slides in), and the dialog asking for the
    /// address.
    fn in_link(&self) {
        let a = self.page_rl();
        let y = a.min.y + 24.0;
        let blue = Color32::from_rgb(30, 90, 200);
        let k = self.reveal(0.1, 0.45);
        self.p.rect_filled(Rect::from_min_max(pos2(a.max.x - 40.0, y - 4.0), pos2(a.max.x - 10.0, y + 4.0)), 0.0, Color32::WHITE);
        self.line(pos2(a.max.x - 10.0 - 30.0 * k, y), pos2(a.max.x - 10.0, y), blue, 1.5);
        self.line(pos2(a.max.x - 10.0 - 30.0 * k, y + 3.0), pos2(a.max.x - 10.0, y + 3.0), blue, 1.0);
        let inner = self.dialog("Link", 1);
        self.label(pos2(inner.min.x + 38.0, inner.min.y + 5.0), "https://…", 6.5, blue, Align2::LEFT_CENTER);
        self.button(pos2(inner.center().x, inner.max.y - 12.0), "OK", false);
    }

    /// A ribbon-like bookmark flag drops into the margin next to the marked line.
    fn in_bookmark(&self) {
        let a = self.page_rl();
        let y = a.min.y + 24.0;
        self.selection(Rect::from_min_max(pos2(a.max.x - 36.0, y - 5.0), pos2(a.max.x, y + 5.0)));
        self.line(pos2(a.max.x - 34.0, y), pos2(a.max.x, y), self.ink(), 1.5);
        let k = self.reveal(0.2, 0.6);
        if k > 0.0 {
            let x = a.max.x + 6.0;
            let y = y - (1.0 - k) * 14.0;
            let flag = vec![pos2(x - 4.0, y - 10.0), pos2(x + 4.0, y - 10.0), pos2(x + 4.0, y + 8.0), pos2(x, y + 3.0), pos2(x - 4.0, y + 8.0)];
            self.p.add(Shape::convex_polygon(flag, self.accent().linear_multiply(k.max(0.3)), Stroke::NONE));
        }
        self.label(pos2(a.center().x, a.max.y - 2.0), "اسم", 7.0, self.accent(), Align2::CENTER_CENTER);
    }

    /// A «see …» chip low on the page; its arrow climbs to the heading.
    fn in_cross_reference(&self) {
        let a = self.page_center();
        self.heading_then_text(a, Dir::Rtl, 2, self.ink());
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 34.0), a.max), true, &[56.0, 52.0, 30.0], 10.0, self.grey());
        let y = a.min.y + 66.0;
        self.field(pos2(a.min.x + 10.0, y - 3.5), 30.0);
        let k = self.reveal(0.2, 0.7);
        if k > 0.0 {
            let top = a.min.y + 8.0;
            self.arrow_up(pos2(a.min.x + 6.0, y), pos2(a.min.x + 6.0, y - (y - top) * k));
        }
    }

    /// Selected words; a thin line reaches out to the margin and the note balloon wipes in.
    fn in_comment(&self) {
        let page = self.page(28.0, 6.0, 76.0, 92.0);
        self.paragraph(page, Dir::Rtl, 6, self.grey());
        let y = page.min.y + 24.0;
        self.highlight(Rect::from_min_max(pos2(page.max.x - 38.0, y - 4.0), pos2(page.max.x - 10.0, y + 4.0)));
        self.line(pos2(page.max.x - 38.0, y), pos2(page.max.x - 10.0, y), self.ink(), 1.5);
        let note = Rect::from_min_size(self.at(116.0, 12.0), vec2(52.0, 30.0));
        let reach = self.reveal(0.1, 0.4);
        if reach > 0.0 {
            let from = pos2(page.max.x - 10.0, y);
            self.line(from, from + (note.left_center() - from) * reach, self.accent(), 1.0);
        }
        let wipe = self.reveal(0.4, 0.8);
        if wipe > 0.0 {
            self.in_clipped(Rect::from_min_max(note.min - vec2(2.0, 2.0), pos2(note.min.x + note.width() * wipe + 2.0, note.max.y + 2.0)), |d| {
                d.p.rect_filled(note, 3.0, d.yellow().linear_multiply(0.5));
                d.p.rect_stroke(note, 3.0, d.stroke(d.accent()), egui::StrokeKind::Inside);
                d.text_lines(note.shrink(5.0), true, &[36.0, 28.0], 8.0, d.ink());
            });
        }
    }

    /// An accent band grows at the top (`header`) or bottom of the page, then on a second page.
    fn in_header_footer(&self, header: bool) {
        for (i, x) in [26.0, 94.0].into_iter().enumerate() {
            let a = self.page(x, 8.0, 56.0, 88.0);
            let body = if header { Rect::from_min_max(pos2(a.min.x, a.min.y + 14.0), a.max) } else { a };
            self.paragraph(body, Dir::Rtl, 5, self.grey());
            let y = if header { a.min.y - 2.0 } else { a.max.y + 2.0 };
            let k = self.reveal(0.1 + i as f32 * 0.3, 0.4 + i as f32 * 0.3);
            let half = a.width() / 2.0 * k;
            if k > 0.0 {
                self.line(pos2(a.center().x - half, y), pos2(a.center().x + half, y), self.accent(), 2.5);
            }
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

    /// The header (or footer) band open for editing: the body text fades, the band is outlined
    /// and a pencil writes along it.
    fn in_edit_hf(&self, header: bool) {
        let a = self.page(50.0, 6.0, 76.0, 92.0);
        let body =
            if header { Rect::from_min_max(pos2(a.min.x, a.min.y + 16.0), a.max) } else { Rect::from_min_max(a.min, pos2(a.max.x, a.max.y - 16.0)) };
        self.paragraph(body, Dir::Rtl, 6, self.grey().linear_multiply(0.45));
        let y = if header { a.min.y + 2.0 } else { a.max.y - 4.0 };
        let band = Rect::from_min_max(pos2(a.min.x - 4.0, y - 6.0), pos2(a.max.x + 4.0, y + 6.0));
        self.p.rect_filled(band, 1.0, self.accent().linear_multiply(0.12));
        self.dashed_rect(band, self.accent());
        let k = self.reveal(0.15, 0.7);
        let written = 30.0 * k;
        self.line(pos2(a.max.x - 2.0 - written, y), pos2(a.max.x - 2.0, y), self.accent(), 2.0);
        let tip = pos2(a.max.x - 2.0 - written, y + 1.0);
        self.pencil(tip, tip + vec2(-9.0, -13.0));
    }

    /// The band with its text, then the page without it and a cross where it was.
    fn in_remove_hf(&self, header: bool) {
        self.before_after(
            |d, a| {
                let body = if header {
                    Rect::from_min_max(pos2(a.min.x, a.min.y + 14.0), a.max)
                } else {
                    Rect::from_min_max(a.min, pos2(a.max.x, a.max.y - 14.0))
                };
                d.paragraph(body, Dir::Rtl, 6, d.grey());
                let y = if header { a.min.y + 1.0 } else { a.max.y - 3.0 };
                d.p.rect_filled(
                    Rect::from_min_max(pos2(a.min.x - 4.0, y - 5.0), pos2(a.max.x + 4.0, y + 5.0)),
                    1.0,
                    d.accent().linear_multiply(0.12),
                );
                d.line(pos2(a.center().x - 14.0, y), pos2(a.center().x + 14.0, y), d.accent(), 2.0);
            },
            |d, a| {
                let body = if header {
                    Rect::from_min_max(pos2(a.min.x, a.min.y + 14.0), a.max)
                } else {
                    Rect::from_min_max(a.min, pos2(a.max.x, a.max.y - 14.0))
                };
                d.paragraph(body, Dir::Rtl, 6, d.grey());
                let y = if header { a.min.y + 1.0 } else { a.max.y - 3.0 };
                d.cross(pos2(a.center().x, y));
            },
        );
    }

    /// Three pages whose numbers land one after the other.
    fn in_page_number(&self) {
        for (i, x) in [20.0, 64.0, 108.0].iter().enumerate() {
            let a = self.page(*x, 10.0, 48.0, 80.0);
            self.text_lines(a, true, &[32.0, 28.0, 30.0, 18.0], 10.0, self.grey());
            let k = self.reveal(0.15 + i as f32 * 0.2, 0.3 + i as f32 * 0.2);
            if k > 0.0 {
                self.badge(pos2(a.center().x, a.max.y - 2.0), ["١", "٢", "٣"][i], self.accent().linear_multiply(k));
            }
        }
    }

    /// One page with the number where the preset puts it: 0 top corner, 1 bottom centre, 2 in
    /// the text at the cursor.
    fn in_page_number_at(&self, position: u8) {
        let a = self.page(50.0, 6.0, 76.0, 92.0);
        let k = self.reveal(0.2, 0.6);
        match position {
            0 => {
                self.paragraph(Rect::from_min_max(pos2(a.min.x, a.min.y + 14.0), a.max), Dir::Rtl, 6, self.grey());
                if k > 0.0 {
                    self.badge(pos2(a.max.x - 6.0, a.min.y + 2.0), "١", self.accent().linear_multiply(k));
                }
            }
            1 => {
                self.paragraph(Rect::from_min_max(a.min, pos2(a.max.x, a.max.y - 14.0)), Dir::Rtl, 6, self.grey());
                if k > 0.0 {
                    self.badge(pos2(a.center().x, a.max.y - 2.0), "١", self.accent().linear_multiply(k));
                }
            }
            _ => {
                self.text_lines(a, true, &[60.0, 24.0, 58.0, 56.0, 30.0], 10.0, self.grey());
                let y = a.min.y + 14.0;
                self.caret(pos2(a.max.x - 26.0, y - 5.0), 10.0);
                if k > 0.0 {
                    let x = a.max.x - 28.0 - (1.0 - k) * 6.0;
                    self.label(pos2(x, y), "٣", 8.0, self.accent().linear_multiply(k), Align2::RIGHT_CENTER);
                    self.line(pos2(x - 8.0 - 20.0 * k, y), pos2(x - 8.0, y), self.grey(), 1.5);
                }
            }
        }
    }

    /// A shelf of saved pieces; the chosen one slides onto the page.
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
        let k = self.reveal(0.15, 0.65);
        let from = Rect::from_min_size(self.at(8.0, 42.0), vec2(48.0, 14.0));
        let to = Rect::from_min_size(pos2(a.min.x + 2.0, a.min.y + 24.0), vec2(50.0, 20.0));
        let r = Rect::from_min_size(from.min + (to.min - from.min) * k, from.size() + (to.size() - from.size()) * k);
        if k < 1.0 {
            self.p.rect_filled(r, 2.0, Color32::WHITE);
        }
        self.p.rect_stroke(r, 2.0, self.stroke(self.accent()), egui::StrokeKind::Outside);
        self.text_lines(r.shrink2(vec2(2.0, 2.0)), true, &[40.0, 30.0], 8.0 * (0.6 + 0.4 * k), self.accent());
    }

    /// The decorated letter grows into place over the text.
    fn in_word_art(&self) {
        let a = self.page_center();
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 44.0), a.max), true, &[56.0, 50.0, 30.0], 10.0, self.grey());
        let c = pos2(a.center().x, a.min.y + 20.0);
        let size = 34.0 * (0.3 + 0.7 * self.reveal(0.15, 0.6));
        self.glyph(c + vec2(3.0, 3.0), "A", size, self.grey());
        for off in [vec2(-1.5, 0.0), vec2(1.5, 0.0), vec2(0.0, -1.5), vec2(0.0, 1.5)] {
            self.glyph(c + off, "A", size, self.ink());
        }
        self.glyph(c, "A", size, self.accent());
    }

    /// The first letter swells from text size to three lines tall.
    fn in_drop_cap(&self) {
        let a = self.page_center();
        let big = Rect::from_min_size(pos2(a.max.x - 22.0, a.min.y), vec2(22.0, 28.0));
        let k = self.reveal(0.2, 0.65);
        self.glyph(big.center(), "ك", 10.0 + 16.0 * k, self.accent());
        self.text_lines(Rect::from_min_max(a.min, pos2(a.max.x - 24.0, a.max.y)), true, &[36.0, 34.0, 36.0], 10.0, self.grey());
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 30.0), a.max), true, &[58.0, 54.0, 56.0, 30.0], 10.0, self.grey());
    }

    /// The signing line draws itself, then the X and the name appear.
    fn in_signature_line(&self) {
        let a = self.page_center();
        self.paragraph(a, Dir::Rtl, 3, self.grey());
        let y = a.min.y + 44.0;
        let draw = self.reveal(0.1, 0.5);
        let (x0, x1) = (a.min.x + 4.0, a.max.x - 10.0);
        self.line(pos2(x1 - (x1 - x0) * draw, y + 4.0), pos2(x1, y + 4.0), self.ink(), 1.2);
        let rest = self.reveal(0.5, 0.8);
        if rest > 0.0 {
            self.label(pos2(a.max.x, y), "X", 9.0, self.accent().linear_multiply(rest), Align2::RIGHT_CENTER);
            self.text_lines(Rect::from_min_max(pos2(a.min.x, y + 8.0), a.max), true, &[30.0, 22.0], 8.0, self.grey().linear_multiply(rest));
        }
    }

    /// The date appears in the text; the clock's hand sweeps round to the time.
    fn in_date_time(&self) {
        let a = self.page_rl();
        let y = a.min.y + 24.0;
        self.p.rect_filled(Rect::from_min_max(pos2(a.max.x - 46.0, y - 5.0), pos2(a.max.x + 2.0, y + 5.0)), 1.0, Color32::WHITE);
        let k = self.reveal(0.2, 0.5);
        self.label(pos2(a.max.x, y), "٢٠٢٦/١٠/١٧", 7.5, self.accent().linear_multiply(k.max(0.05)), Align2::RIGHT_CENTER);
        // A small clock; the minute hand turns once during the film and ends pointing up.
        let c = self.at(26.0, 72.0);
        self.p.circle_stroke(c, 11.0, Stroke::new(2.0, self.accent()));
        let turn = if self.phase >= 1.0 { 0.0 } else { (1.0 - self.phase) * std::f32::consts::TAU };
        let hand = vec2(turn.sin(), -turn.cos()) * 7.0;
        self.line(c, c + hand, self.accent(), 1.5);
        self.line(c, c + vec2(5.0, 2.0), self.accent(), 1.5);
    }

    /// A file (a page with a folded corner) whose contents wipe onto the page.
    fn in_object(&self) {
        let f = Rect::from_min_size(self.at(14.0, 24.0), vec2(40.0, 50.0));
        self.p.rect_filled(f, 1.0, Color32::WHITE);
        self.p.rect_stroke(f, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        let fold = vec![pos2(f.max.x - 12.0, f.min.y), pos2(f.max.x, f.min.y + 12.0), pos2(f.max.x - 12.0, f.min.y + 12.0)];
        self.p.add(Shape::convex_polygon(fold, self.t.hover, Stroke::new(1.0, self.t.border_strong)));
        self.text_lines(f.shrink2(vec2(5.0, 16.0)), true, &[28.0, 24.0, 26.0], 8.0, self.accent());
        let arrow = self.reveal(0.05, 0.3);
        if arrow > 0.0 {
            self.arrow_right(self.at(60.0, 50.0), self.at(60.0 + 22.0 * arrow, 50.0));
        }
        let a = self.page(90.0, 8.0, 70.0, 88.0);
        self.text_lines(a, true, &[50.0, 44.0], 10.0, self.grey());
        let wipe = self.reveal(0.3, 0.7);
        if wipe > 0.0 {
            let band = Rect::from_min_max(pos2(a.min.x - 2.0, a.min.y + 18.0), pos2(a.min.x - 2.0 + (a.width() + 4.0) * wipe, a.min.y + 50.0));
            self.in_clipped(band, |d| {
                d.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 22.0), a.max), true, &[48.0, 52.0, 40.0], 10.0, d.accent());
            });
        }
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 54.0), a.max), true, &[46.0, 24.0], 10.0, self.grey());
    }

    /// The formula's parts appear one after the other.
    fn in_equation(&self) {
        let a = self.page_rl();
        let box_ = Rect::from_min_max(pos2(a.min.x + 2.0, a.min.y + 18.0), pos2(a.max.x - 2.0, a.min.y + 48.0));
        self.p.rect_filled(box_, 2.0, Color32::WHITE);
        self.p.rect_stroke(box_, 2.0, self.stroke(self.t.border), egui::StrokeKind::Inside);
        let c = box_.center();
        let k1 = self.reveal(0.1, 0.3);
        self.glyph(c + vec2(-16.0, 0.0), "∑", 20.0, self.accent().linear_multiply(k1));
        let k2 = self.reveal(0.3, 0.5);
        self.glyph(c + vec2(6.0, -6.0), "x²", 9.0, self.ink().linear_multiply(k2));
        let k3 = self.reveal(0.5, 0.65);
        self.line(c + vec2(-1.0, 0.0), c + vec2(-1.0 + 15.0 * k3, 0.0), self.ink(), 1.2);
        let k4 = self.reveal(0.65, 0.8);
        self.glyph(c + vec2(6.0, 7.0), "n", 9.0, self.ink().linear_multiply(k4));
    }

    /// A palette of symbols to pick from; the chosen one flies up into the text.
    fn in_symbol(&self) {
        let a = self.page_rl();
        let y = a.min.y + 24.0;
        self.p.rect_filled(Rect::from_min_max(pos2(a.max.x - 30.0, y - 6.0), pos2(a.max.x - 14.0, y + 6.0)), 0.0, Color32::WHITE);
        let g = Rect::from_min_size(self.at(10.0, 56.0), vec2(48.0, 32.0));
        self.p.rect_filled(g, 2.0, self.t.menu);
        self.grid(g, 2, 3, self.t.border);
        let mut from = pos2(a.max.x - 22.0, y);
        for (i, s) in ["©", "±", "€", "→", "√", "∞"].iter().enumerate() {
            let cell = self.cell(g, 2, 3, i / 3, i % 3);
            if i == 1 {
                from = cell.center();
                self.selection(cell.shrink(1.0));
            }
            self.label(cell.center(), s, 8.0, if i == 1 { self.accent() } else { self.ink() }, Align2::CENTER_CENTER);
        }
        let k = self.reveal(0.2, 0.7);
        let to = pos2(a.max.x - 22.0, y);
        self.glyph(from + (to - from) * k, "Ω", 8.0 + 4.0 * k, self.accent());
    }

    // ---- Tables menu ----

    /// The Insert Table box: rows and columns to fill in, and a small grid of the result.
    fn in_table_dialog(&self) {
        let inner = self.dialog("Table", 2);
        self.label(pos2(inner.min.x + 38.0, inner.min.y + 5.0), "٥", 7.0, self.ink(), Align2::LEFT_CENTER);
        self.label(pos2(inner.min.x + 38.0, inner.min.y + 19.0), "٣", 7.0, self.ink(), Align2::LEFT_CENTER);
        let k = self.reveal(0.2, 0.7);
        let g = Rect::from_min_size(pos2(inner.min.x, inner.min.y + 32.0), vec2(inner.width(), 20.0));
        self.in_clipped(Rect::from_min_max(g.min, pos2(g.min.x + g.width() * k, g.max.y)), |d| d.grid(g, 2, 5, d.accent()));
    }

    /// Lines with commas between their words become the rows and cells of a table.
    fn in_from_text(&self) {
        self.before_after(
            |d, a| {
                for i in 0..3 {
                    let y = a.min.y + 8.0 + i as f32 * 12.0;
                    let mut x = a.max.x;
                    for w in [14.0, 10.0, 16.0] {
                        d.line(pos2(x - w, y), pos2(x, y), d.grey(), 1.5);
                        x -= w + 3.0;
                        d.label(pos2(x, y + 2.0), "،", 7.0, d.accent(), Align2::CENTER_CENTER);
                        x -= 5.0;
                    }
                }
            },
            |d, a| d.table(Rect::from_min_size(pos2(a.min.x, a.min.y + 2.0), vec2(a.width(), 38.0)), 3, 3),
        );
    }

    /// A ready-made table wipes down the page: 0 a tabular list (a heading row, rules only
    /// between rows), 1 a matrix (headings across and down), 2 a month's calendar.
    fn in_quick_table(&self, kind: u8) {
        let a = self.page_center();
        let r = Rect::from_min_size(pos2(a.min.x, a.min.y + 12.0), vec2(a.width(), 52.0));
        let k = self.reveal(0.2, 0.7);
        self.in_clipped(Rect::from_min_max(r.min - vec2(2.0, 2.0), pos2(r.max.x + 2.0, r.min.y + r.height() * k + 2.0)), |d| match kind {
            0 => {
                d.p.rect_filled(r, 1.0, Color32::WHITE);
                let rows = 4;
                let rh = r.height() / rows as f32;
                d.p.rect_filled(Rect::from_min_size(r.min, vec2(r.width(), rh)), 0.0, d.t.hover);
                for i in 0..=rows {
                    let y = r.min.y + i as f32 * rh;
                    d.line(pos2(r.min.x, y), pos2(r.max.x, y), if i == 0 || i == 1 || i == rows { d.t.border_strong } else { d.t.border }, 1.0);
                }
                for i in 0..rows {
                    let y = r.min.y + (i as f32 + 0.5) * rh;
                    for j in 0..3 {
                        let x = r.max.x - 3.0 - j as f32 * r.width() / 3.0;
                        d.line(pos2(x - 12.0, y), pos2(x, y), if i == 0 { d.ink() } else { d.grey() }, 1.5);
                    }
                }
            }
            1 => {
                d.table(r, 4, 4);
                let cw = r.width() / 4.0;
                d.p.rect_filled(Rect::from_min_size(pos2(r.max.x - cw, r.min.y), vec2(cw, r.height())), 0.0, d.t.hover);
                for i in 0..4 {
                    let y = r.min.y + i as f32 * r.height() / 4.0;
                    d.line(pos2(r.max.x - cw, y), pos2(r.max.x, y), d.t.border, 1.0);
                    let y = y + r.height() / 8.0;
                    d.line(pos2(r.max.x - cw + 3.0, y), pos2(r.max.x - cw + 3.0 + cw * 0.6, y), d.ink(), 1.5);
                }
                d.p.rect_stroke(r, 1.0, d.stroke(d.t.border_strong), egui::StrokeKind::Inside);
            }
            _ => {
                let title = Rect::from_min_size(r.min, vec2(r.width(), 12.0));
                d.p.rect_filled(title, 1.0, d.accent().linear_multiply(0.15));
                d.label(title.center(), "تشرين الأول", 6.5, d.accent(), Align2::CENTER_CENTER);
                let g = Rect::from_min_max(pos2(r.min.x, r.min.y + 12.0), r.max);
                d.p.rect_filled(g, 0.0, Color32::WHITE);
                d.grid(g, 5, 7, d.t.border);
                for (i, s) in ["س", "ح", "ن", "ث", "ر", "خ", "ج"].iter().enumerate() {
                    let cell = d.cell(g, 5, 7, 0, 6 - i);
                    d.label(cell.center(), s, 5.0, d.ink(), Align2::CENTER_CENTER);
                }
                for row in 1..5 {
                    for col in 0..7 {
                        let cell = d.cell(g, 5, 7, row, col);
                        d.p.circle_filled(cell.center(), 0.9, d.grey());
                    }
                }
            }
        });
    }

    // ---- Shapes menu ----

    /// One shape of the menu, grown onto the page: 0 rectangle, 1 rounded rectangle, 2 oval,
    /// 3 triangle, 4 diamond, 5 line, 6 arrow, 7 star, 8 heart.
    fn in_shape_kind(&self, kind: u8) {
        let a = self.page_rl();
        let c = pos2(a.center().x, a.min.y + 36.0);
        let k = 0.25 + 0.75 * self.reveal(0.15, 0.6);
        let fill = self.accent().linear_multiply(0.35);
        let stroke = Stroke::new(1.5, self.accent());
        let r = Rect::from_center_size(c, vec2(40.0, 28.0) * k);
        self.p.rect_filled(r.expand(4.0), 0.0, Color32::WHITE);
        match kind {
            0 => {
                self.p.rect_filled(r, 0.0, fill);
                self.p.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Inside);
            }
            1 => {
                self.p.rect_filled(r, 7.0, fill);
                self.p.rect_stroke(r, 7.0, stroke, egui::StrokeKind::Inside);
            }
            2 => {
                let pts: Vec<Pos2> = (0..32)
                    .map(|i| {
                        let t = i as f32 / 32.0 * std::f32::consts::TAU;
                        c + vec2(t.cos() * r.width() / 2.0, t.sin() * r.height() / 2.0)
                    })
                    .collect();
                self.p.add(Shape::convex_polygon(pts, fill, stroke));
            }
            3 => {
                let pts = vec![r.center_top(), r.right_bottom(), r.left_bottom()];
                self.p.add(Shape::convex_polygon(pts, fill, stroke));
            }
            4 => {
                let pts = vec![r.center_top(), r.right_center(), r.center_bottom(), r.left_center()];
                self.p.add(Shape::convex_polygon(pts, fill, stroke));
            }
            5 => self.line(r.left_bottom(), r.right_top(), self.accent(), 2.0),
            6 => {
                self.line(r.left_bottom(), r.right_top(), self.accent(), 2.0);
                let d = (r.right_top() - r.left_bottom()).normalized();
                let n = vec2(-d.y, d.x);
                let tip = r.right_top();
                self.p.add(Shape::convex_polygon(vec![tip, tip - d * 7.0 + n * 4.0, tip - d * 7.0 - n * 4.0], self.accent(), Stroke::NONE));
            }
            7 => {
                self.p.add(Shape::closed_line(Draw::in_star(c, 16.0 * k, 7.0 * k), Stroke::new(2.0, self.accent())));
            }
            _ => {
                let s = 0.95 * k;
                let pts: Vec<Pos2> = (0..40)
                    .map(|i| {
                        let t = i as f32 / 40.0 * std::f32::consts::TAU;
                        let x = 16.0 * t.sin().powi(3);
                        let y = 13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos();
                        c + vec2(x * s, -y * s + 1.0)
                    })
                    .collect();
                self.p.add(Shape::closed_line(pts, Stroke::new(2.0, self.accent())));
            }
        }
    }

    /// Two boxes and the connector between them, drawn from one to the other: 0 straight,
    /// 1 elbow, 2 curved.
    fn in_connector(&self, kind: u8) {
        let a = self.page_rl();
        let s1 = Rect::from_min_size(pos2(a.min.x + 1.0, a.min.y + 14.0), vec2(20.0, 14.0));
        let s2 = Rect::from_min_size(pos2(a.max.x - 21.0, a.max.y - 32.0), vec2(20.0, 14.0));
        self.p.rect_filled(s1.expand(2.0), 0.0, Color32::WHITE);
        self.p.rect_filled(s2.expand(2.0), 0.0, Color32::WHITE);
        self.shape(s1);
        self.shape(s2);
        let (p0, p1) = (s1.right_center(), s2.left_center());
        let mx = (p0.x + p1.x) / 2.0;
        let pts: Vec<Pos2> = match kind {
            0 => vec![p0, p1],
            1 => vec![p0, pos2(mx, p0.y), pos2(mx, p1.y), p1],
            _ => (0..=16)
                .map(|i| {
                    let t = i as f32 / 16.0;
                    let (c1, c2) = (pos2(mx + 8.0, p0.y), pos2(mx - 8.0, p1.y));
                    let u = 1.0 - t;
                    let x = u * u * u * p0.x + 3.0 * u * u * t * c1.x + 3.0 * u * t * t * c2.x + t * t * t * p1.x;
                    let y = u * u * u * p0.y + 3.0 * u * u * t * c1.y + 3.0 * u * t * t * c2.y + t * t * t * p1.y;
                    pos2(x, y)
                })
                .collect(),
        };
        self.in_path(&pts, self.reveal(0.15, 0.7), self.accent(), 1.5, true);
    }

    /// The first `k` of the polyline `pts` (by length), with an arrowhead at its end when `head`.
    pub(super) fn in_path(&self, pts: &[Pos2], k: f32, c: Color32, w: f32, head: bool) {
        let total: f32 = pts.windows(2).map(|s| if let [a, b] = s { (*b - *a).length() } else { 0.0 }).sum();
        let mut left = total * k.clamp(0.0, 1.0);
        let mut last: Option<(Pos2, Pos2)> = None;
        for s in pts.windows(2) {
            let [a, b] = s else { continue };
            let len = (*b - *a).length();
            if left <= 0.0 {
                break;
            }
            let end = if len <= left { *b } else { *a + (*b - *a) * (left / len.max(0.001)) };
            self.line(*a, end, c, w);
            last = Some((*a, end));
            left -= len;
        }
        if let (true, Some((a, b))) = (head, last) {
            let d = (b - a).normalized();
            if d.length() > 0.0 {
                let n = vec2(-d.y, d.x);
                self.line(b, b - d * 5.0 + n * 3.5, c, w);
                self.line(b, b - d * 5.0 - n * 3.5, c, w);
            }
        }
    }

    // ---- Field and symbol dialogs ----

    /// The Field box: a list of field names and the «field» chip they become.
    fn in_field_dialog(&self) {
        let inner = self.dialog("Field", 0);
        for (i, name) in ["DATE", "NUMPAGES", "FILENAME"].iter().enumerate() {
            let y = inner.min.y + 2.0 + i as f32 * 12.0;
            let row = Rect::from_min_size(pos2(inner.min.x, y - 5.0), vec2(50.0, 11.0));
            if i == 1 {
                self.selection(row);
            }
            self.label(pos2(inner.min.x + 3.0, y), name, 6.0, if i == 1 { self.accent() } else { self.ink() }, Align2::LEFT_CENTER);
        }
        let k = self.reveal(0.2, 0.6);
        let chip = pos2(inner.max.x - 30.0, inner.min.y + 12.0);
        self.field(chip, 28.0 * k.max(0.1));
        self.label(chip + vec2(14.0 * k.max(0.1), 3.5), "٧", 6.5, self.accent().linear_multiply(k), Align2::CENTER_CENTER);
        self.button(pos2(inner.center().x, inner.max.y - 8.0), "OK", false);
    }

    /// The Symbol box: a wide grid of characters, one chosen.
    fn in_symbol_dialog(&self) {
        let inner = self.dialog("Symbol", 0);
        let g = Rect::from_min_size(inner.min, vec2(inner.width(), 36.0));
        self.p.rect_filled(g, 0.0, Color32::WHITE);
        self.grid(g, 3, 8, self.t.border);
        let k = self.reveal(0.2, 0.6);
        let syms = ["©", "®", "™", "°", "±", "×", "÷", "≠", "≤", "≥", "∞", "√", "∑", "π", "Ω", "µ", "←", "→", "↑", "↓", "☑", "☐", "★", "♥"];
        for (i, s) in syms.iter().enumerate() {
            let cell = self.cell(g, 3, 8, i / 8, i % 8);
            if i == 20 {
                self.p.rect_filled(cell.shrink(0.5), 0.0, self.t.selection.linear_multiply(k));
            }
            self.label(cell.center(), s, 6.0, if i == 20 { self.accent() } else { self.ink() }, Align2::CENTER_CENTER);
        }
        self.button(pos2(inner.center().x, inner.max.y - 7.0), "OK", false);
    }
}
