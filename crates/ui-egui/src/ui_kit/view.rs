//! View tab thumbnails: the document views, zoom levels, dark mode, windows and macros. Every
//! scene is a page (or a window) drawn with the shared kit; the part the button changes is in
//! the accent colour. Each one a little film: the page turns, the text spreads to the window,
//! the page grows under the magnifier, the dark half wipes across, the split bar slides in.

use egui::{Align2, Color32, Rect, Stroke, pos2, vec2};

use super::Entry;
use super::draw::{Dir, Draw, SIZE};

pub const ENTRIES: &[Entry] = &[
    Entry::new(
        "view.readMode",
        "Shows the document as two pages side by side, like an open book, with the ribbon hidden, for reading rather than editing. Example: turn the pages with the arrow keys and read the letter in comfort.",
        |d| d.vw_read_mode(),
    ),
    Entry::new(
        "view.printLayout",
        "Shows every page exactly as it will print, with its edges and margins, on a grey background; this is the usual view for writing a letter. Example: you see where page 1 ends and page 2 begins.",
        |d| d.vw_print_layout(),
    ),
    Entry::new(
        "view.webLayout",
        "Spreads the text across the whole window with no page edges, the way a web page looks. Example: a wide table shows in full instead of being cut by the page edge.",
        |d| d.vw_web_layout(),
    ),
    Entry::new(
        "view.outline",
        "Shows only the headings, stepped in by level, so you can see the structure and drag whole sections to a new place. Example: move chapter 3 before chapter 2 with one drag.",
        |d| d.vw_outline(),
    ),
    Entry::new(
        "view.draft",
        "Shows the text alone, with no page edges, pictures or headers, so typing and scrolling are fast; a dotted line marks each page break. Example: a long report scrolls as one column of text.",
        |d| d.vw_draft(),
    ),
    Entry::new(
        "view.focus",
        "Hides everything except the page on a dark background, so nothing distracts you while you write. Example: no ribbon, no rulers, just the page in the middle.",
        |d| d.vw_focus(),
    ),
    Entry::new(
        "view.immersive",
        "Shows the text in larger letters with more space between the lines, which is easier on the eyes; printing is not changed. Example: a dense paragraph becomes airy and readable.",
        |d| d.vw_immersive(),
    ),
    Entry::new(
        "view.vertical",
        "Scrolls through the pages up and down, one below the other, the usual way. Example: roll the mouse wheel and page 2 comes up under page 1.",
        |d| d.vw_vertical(),
    ),
    Entry::new(
        "view.sideToSide",
        "Lays the pages out in a row and flips through them sideways, like turning the pages of a book. Example: swipe or scroll sideways to go from page 1 to page 2.",
        |d| d.vw_side_to_side(),
    ),
    Entry::new(
        "ui.dialog:zoom",
        "Opens a box where you type or choose how large the page appears on the screen; the printed size does not change. Example: choose 150% to read small print.",
        |d| d.vw_zoom_dialog(),
    ),
    Entry::new(
        "view.zoom100",
        "Shows the page at its real printed size on the screen, neither enlarged nor reduced. Example: a 12-point letter looks the same size as on paper.",
        |d| d.vw_zoom_100(),
    ),
    Entry::new(
        "view.zoomIn",
        "Makes the page appear larger on the screen, one step at a time; the printed size does not change. Example: from 100% to 110% to 120%.",
        |d| d.vw_zoom_step(true),
    ),
    Entry::new(
        "view.zoomOut",
        "Makes the page appear smaller on the screen, one step at a time, so you see more of it. Example: from 100% down to 90% to 80%.",
        |d| d.vw_zoom_step(false),
    ),
    Entry::new(
        "view.onePage",
        "Shrinks the view until one whole page fits in the window, top to bottom. Example: see the whole letter at once to check its layout.",
        |d| d.vw_fit_pages(1),
    ),
    Entry::new(
        "view.multiplePages",
        "Shrinks the view until several whole pages fit in the window side by side. Example: see pages 1, 2 and 3 together to compare them.",
        |d| d.vw_fit_pages(3),
    ),
    Entry::new(
        "view.pageWidth",
        "Enlarges the view until the page is exactly as wide as the window, so the lines are as big as they can be with nothing cut off. Example: the text fills the screen from edge to edge.",
        |d| d.vw_page_width(),
    ),
    Entry::new(
        "view.darkMode",
        "Switches the page between white with dark text and dark with light text, which is easier on the eyes at night; printing stays black on white. Example: the white page turns dark grey and the letters turn light.",
        |d| d.vw_switch_modes(),
    ),
    Entry::new(
        "view.newWindow",
        "Opens the same document in a second window; what you change in one shows in the other. Example: keep page 1 open in one window while you edit page 9 in the other.",
        |d| d.vw_new_window(),
    ),
    Entry::new(
        "view.arrangeAll",
        "Places all the open windows side by side on the screen so you can see every one. Example: two documents, one on the left and one on the right.",
        |d| d.vw_arrange_all(),
    ),
    Entry::new(
        "view.split",
        "Divides the window into a top and a bottom part that scroll separately through the same document. Example: keep the table on top while you type under it in the lower part.",
        |d| d.vw_split(),
    ),
    Entry::new(
        "view.fullScreen",
        "Makes the window fill the whole screen, hiding the title bar and the ribbon, to get the most room for the page. Example: press it before reading a long document; press again to return.",
        |d| d.vw_full_screen(),
    ),
    Entry::new(
        "tools.macros",
        "Lists the recorded macros (saved series of steps) so you can run, edit or delete one. Example: run «ختم الكتاب» to stamp and number the letter in one go.",
        |d| d.vw_macros(),
    ),
];

/// The dark background of focus mode and the dark half of a page.
const DARK: Color32 = Color32::from_rgb(44, 44, 48);

impl Draw<'_> {
    /// A window: a frame with a title bar, at `x, y` of `w × h`; returns its content area.
    fn vw_window(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        let r = Rect::from_min_size(self.at(x, y), vec2(w, h));
        self.p.rect_filled(r, 2.0, self.t.canvas);
        self.p.rect_stroke(r, 2.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        self.p.rect_filled(Rect::from_min_size(r.min, vec2(w, 10.0)), 2.0, self.t.hover);
        self.line(pos2(r.min.x + 4.0, r.min.y + 5.0), pos2(r.min.x + w * 0.4, r.min.y + 5.0), self.t.text, 2.0);
        Rect::from_min_max(pos2(r.min.x, r.min.y + 10.0), r.max)
    }

    /// A small page with a few text lines inside `area`, centred.
    fn vw_mini_page(&self, area: Rect) {
        let w = (area.width() * 0.6).min(44.0);
        let h = (area.height() - 8.0).min(w * 1.3);
        let r = Rect::from_center_size(area.center(), vec2(w, h));
        self.p.rect_filled(r, 1.0, Color32::WHITE);
        self.p.rect_stroke(r, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        self.paragraph(r.shrink(5.0), Dir::Ltr, ((h - 10.0) / 7.0).max(1.0) as usize, self.grey());
    }

    /// The whole thumbnail painted dark (focus mode, the dark half of a page).
    fn vw_dark_fill(&self, r: Rect) {
        self.p.rect_filled(r, 2.0, DARK);
    }

    /// Two facing pages; the right one is turned in like a book page.
    fn vw_read_mode(&self) {
        let l = self.page(20.0, 8.0, 66.0, 88.0);
        self.paragraph(l, Dir::Ltr, 7, self.grey());
        let k = self.reveal(0.15, 0.6);
        // The right page turns in: it widens from the spine as it comes round.
        let w = 66.0 * k;
        if w > 18.0 {
            let r = self.page(90.0, 8.0, w, 88.0);
            let clipped = self.p.with_clip_rect(r.expand(2.0).intersect(self.p.clip_rect()));
            self.with_painter(&clipped).paragraph(Rect::from_min_size(r.min, vec2(66.0 - 16.0, r.height())), Dir::Ltr, 7, self.grey());
        }
        self.arrow_left(self.at(84.0, 100.0), self.at(76.0, 100.0));
        self.arrow_right(self.at(92.0, 100.0), self.at(100.0, 100.0));
    }

    /// The page on its grey desk; the margin box fades in.
    fn vw_print_layout(&self) {
        self.p.rect_filled(self.r, 2.0, self.t.canvas);
        let a = self.page_center();
        let k = self.reveal(0.2, 0.6);
        if k > 0.0 {
            self.dashed_rect(a.expand(3.0), self.accent().linear_multiply(k));
        }
        self.paragraph(a, Dir::Ltr, 7, self.grey());
    }

    /// The text spreads from page width to the whole window.
    fn vw_web_layout(&self) {
        self.p.rect_filled(self.r, 2.0, Color32::WHITE);
        let k = self.reveal(0.15, 0.65);
        let inset = 10.0 + (1.0 - k) * 40.0;
        let a = Rect::from_min_max(self.at(inset, 10.0), self.at(SIZE.x - inset, SIZE.y - 10.0));
        self.heading_then_text(a, Dir::Ltr, 7, self.accent());
    }

    /// The headings, stepped in by level, appear one by one.
    fn vw_outline(&self) {
        let a = self.page_center();
        let rows: [(f32, &str, f32); 7] =
            [(0.0, "–", 3.0), (1.0, "–", 2.0), (2.0, "+", 1.5), (2.0, "+", 1.5), (1.0, "–", 2.0), (2.0, "+", 1.5), (0.0, "+", 3.0)];
        for (i, (level, mark, w)) in rows.iter().enumerate() {
            let from = 0.1 + 0.6 * i as f32 / 7.0;
            let k = self.reveal(from, from + 0.15);
            if k <= 0.0 {
                continue;
            }
            let y = a.min.y + 5.0 + i as f32 * 12.0;
            let x = a.min.x + level * 9.0;
            self.label(pos2(x + 2.0, y), mark, 8.0, self.accent().linear_multiply(k), Align2::CENTER_CENTER);
            let c = if *w > 2.0 { self.ink() } else { self.grey() };
            self.line(pos2(x + 8.0, y), pos2(x + 8.0 + (a.max.x - level * 4.0 - x - 8.0) * k, y), c, *w);
        }
    }

    /// The page edge fades away, leaving the text and a dotted page break.
    fn vw_draft(&self) {
        self.p.rect_filled(self.r, 2.0, Color32::WHITE);
        let a = Rect::from_min_max(self.at(16.0, 8.0), self.at(SIZE.x - 16.0, SIZE.y - 8.0));
        let k = self.reveal(0.15, 0.6);
        if k < 1.0 {
            self.p.rect_stroke(a.expand(6.0), 2.0, self.stroke(self.t.border_strong.linear_multiply(1.0 - k)), egui::StrokeKind::Inside);
        }
        self.paragraph(Rect::from_min_size(a.min, vec2(a.width(), 44.0)), Dir::Ltr, 4, self.grey());
        // A page break: a dotted line across, no page edge.
        let y = a.min.y + 50.0;
        let mut x = a.min.x;
        while x < a.min.x + a.width() * k {
            self.line(pos2(x, y), pos2((x + 3.0).min(a.max.x), y), self.accent(), 1.0);
            x += 6.0;
        }
        self.paragraph(Rect::from_min_size(pos2(a.min.x, y + 4.0), vec2(a.width(), 40.0)), Dir::Ltr, 3, self.grey());
    }

    /// The dark closes in around the page.
    fn vw_focus(&self) {
        let k = self.reveal(0.1, 0.6);
        self.vw_dark_fill(self.r);
        if k < 1.0 {
            // The light still showing around the page, shrinking to nothing.
            self.p.rect_filled(self.r.shrink2(vec2(60.0 * k, 36.0 * k)), 2.0, self.t.canvas.linear_multiply(1.0 - k));
        }
        let a = self.page_center();
        self.paragraph(a, Dir::Ltr, 7, self.grey());
    }

    /// Thin, close lines grow thicker and move apart: the big, airy reading font.
    fn vw_immersive(&self) {
        let a = self.page_wide();
        let k = self.reveal(0.15, 0.65);
        let pitch = 10.0 + 8.0 * k;
        let thick = 1.5 + 1.5 * k;
        for (i, w) in [100.0, 96.0, 104.0, 60.0].iter().enumerate() {
            let y = a.min.y + 6.0 + i as f32 * pitch;
            self.line(pos2(a.min.x, y), pos2(a.min.x + w, y), self.ink(), thick);
        }
    }

    /// Two pages, one under the other, scrolling up into place.
    fn vw_vertical(&self) {
        let k = self.reveal(0.1, 0.6);
        let shift = (1.0 - k) * 14.0;
        let clipped = self.p.with_clip_rect(Rect::from_min_max(self.at(40.0, 4.0), self.at(136.0, 100.0)).intersect(self.p.clip_rect()));
        let d = self.with_painter(&clipped);
        for y in [6.0, 56.0] {
            let a = d.page(62.0, y + shift, 52.0, 42.0);
            d.paragraph(a, Dir::Ltr, 3, d.grey());
        }
        self.measure_v(self.at(130.0, 12.0), self.at(130.0, 92.0));
    }

    /// Three pages in a row, sliding in from the side.
    fn vw_side_to_side(&self) {
        let k = self.reveal(0.1, 0.6);
        let shift = (1.0 - k) * 24.0;
        let clipped = self.p.with_clip_rect(Rect::from_min_max(self.at(4.0, 4.0), self.at(172.0, 100.0)).intersect(self.p.clip_rect()));
        let d = self.with_painter(&clipped);
        let w = 34.0;
        let total = 3.0 * (w + 8.0) - 8.0;
        let x0 = (SIZE.x - total) / 2.0 + shift;
        for i in 0..3 {
            let a = d.page(x0 + i as f32 * (w + 8.0), 24.0, w, 48.0);
            d.paragraph(a, Dir::Ltr, 4, d.grey());
        }
        self.measure_h(self.at(30.0, 86.0), self.at(146.0, 86.0));
    }

    fn vw_zoom_dialog(&self) {
        let inner = self.dialog("Zoom", 2);
        self.label(pos2(inner.max.x - 2.0, inner.max.y - 8.0), "%", 9.0, self.accent(), Align2::RIGHT_CENTER);
        self.magnifier(pos2(inner.min.x + 10.0, inner.max.y - 12.0));
    }

    /// The page, and the 100% badge pops beside it.
    fn vw_zoom_100(&self) {
        let a = self.page_lr();
        let k = self.reveal(0.2, 0.5);
        if k > 0.0 {
            self.badge(pos2(a.max.x + 4.0, a.max.y - 6.0 + (1.0 - k) * 8.0), "100%", self.accent().linear_multiply(k.max(0.3)));
        }
    }

    /// The small page and the big one; the arrow grows the way the view goes and the magnifier
    /// slides with it.
    fn vw_zoom_step(&self, bigger: bool) {
        let small = self.page(14.0, 30.0, 36.0, 44.0);
        self.paragraph(small, Dir::Ltr, 3, self.grey());
        let big = self.page(98.0, 10.0, 66.0, 84.0);
        self.paragraph(big, Dir::Ltr, 5, self.grey());
        let k = self.reveal(0.1, 0.5);
        if k > 0.0 {
            if bigger {
                self.arrow_right(self.at(58.0, 52.0), self.at(58.0 + 32.0 * k, 52.0));
            } else {
                self.arrow_left(self.at(90.0, 52.0), self.at(90.0 - 32.0 * k, 52.0));
            }
        }
        let c = self.at(74.0, 28.0);
        self.magnifier(c);
        self.glyph(c, if bigger { "+" } else { "−" }, 12.0, self.accent());
    }

    /// The pages shrink into the window until `n` of them fit.
    fn vw_fit_pages(&self, n: usize) {
        let area = self.vw_window(8.0, 4.0, SIZE.x - 16.0, SIZE.y - 8.0);
        let n = n.clamp(1, 4);
        let k = self.reveal(0.1, 0.6);
        // Bigger than the window at the start, the pages shrink to fit.
        let scale = 1.0 + (1.0 - k) * 0.6;
        let w = 34.0 * scale;
        let h = 54.0 * scale;
        let total = n as f32 * (w + 8.0) - 8.0;
        let x0 = area.center().x - total / 2.0 - self.r.min.x;
        let clipped = self.p.with_clip_rect(area.intersect(self.p.clip_rect()));
        let d = self.with_painter(&clipped);
        for i in 0..n {
            let a = d.page(x0 + i as f32 * (w + 8.0), area.min.y - self.r.min.y + 12.0, w, h);
            d.paragraph(a, Dir::Ltr, 4, d.grey());
        }
    }

    /// The page widens until it is exactly as wide as the window.
    fn vw_page_width(&self) {
        let area = self.vw_window(8.0, 4.0, SIZE.x - 16.0, SIZE.y - 8.0);
        self.measure_h(pos2(area.min.x + 6.0, area.min.y + 6.0), pos2(area.max.x - 6.0, area.min.y + 6.0));
        let k = self.reveal(0.1, 0.6);
        let full = SIZE.x - 28.0;
        let w = full * (0.55 + 0.45 * k);
        let x = 14.0 + (full - w) / 2.0;
        let a = self.page(x, area.min.y - self.r.min.y + 12.0, w, 72.0);
        self.paragraph(a, Dir::Ltr, 5, self.grey());
    }

    /// The dark half wipes across the page from the right.
    fn vw_switch_modes(&self) {
        let a = self.page_center();
        let outer = a.expand(8.0);
        let k = self.reveal(0.15, 0.6);
        // The boundary moves from the right edge to the middle.
        let mid = outer.max.x - (outer.max.x - outer.center().x) * k;
        let right = Rect::from_min_max(pos2(mid, outer.min.y), outer.max);
        self.vw_dark_fill(right);
        let light = Color32::from_rgb(210, 210, 215);
        for i in 0..7 {
            let y = a.min.y + 4.0 + i as f32 * 10.0;
            let w = if i == 6 { 0.55 } else { 0.95 };
            let x1 = a.min.x + a.width() * w;
            self.line(pos2(a.min.x, y), pos2(x1.min(mid), y), self.grey(), 1.5);
            if x1 > mid {
                self.line(pos2(mid, y), pos2(x1, y), light, 1.5);
            }
        }
        // A sun on the light half, a crescent on the dark half.
        let sun = pos2(a.min.x + 8.0, a.max.y - 2.0);
        self.p.circle_filled(sun, 4.0, self.accent());
        for i in 0..8 {
            let ang = i as f32 * std::f32::consts::FRAC_PI_4;
            let dir = vec2(ang.cos(), ang.sin());
            self.line(sun + dir * 5.5, sun + dir * 8.0, self.accent(), 1.5);
        }
        if k >= 1.0 {
            let moon = pos2(a.max.x - 8.0, a.max.y - 2.0);
            self.p.circle_filled(moon, 6.0, self.accent());
            self.p.circle_filled(moon + vec2(3.0, -2.0), 5.0, DARK);
        }
    }

    /// The second window slides out from behind the first.
    fn vw_new_window(&self) {
        let back = self.vw_window(16.0, 8.0, 100.0, 70.0);
        self.vw_mini_page(back);
        let k = self.reveal(0.15, 0.6);
        let (x, y) = (16.0 + 44.0 * k, 8.0 + 20.0 * k);
        let front = self.vw_window(x, y, 100.0, 70.0);
        self.vw_mini_page(front);
        let whole = Rect::from_min_max(pos2(front.min.x, front.min.y - 10.0), front.max);
        self.p.rect_stroke(whole, 2.0, Stroke::new(1.5, self.accent()), egui::StrokeKind::Inside);
    }

    /// Two overlapping windows move apart to sit side by side (Arrange All).
    fn vw_arrange_all(&self) {
        let k = self.reveal(0.15, 0.6);
        let l = self.vw_window(6.0 + 20.0 * (1.0 - k), 8.0, 80.0, 88.0);
        self.vw_mini_page(l);
        let r = self.vw_window(90.0 - 20.0 * (1.0 - k), 8.0, 80.0, 88.0);
        self.vw_mini_page(r);
    }

    /// The split bar slides down from the top of the window to its middle.
    fn vw_split(&self) {
        let a = self.vw_window(28.0, 4.0, 120.0, 96.0);
        let k = self.reveal(0.15, 0.6);
        let y = a.min.y + (a.center().y - a.min.y) * k;
        let top = Rect::from_min_max(a.min, pos2(a.max.x, y - 3.0));
        let bottom = Rect::from_min_max(pos2(a.min.x, y + 3.0), a.max);
        if top.height() > 10.0 {
            self.p.rect_filled(top.shrink(4.0), 1.0, Color32::WHITE);
            let clipped = self.p.with_clip_rect(top.shrink(4.0).intersect(self.p.clip_rect()));
            self.with_painter(&clipped).paragraph(top.shrink(8.0), Dir::Ltr, 3, self.grey());
        }
        self.p.rect_filled(bottom.shrink(4.0), 1.0, Color32::WHITE);
        let clipped = self.p.with_clip_rect(bottom.shrink(4.0).intersect(self.p.clip_rect()));
        self.with_painter(&clipped).paragraph(bottom.shrink(8.0), Dir::Ltr, 3, self.grey());
        self.line(pos2(a.min.x, y), pos2(a.max.x, y), self.accent(), 4.0);
    }

    /// The page grows to fill the thumbnail; the corner marks push outward.
    fn vw_full_screen(&self) {
        let k = self.reveal(0.1, 0.6);
        let inset = 4.0 + (1.0 - k) * 18.0;
        let a = self.page(inset, inset, SIZE.x - 2.0 * inset, SIZE.y - 2.0 * inset);
        self.paragraph(a.shrink2(vec2(24.0 * k, 8.0)), Dir::Ltr, 6, self.grey());
        let o = a.expand(2.0);
        let c = self.accent();
        for (corner, dx, dy) in [(o.left_top(), 1.0, 1.0), (o.right_top(), -1.0, 1.0), (o.left_bottom(), 1.0, -1.0), (o.right_bottom(), -1.0, -1.0)] {
            let inner = corner + vec2(dx * 12.0, dy * 12.0);
            self.line(inner, corner, c, 1.5);
            self.line(corner, corner + vec2(dx * 7.0, 0.0), c, 1.5);
            self.line(corner, corner + vec2(0.0, dy * 7.0), c, 1.5);
        }
    }

    /// The macros list; the selection settles on the first one and the Run button appears.
    fn vw_macros(&self) {
        let inner = self.dialog("Macros", 0);
        let rows = 3;
        let k = self.reveal(0.1, 0.5);
        for i in 0..rows {
            let y = inner.min.y + 2.0 + i as f32 * 12.0;
            let row = Rect::from_min_size(pos2(inner.min.x, y - 4.0), vec2(inner.width() - 30.0, 10.0));
            // The selection walks up the list to the first row.
            if i == ((1.0 - k) * 2.999) as usize {
                self.selection(row);
            }
            self.line(pos2(row.min.x + 3.0, y + 1.0), pos2(row.min.x + 30.0 + i as f32 * 8.0, y + 1.0), self.ink(), 1.5);
        }
        // A Run button: a small play triangle.
        let c = pos2(inner.max.x - 14.0, inner.min.y + 8.0);
        self.button(c, "   ", false);
        self.p.add(egui::Shape::convex_polygon(vec![c + vec2(-4.0, -5.0), c + vec2(5.0, 0.0), c + vec2(-4.0, 5.0)], self.accent(), Stroke::NONE));
    }
}
