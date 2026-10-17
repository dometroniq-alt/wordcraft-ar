//! Design tab thumbnails: themes (colours, fonts, spacing, effects) and the page background
//! (watermark, page colour, page borders), plus the items of their menus — each theme in its
//! own colours, each spacing preset, each watermark word — and the Page Setup menus that the
//! Layout tab shares (margins, orientation, columns, text direction), whose items are drawn
//! here beside the Design ones. Little films: swatches pop in, a stamp comes down, a page
//! turns on its side, margins measure themselves.

use egui::{Align2, Color32, Rect, Stroke, pos2, vec2};

use super::Entry;
use super::draw::{Dir, Draw};

pub const ENTRIES: &[Entry] = &[
    Entry::new(
        "design.theme",
        "Applies a matched set of colours, fonts and effects to the whole document in one click. Example: choose a theme and every heading, shape and chart changes to its colours.",
        |d| d.ds_theme(None),
    ),
    Entry::new(
        "design.theme:Craft",
        "The Craft theme: deep blue headings with orange accents, in the program's own fonts. Example: headings turn blue and shapes orange.",
        |d| d.ds_theme(Some("Craft")),
    ),
    Entry::new(
        "design.theme:Studio",
        "The Studio theme: wine-red headings with gold accents, in the classic Georgia font. Example: headings turn dark red and the text takes a book-like look.",
        |d| d.ds_theme(Some("Studio")),
    ),
    Entry::new(
        "design.theme:Gallery",
        "The Gallery theme: black headings with bright red accents, in the plain Helvetica Neue font. Example: headings turn black and shapes red.",
        |d| d.ds_theme(Some("Gallery")),
    ),
    Entry::new(
        "design.theme:Atelier",
        "The Atelier theme: soft purple headings with rose accents, in the Palatino font. Example: headings turn mauve and shapes pink.",
        |d| d.ds_theme(Some("Atelier")),
    ),
    Entry::new(
        "design.theme:Darkroom",
        "The Darkroom theme: dark teal headings with turquoise and yellow accents, in the Futura and Avenir Next fonts. Example: headings turn deep green-blue and shapes turquoise.",
        |d| d.ds_theme(Some("Darkroom")),
    ),
    Entry::new(
        "design.theme:Sketchbook",
        "The Sketchbook theme: leaf-green headings with soft green and sand accents, in the Gill Sans font. Example: headings turn green and shapes pale green.",
        |d| d.ds_theme(Some("Sketchbook")),
    ),
    Entry::new(
        "design.themeColors",
        "Changes only the set of colours used by headings, shapes and charts throughout the document; the fonts stay as they are. Example: a blue document becomes a green one in one click.",
        |d| d.ds_theme_colors(None),
    ),
    Entry::new(
        "design.themeColors:Craft",
        "Only the Craft colours: deep blue with orange; the fonts stay as they are. Example: headings turn blue and shapes orange, in the same font.",
        |d| d.ds_theme_colors(Some("Craft")),
    ),
    Entry::new(
        "design.themeColors:Studio",
        "Only the Studio colours: wine red with gold; the fonts stay as they are. Example: headings turn dark red and shapes gold, in the same font.",
        |d| d.ds_theme_colors(Some("Studio")),
    ),
    Entry::new(
        "design.themeColors:Gallery",
        "Only the Gallery colours: black with bright red; the fonts stay as they are. Example: headings turn black and shapes red, in the same font.",
        |d| d.ds_theme_colors(Some("Gallery")),
    ),
    Entry::new(
        "design.themeColors:Atelier",
        "Only the Atelier colours: soft purple with rose; the fonts stay as they are. Example: headings turn mauve and shapes pink, in the same font.",
        |d| d.ds_theme_colors(Some("Atelier")),
    ),
    Entry::new(
        "design.themeColors:Darkroom",
        "Only the Darkroom colours: dark teal with turquoise and yellow; the fonts stay as they are. Example: headings turn deep green-blue and shapes turquoise, in the same font.",
        |d| d.ds_theme_colors(Some("Darkroom")),
    ),
    Entry::new(
        "design.themeColors:Sketchbook",
        "Only the Sketchbook colours: leaf green with soft green and sand; the fonts stay as they are. Example: headings turn green and shapes pale green, in the same font.",
        |d| d.ds_theme_colors(Some("Sketchbook")),
    ),
    Entry::new(
        "design.themeFonts",
        "Changes the pair of fonts used for headings and for body text throughout the document. Example: every heading switches to Georgia and the body text with it.",
        |d| d.ds_theme_fonts(),
    ),
    Entry::new(
        "design.paragraphSpacing",
        "Sets, for the whole document at once, how much space lies between paragraphs and between lines. Example: choose Open and every paragraph gets a blank band after it.",
        |d| {
            d.before_after(
                |d, a| {
                    d.text_lines(a, true, &[50.0, 46.0, 28.0], 9.0, d.grey());
                    d.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 28.0), a.max), true, &[48.0, 50.0, 24.0], 9.0, d.grey());
                },
                |d, a| {
                    d.text_lines(a, true, &[50.0, 46.0, 28.0], 10.0, d.grey());
                    d.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 44.0), a.max), true, &[48.0, 50.0, 24.0], 10.0, d.grey());
                    d.measure_v(pos2(a.min.x + 3.0, a.min.y + 28.0), pos2(a.min.x + 3.0, a.min.y + 44.0));
                },
            )
        },
    ),
    Entry::new(
        "design.paragraphSpacing:default",
        "The normal spacing: a small gap after each paragraph and the lines slightly apart. Example: a letter with light air between its paragraphs.",
        |d| d.ds_spacing(8.0, 1.15),
    ),
    Entry::new(
        "design.paragraphSpacing:none",
        "No gap between paragraphs and the lines close together: the most text on a page. Example: a long list with no space between its items.",
        |d| d.ds_spacing(0.0, 1.0),
    ),
    Entry::new(
        "design.paragraphSpacing:compact",
        "A tiny gap after each paragraph and the lines close together. Example: paragraphs almost touching, yet still told apart.",
        |d| d.ds_spacing(4.0, 1.0),
    ),
    Entry::new(
        "design.paragraphSpacing:tight",
        "A small gap after each paragraph and the lines a little apart. Example: a dense report that still breathes.",
        |d| d.ds_spacing(6.0, 1.15),
    ),
    Entry::new(
        "design.paragraphSpacing:open",
        "A wide gap after each paragraph and the lines a little apart. Example: paragraphs clearly separated by a blank band.",
        |d| d.ds_spacing(10.0, 1.15),
    ),
    Entry::new(
        "design.paragraphSpacing:relaxed",
        "A small gap after each paragraph and the lines well apart, one and a half lines. Example: text that is easy to read aloud from.",
        |d| d.ds_spacing(6.0, 1.5),
    ),
    Entry::new(
        "design.paragraphSpacing:double",
        "A gap after each paragraph and the lines twice as far apart as normal. Example: a draft with room to write corrections between the lines.",
        |d| d.ds_spacing(8.0, 2.0),
    ),
    Entry::new(
        "design.effects",
        "Changes how shapes and pictures look everywhere in the document: flat, with soft shadows, or with strong edges. Example: every box in the document gets a soft shadow.",
        |d| d.ds_effects(),
    ),
    Entry::new(
        "design.setDefault",
        "Makes the current theme, fonts and spacing the starting point for every new document. Example: set your office font once and every new letter opens with it.",
        |d| d.ds_set_default(),
    ),
    Entry::new(
        "design.watermark",
        "Prints a faint word diagonally behind the text of every page. Example: «نموذج» in light grey across each page of a draft.",
        |d| {
            d.page_rl();
            d.ds_stamp(d.at(88.0, 52.0), "نموذج", 16.0);
        },
    ),
    Entry::new(
        "design.watermark:DRAFT",
        "Prints the faint word DRAFT diagonally behind the text of every page. Example: a letter not yet approved is marked DRAFT on each page.",
        |d| d.ds_watermark_word("DRAFT"),
    ),
    Entry::new(
        "design.watermark:CONFIDENTIAL",
        "Prints the faint word CONFIDENTIAL diagonally behind the text of every page. Example: a personnel file marked CONFIDENTIAL on each page.",
        |d| d.ds_watermark_word("CONFIDENTIAL"),
    ),
    Entry::new(
        "design.watermark:DO NOT COPY",
        "Prints the faint words DO NOT COPY diagonally behind the text of every page. Example: an original document marked DO NOT COPY on each page.",
        |d| d.ds_watermark_word("DO NOT COPY"),
    ),
    Entry::new(
        "design.watermark:SAMPLE",
        "Prints the faint word SAMPLE diagonally behind the text of every page. Example: a form shown only as an example is marked SAMPLE.",
        |d| d.ds_watermark_word("SAMPLE"),
    ),
    Entry::new(
        "design.watermark:ASAP",
        "Prints the faint word ASAP (as soon as possible) diagonally behind the text of every page. Example: a request that must be handled at once is marked ASAP.",
        |d| d.ds_watermark_word("ASAP"),
    ),
    Entry::new(
        "design.watermark:URGENT",
        "Prints the faint word URGENT diagonally behind the text of every page. Example: an urgent memo is marked URGENT on each page.",
        |d| d.ds_watermark_word("URGENT"),
    ),
    Entry::new(
        "ui.dialog:watermark",
        "Opens a box where you type your own watermark word, pick its colour and choose whether it lies diagonally or straight. Example: the faint word «سري» in red across each page.",
        |d| {
            let inner = d.dialog("Watermark", 2);
            d.label(pos2(inner.min.x + 40.0, inner.min.y + 5.0), "سري", 7.0, d.ink(), Align2::LEFT_CENTER);
            d.ds_stamp(pos2(inner.max.x - 18.0, inner.max.y - 12.0), "سري", 11.0);
        },
    ),
    Entry::new(
        "design.watermark:remove",
        "Takes the faint word off every page. Example: the DRAFT mark disappears once the letter is approved.",
        |d| {
            d.before_after(
                |d, a| {
                    d.paragraph(a, Dir::Rtl, 6, d.grey());
                    d.ds_stamp(a.center(), "DRAFT", 12.0);
                },
                |d, a| d.paragraph(a, Dir::Rtl, 6, d.grey()),
            )
        },
    ),
    Entry::new("design.pageColor", "Colours the background of every page instead of plain white. Example: a cream page for a certificate.", |d| {
        d.ds_page_color()
    }),
    Entry::new("design.pageBorders", "Draws a frame around the edge of every page. Example: a certificate with a thin frame along its edges.", |d| {
        d.ds_page_borders()
    }),
    Entry::new(
        "layout.orientation:portrait",
        "Stands the page upright: taller than wide, as a letter is. Example: an A4 letter page, 21 cm wide and 29.7 cm tall.",
        |d| d.ds_orientation(false),
    ),
    Entry::new(
        "layout.orientation:landscape",
        "Turns the page on its side: wider than tall. Example: a wide table of twelve columns fits on a sideways page.",
        |d| d.ds_orientation(true),
    ),
    Entry::new(
        "ui.dialog:pageSetup",
        "Opens a box where you type the margins, the orientation and the paper size exactly. Example: set a 3 cm margin on the right for binding.",
        |d| {
            let inner = d.dialog("Page Setup", 2);
            let pg = d.page(inner.max.x - 26.0, inner.min.y + 2.0, 22.0, 30.0);
            d.dashed_rect(pg.expand(4.0), d.accent());
            d.label(pos2(inner.min.x + 40.0, inner.min.y + 5.0), "3 cm", 7.0, d.accent(), Align2::LEFT_CENTER);
        },
    ),
    Entry::new(
        "layout.margins:normal",
        "Normal margins: 2.54 cm of blank space on all four sides of the page. Example: the usual margins of a letter.",
        |d| d.ds_margins([8.0, 8.0, 8.0, 8.0], false),
    ),
    Entry::new(
        "layout.margins:narrow",
        "Narrow margins: 1.27 cm on all four sides, so more text fits on the page. Example: a long table squeezed onto one page.",
        |d| d.ds_margins([4.0, 4.0, 4.0, 4.0], false),
    ),
    Entry::new(
        "layout.margins:moderate",
        "Moderate margins: 2.54 cm top and bottom, 1.9 cm on the sides. Example: a little more text on each line than normal.",
        |d| d.ds_margins([8.0, 8.0, 6.0, 6.0], false),
    ),
    Entry::new(
        "layout.margins:wide",
        "Wide margins: 2.54 cm top and bottom, 5.08 cm on the sides, for short lines and room for notes. Example: a draft with space in the margins for handwritten remarks.",
        |d| d.ds_margins([8.0, 8.0, 16.0, 16.0], false),
    ),
    Entry::new(
        "layout.margins:mirrored",
        "Mirrored margins for pages printed on both sides: the inner margin, by the binding, is wider on every page. Example: a booklet bound in the middle.",
        |d| d.ds_margins([8.0, 8.0, 10.0, 8.0], true),
    ),
    Entry::new(
        "layout.margins:office2003",
        "The margins of older documents: 2.54 cm top and bottom, 3.17 cm on the sides. Example: match a form made years ago.",
        |d| d.ds_margins([8.0, 8.0, 10.0, 10.0], false),
    ),
    Entry::new(
        "ui.dialog:columns",
        "Opens a box where you set the number of columns, their widths and the gap between them. Example: two columns with a line between them.",
        |d| {
            let inner = d.dialog("Columns", 1);
            d.label(pos2(inner.min.x + 40.0, inner.min.y + 5.0), "2", 8.0, d.accent(), Align2::LEFT_CENTER);
            let pg = d.page(inner.min.x + 20.0, inner.min.y + 18.0, 64.0, 30.0);
            let half = pg.width() / 2.0 - 3.0;
            d.text_lines(Rect::from_min_max(pg.min, pos2(pg.min.x + half, pg.max.y)), true, &[half, half, half * 0.6], 5.0, d.grey());
            d.text_lines(Rect::from_min_max(pos2(pg.max.x - half, pg.min.y), pg.max), true, &[half, half, half * 0.6], 5.0, d.grey());
            d.line(pos2(pg.center().x, pg.min.y), pos2(pg.center().x, pg.max.y), d.accent(), 1.0);
        },
    ),
    Entry::new(
        "layout.columns:left",
        "Two columns with the narrow one on the left and the wide one on the right. Example: a short note beside the main text.",
        |d| d.ds_columns(true),
    ),
    Entry::new(
        "layout.columns:right",
        "Two columns with the narrow one on the right and the wide one on the left. Example: a sidebar of notes to the right of the main text.",
        |d| d.ds_columns(false),
    ),
    Entry::new(
        "layout.textDirection:lrTb",
        "Text runs the normal way: in lines across the page, from the top down. Example: an ordinary letter.",
        |d| d.ds_text_direction(0),
    ),
    Entry::new(
        "layout.textDirection:tbRl",
        "Turns all the text 90°: the lines run from the top down, stacked from the right to the left. Example: a label read with the page turned on its side.",
        |d| d.ds_text_direction(1),
    ),
    Entry::new(
        "layout.textDirection:btLr",
        "Turns all the text 270°: the lines run from the bottom up, stacked from the left to the right. Example: a tall table heading read from the bottom.",
        |d| d.ds_text_direction(2),
    ),
];

impl Draw<'_> {
    /// The first three colours of a named theme, or the kit's own trio.
    fn ds_palette(&self, name: Option<&str>) -> [Color32; 3] {
        let own = [self.accent(), self.green(), Color32::from_rgb(220, 150, 20)];
        let Some(name) = name else { return own };
        wordcraft_engine::cmd::design::THEMES
            .iter()
            .find(|t| t.0 == name)
            .map(|t| [crate::theme::c32(t.3[0]), crate::theme::c32(t.3[1]), crate::theme::c32(t.3[2])])
            .unwrap_or(own)
    }

    /// Three colour swatches in a row at `y`, popping in one after the other.
    fn ds_swatches(&self, x: f32, y: f32, colours: [Color32; 3]) {
        for (i, c) in colours.iter().enumerate() {
            let pop = self.reveal(0.05 + i as f32 * 0.08, 0.2 + i as f32 * 0.08);
            if pop <= 0.0 {
                continue;
            }
            let r = Rect::from_center_size(self.at(x + 6.0 + i as f32 * 16.0, y + 6.0), vec2(12.0, 12.0) * pop);
            self.p.rect_filled(r, 2.0, *c);
        }
    }

    /// Swatches plus a font sample beside a page whose heading and shape take the colours.
    fn ds_theme(&self, name: Option<&str>) {
        let [c0, c1, _] = self.ds_palette(name);
        self.ds_swatches(10.0, 18.0, self.ds_palette(name));
        self.glyph(self.at(32.0, 48.0), "Aa", 16.0, self.ink());
        if let Some(n) = name {
            self.label(self.at(32.0, 70.0), n, 7.5, c0, Align2::CENTER_CENTER);
        }
        let go = self.reveal(0.25, 0.45);
        if go > 0.0 {
            self.arrow_right(self.at(62.0, 50.0), self.at(62.0 + 18.0 * go, 50.0));
        }
        let a = self.page(90.0, 8.0, 70.0, 88.0);
        let k = self.reveal(0.45, 0.8);
        let heading = self.ds_mix(self.grey(), c0, k);
        self.heading_then_text(a, Dir::Rtl, 2, heading);
        let r = Rect::from_min_size(pos2(a.min.x + 10.0, a.min.y + 40.0), vec2(34.0, 20.0));
        let fill = self.ds_mix(self.grey(), c1, k);
        self.p.rect_filled(r, 4.0, fill.linear_multiply(0.35));
        self.p.rect_stroke(r, 4.0, Stroke::new(1.5, fill), egui::StrokeKind::Inside);
    }

    /// A short horizontal measure: a line with a tick at each end (for spans too short for
    /// arrowheads).
    fn ds_ticks_h(&self, a: egui::Pos2, b: egui::Pos2) {
        let c = self.accent();
        self.line(a, b, c, 1.5);
        self.line(a + vec2(0.0, -3.0), a + vec2(0.0, 3.0), c, 1.5);
        self.line(b + vec2(0.0, -3.0), b + vec2(0.0, 3.0), c, 1.5);
    }

    /// A short vertical measure with end ticks.
    fn ds_ticks_v(&self, a: egui::Pos2, b: egui::Pos2) {
        let c = self.accent();
        self.line(a, b, c, 1.5);
        self.line(a + vec2(-3.0, 0.0), a + vec2(3.0, 0.0), c, 1.5);
        self.line(b + vec2(-3.0, 0.0), b + vec2(3.0, 0.0), c, 1.5);
    }

    /// `a` blended towards `b` by `k`.
    fn ds_mix(&self, a: Color32, b: Color32, k: f32) -> Color32 {
        let ch = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * k).round().clamp(0.0, 255.0) as u8;
        Color32::from_rgb(ch(a.r(), b.r()), ch(a.g(), b.g()), ch(a.b(), b.b()))
    }

    /// Two colour sets: the page's heading and shape change from grey to the theme's colours.
    fn ds_theme_colors(&self, name: Option<&str>) {
        let [c0, c1, _] = self.ds_palette(name);
        self.before_after(
            |d, a| {
                d.heading_then_text(a, Dir::Rtl, 2, d.grey());
                let r = Rect::from_min_size(pos2(a.min.x + 8.0, a.min.y + 40.0), vec2(34.0, 20.0));
                d.p.rect_filled(r, 4.0, d.grey().linear_multiply(0.4));
                d.p.rect_stroke(r, 4.0, Stroke::new(1.5, d.grey()), egui::StrokeKind::Inside);
            },
            |d, a| {
                d.heading_then_text(a, Dir::Rtl, 2, c0);
                let r = Rect::from_min_size(pos2(a.min.x + 8.0, a.min.y + 40.0), vec2(34.0, 20.0));
                d.p.rect_filled(r, 4.0, c1.linear_multiply(0.35));
                d.p.rect_stroke(r, 4.0, Stroke::new(1.5, c1), egui::StrokeKind::Inside);
            },
        );
        self.ds_swatches(66.0, 80.0, self.ds_palette(name));
    }

    /// A heading "Aa" and a body "Aa" in two styles on one page; the heading grows in.
    fn ds_theme_fonts(&self) {
        let a = self.page_wide();
        let k = self.reveal(0.15, 0.55);
        self.glyph(pos2(a.max.x - 22.0, a.min.y + 14.0), "Aa", 10.0 + 8.0 * k, self.accent());
        self.line(pos2(a.max.x - 48.0 - (a.width() - 48.0) * k, a.min.y + 14.0), pos2(a.max.x - 48.0, a.min.y + 14.0), self.accent(), 3.0);
        self.label(pos2(a.max.x - 16.0, a.min.y + 40.0), "Aa", 10.0, self.ink(), Align2::CENTER_CENTER);
        self.text_lines(
            Rect::from_min_max(pos2(a.min.x, a.min.y + 32.0), pos2(a.max.x - 34.0, a.max.y)),
            true,
            &[60.0, 66.0, 40.0],
            9.0,
            self.grey(),
        );
        self.label(pos2(a.center().x, a.max.y - 2.0), "عنوان / نص", 7.0, self.grey(), Align2::CENTER_CENTER);
    }

    /// Three two-line paragraphs on one page, `after` points of gap between them and lines
    /// `line` times the normal distance apart; in motion they spread out from no spacing.
    fn ds_spacing(&self, after: f32, line: f32) {
        let a = self.page_center();
        let k = self.reveal(0.15, 0.6);
        let pitch = 5.0 * (1.0 + (line - 1.0) * k);
        let gap = after * 0.75 * k;
        let widths = [[1.0, 0.6], [0.95, 0.8], [0.9, 0.5]];
        let mut y = a.min.y + 4.0;
        for (i, ws) in widths.iter().enumerate() {
            for w in ws {
                self.line(pos2(a.max.x - a.width() * w, y), pos2(a.max.x, y), self.grey(), 1.5);
                y += pitch;
            }
            // The gap after the first paragraph: a tinted band the height of the space.
            if i == 0 && gap > 1.0 {
                let band = Rect::from_min_max(pos2(a.min.x, y - pitch * 0.5), pos2(a.max.x, y - pitch * 0.5 + gap));
                self.p.rect_filled(band, 1.0, self.accent().linear_multiply(0.18));
            }
            y += gap;
        }
        if line > 1.0 {
            self.ds_ticks_v(pos2(a.max.x + 4.0, a.min.y + 4.0), pos2(a.max.x + 4.0, a.min.y + 4.0 + pitch));
        }
    }

    /// The same shape flat and with a soft shadow.
    fn ds_effects(&self) {
        self.before_after(
            |d, a| d.shape(Rect::from_center_size(a.center(), vec2(40.0, 30.0))),
            |d, a| {
                let r = Rect::from_center_size(a.center(), vec2(40.0, 30.0));
                for i in (1..=4).rev() {
                    d.p.rect_filled(r.translate(vec2(i as f32, i as f32)), 4.0, d.grey().linear_multiply(0.12));
                }
                d.shape(r);
            },
        );
    }

    /// The current page's look, ticked, flowing into a stack of new pages that appear one
    /// after the other.
    fn ds_set_default(&self) {
        let a = self.page(8.0, 8.0, 60.0, 80.0);
        self.heading_then_text(a, Dir::Rtl, 4, self.accent());
        if self.reveal(0.05, 0.2) > 0.0 {
            self.check(self.at(62.0, 80.0));
        }
        let go = self.reveal(0.2, 0.4);
        if go > 0.0 {
            self.arrow_right(self.at(76.0, 48.0), self.at(76.0 + 18.0 * go, 48.0));
        }
        let n = 1 + (self.reveal(0.4, 0.8) * 2.999) as usize;
        let top = self.page_stack(100.0, 8.0, 56.0, 68.0, n.min(3));
        self.heading_then_text(top, Dir::Rtl, 3, self.accent());
    }

    /// A faint diagonal word centred at `c`, `size` tall when stamped: it comes down big and
    /// fainter, and settles.
    fn ds_stamp(&self, c: egui::Pos2, text: &str, size: f32) {
        let land = self.reveal(0.15, 0.55);
        if land <= 0.0 {
            return;
        }
        let colour = self.accent().linear_multiply(0.12 + 0.33 * land);
        let galley = self.p.layout_no_wrap(crate::i18n::artwork(text).to_string(), self.font(size + (1.0 - land) * 8.0), colour);
        let shape = egui::epaint::TextShape::new(c, galley, colour).with_angle_and_anchor(-std::f32::consts::FRAC_PI_4, Align2::CENTER_CENTER);
        self.p.add(shape);
    }

    /// A page of text with `word` stamped faintly across it, sized to fit the page.
    fn ds_watermark_word(&self, word: &str) {
        let a = self.page_rl();
        let size = (90.0 / word.chars().count().max(1) as f32).clamp(7.0, 15.0);
        self.ds_stamp(a.center(), word, size);
    }

    /// A page tinted in a colour instead of white.
    fn ds_page_color(&self) {
        self.before_after(
            |d, a| d.paragraph(a, Dir::Rtl, 6, d.grey()),
            |d, a| {
                d.p.rect_filled(a.expand(7.0), 2.0, d.accent().linear_multiply(0.18));
                d.paragraph(a, Dir::Rtl, 6, d.grey());
            },
        );
    }

    /// An accent frame just inside the page edge, on two pages, fading in.
    fn ds_page_borders(&self) {
        let k = self.reveal(0.15, 0.55);
        for x in [26.0, 94.0] {
            let a = self.page(x, 8.0, 56.0, 88.0);
            self.p.rect_stroke(a.expand(4.0), 0.0, Stroke::new(2.0, self.accent().linear_multiply(k.max(0.02))), egui::StrokeKind::Inside);
            self.paragraph(a.shrink2(vec2(2.0, 4.0)), Dir::Rtl, 5, self.grey());
        }
    }

    /// One page that turns from upright to sideways (`landscape`) or back, text reflowing.
    fn ds_orientation(&self, landscape: bool) {
        let k = self.reveal(0.15, 0.6);
        let (w0, h0, w1, h1) = if landscape { (76.0, 92.0, 120.0, 84.0) } else { (120.0, 84.0, 76.0, 92.0) };
        let w = w0 + (w1 - w0) * k;
        let h = h0 + (h1 - h0) * k;
        let a = self.page((super::SIZE.x - w) / 2.0, (super::SIZE.y - h) / 2.0, w, h);
        let n = ((a.height() - 4.0) / 10.0).max(1.0) as usize;
        self.paragraph(a, Dir::Rtl, n.min(7), self.grey());
        if landscape {
            self.measure_h(pos2(a.min.x, a.max.y + 4.0), pos2(a.max.x, a.max.y + 4.0));
        } else {
            self.measure_v(pos2(a.max.x + 4.0, a.min.y), pos2(a.max.x + 4.0, a.max.y));
        }
    }

    /// A page with its text held inside dashed margins `[top, bottom, left, right]` (drawing
    /// units); `mirrored` shows two facing pages whose inner margins are the wider ones.
    fn ds_margins(&self, m: [f32; 4], mirrored: bool) {
        let [top, bottom, left, right] = m;
        let k = self.reveal(0.15, 0.5);
        let guide = self.accent().linear_multiply(k.max(0.02));
        if mirrored {
            for (i, x) in [30.0, 92.0].into_iter().enumerate() {
                let r = Rect::from_min_size(self.at(x, 10.0), vec2(54.0, 84.0));
                self.p.rect_filled(r, 2.0, Color32::WHITE);
                self.p.rect_stroke(r, 2.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
                // The inner edge (by the binding) takes the wider margin.
                let (l, rr) = if i == 0 { (right, left) } else { (left, right) };
                let inner = Rect::from_min_max(pos2(r.min.x + l, r.min.y + top), pos2(r.max.x - rr, r.max.y - bottom));
                self.dashed_rect(inner, guide);
                self.paragraph(inner.shrink2(vec2(1.0, 2.0)), Dir::Rtl, 6, self.grey());
            }
            self.line(self.at(88.0, 8.0), self.at(88.0, 96.0), self.grey(), 1.0);
            return;
        }
        let r = Rect::from_min_size(self.at(50.0, 6.0), vec2(76.0, 92.0));
        self.p.rect_filled(r, 2.0, Color32::WHITE);
        self.p.rect_stroke(r, 2.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        let inner = Rect::from_min_max(pos2(r.min.x + left, r.min.y + top), pos2(r.max.x - right, r.max.y - bottom));
        self.dashed_rect(inner, guide);
        self.paragraph(inner.shrink2(vec2(1.0, 2.0)), Dir::Rtl, 7, self.grey());
        if k > 0.0 {
            self.ds_ticks_h(pos2(r.max.x - right * k, r.min.y + 20.0), pos2(r.max.x, r.min.y + 20.0));
            self.ds_ticks_v(pos2(r.min.x + 20.0, r.min.y), pos2(r.min.x + 20.0, r.min.y + top * k));
        }
    }

    /// A wide page split into a narrow column (on the `left`) and a wide one.
    fn ds_columns(&self, left: bool) {
        let a = self.page_wide();
        let k = self.reveal(0.15, 0.55);
        let narrow = a.width() * 0.3;
        let gap = 6.0;
        let (n, w) = if left {
            (Rect::from_min_max(a.min, pos2(a.min.x + narrow, a.max.y)), Rect::from_min_max(pos2(a.min.x + narrow + gap, a.min.y), a.max))
        } else {
            (Rect::from_min_max(pos2(a.max.x - narrow, a.min.y), a.max), Rect::from_min_max(a.min, pos2(a.max.x - narrow - gap, a.max.y)))
        };
        self.paragraph(w, Dir::Rtl, 6, self.grey());
        let nw = n.width() * k;
        self.text_lines(n, true, &[nw, nw * 0.9, nw * 0.95, nw * 0.6], 10.0, self.accent());
        let x = if left { n.max.x + gap / 2.0 } else { n.min.x - gap / 2.0 };
        self.line(pos2(x, a.min.y), pos2(x, a.max.y), self.t.border, 1.0);
    }

    /// Text lines running across (0), down from the top stacked right to left (1), or up from
    /// the bottom stacked left to right (2), with an arrow for the reading direction.
    fn ds_text_direction(&self, how: u8) {
        let a = self.page_center();
        let k = self.reveal(0.15, 0.55);
        let widths = [1.0, 0.94, 0.98, 0.9, 0.96, 0.55];
        match how {
            0 => {
                self.paragraph(a, Dir::Rtl, 6, self.grey());
                self.arrow_left(pos2(a.max.x, a.max.y - 2.0), pos2(a.max.x - 24.0 * k.max(0.1), a.max.y - 2.0));
            }
            1 => {
                for (i, f) in widths.iter().enumerate() {
                    let x = a.max.x - 4.0 - i as f32 * 10.0;
                    let len = a.height() * f * k;
                    self.line(pos2(x, a.min.y), pos2(x, a.min.y + len), self.grey(), 1.5);
                }
                self.arrow_down(pos2(a.min.x + 2.0, a.min.y), pos2(a.min.x + 2.0, a.min.y + 24.0 * k.max(0.1)));
            }
            _ => {
                for (i, f) in widths.iter().enumerate() {
                    let x = a.min.x + 4.0 + i as f32 * 10.0;
                    let len = a.height() * f * k;
                    self.line(pos2(x, a.max.y), pos2(x, a.max.y - len), self.grey(), 1.5);
                }
                self.arrow_up(pos2(a.max.x - 2.0, a.max.y), pos2(a.max.x - 2.0, a.max.y - 24.0 * k.max(0.1)));
            }
        }
    }
}
