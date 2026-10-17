//! Layout tab thumbnails: page setup (margins, orientation, size, columns, breaks, line numbers,
//! hyphenation, text direction) and arranging pictures and shapes on the page — plus the items
//! of the tab's menus (the seven break kinds, the line-number modes, the rotations) and the
//! contextual Picture Format, Shape Format and Chart Design menus. Each a little film: the page
//! turns, the cut draws, the picture slides, the colour drains.

use egui::{Align2, Color32, Pos2, Rect, Shape, Stroke, pos2, vec2};

use super::Entry;
use super::draw::{Dir, Draw};

pub const ENTRIES: &[Entry] = &[
    Entry::new(
        "layout.margins",
        "Sets how much empty space is left between the edge of the paper and the text, on all four sides. Example: narrow margins fit more words on a page; wide margins leave room to write notes.",
        |d| d.lay_margins(),
    ),
    Entry::new(
        "layout.orientation",
        "Turns the page sideways (wider than it is tall) or back upright. Example: a wide table fits on a sideways page.",
        |d| d.lay_orientation(),
    ),
    Entry::new(
        "layout.size",
        "Picks the size of the paper the pages are made for, so printing matches. Example: A4 for letters, A5 for a small booklet.",
        |d| d.lay_paper_size(),
    ),
    Entry::new(
        "layout.columns",
        "Splits the text into two or more columns side by side, like a newspaper page. Example: a long list in two columns uses half the pages.",
        |d| d.lay_columns(),
    ),
    Entry::new(
        "layout.break",
        "Starts a new page, column or section at the cursor, instead of pressing Enter many times. Example: the next heading begins at the top of a fresh page, and stays there when you edit the text above it.",
        |d| d.lay_breaks(false),
    ),
    Entry::new(
        "layout.break:page",
        "Ends the page at the cursor; the text after it starts at the top of the next page. Example: the cursor is after the last line of a chapter and the next chapter jumps to a new page.",
        |d| d.lay_breaks(true),
    ),
    Entry::new(
        "layout.break:column",
        "Ends the column at the cursor; the text after it starts at the top of the next column. Example: on a two-column page, the list goes on at the top of the second column instead of the bottom of the first.",
        |d| d.lay_break_column(),
    ),
    Entry::new(
        "layout.break:textWrapping",
        "Ends the line at the cursor and goes on at the next line without starting a new paragraph, so no extra space is added. Example: an address broken into lines that stay one paragraph.",
        |d| d.lay_break_wrap(),
    ),
    Entry::new(
        "layout.break:nextPage",
        "Starts a new section on the next page; each section can have its own margins, orientation, header or page numbers. Example: the appendix starts on a new page with a sideways layout of its own.",
        |d| d.lay_section_next_page(),
    ),
    Entry::new(
        "layout.break:continuous",
        "Starts a new section on the same page, without a new page, so the part below can have different columns or margins. Example: the middle of the page switches to two columns.",
        |d| d.lay_section_continuous(),
    ),
    Entry::new(
        "layout.break:evenPage",
        "Starts a new section on the next even-numbered page, leaving a blank page if needed. Example: the text ends on page ٢, page ٣ stays empty, and the new section starts on page ٤.",
        |d| d.lay_section_parity(true),
    ),
    Entry::new(
        "layout.break:oddPage",
        "Starts a new section on the next odd-numbered page, leaving a blank page if needed, so every chapter opens on the same side of the book. Example: the text ends on page ١, page ٢ stays empty, and the new section starts on page ٣.",
        |d| d.lay_section_parity(false),
    ),
    Entry::new(
        "layout.lineNumbers",
        "Writes a small number in the margin beside every line of text, so people can refer to a line. Example: «see line ١٢» instead of describing where the sentence is.",
        |d| d.lay_line_numbers(),
    ),
    Entry::new(
        "layout.lineNumbers:none",
        "Removes the line numbers from the margin. Example: the numbers ١ to ٢٥ beside the lines disappear.",
        |d| d.lay_line_numbers_none(),
    ),
    Entry::new(
        "layout.lineNumbers:continuous",
        "Numbers every line of the whole document in one count, going on from page to page. Example: page ١ has lines ١ to ٤٠ and page ٢ goes on from ٤١.",
        |d| d.lay_line_numbers_pages(false),
    ),
    Entry::new(
        "layout.lineNumbers:restartPage",
        "Numbers the lines from ١ again at the top of every page. Example: every page starts with line ١ in the margin.",
        |d| d.lay_line_numbers_pages(true),
    ),
    Entry::new(
        "layout.lineNumbers:restartSection",
        "Numbers the lines from ١ again at the start of every section. Example: the count starts over at ١ where the new chapter's section begins.",
        |d| d.lay_line_numbers_section(),
    ),
    Entry::new(
        "ui.dialog:lineNumbers",
        "Opens a box with the details of line numbering: the number to start from, how far from the text, and whether to number every line or every fifth. Example: show a number only on lines ٥، ١٠، ١٥ and so on.",
        |d| {
            let inner = d.dialog("Line Numbers", 3);
            d.label(pos2(inner.min.x + 38.0, inner.min.y + 5.0), "١", 7.0, d.ink(), Align2::LEFT_CENTER);
            d.label(pos2(inner.min.x + 38.0, inner.min.y + 33.0), "٥", 7.0, d.accent(), Align2::LEFT_CENTER);
        },
    ),
    Entry::new(
        "layout.hyphenation",
        "Lets long words split at the end of a line with a hyphen, so the lines are even and have fewer gaps. Example: «inter-» at the end of one line and «national» at the start of the next.",
        |d| d.lay_hyphenation(),
    ),
    Entry::new(
        "ui.dialog:manualHyphenation",
        "Goes through the long words at line ends one by one and asks you where each may be split, so you decide. Example: it suggests «inter-national» and you accept or move the hyphen.",
        |d| {
            let inner = d.dialog("Hyphenate", 1);
            d.label(pos2(inner.min.x + 38.0, inner.min.y + 5.0), "inter-national", 6.5, d.accent(), Align2::LEFT_CENTER);
            d.button(pos2(inner.center().x - 22.0, inner.max.y - 12.0), "Yes", false);
            d.button(pos2(inner.center().x + 22.0, inner.max.y - 12.0), "No", false);
        },
    ),
    Entry::new(
        "ui.dialog:hyphenation",
        "Opens a box with the hyphenation settings: automatic or not, and how many lines in a row may end with a hyphen. Example: allow at most two lines in a row to end with a hyphen.",
        |d| {
            let inner = d.dialog("Hyphenation", 2);
            d.label(pos2(inner.min.x + 38.0, inner.min.y + 19.0), "٢", 7.0, d.ink(), Align2::LEFT_CENTER);
            d.button(pos2(inner.center().x, inner.max.y - 10.0), "OK", false);
        },
    ),
    Entry::new(
        "layout.pageNumberFormat",
        "Opens a box to choose how page numbers look (١، ٢، ٣ or أ، ب، ج) and which number the first page starts from. Example: start the numbering from ٥ because the cover pages are counted elsewhere.",
        |d| {
            let inner = d.dialog("Page Number Format", 1);
            d.chip(pos2(inner.min.x + 24.0, inner.min.y + 24.0), "١، ٢، ٣", 44.0);
            d.chip(pos2(inner.max.x - 24.0, inner.min.y + 24.0), "أ، ب، ج", 44.0);
            d.label(pos2(inner.min.x + 38.0, inner.min.y + 5.0), "٥", 7.0, d.accent(), Align2::LEFT_CENTER);
        },
    ),
    Entry::new(
        "layout.textDirection",
        "Turns the text to run from top to bottom in the selected cells or text box, instead of across. Example: a tall narrow heading cell in a table reads downwards.",
        |d| d.lay_text_direction(),
    ),
    Entry::new(
        "arrange.position",
        "Moves the selected picture or shape to a set spot on the page (a corner, the middle, a side) and makes the text flow around it. Example: the photo goes to the top corner and the text wraps round it.",
        |d| d.lay_position(),
    ),
    Entry::new(
        "arrange.wrap",
        "Chooses how the text goes around the selected picture: around it, above and below it only, behind it or in front of it. Example: «Square» leaves a clean rectangle of text around the photo.",
        |d| d.lay_wrap(),
    ),
    Entry::new(
        "arrange.wrapText",
        "Chooses which sides of the selected picture the text may flow on: both, left only, right only or the larger side. Example: «Right Only» keeps the left side of the photo free of text.",
        |d| d.lay_wrap_sides(),
    ),
    Entry::new(
        "arrange.bringForward",
        "Moves the selected shape one step up in the pile, so it shows on top of the shape it overlaps. Example: the blue box that was hidden now covers the grey one.",
        |d| d.lay_stack_order(true),
    ),
    Entry::new(
        "arrange.sendBackward",
        "Moves the selected shape one step down in the pile, so the shape it overlaps shows on top of it. Example: the blue box slips behind the grey one.",
        |d| d.lay_stack_order(false),
    ),
    Entry::new(
        "arrange.selectionPane",
        "Opens a list of every picture, shape and text box on the page, so you can select or hide any one, even when it is covered. Example: click «صورة ٢» in the list to select a picture hidden behind a box.",
        |d| d.lay_selection_pane(),
    ),
    Entry::new(
        "arrange.align",
        "Lines up the selected shapes with each other or with the edges of the page. Example: three boxes at different heights line up along one edge.",
        |d| d.lay_align(),
    ),
    Entry::new(
        "arrange.group",
        "Joins the selected shapes into one object that moves and resizes as one piece. Example: a box and its arrow move together.",
        |d| d.lay_group(),
    ),
    Entry::new(
        "arrange.ungroup",
        "Splits a grouped object back into its separate shapes, so each one can be moved on its own. Example: the box and its arrow become two separate shapes again.",
        |d| d.lay_ungroup(),
    ),
    Entry::new(
        "arrange.rotate",
        "Turns the selected picture or shape a quarter turn, or flips it like a mirror. Example: a sideways photo turns upright.",
        |d| d.lay_rotate(),
    ),
    Entry::new(
        "arrange.rotate:right",
        "Turns the selected picture or shape a quarter turn clockwise. Example: a photo lying on its left side comes upright.",
        |d| d.lay_rotate_kind(0),
    ),
    Entry::new(
        "arrange.rotate:left",
        "Turns the selected picture or shape a quarter turn counter-clockwise. Example: a photo lying on its right side comes upright.",
        |d| d.lay_rotate_kind(1),
    ),
    Entry::new(
        "arrange.rotate:flipVertical",
        "Flips the selected picture or shape upside down, like a reflection in water. Example: an arrow pointing up now points down.",
        |d| d.lay_rotate_kind(2),
    ),
    Entry::new(
        "arrange.rotate:flipHorizontal",
        "Flips the selected picture or shape left to right, like a mirror. Example: a person looking left now looks right.",
        |d| d.lay_rotate_kind(3),
    ),
    // ---- Chart Design ----
    Entry::new(
        "chart.dataLabels",
        "Shows or hides the value of every bar, point or slice written on the chart itself. Example: «٤٠» appears above the bar for March, so no one has to guess from the axis.",
        |d| {
            let a = d.page_rl();
            d.in_chart_bars(Rect::from_min_size(pos2(a.min.x + 4.0, a.min.y + 20.0), vec2(a.width() - 8.0, 40.0)), 1.0, true);
        },
    ),
    // ---- Shape Format ----
    Entry::new(
        "shape.fill",
        "Colours the inside of the selected shape, or leaves it empty with «No Color». Example: the rectangle becomes light blue inside.",
        |d| d.lay_shape_fill(),
    ),
    Entry::new(
        "shape.outline",
        "Colours the line around the selected shape and sets how thick it is, or removes it with «No Color». Example: a thick dark line around the box.",
        |d| d.lay_shape_outline(),
    ),
    Entry::new(
        "shape.effects",
        "Adds a shadow, a glow or soft edges to the selected shape, or removes them with the «No …» items. Example: a shadow under the box makes it look lifted off the page.",
        |d| d.lay_shape_effects(),
    ),
    Entry::new(
        "canvas.fit",
        "Shrinks the drawing area so it hugs the shapes inside it, leaving no empty space around them. Example: the big empty canvas tightens around its three boxes.",
        |d| d.lay_canvas(0),
    ),
    Entry::new(
        "canvas.expand",
        "Makes the drawing area bigger on every side, leaving room to add more shapes. Example: more space opens around the three boxes.",
        |d| d.lay_canvas(1),
    ),
    Entry::new(
        "canvas.scale",
        "Makes the whole drawing bigger or smaller by a percentage, shapes and all. Example: ٥٠٪ halves the drawing; ٢٠٠٪ doubles it.",
        |d| d.lay_canvas(2),
    ),
    // ---- Picture Format ----
    Entry::new(
        "picture.corrections",
        "Makes the selected picture brighter or darker, with more or less contrast, sharper or softer. Example: a dark photo at +20% brightness becomes easy to see.",
        |d| d.lay_corrections(),
    ),
    Entry::new(
        "picture.color:saturation",
        "Makes the colours of the selected picture stronger or weaker. Example: ٠٪ removes all colour, ٢٠٠٪ makes it vivid.",
        |d| d.lay_picture_tone(0),
    ),
    Entry::new(
        "picture.color:grayscale",
        "Turns the selected picture into shades of grey, like a black-and-white photo. Example: a colour photo prints well on a black-and-white printer.",
        |d| d.lay_picture_tone(1),
    ),
    Entry::new(
        "picture.color:sepia",
        "Gives the selected picture a brownish old-photograph look. Example: a modern photo looks like one from decades ago.",
        |d| d.lay_picture_tone(2),
    ),
    Entry::new(
        "picture.color:washout",
        "Makes the selected picture very pale, so text can be read over it. Example: a logo faded into the background of a page.",
        |d| d.lay_picture_tone(3),
    ),
    Entry::new(
        "picture.transparency",
        "Makes the selected picture see-through by a percentage, so what is behind it shows. Example: at ٥٠٪ the text behind the picture shows through it.",
        |d| d.lay_transparency(),
    ),
    Entry::new("picture.style:simpleFrame", "Puts a thin frame around the selected picture. Example: a thin dark line around the photo.", |d| {
        d.lay_picture_style(0)
    }),
    Entry::new(
        "picture.style:thickFrame",
        "Puts a wide frame around the selected picture. Example: a wide white border, like a printed photo.",
        |d| d.lay_picture_style(1),
    ),
    Entry::new("picture.style:rounded", "Rounds the corners of the selected picture. Example: the photo's sharp corners become soft curves.", |d| {
        d.lay_picture_style(2)
    }),
    Entry::new(
        "picture.style:softEdge",
        "Fades the edges of the selected picture into the page, with no hard border. Example: the photo melts into the white page at its sides.",
        |d| d.lay_picture_style(3),
    ),
    Entry::new(
        "picture.style:shadow",
        "Puts a shadow under the selected picture, so it looks lifted off the page. Example: a grey shadow at the bottom corner of the photo.",
        |d| d.lay_picture_style(4),
    ),
    Entry::new("picture.border", "Draws a line around the selected picture: thin, medium or thick. Example: a thick line frames the photo.", |d| {
        d.lay_picture_border()
    }),
    Entry::new(
        "ui.changePicture",
        "Swaps the selected picture for another file, keeping its size, place and frame. Example: replace the old logo with the new one without moving anything.",
        |d| d.lay_change_picture(),
    ),
];

/// The colours of the kit's landscape picture: sky, near hill, far hill, sun.
const PIC_NORMAL: [Color32; 4] =
    [Color32::from_rgb(214, 232, 246), Color32::from_rgb(96, 150, 96), Color32::from_rgb(70, 120, 80), Color32::from_rgb(240, 190, 60)];

impl Draw<'_> {
    /// The same drawing through a painter clipped to `clip` (a wipe).
    fn lay_clipped(&self, clip: Rect, f: impl FnOnce(&Draw<'_>)) {
        let p = self.p.with_clip_rect(clip.intersect(self.p.clip_rect()));
        f(&self.with_painter(&p));
    }

    /// `a` blended towards `b` by `k`.
    fn lay_mix(a: Color32, b: Color32, k: f32) -> Color32 {
        let k = k.clamp(0.0, 1.0);
        let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * k).round() as u8;
        Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
    }

    /// A filled polygon drawn with a consistent winding whatever transform produced the points.
    fn lay_poly(&self, mut pts: Vec<Pos2>, fill: Color32, stroke: Stroke) {
        let area: f32 = pts.iter().zip(pts.iter().cycle().skip(1)).map(|(a, b)| a.x * b.y - b.x * a.y).sum();
        if area < 0.0 {
            pts.reverse();
        }
        self.p.add(Shape::convex_polygon(pts, fill, stroke));
    }

    /// Two dashed lines close together across `x0..x1` at `y`: a section break.
    fn lay_section_mark(&self, x0: f32, x1: f32, y: f32, k: f32) {
        let end = x0 + (x1 - x0) * k;
        for dy in [-1.5, 1.5] {
            let mut x = x0;
            while x < end {
                self.line(pos2(x, y + dy), pos2((x + 3.0).min(end), y + dy), self.accent(), 1.0);
                x += 5.0;
            }
        }
    }

    /// The landscape picture `w × h` centred at `c`, every point passed through `xf` (a
    /// rotation, a flip), in the colours `tone` at opacity `alpha`.
    fn lay_pic_xf(&self, c: Pos2, w: f32, h: f32, tone: [Color32; 4], alpha: f32, xf: &dyn Fn(egui::Vec2) -> egui::Vec2) {
        let at = |x: f32, y: f32| c + xf(vec2(x, y));
        let (l, t, r, b) = (-w / 2.0, -h / 2.0, w / 2.0, h / 2.0);
        let [sky, hill1, hill2, sun] = tone;
        let a = |col: Color32| col.linear_multiply(alpha);
        self.lay_poly(vec![at(l, t), at(r, t), at(r, b), at(l, b)], a(sky), Stroke::NONE);
        let base = b - 2.0;
        self.lay_poly(vec![at(l + 2.0, base), at(l + w * 0.35, t + h * 0.45), at(l + w * 0.6, base)], a(hill1), Stroke::NONE);
        self.lay_poly(vec![at(l + w * 0.4, base), at(l + w * 0.7, t + h * 0.3), at(r - 2.0, base)], a(hill2), Stroke::NONE);
        self.p.circle_filled(at(l + w * 0.22, t + h * 0.25), h * 0.1, a(sun));
        self.p.add(Shape::closed_line(vec![at(l, t), at(r, t), at(r, b), at(l, b)], Stroke::new(1.0, a(self.t.border_strong))));
    }

    /// The landscape picture in `r`, upright, in the colours `tone` at opacity `alpha`.
    fn lay_pic_toned(&self, r: Rect, tone: [Color32; 4], alpha: f32) {
        self.lay_pic_xf(r.center(), r.width(), r.height(), tone, alpha, &|v| v);
    }

    /// A page whose text area is outlined with a dashed box; the margins grow and are measured.
    fn lay_margins(&self) {
        let page = Rect::from_min_size(self.at(50.0, 6.0), vec2(76.0, 92.0));
        let area = self.page(50.0, 6.0, 76.0, 92.0);
        let k = self.reveal(0.15, 0.6);
        let inner = area.shrink2(vec2(4.0, 6.0) * k);
        self.dashed_rect(inner, self.accent());
        self.paragraph(inner.shrink(3.0), Dir::Ltr, 5, self.grey());
        // The left margin and the top margin, measured.
        self.measure_h(pos2(page.min.x + 1.0, inner.center().y), pos2(inner.min.x, inner.center().y));
        self.measure_v(pos2(inner.center().x, page.min.y + 1.0), pos2(inner.center().x, inner.min.y));
    }

    /// An upright page turns into a sideways one.
    fn lay_orientation(&self) {
        let before = self.page(10.0, 16.0, 52.0, 72.0);
        self.paragraph(before, Dir::Ltr, 5, self.grey());
        let arrow = self.reveal(0.05, 0.3);
        if arrow > 0.0 {
            self.arrow_right(self.at(68.0, 52.0), self.at(68.0 + 16.0 * arrow, 52.0));
        }
        let k = self.reveal(0.3, 0.75);
        let (x, y, w, h) = (104.0 - 14.0 * k, 16.0 + 10.0 * k, 52.0 + 28.0 * k, 72.0 - 20.0 * k);
        let after = self.page(x, y, w, h);
        self.paragraph(after, Dir::Ltr, 3, self.grey());
    }

    /// A page with its paper name on it; the width is measured.
    fn lay_paper_size(&self) {
        let area = self.page_lr();
        let k = self.reveal(0.1, 0.5);
        if k > 0.0 {
            self.badge(area.center(), "A4", self.accent().linear_multiply(k));
        }
        let m = self.reveal(0.4, 0.8);
        let (x0, x1) = (area.min.x - 8.0, area.max.x + 8.0);
        let half = (x1 - x0) / 2.0 * m;
        if m > 0.0 {
            self.measure_h(pos2(area.center().x - half, area.max.y + 4.0), pos2(area.center().x + half, area.max.y + 4.0));
        }
    }

    /// One block of text becomes two columns.
    fn lay_columns(&self) {
        self.before_after(
            |d, a| d.paragraph(a, Dir::Ltr, 7, d.grey()),
            |d, a| {
                let w = (a.width() - 6.0) / 2.0;
                let left = Rect::from_min_size(a.min, vec2(w, a.height()));
                let right = Rect::from_min_size(pos2(a.max.x - w, a.min.y), vec2(w, a.height()));
                d.paragraph(left, Dir::Ltr, 7, d.accent());
                d.paragraph(right, Dir::Ltr, 7, d.accent());
            },
        );
    }

    /// Text stops halfway down one page at a dashed cut and goes on at the top of the next;
    /// `numbered` badges the next page (the Page break item).
    fn lay_breaks(&self, numbered: bool) {
        let first = self.page(14.0, 8.0, 66.0, 88.0);
        self.text_lines(first, false, &[50.0, 46.0, 48.0, 30.0], 10.0, self.grey());
        let y = first.min.y + 42.0;
        let cut = self.reveal(0.05, 0.3);
        let end = first.min.x - 4.0 + (first.width() + 8.0) * cut;
        let mut x = first.min.x - 4.0;
        while x < end {
            self.line(pos2(x, y), pos2((x + 2.4).min(end), y), self.accent(), 1.0);
            x += 4.0;
        }
        let arrow = self.reveal(0.3, 0.45);
        if arrow > 0.0 {
            self.arrow_right(self.at(84.0, 52.0), self.at(84.0 + 8.0 * arrow, 52.0));
        }
        let second = self.page(96.0, 8.0, 66.0, 88.0);
        let wipe = self.reveal(0.45, 0.8);
        if wipe > 0.0 {
            self.lay_clipped(Rect::from_min_max(second.min - vec2(2.0, 6.0), pos2(second.max.x + 2.0, second.min.y + 54.0 * wipe)), |d| {
                d.text_lines(second, false, &[50.0, 46.0, 48.0, 44.0, 28.0], 10.0, d.accent());
            });
        }
        if numbered {
            self.badge(pos2(second.center().x, second.max.y - 2.0), "٢", self.accent());
        }
    }

    /// Two columns: the text stops in the first at a cut and jumps to the top of the second.
    fn lay_break_column(&self) {
        let a = self.page_center();
        let w = (a.width() - 8.0) / 2.0;
        let left = Rect::from_min_size(a.min, vec2(w, a.height()));
        let right = Rect::from_min_size(pos2(a.max.x - w, a.min.y), vec2(w, a.height()));
        self.text_lines(left, false, &[w, w * 0.94, w, w * 0.6], 10.0, self.grey());
        let y = left.min.y + 42.0;
        let cut = self.reveal(0.05, 0.25);
        let end = left.min.x - 2.0 + (w + 4.0) * cut;
        let mut x = left.min.x - 2.0;
        while x < end {
            self.line(pos2(x, y), pos2((x + 2.4).min(end), y), self.accent(), 1.0);
            x += 4.0;
        }
        let gx = a.center().x;
        let path = [pos2(left.max.x + 1.0, y), pos2(gx, y), pos2(gx, a.min.y - 3.0), pos2(right.min.x - 1.0, a.min.y - 3.0)];
        self.in_path(&path, self.reveal(0.25, 0.55), self.accent(), 1.2, true);
        let wipe = self.reveal(0.55, 0.85);
        if wipe > 0.0 {
            self.lay_clipped(Rect::from_min_max(right.min - vec2(2.0, 6.0), pos2(right.max.x + 2.0, right.min.y + 54.0 * wipe)), |d| {
                d.text_lines(right, false, &[w, w * 0.94, w, w * 0.9, w * 0.5], 10.0, d.accent());
            });
        }
    }

    /// A line ends early with a ↵ mark and the rest of its words move to the next line, with
    /// no paragraph gap.
    fn lay_break_wrap(&self) {
        let a = self.page_center();
        let full = a.width();
        let k = self.reveal(0.2, 0.6);
        let (y1, y2) = (a.min.y + 14.0, a.min.y + 24.0);
        self.line(pos2(a.min.x, a.min.y + 4.0), pos2(a.max.x, a.min.y + 4.0), self.grey(), 1.5);
        // Line two shrinks to the cut; line three gains what it lost.
        self.line(pos2(a.min.x, y1), pos2(a.min.x + full - full * 0.42 * k, y1), self.grey(), 1.5);
        self.line(pos2(a.min.x, y2), pos2(a.min.x + full * 0.42 + full * 0.52 * k, y2), self.accent(), 1.5);
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 30.0), a.max), false, &[full * 0.96, full * 0.9, full * 0.55], 10.0, self.grey());
        if k > 0.0 {
            self.label(pos2(a.min.x + full * 0.58 + 4.0, y1), "↵", 9.0, self.accent().linear_multiply(k), Align2::LEFT_CENTER);
        }
    }

    /// A section ends on one page and the next section starts on a fresh page, tagged.
    fn lay_section_next_page(&self) {
        let first = self.page(14.0, 8.0, 66.0, 88.0);
        self.chip(pos2(first.center().x, first.min.y + 6.0), "قسم ١", 34.0);
        self.text_lines(Rect::from_min_max(pos2(first.min.x, first.min.y + 14.0), first.max), false, &[50.0, 46.0, 30.0], 10.0, self.grey());
        self.lay_section_mark(first.min.x - 4.0, first.max.x + 4.0, first.min.y + 50.0, self.reveal(0.05, 0.3));
        let arrow = self.reveal(0.3, 0.45);
        if arrow > 0.0 {
            self.arrow_right(self.at(84.0, 52.0), self.at(84.0 + 8.0 * arrow, 52.0));
        }
        let second = self.page(96.0, 8.0, 66.0, 88.0);
        let wipe = self.reveal(0.45, 0.8);
        if wipe > 0.0 {
            self.lay_clipped(Rect::from_min_max(second.min - vec2(2.0, 6.0), pos2(second.max.x + 2.0, second.min.y + 70.0 * wipe)), |d| {
                d.chip(pos2(second.center().x, second.min.y + 6.0), "قسم ٢", 34.0);
                d.text_lines(
                    Rect::from_min_max(pos2(second.min.x, second.min.y + 14.0), second.max),
                    false,
                    &[50.0, 46.0, 48.0, 28.0],
                    10.0,
                    d.accent(),
                );
            });
        }
    }

    /// One page: the section mark across the middle, and the part below it in two columns.
    fn lay_section_continuous(&self) {
        let a = self.page_center();
        self.chip(pos2(a.center().x, a.min.y + 6.0), "قسم ١", 34.0);
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 14.0), a.max), false, &[60.0, 56.0, 40.0], 10.0, self.grey());
        self.lay_section_mark(a.min.x - 4.0, a.max.x + 4.0, a.min.y + 46.0, self.reveal(0.05, 0.3));
        let wipe = self.reveal(0.35, 0.75);
        if wipe > 0.0 {
            let w = (a.width() - 6.0) / 2.0;
            let below = Rect::from_min_max(pos2(a.min.x - 2.0, a.min.y + 50.0), pos2(a.min.x - 2.0 + (a.width() + 4.0) * wipe, a.max.y + 2.0));
            self.lay_clipped(below, |d| {
                d.chip(pos2(a.center().x, a.min.y + 56.0), "قسم ٢", 34.0);
                let top = a.min.y + 62.0;
                d.text_lines(Rect::from_min_max(pos2(a.min.x, top), pos2(a.min.x + w, a.max.y)), false, &[w, w, w * 0.7], 8.0, d.accent());
                d.text_lines(Rect::from_min_max(pos2(a.max.x - w, top), a.max), false, &[w, w, w * 0.7], 8.0, d.accent());
            });
        }
    }

    /// Three numbered pages: the text ends on the first, the next section skips a blank page
    /// to land on an even (`even`) or odd page.
    fn lay_section_parity(&self, even: bool) {
        let numbers = if even { ["٢", "٣", "٤"] } else { ["١", "٢", "٣"] };
        for (i, n) in numbers.iter().enumerate() {
            let a = self.page(12.0 + i as f32 * 54.0, 14.0, 44.0, 72.0);
            match i {
                0 => {
                    self.text_lines(a, false, &[28.0, 26.0, 16.0], 9.0, self.grey());
                    self.lay_section_mark(a.min.x - 3.0, a.max.x + 3.0, a.min.y + 32.0, self.reveal(0.05, 0.3));
                    self.badge(pos2(a.center().x, a.max.y - 2.0), n, self.grey());
                }
                1 => {
                    let k = self.reveal(0.3, 0.5);
                    self.label(a.center(), "فارغة", 6.5, self.grey().linear_multiply(k), Align2::CENTER_CENTER);
                    self.badge(pos2(a.center().x, a.max.y - 2.0), n, self.grey().linear_multiply(0.5));
                }
                _ => {
                    let wipe = self.reveal(0.5, 0.85);
                    if wipe > 0.0 {
                        self.lay_clipped(Rect::from_min_max(a.min - vec2(2.0, 6.0), pos2(a.max.x + 2.0, a.min.y + 40.0 * wipe)), |d| {
                            d.text_lines(a, false, &[28.0, 26.0, 27.0, 14.0], 9.0, d.accent());
                        });
                    }
                    self.badge(pos2(a.center().x, a.max.y - 2.0), n, self.accent());
                }
            }
        }
    }

    /// A number appears beside every line, one after the other.
    fn lay_line_numbers(&self) {
        let area = self.page_center();
        let text = Rect::from_min_max(pos2(area.min.x + 12.0, area.min.y), area.max);
        self.paragraph(text, Dir::Ltr, 7, self.grey());
        for (i, n) in ["١", "٢", "٣", "٤", "٥", "٦", "٧"].iter().enumerate() {
            let k = self.reveal(0.1 + i as f32 * 0.08, 0.2 + i as f32 * 0.08);
            if k <= 0.0 {
                continue;
            }
            let y = text.min.y + 4.0 + i as f32 * 10.0;
            self.label(pos2(area.min.x + 6.0, y), n, 6.0, self.accent().linear_multiply(k), Align2::RIGHT_CENTER);
        }
    }

    /// The numbers in the margin, struck out by a red line.
    fn lay_line_numbers_none(&self) {
        let area = self.page_center();
        let text = Rect::from_min_max(pos2(area.min.x + 12.0, area.min.y), area.max);
        self.paragraph(text, Dir::Ltr, 7, self.grey());
        let k = self.reveal(0.15, 0.6);
        for (i, n) in ["١", "٢", "٣", "٤", "٥", "٦", "٧"].iter().enumerate() {
            let y = text.min.y + 4.0 + i as f32 * 10.0;
            self.label(pos2(area.min.x + 6.0, y), n, 6.0, self.grey().linear_multiply(1.0 - 0.6 * k), Align2::RIGHT_CENTER);
        }
        self.line(pos2(area.min.x + 3.0, text.min.y), pos2(area.min.x + 3.0, text.min.y + 64.0 * k), self.red(), 1.5);
    }

    /// Two pages of numbered lines: the count goes on (`١`…`١٠`) or starts again on each page.
    fn lay_line_numbers_pages(&self, restart: bool) {
        let second = if restart { ["١", "٢", "٣", "٤", "٥"] } else { ["٦", "٧", "٨", "٩", "١٠"] };
        for (p, (x, nums)) in [(14.0, ["١", "٢", "٣", "٤", "٥"]), (96.0, second)].into_iter().enumerate() {
            let a = self.page(x, 8.0, 66.0, 88.0);
            let text = Rect::from_min_max(pos2(a.min.x + 12.0, a.min.y + 14.0), a.max);
            self.paragraph(text, Dir::Ltr, 5, self.grey());
            for (i, n) in nums.iter().enumerate() {
                let k = self.reveal(0.1 + (p * 5 + i) as f32 * 0.07, 0.17 + (p * 5 + i) as f32 * 0.07);
                if k <= 0.0 {
                    continue;
                }
                let y = text.min.y + 4.0 + i as f32 * 10.0;
                let c = if p == 1 && (restart || i == 0) { self.accent() } else { self.grey() };
                self.label(pos2(a.min.x + 8.0, y), n, 6.0, c.linear_multiply(k), Align2::RIGHT_CENTER);
            }
        }
        self.arrow_right(self.at(84.0, 52.0), self.at(92.0, 52.0));
    }

    /// One page with a section mark across the middle; the numbers start again below it.
    fn lay_line_numbers_section(&self) {
        let area = self.page_center();
        let text = Rect::from_min_max(pos2(area.min.x + 12.0, area.min.y), area.max);
        self.text_lines(text, false, &[46.0, 44.0, 30.0], 10.0, self.grey());
        self.lay_section_mark(area.min.x - 4.0, area.max.x + 4.0, text.min.y + 32.0, self.reveal(0.05, 0.3));
        let lower = Rect::from_min_max(pos2(text.min.x, text.min.y + 36.0), text.max);
        self.text_lines(lower, false, &[46.0, 44.0, 45.0, 26.0], 10.0, self.grey());
        for (i, n) in ["١", "٢", "٣"].iter().enumerate() {
            let y = text.min.y + 4.0 + i as f32 * 10.0;
            self.label(pos2(area.min.x + 6.0, y), n, 6.0, self.grey(), Align2::RIGHT_CENTER);
        }
        for (i, n) in ["١", "٢", "٣", "٤"].iter().enumerate() {
            let k = self.reveal(0.35 + i as f32 * 0.1, 0.45 + i as f32 * 0.1);
            if k <= 0.0 {
                continue;
            }
            let y = lower.min.y + 4.0 + i as f32 * 10.0;
            self.label(pos2(area.min.x + 6.0, y), n, 6.0, self.accent().linear_multiply(k), Align2::RIGHT_CENTER);
        }
    }

    /// Ragged lines become even ones, a word split at a line end with a hyphen.
    fn lay_hyphenation(&self) {
        self.before_after(
            |d, a| d.text_lines(a, false, &[a.width(), a.width() * 0.7, a.width() * 0.95, a.width() * 0.6, a.width() * 0.5], 10.0, d.grey()),
            |d, a| {
                let w = a.width();
                d.text_lines(a, false, &[w - 6.0, w - 6.0, w - 6.0, w - 6.0, w * 0.5], 10.0, d.grey());
                for row in [1.0, 3.0] {
                    let y = a.min.y + 4.0 + row * 10.0;
                    d.label(pos2(a.max.x - 5.0, y), "-", 11.0, d.accent(), Align2::LEFT_CENTER);
                }
            },
        );
    }

    /// Lines that run across become lines that run down.
    fn lay_text_direction(&self) {
        self.before_after(
            |d, a| d.paragraph(a, Dir::Ltr, 6, d.grey()),
            |d, a| {
                let full = a.height();
                for (i, f) in [1.0, 0.94, 0.98, 0.9, 0.55].iter().enumerate() {
                    let x = a.max.x - 4.0 - i as f32 * 10.0;
                    d.line(pos2(x, a.min.y), pos2(x, a.min.y + full * f), d.accent(), 1.5);
                }
                d.arrow_down(pos2(a.min.x + 2.0, a.min.y + 2.0), pos2(a.min.x + 2.0, a.max.y - 2.0));
            },
        );
    }

    /// A picture slides from the middle of the page to its top-right corner; the text flows round it.
    fn lay_position(&self) {
        let area = self.page_center();
        let was = Rect::from_center_size(area.center() + vec2(0.0, 6.0), vec2(26.0, 20.0));
        self.dashed_rect(was, self.grey());
        let pic = Rect::from_min_size(pos2(area.max.x - 26.0, area.min.y), vec2(26.0, 20.0));
        self.text_lines(area, false, &[30.0, 28.0, 30.0], 7.0, self.grey());
        self.text_lines(
            Rect::from_min_max(pos2(area.min.x, area.min.y + 24.0), area.max),
            false,
            &[60.0, 56.0, 58.0, 54.0, 60.0, 40.0],
            7.0,
            self.grey(),
        );
        let k = self.reveal(0.15, 0.65);
        self.picture(Rect::from_min_size(was.min + (pic.min - was.min) * k, pic.size()));
        self.arrow_right(was.right_top() + vec2(2.0, -2.0), pic.left_bottom() + vec2(-2.0, 2.0));
    }

    /// Text that ran behind a picture moves aside to run down both sides of it.
    fn lay_wrap(&self) {
        let area = self.page_center();
        let pic = Rect::from_center_size(area.center(), vec2(26.0, 24.0));
        let k = self.reveal(0.15, 0.65);
        let side = 60.0 - 46.0 * k;
        self.text_lines(area, false, &[60.0, 56.0], 7.0, self.grey());
        let band = Rect::from_min_max(pos2(area.min.x, area.min.y + 14.0), pos2(area.max.x, pic.max.y));
        self.text_lines(band, false, &[side, side, side, side], 7.0, self.accent());
        self.text_lines(band, true, &[side, side, side, side], 7.0, self.accent());
        self.text_lines(Rect::from_min_max(pos2(area.min.x, area.min.y + 44.0), area.max), false, &[60.0, 58.0, 36.0], 7.0, self.grey());
        self.picture(pic);
    }

    /// A picture in the middle with text on both sides, arrows marking the sides it may use.
    fn lay_wrap_sides(&self) {
        let area = self.page_center();
        let pic = Rect::from_center_size(area.center(), vec2(24.0, 26.0));
        self.text_lines(area, false, &[60.0, 56.0], 7.0, self.grey());
        let k = self.reveal(0.15, 0.6);
        let side = 14.0 * k;
        let band = Rect::from_min_max(pos2(area.min.x, area.min.y + 14.0), pos2(area.max.x, pic.max.y));
        self.text_lines(band, false, &[side, side, side, side], 7.0, self.accent());
        self.text_lines(band, true, &[side, side, side, side], 7.0, self.accent());
        self.text_lines(Rect::from_min_max(pos2(area.min.x, area.min.y + 46.0), area.max), false, &[60.0, 58.0, 36.0], 7.0, self.grey());
        self.picture(pic);
        self.arrow_left(pos2(pic.min.x - 3.0, pic.center().y), pos2(area.min.x + 2.0, pic.center().y));
        self.arrow_right(pos2(pic.max.x + 3.0, pic.center().y), pos2(area.max.x - 2.0, pic.center().y));
    }

    /// Two overlapping shapes: the accent one goes on top (`forward`) or underneath.
    fn lay_stack_order(&self, forward: bool) {
        self.before_after(
            |d, a| d.lay_overlap(a, !forward),
            |d, a| {
                d.lay_overlap(a, forward);
                let x = a.max.x - 4.0;
                if forward {
                    d.arrow_up(pos2(x, a.max.y - 2.0), pos2(x, a.max.y - 22.0));
                } else {
                    d.arrow_down(pos2(x, a.max.y - 22.0), pos2(x, a.max.y - 2.0));
                }
            },
        );
    }

    /// A grey box and an accent box overlapping; `accent_on_top` picks the drawing order.
    fn lay_overlap(&self, a: Rect, accent_on_top: bool) {
        let grey = Rect::from_min_size(a.min + vec2(4.0, 8.0), vec2(30.0, 26.0));
        let acc = Rect::from_min_size(a.min + vec2(18.0, 22.0), vec2(30.0, 26.0));
        let grey_box = |d: &Self| {
            d.p.rect_filled(grey, 3.0, d.t.hover);
            d.p.rect_stroke(grey, 3.0, Stroke::new(1.5, d.grey()), egui::StrokeKind::Inside);
        };
        if accent_on_top {
            grey_box(self);
            self.shape(acc);
        } else {
            self.shape(acc);
            grey_box(self);
        }
    }

    /// A page with a picture and a shape; the pane that lists them slides in from the side.
    fn lay_selection_pane(&self) {
        let area = self.page(10.0, 8.0, 84.0, 88.0);
        self.text_lines(area, false, &[60.0, 56.0], 7.0, self.grey());
        self.picture(Rect::from_min_size(area.min + vec2(0.0, 16.0), vec2(30.0, 22.0)));
        self.shape(Rect::from_min_size(area.min + vec2(38.0, 20.0), vec2(28.0, 18.0)));
        self.text_lines(Rect::from_min_max(pos2(area.min.x, area.min.y + 46.0), area.max), false, &[64.0, 60.0, 40.0], 7.0, self.grey());
        let k = self.reveal(0.15, 0.6);
        if k <= 0.0 {
            return;
        }
        self.lay_clipped(Rect::from_min_max(pos2(self.r.max.x - 74.0 * k, self.r.min.y), self.r.max), |d| {
            let pane = d.pane(true, 66.0, 3);
            // The first row (the picture) is the selected one.
            let row = Rect::from_min_size(pos2(pane.min.x + 3.0, pane.min.y + 15.0), vec2(pane.width() - 6.0, 10.0));
            d.selection(row);
            d.line(pos2(row.min.x + 3.0, row.center().y), pos2(row.max.x - 12.0, row.center().y), d.accent(), 1.5);
        });
    }

    /// Scattered shapes line up on one guide.
    fn lay_align(&self) {
        let sizes = [vec2(22.0, 12.0), vec2(16.0, 12.0), vec2(28.0, 12.0)];
        self.before_after(
            |d, a| {
                for (i, (s, dx)) in sizes.iter().zip([14.0, 2.0, 24.0]).enumerate() {
                    d.shape(Rect::from_min_size(a.min + vec2(dx, 6.0 + i as f32 * 22.0), *s));
                }
            },
            |d, a| {
                let x = a.min.x + 8.0;
                d.line(pos2(x, a.min.y), pos2(x, a.max.y), d.accent(), 1.0);
                for (i, s) in sizes.iter().enumerate() {
                    d.shape(Rect::from_min_size(pos2(x, a.min.y + 6.0 + i as f32 * 22.0), *s));
                }
            },
        );
    }

    /// The two boxes of the group thumbnails, in a page area.
    fn lay_pair(a: Rect) -> (Rect, Rect) {
        (Rect::from_min_size(a.min + vec2(6.0, 10.0), vec2(26.0, 20.0)), Rect::from_min_size(a.min + vec2(22.0, 36.0), vec2(26.0, 20.0)))
    }

    /// Two shapes, each with its own handles, become one framed object with one set of handles.
    fn lay_group(&self) {
        self.before_after(
            |d, a| {
                let (x, y) = Draw::lay_pair(a);
                for r in [x, y] {
                    d.shape(r);
                    d.handles(r);
                }
            },
            |d, a| {
                let (x, y) = Draw::lay_pair(a);
                let frame = x.union(y).expand(3.0);
                d.dashed_rect(frame, d.accent());
                d.shape(x);
                d.shape(y);
                d.handles(frame);
            },
        );
    }

    /// One framed object becomes two shapes, each with its own handles.
    fn lay_ungroup(&self) {
        self.before_after(
            |d, a| {
                let (x, y) = Draw::lay_pair(a);
                let frame = x.union(y).expand(3.0);
                d.dashed_rect(frame, d.accent());
                d.shape(x);
                d.shape(y);
                d.handles(frame);
            },
            |d, a| {
                let (x, y) = Draw::lay_pair(a);
                for r in [x, y] {
                    d.shape(r);
                    d.handles(r);
                }
            },
        );
    }

    /// A curved arrow around `c` at radius `r` from angle `a0` to `a1`, the first `k` of it.
    fn lay_arc_arrow(&self, c: Pos2, r: f32, a0: f32, a1: f32, k: f32) {
        if k <= 0.0 {
            return;
        }
        let a1 = a0 + (a1 - a0) * k;
        let n = 16;
        let pts: Vec<_> = (0..=n).map(|i| a0 + (a1 - a0) * i as f32 / n as f32).map(|ang| c + vec2(ang.cos(), ang.sin()) * r).collect();
        self.p.add(Shape::line(pts, Stroke::new(2.0, self.accent())));
        let tip = c + vec2(a1.cos(), a1.sin()) * r;
        let tangent = vec2(-a1.sin(), a1.cos()) * if a1 >= a0 { 1.0 } else { -1.0 };
        let back = tip - tangent * 7.0;
        let side = vec2(tangent.y, -tangent.x) * 4.5;
        self.p.add(Shape::convex_polygon(vec![tip + tangent * 2.0, back + side, back - side], self.accent(), Stroke::NONE));
    }

    /// A picture with a curved arrow drawing itself around it.
    fn lay_rotate(&self) {
        let c = self.at(88.0, 54.0);
        self.picture(Rect::from_center_size(c, vec2(44.0, 34.0)));
        self.lay_arc_arrow(c, 36.0, -2.2, -0.35, self.reveal(0.1, 0.7));
    }

    /// The picture turns a quarter turn right (0) or left (1), or flips over (2) or across (3).
    fn lay_rotate_kind(&self, kind: u8) {
        let c = self.at(88.0, 54.0);
        let k = self.reveal(0.15, 0.7);
        let (w, h) = (44.0, 34.0);
        match kind {
            0 | 1 => {
                let ang = std::f32::consts::FRAC_PI_2 * k * if kind == 0 { 1.0 } else { -1.0 };
                let (s, co) = ang.sin_cos();
                self.lay_pic_xf(c, w, h, PIC_NORMAL, 1.0, &|v| vec2(v.x * co - v.y * s, v.x * s + v.y * co));
                if kind == 0 {
                    self.lay_arc_arrow(c, 38.0, -2.2, -0.35, 1.0);
                } else {
                    self.lay_arc_arrow(c, 38.0, -0.94, -2.79, 1.0);
                }
            }
            2 => {
                let f = (1.0 - 2.0 * k).clamp(-1.0, 1.0);
                let f = if f.abs() < 0.06 { 0.06 } else { f };
                self.lay_pic_xf(c, w, h, PIC_NORMAL, 1.0, &|v| vec2(v.x, v.y * f));
                self.measure_v(c + vec2(w / 2.0 + 10.0, -h / 2.0), c + vec2(w / 2.0 + 10.0, h / 2.0));
                self.line(c + vec2(-w / 2.0 - 6.0, 0.0), c + vec2(w / 2.0 + 4.0, 0.0), self.accent().linear_multiply(0.5), 1.0);
            }
            _ => {
                let f = (1.0 - 2.0 * k).clamp(-1.0, 1.0);
                let f = if f.abs() < 0.06 { 0.06 } else { f };
                self.lay_pic_xf(c, w, h, PIC_NORMAL, 1.0, &|v| vec2(v.x * f, v.y));
                self.measure_h(c + vec2(-w / 2.0, h / 2.0 + 10.0), c + vec2(w / 2.0, h / 2.0 + 10.0));
                self.line(c + vec2(0.0, -h / 2.0 - 6.0), c + vec2(0.0, h / 2.0 + 4.0), self.accent().linear_multiply(0.5), 1.0);
            }
        }
    }

    // ---- Shape Format ----

    /// The shape on a page and the colour swatches under it; the fill rises from the bottom.
    fn lay_shape_fill(&self) {
        let a = self.page_rl();
        let r = Rect::from_center_size(pos2(a.center().x, a.min.y + 30.0), vec2(40.0, 26.0));
        self.p.rect_filled(r.expand(3.0), 0.0, Color32::WHITE);
        let k = self.reveal(0.15, 0.6);
        self.p.rect_filled(Rect::from_min_max(pos2(r.min.x, r.max.y - r.height() * k), r.max), 3.0, self.accent().linear_multiply(0.35));
        self.p.rect_stroke(r, 3.0, Stroke::new(1.5, self.accent()), egui::StrokeKind::Inside);
        for (i, c) in [self.accent(), self.green(), self.red(), self.yellow()].into_iter().enumerate() {
            let s = Rect::from_min_size(pos2(a.min.x + 8.0 + i as f32 * 14.0, a.max.y - 14.0), vec2(10.0, 10.0));
            self.p.rect_filled(s.expand(1.5), 0.0, Color32::WHITE);
            self.p.rect_filled(s, 1.0, c);
            if i == 0 {
                self.p.rect_stroke(s.expand(1.5), 1.0, self.stroke(self.ink()), egui::StrokeKind::Outside);
            }
        }
    }

    /// The shape's outline thickens; the weights to choose from under it.
    fn lay_shape_outline(&self) {
        let a = self.page_rl();
        let r = Rect::from_center_size(pos2(a.center().x, a.min.y + 30.0), vec2(40.0, 26.0));
        self.p.rect_filled(r.expand(4.0), 0.0, Color32::WHITE);
        let k = self.reveal(0.15, 0.6);
        self.p.rect_filled(r, 3.0, self.accent().linear_multiply(0.15));
        self.p.rect_stroke(r, 3.0, Stroke::new(0.8 + 2.7 * k, self.accent()), egui::StrokeKind::Inside);
        for (i, w) in [0.8, 1.8, 3.5].into_iter().enumerate() {
            let y = a.max.y - 16.0 + i as f32 * 6.0;
            self.p.rect_filled(Rect::from_min_max(pos2(a.min.x + 6.0, y - 2.5), pos2(a.min.x + 46.0, y + 2.5)), 0.0, Color32::WHITE);
            self.line(pos2(a.min.x + 8.0, y), pos2(a.min.x + 44.0, y), if i == 2 { self.accent() } else { self.ink() }, w);
        }
    }

    /// A glow spreads round the shape and a shadow slides out from under it.
    fn lay_shape_effects(&self) {
        let a = self.page_rl();
        let r = Rect::from_center_size(a.center(), vec2(44.0, 30.0));
        self.p.rect_filled(r.expand(12.0), 0.0, Color32::WHITE);
        let k = self.reveal(0.15, 0.65);
        for i in (1..=4).rev() {
            let g = r.expand(i as f32 * 2.0 * k);
            self.p.rect_filled(g, 4.0, self.accent().linear_multiply(0.08));
        }
        self.p.rect_filled(r.translate(vec2(4.0, 4.0) * k), 4.0, self.grey().linear_multiply(0.55));
        self.shape(r);
    }

    /// A drawing canvas (dashed) with three shapes: it hugs them (0), grows (1), or the
    /// drawing scales up inside it (2).
    fn lay_canvas(&self, kind: u8) {
        let a = self.page_wide();
        let k = self.reveal(0.15, 0.65);
        let tight = Rect::from_min_size(pos2(a.min.x + 20.0, a.min.y + 12.0), vec2(64.0, 40.0));
        let loose = tight.expand2(vec2(16.0, 10.0));
        let canvas = match kind {
            0 => Rect::from_min_max(loose.min + (tight.min - loose.min) * k, loose.max + (tight.max - loose.max) * k),
            1 => Rect::from_min_max(tight.min + (loose.min - tight.min) * k, tight.max + (loose.max - tight.max) * k),
            _ => loose,
        };
        self.p.rect_filled(canvas, 0.0, self.t.panel);
        self.dashed_rect(canvas, self.accent());
        let s = if kind == 2 { 0.6 + 0.4 * k } else { 1.0 };
        let c = tight.center();
        for (dx, dy) in [(-20.0, -8.0), (6.0, -8.0), (-7.0, 10.0)] {
            let r = Rect::from_center_size(c + vec2(dx, dy) * s, vec2(20.0, 14.0) * s);
            self.shape(r);
        }
        if kind == 2 {
            self.badge(pos2(a.max.x - 16.0, a.max.y - 8.0), "١٥٠٪", self.accent());
        } else {
            let (x0, x1) = (canvas.min.x, canvas.max.x);
            self.measure_h(pos2(x0, a.max.y - 4.0), pos2(x1, a.max.y - 4.0));
        }
    }

    // ---- Picture Format ----

    /// A dark picture becomes a bright one.
    fn lay_corrections(&self) {
        self.before_after(
            |d, a| {
                let r = Rect::from_min_size(pos2(a.min.x + 4.0, a.min.y + 18.0), vec2(a.width() - 8.0, 36.0));
                d.picture(r);
                d.p.rect_filled(r, 1.0, Color32::from_black_alpha(110));
            },
            |d, a| {
                let r = Rect::from_min_size(pos2(a.min.x + 4.0, a.min.y + 18.0), vec2(a.width() - 8.0, 36.0));
                d.picture(r);
                let c = pos2(r.max.x - 8.0, r.min.y - 7.0);
                d.p.circle_filled(c, 3.5, d.yellow());
                for i in 0..8 {
                    let ang = i as f32 * std::f32::consts::FRAC_PI_4;
                    let dir = vec2(ang.cos(), ang.sin());
                    d.line(c + dir * 5.0, c + dir * 7.5, d.yellow(), 1.2);
                }
            },
        );
    }

    /// The picture's colours shift to the mode's: 0 saturation (with its grey-to-colour bar),
    /// 1 grayscale, 2 sepia, 3 washout.
    fn lay_picture_tone(&self, mode: u8) {
        let a = self.page_rl();
        let r = Rect::from_min_size(pos2(a.min.x + 6.0, a.min.y + 16.0), vec2(a.width() - 12.0, 38.0));
        let tone = match mode {
            0 => [Color32::from_rgb(150, 205, 250), Color32::from_rgb(50, 170, 50), Color32::from_rgb(20, 125, 45), Color32::from_rgb(255, 165, 10)],
            1 => [
                Color32::from_rgb(228, 228, 228),
                Color32::from_rgb(130, 130, 130),
                Color32::from_rgb(106, 106, 106),
                Color32::from_rgb(190, 190, 190),
            ],
            2 => {
                [Color32::from_rgb(236, 222, 190), Color32::from_rgb(152, 122, 80), Color32::from_rgb(120, 96, 60), Color32::from_rgb(232, 204, 130)]
            }
            _ => [
                Color32::from_rgb(245, 250, 253),
                Color32::from_rgb(216, 229, 216),
                Color32::from_rgb(209, 221, 211),
                Color32::from_rgb(252, 239, 206),
            ],
        };
        let k = self.reveal(0.2, 0.7);
        let mixed = [0, 1, 2, 3].map(|i| Draw::lay_mix(PIC_NORMAL[i], tone[i], k));
        self.p.rect_filled(r.expand(2.0), 0.0, Color32::WHITE);
        self.lay_pic_toned(r, mixed, 1.0);
        if mode == 0 {
            // The saturation bar: grey on the left, full colour on the right.
            let bar = Rect::from_min_size(pos2(r.min.x, r.max.y + 6.0), vec2(r.width(), 6.0));
            let n = 10;
            for i in 0..n {
                let x0 = bar.min.x + bar.width() * i as f32 / n as f32;
                let c = Draw::lay_mix(Color32::from_rgb(150, 150, 150), tone[1], i as f32 / (n - 1) as f32);
                self.p.rect_filled(Rect::from_min_max(pos2(x0, bar.min.y), pos2(x0 + bar.width() / n as f32 + 0.5, bar.max.y)), 0.0, c);
            }
            self.p.rect_stroke(bar, 0.0, self.stroke(self.t.border_strong), egui::StrokeKind::Outside);
        }
    }

    /// Text lines show through a picture that turns half transparent.
    fn lay_transparency(&self) {
        let a = self.page_center();
        self.paragraph(a, Dir::Rtl, 7, self.grey());
        let r = Rect::from_min_size(pos2(a.min.x + 4.0, a.min.y + 14.0), vec2(a.width() - 8.0, 40.0));
        let k = self.reveal(0.2, 0.7);
        self.lay_pic_toned(r, PIC_NORMAL, 1.0 - 0.55 * k);
        self.badge(pos2(a.center().x, a.max.y - 6.0), "٥٠٪", self.accent().linear_multiply(k.max(0.15)));
    }

    /// The picture with one frame style: 0 simple frame, 1 thick frame, 2 rounded corners,
    /// 3 soft edges, 4 shadow.
    fn lay_picture_style(&self, style: u8) {
        let a = self.page_rl();
        let r = Rect::from_min_size(pos2(a.min.x + 8.0, a.min.y + 16.0), vec2(a.width() - 16.0, 36.0));
        self.p.rect_filled(r.expand(8.0), 0.0, Color32::WHITE);
        let k = self.reveal(0.2, 0.65);
        match style {
            0 => {
                self.picture(r);
                self.p.rect_stroke(r, 0.0, Stroke::new(1.5, self.ink().linear_multiply(k.max(0.1))), egui::StrokeKind::Outside);
            }
            1 => {
                let frame = r.expand(5.0 * k);
                self.p.rect_filled(frame, 0.0, Color32::WHITE);
                self.p.rect_stroke(frame, 0.0, self.stroke(self.t.border_strong), egui::StrokeKind::Outside);
                self.picture(r);
            }
            2 => {
                self.picture(r);
                // A white band along a rounded path just outside the picture hides its corners.
                let radius = 4.0 + 8.0 * k;
                self.p.rect_stroke(r.expand(4.0), radius, Stroke::new(8.0, Color32::WHITE), egui::StrokeKind::Middle);
                self.p.rect_stroke(r, 8.0 * k, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
            }
            3 => {
                self.picture(r);
                for i in 0..7 {
                    let alpha = (1.0 - i as f32 / 7.0) * k;
                    self.p.rect_stroke(r.shrink(i as f32), 0.0, Stroke::new(1.2, Color32::WHITE.linear_multiply(alpha)), egui::StrokeKind::Inside);
                }
            }
            _ => {
                self.p.rect_filled(r.translate(vec2(5.0, 5.0) * k), 1.0, self.grey().linear_multiply(0.55));
                self.picture(r);
            }
        }
    }

    /// Three small pictures with a thin, a medium and a thick border.
    fn lay_picture_border(&self) {
        let k = self.reveal(0.15, 0.6);
        for (i, w) in [1.0, 2.5, 4.5].into_iter().enumerate() {
            let r = Rect::from_min_size(self.at(16.0 + i as f32 * 52.0, 34.0), vec2(40.0, 32.0));
            self.picture(r);
            self.p.rect_stroke(r, 0.0, Stroke::new(w * k.max(0.05), self.ink()), egui::StrokeKind::Outside);
        }
    }

    /// The picture in its frame is swapped for another, the frame staying where it is.
    fn lay_change_picture(&self) {
        let sunset =
            [Color32::from_rgb(250, 200, 150), Color32::from_rgb(120, 90, 110), Color32::from_rgb(80, 60, 90), Color32::from_rgb(250, 120, 60)];
        let r = |a: Rect| Rect::from_min_size(pos2(a.min.x + 4.0, a.min.y + 18.0), vec2(a.width() - 8.0, 36.0));
        self.before_after(
            |d, a| {
                d.text_lines(a, true, &[50.0, 44.0], 7.0, d.grey());
                d.picture(r(a));
                d.p.rect_stroke(r(a).expand(2.0), 0.0, Stroke::new(1.5, d.accent()), egui::StrokeKind::Outside);
            },
            |d, a| {
                d.text_lines(a, true, &[50.0, 44.0], 7.0, d.grey());
                d.lay_pic_toned(r(a), sunset, 1.0);
                d.p.rect_stroke(r(a).expand(2.0), 0.0, Stroke::new(1.5, d.accent()), egui::StrokeKind::Outside);
            },
        );
    }
}
