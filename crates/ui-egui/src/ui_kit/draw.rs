//! The thumbnail drawing kit: a small set of primitives (pages, text lines, highlights, arrows,
//! objects, glyphs) that every visual-help thumbnail is composed from. Original drawings only —
//! lines and boxes on a mini page, never product artwork.
//!
//! A thumbnail is [`SIZE`] points; `Draw::at(x, y)` addresses it from its top-left corner. The
//! conventions the entries rely on:
//! - `page_lr()` / `page_rl()`: one centred page, text running left-to-right or right-to-left.
//! - `before_after(rtl)`: two pages with an arrow between them; the closure draws into each.
//! - Grey is "ordinary text", the theme accent is "what the button touches", `selection` is a
//!   selected range, `find_highlight` is a yellow mark.

use egui::{Align2, Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

use crate::theme::Tokens;

/// Thumbnail size, in points.
pub const SIZE: egui::Vec2 = egui::Vec2::new(176.0, 104.0);

/// The painter, the thumbnail's rect and the theme, bundled for the drawing helpers.
pub struct Draw<'a> {
    pub p: &'a Painter,
    pub r: Rect,
    pub t: &'a Tokens,
}

/// Which way a drawn "paragraph" runs.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Ltr,
    Rtl,
}

impl Dir {
    pub fn rtl(self) -> bool {
        self == Dir::Rtl
    }
}

impl<'a> Draw<'a> {
    pub fn new(p: &'a Painter, r: Rect, t: &'a Tokens) -> Self {
        Self { p, r, t }
    }

    // ---- coordinates and colours ----

    pub fn at(&self, x: f32, y: f32) -> Pos2 {
        pos2(self.r.min.x + x, self.r.min.y + y)
    }
    pub fn stroke(&self, c: Color32) -> Stroke {
        Stroke::new(1.0, c)
    }
    pub fn grey(&self) -> Color32 {
        self.t.text_dim
    }
    pub fn accent(&self) -> Color32 {
        self.t.accent
    }
    pub fn ink(&self) -> Color32 {
        self.t.text
    }
    pub fn red(&self) -> Color32 {
        Color32::from_rgb(200, 48, 48)
    }
    pub fn green(&self) -> Color32 {
        Color32::from_rgb(46, 139, 87)
    }
    pub fn yellow(&self) -> Color32 {
        self.t.find_highlight
    }

    // ---- pages ----

    /// A page outline (white, grey border) at `x, y` of `w × h`; returns its text area.
    pub fn page(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        let r = Rect::from_min_size(self.at(x, y), vec2(w, h));
        self.p.rect_filled(r, 2.0, Color32::WHITE);
        self.p.rect_stroke(r, 2.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        r.shrink2(vec2(8.0, 8.0))
    }
    /// One centred portrait page; returns its text area.
    pub fn page_center(&self) -> Rect {
        self.page(50.0, 6.0, 76.0, 92.0)
    }
    /// One wide centred page (for landscape, columns, wide tables); returns its text area.
    pub fn page_wide(&self) -> Rect {
        self.page(28.0, 10.0, 120.0, 84.0)
    }
    /// A centred page filled with ordinary text lines, left-to-right.
    pub fn page_lr(&self) -> Rect {
        let a = self.page_center();
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        a
    }
    /// A centred page filled with ordinary text lines, right-to-left.
    pub fn page_rl(&self) -> Rect {
        let a = self.page_center();
        self.paragraph(a, Dir::Rtl, 7, self.grey());
        a
    }
    /// Two pages with an arrow between them: `before(area)` then `after(area)`.
    pub fn before_after(&self, before: impl FnOnce(&Self, Rect), after: impl FnOnce(&Self, Rect)) {
        let b = self.page(6.0, 8.0, 70.0, 88.0);
        let a = self.page(100.0, 8.0, 70.0, 88.0);
        self.arrow_right(self.at(80.0, 52.0), self.at(96.0, 52.0));
        before(self, b);
        after(self, a);
    }
    /// A stack of `n` pages fanned to the bottom-right; returns the top page's text area.
    pub fn page_stack(&self, x: f32, y: f32, w: f32, h: f32, n: usize) -> Rect {
        let mut area = Rect::NOTHING;
        for i in 0..n.max(1) {
            let off = i as f32 * 8.0;
            area = self.page(x + off, y + off, w, h);
        }
        area
    }
    /// A small `n`-page row (thumbnails of pages, as in the View tab).
    pub fn pages_row(&self, n: usize) {
        let n = n.clamp(1, 4);
        let w = 34.0;
        let total = n as f32 * (w + 8.0) - 8.0;
        let x0 = (SIZE.x - total) / 2.0;
        for i in 0..n {
            let a = self.page(x0 + i as f32 * (w + 8.0), 24.0, w, 48.0);
            self.paragraph(a, Dir::Ltr, 4, self.grey());
        }
    }

    // ---- text ----

    pub fn line(&self, a: Pos2, b: Pos2, c: Color32, w: f32) {
        self.p.add(Shape::line_segment([a, b], Stroke::new(w, c)));
    }
    /// Grey "text" lines of the given widths in `area`, right-aligned when `rtl`, `pitch` apart.
    pub fn text_lines(&self, area: Rect, rtl: bool, widths: &[f32], pitch: f32, c: Color32) {
        for (i, w) in widths.iter().enumerate() {
            let y = area.min.y + 4.0 + i as f32 * pitch;
            let (x0, x1) = if rtl { (area.max.x - w, area.max.x) } else { (area.min.x, area.min.x + w) };
            self.line(pos2(x0, y), pos2(x1, y), c, 1.5);
        }
    }
    /// `n` lines of a paragraph filling `area`'s width in a natural ragged shape (the last line
    /// short), 10 pt apart.
    pub fn paragraph(&self, area: Rect, dir: Dir, n: usize, c: Color32) {
        let full = area.width();
        let widths: Vec<f32> = (0..n).map(|i| if i + 1 == n { full * 0.55 } else { full * [1.0, 0.94, 0.98, 0.9, 0.96, 0.92][i % 6] }).collect();
        self.text_lines(area, dir.rtl(), &widths, 10.0, c);
    }
    /// A paragraph whose every line reaches both margins (justified); `last_too` for Distribute.
    pub fn paragraph_justified(&self, area: Rect, n: usize, last_too: bool, c: Color32) {
        let full = area.width();
        let widths: Vec<f32> = (0..n).map(|i| if i + 1 == n && !last_too { full * 0.55 } else { full }).collect();
        self.text_lines(area, false, &widths, 10.0, c);
    }
    /// A heading line (thicker, in `c`) at the top of `area`, then `n` ordinary lines below it.
    pub fn heading_then_text(&self, area: Rect, dir: Dir, n: usize, c: Color32) {
        let y = area.min.y + 4.0;
        let w = area.width() * 0.6;
        let (x0, x1) = if dir.rtl() { (area.max.x - w, area.max.x) } else { (area.min.x, area.min.x + w) };
        self.line(pos2(x0, y), pos2(x1, y), c, 3.0);
        self.paragraph(Rect::from_min_max(pos2(area.min.x, area.min.y + 10.0), area.max), dir, n, self.grey());
    }
    /// A list: `n` items with a marker (`•` or a digit) and a line each; `level` steps them in.
    pub fn list(&self, area: Rect, dir: Dir, markers: &[&str], c: Color32) {
        for (i, m) in markers.iter().enumerate() {
            let y = area.min.y + 5.0 + i as f32 * 12.0;
            if dir.rtl() {
                self.label(pos2(area.max.x, y), m, 7.0, c, Align2::RIGHT_CENTER);
                self.line(pos2(area.min.x, y), pos2(area.max.x - 16.0, y), self.grey(), 1.5);
            } else {
                self.label(pos2(area.min.x, y), m, 7.0, c, Align2::LEFT_CENTER);
                self.line(pos2(area.min.x + 14.0, y), pos2(area.max.x, y), self.grey(), 1.5);
            }
        }
    }
    /// A word-sized box of text (one "word") at `x, y`, `w` wide.
    pub fn word(&self, x: f32, y: f32, w: f32, c: Color32) {
        self.p.rect_filled(Rect::from_min_size(pos2(x, y - 3.0), vec2(w, 6.0)), 1.0, c);
    }
    /// Text set in the thumbnail's own font.
    pub fn label(&self, pos: Pos2, s: &str, size: f32, c: Color32, anchor: Align2) {
        self.p.text(pos, anchor, s, crate::theme::semibold(size), c);
    }
    /// A large glyph (a letter, a symbol) in the middle of the thumbnail or at `pos`.
    pub fn glyph(&self, pos: Pos2, s: &str, size: f32, c: Color32) {
        self.p.text(pos, Align2::CENTER_CENTER, s, crate::theme::semibold(size), c);
    }
    /// A selection band over `rect` (behind the text, so draw it first).
    pub fn selection(&self, rect: Rect) {
        self.p.rect_filled(rect, 1.0, self.t.selection);
    }
    /// A yellow highlight band.
    pub fn highlight(&self, rect: Rect) {
        self.p.rect_filled(rect, 1.0, self.yellow());
    }
    /// The band of `rows` lines (0-based, inclusive) of a paragraph drawn with `paragraph` in
    /// `area`, for `selection`/`highlight`.
    pub fn band(&self, area: Rect, rows: std::ops::RangeInclusive<usize>) -> Rect {
        let y0 = area.min.y + 4.0 + *rows.start() as f32 * 10.0 - 5.0;
        let y1 = area.min.y + 4.0 + *rows.end() as f32 * 10.0 + 5.0;
        Rect::from_min_max(pos2(area.min.x - 2.0, y0), pos2(area.max.x + 2.0, y1))
    }
    /// A text caret `h` tall at `pos` (its top).
    pub fn caret(&self, pos: Pos2, h: f32) {
        self.line(pos, pos + vec2(0.0, h), self.ink(), 1.5);
    }

    // ---- arrows and marks ----

    pub fn arrow_right(&self, a: Pos2, b: Pos2) {
        let c = self.t.accent;
        self.line(a, b, c, 1.5);
        self.line(b, pos2(b.x - 5.0, b.y - 4.0), c, 1.5);
        self.line(b, pos2(b.x - 5.0, b.y + 4.0), c, 1.5);
    }
    pub fn arrow_left(&self, a: Pos2, b: Pos2) {
        let c = self.t.accent;
        self.line(a, b, c, 1.5);
        self.line(b, pos2(b.x + 5.0, b.y - 4.0), c, 1.5);
        self.line(b, pos2(b.x + 5.0, b.y + 4.0), c, 1.5);
    }
    pub fn arrow_down(&self, a: Pos2, b: Pos2) {
        let c = self.t.accent;
        self.line(a, b, c, 1.5);
        self.line(b, pos2(b.x - 4.0, b.y - 5.0), c, 1.5);
        self.line(b, pos2(b.x + 4.0, b.y - 5.0), c, 1.5);
    }
    pub fn arrow_up(&self, a: Pos2, b: Pos2) {
        let c = self.t.accent;
        self.line(a, b, c, 1.5);
        self.line(b, pos2(b.x - 4.0, b.y + 5.0), c, 1.5);
        self.line(b, pos2(b.x + 4.0, b.y + 5.0), c, 1.5);
    }
    /// A double-headed horizontal arrow (widths, margins).
    pub fn measure_h(&self, a: Pos2, b: Pos2) {
        let c = self.t.accent;
        self.line(a, b, c, 1.5);
        self.line(a, a + vec2(5.0, -4.0), c, 1.5);
        self.line(a, a + vec2(5.0, 4.0), c, 1.5);
        self.line(b, b + vec2(-5.0, -4.0), c, 1.5);
        self.line(b, b + vec2(-5.0, 4.0), c, 1.5);
    }
    /// A double-headed vertical arrow.
    pub fn measure_v(&self, a: Pos2, b: Pos2) {
        let c = self.t.accent;
        self.line(a, b, c, 1.5);
        self.line(a, a + vec2(-4.0, 5.0), c, 1.5);
        self.line(a, a + vec2(4.0, 5.0), c, 1.5);
        self.line(b, b + vec2(-4.0, -5.0), c, 1.5);
        self.line(b, b + vec2(4.0, -5.0), c, 1.5);
    }
    /// A green tick in a circle at `c`.
    pub fn check(&self, c: Pos2) {
        self.p.circle_filled(c, 11.0, self.green());
        self.line(c + vec2(-5.0, 0.0), c + vec2(-1.5, 4.0), Color32::WHITE, 2.5);
        self.line(c + vec2(-1.5, 4.0), c + vec2(6.0, -5.0), Color32::WHITE, 2.5);
    }
    /// A red cross in a circle at `c`.
    pub fn cross(&self, c: Pos2) {
        self.p.circle_filled(c, 11.0, self.red());
        self.line(c + vec2(-5.0, -5.0), c + vec2(5.0, 5.0), Color32::WHITE, 2.5);
        self.line(c + vec2(-5.0, 5.0), c + vec2(5.0, -5.0), Color32::WHITE, 2.5);
    }
    /// An orange warning triangle at `w` (its centre).
    pub fn warning(&self, w: Pos2) {
        self.p.add(Shape::convex_polygon(
            vec![w + vec2(0.0, -8.0), w + vec2(8.0, 6.0), w + vec2(-8.0, 6.0)],
            Color32::from_rgb(220, 150, 20),
            Stroke::NONE,
        ));
        self.label(w + vec2(0.0, 1.0), "!", 8.0, Color32::WHITE, Align2::CENTER_CENTER);
    }
    /// A magnifying glass centred at `c`.
    pub fn magnifier(&self, c: Pos2) {
        self.p.circle_stroke(c, 10.0, Stroke::new(2.5, self.t.accent));
        self.line(c + vec2(7.0, 7.0), c + vec2(16.0, 16.0), self.t.accent, 3.0);
    }
    /// A pencil from `a` (tip) to `b`.
    pub fn pencil(&self, a: Pos2, b: Pos2) {
        self.line(a, b, self.t.accent, 4.0);
        self.line(a, a + (a - b).normalized() * 4.0, self.t.text, 2.0);
    }
    /// A gear centred at `c`.
    pub fn gear(&self, c: Pos2, r: f32) {
        self.p.circle_stroke(c, r, Stroke::new(2.0, self.t.accent));
        for i in 0..8 {
            let ang = i as f32 * std::f32::consts::FRAC_PI_4;
            let dir = vec2(ang.cos(), ang.sin());
            self.line(c + dir * (r + 1.5), c + dir * (r + 5.5), self.t.accent, 2.0);
        }
    }
    /// A mouse pointer at `tip`.
    pub fn pointer(&self, tip: Pos2) {
        let pts = vec![
            tip,
            tip + vec2(0.0, 14.0),
            tip + vec2(4.0, 10.5),
            tip + vec2(7.0, 16.0),
            tip + vec2(9.5, 15.0),
            tip + vec2(6.5, 9.5),
            tip + vec2(11.0, 9.0),
        ];
        self.p.add(Shape::convex_polygon(pts.clone(), Color32::WHITE, Stroke::NONE));
        self.p.add(Shape::closed_line(pts, Stroke::new(1.2, self.ink())));
    }
    /// A speaker (sound) icon at `c`.
    pub fn speaker(&self, c: Pos2) {
        let pts =
            vec![c + vec2(-10.0, -4.0), c + vec2(-5.0, -4.0), c + vec2(2.0, -10.0), c + vec2(2.0, 10.0), c + vec2(-5.0, 4.0), c + vec2(-10.0, 4.0)];
        self.p.add(Shape::convex_polygon(pts, self.t.accent, Stroke::NONE));
        for (r, w) in [(6.0, 1.5), (10.0, 1.5)] {
            let n = 10;
            let pts: Vec<Pos2> = (0..=n)
                .map(|i| c + vec2(2.0, 0.0) + vec2(r * (i as f32 / n as f32 * 1.2 - 0.6).cos(), r * (i as f32 / n as f32 * 1.2 - 0.6).sin()))
                .collect();
            self.p.add(Shape::line(pts, Stroke::new(w, self.t.accent)));
        }
    }

    // ---- objects on the page ----

    /// A `rows × cols` table in `r` with a shaded header row and short text in every cell.
    pub fn table(&self, r: Rect, rows: usize, cols: usize) {
        let (rows, cols) = (rows.max(1), cols.max(1));
        self.p.rect_filled(r, 1.0, Color32::WHITE);
        self.p.rect_stroke(r, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        let rh = r.height() / rows as f32;
        let cw = r.width() / cols as f32;
        self.p.rect_filled(Rect::from_min_size(r.min, vec2(r.width(), rh)), 0.0, self.t.hover);
        for i in 1..rows {
            let y = r.min.y + i as f32 * rh;
            self.line(pos2(r.min.x, y), pos2(r.max.x, y), self.t.border, 1.0);
        }
        for j in 1..cols {
            let x = r.min.x + j as f32 * cw;
            self.line(pos2(x, r.min.y), pos2(x, r.max.y), self.t.border, 1.0);
        }
        for i in 0..rows {
            for j in 0..cols {
                let y = r.min.y + (i as f32 + 0.5) * rh;
                let x = r.min.x + j as f32 * cw + 3.0;
                self.line(pos2(x, y), pos2(x + cw * 0.6, y), if i == 0 { self.t.text } else { self.t.text_dim }, 1.5);
            }
        }
    }
    /// A bare grid (no text) in `r`, lines in `c`.
    pub fn grid(&self, r: Rect, rows: usize, cols: usize, c: Color32) {
        let (rows, cols) = (rows.max(1), cols.max(1));
        self.p.rect_stroke(r, 0.0, self.stroke(c), egui::StrokeKind::Inside);
        for i in 1..rows {
            let y = r.min.y + i as f32 * r.height() / rows as f32;
            self.line(pos2(r.min.x, y), pos2(r.max.x, y), c, 1.0);
        }
        for j in 1..cols {
            let x = r.min.x + j as f32 * r.width() / cols as f32;
            self.line(pos2(x, r.min.y), pos2(x, r.max.y), c, 1.0);
        }
    }
    /// The rect of cell `(row, col)` of a `rows × cols` grid in `r`.
    pub fn cell(&self, r: Rect, rows: usize, cols: usize, row: usize, col: usize) -> Rect {
        let rh = r.height() / rows.max(1) as f32;
        let cw = r.width() / cols.max(1) as f32;
        Rect::from_min_size(pos2(r.min.x + col as f32 * cw, r.min.y + row as f32 * rh), vec2(cw, rh))
    }
    /// A picture: a framed landscape (hill, sun).
    pub fn picture(&self, r: Rect) {
        self.p.rect_filled(r, 1.0, Color32::from_rgb(214, 232, 246));
        self.p.rect_stroke(r, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        let base = r.max.y - 2.0;
        let hill =
            vec![pos2(r.min.x + 2.0, base), pos2(r.min.x + r.width() * 0.35, r.min.y + r.height() * 0.45), pos2(r.min.x + r.width() * 0.6, base)];
        self.p.add(Shape::convex_polygon(hill, Color32::from_rgb(96, 150, 96), Stroke::NONE));
        let hill2 =
            vec![pos2(r.min.x + r.width() * 0.4, base), pos2(r.min.x + r.width() * 0.7, r.min.y + r.height() * 0.3), pos2(r.max.x - 2.0, base)];
        self.p.add(Shape::convex_polygon(hill2, Color32::from_rgb(70, 120, 80), Stroke::NONE));
        self.p.circle_filled(pos2(r.min.x + r.width() * 0.22, r.min.y + r.height() * 0.25), r.height() * 0.1, Color32::from_rgb(240, 190, 60));
    }
    /// A filled shape (a rounded rectangle with an accent fill) in `r`.
    pub fn shape(&self, r: Rect) {
        self.p.rect_filled(r, 4.0, self.t.accent.linear_multiply(0.35));
        self.p.rect_stroke(r, 4.0, Stroke::new(1.5, self.t.accent), egui::StrokeKind::Inside);
    }
    /// A text box: a framed box with text lines inside.
    pub fn text_box(&self, r: Rect) {
        self.p.rect_filled(r, 0.0, Color32::WHITE);
        self.p.rect_stroke(r, 0.0, Stroke::new(1.5, self.t.accent), egui::StrokeKind::Inside);
        self.paragraph(r.shrink(4.0), Dir::Ltr, ((r.height() - 8.0) / 10.0).max(1.0) as usize, self.grey());
    }
    /// Eight selection handles around `r`.
    pub fn handles(&self, r: Rect) {
        for p in
            [r.left_top(), r.center_top(), r.right_top(), r.left_center(), r.right_center(), r.left_bottom(), r.center_bottom(), r.right_bottom()]
        {
            self.p.circle_filled(p, 2.2, Color32::WHITE);
            self.p.circle_stroke(p, 2.2, Stroke::new(1.0, self.t.accent));
        }
    }
    /// A simple bar chart in `r`.
    pub fn chart(&self, r: Rect) {
        self.p.rect_filled(r, 1.0, Color32::WHITE);
        self.p.rect_stroke(r, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        let n = 5;
        let bw = (r.width() - 8.0) / n as f32;
        for (i, h) in [0.5, 0.8, 0.35, 0.95, 0.6].iter().enumerate() {
            let x = r.min.x + 4.0 + i as f32 * bw + 2.0;
            let top = r.max.y - 4.0 - (r.height() - 8.0) * h;
            self.p.rect_filled(
                Rect::from_min_max(pos2(x, top), pos2(x + bw - 4.0, r.max.y - 4.0)),
                1.0,
                if i % 2 == 0 { self.t.accent } else { self.t.accent.linear_multiply(0.5) },
            );
        }
    }
    /// A «field» chip at `at`, `w` wide.
    pub fn field(&self, at: Pos2, w: f32) {
        let r = Rect::from_min_size(at, vec2(w, 7.0));
        self.p.rect_filled(r, 1.0, self.t.accent.linear_multiply(0.25));
        self.p.rect_stroke(r, 1.0, self.stroke(self.t.accent), egui::StrokeKind::Inside);
    }
    /// A labelled chip (a tag, a style name, a button) centred at `c`.
    pub fn chip(&self, c: Pos2, text: &str, w: f32) {
        let r = Rect::from_center_size(c, vec2(w, 14.0));
        self.p.rect_filled(r, 3.0, Color32::WHITE);
        self.p.rect_stroke(r, 3.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        self.label(c, text, 7.5, self.t.text, Align2::CENTER_CENTER);
    }
    /// A coloured badge with white text (PDF, 100%, A4…) centred at `c`.
    pub fn badge(&self, c: Pos2, text: &str, fill: Color32) {
        let w = (text.chars().count() as f32 * 6.5 + 10.0).max(24.0);
        let r = Rect::from_center_size(c, vec2(w, 18.0));
        self.p.rect_filled(r, 3.0, fill);
        self.label(c, text, 9.0, Color32::WHITE, Align2::CENTER_CENTER);
    }
    /// A side pane (a task pane) `w` wide on the `right` or left of the thumbnail, with a title
    /// bar and a few rows.
    pub fn pane(&self, right: bool, w: f32, rows: usize) -> Rect {
        let r = if right {
            Rect::from_min_size(self.at(SIZE.x - w - 4.0, 4.0), vec2(w, SIZE.y - 8.0))
        } else {
            Rect::from_min_size(self.at(4.0, 4.0), vec2(w, SIZE.y - 8.0))
        };
        self.p.rect_filled(r, 2.0, self.t.panel);
        self.p.rect_stroke(r, 2.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        self.p.rect_filled(Rect::from_min_size(r.min, vec2(w, 12.0)), 2.0, self.t.hover);
        self.line(pos2(r.min.x + 4.0, r.min.y + 6.0), pos2(r.min.x + w * 0.6, r.min.y + 6.0), self.t.text, 2.0);
        let area = Rect::from_min_max(pos2(r.min.x + 6.0, r.min.y + 16.0), pos2(r.max.x - 6.0, r.max.y - 4.0));
        self.text_lines(area, false, &vec![area.width() * 0.8; rows], 11.0, self.grey());
        r
    }
    /// A dialog box in the middle of the thumbnail with a title and `rows` fields; returns its
    /// inner area.
    pub fn dialog(&self, title: &str, rows: usize) -> Rect {
        let r = Rect::from_center_size(self.at(SIZE.x / 2.0, SIZE.y / 2.0), vec2(120.0, 80.0));
        self.p.rect_filled(r, 3.0, self.t.menu);
        self.p.rect_stroke(r, 3.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        self.p.rect_filled(Rect::from_min_size(r.min, vec2(r.width(), 14.0)), 3.0, self.t.hover);
        self.label(pos2(r.min.x + 6.0, r.min.y + 7.0), title, 7.5, self.t.text, Align2::LEFT_CENTER);
        let inner = Rect::from_min_max(pos2(r.min.x + 8.0, r.min.y + 20.0), pos2(r.max.x - 8.0, r.max.y - 6.0));
        for i in 0..rows {
            let y = inner.min.y + i as f32 * 14.0;
            self.line(pos2(inner.min.x, y + 5.0), pos2(inner.min.x + 28.0, y + 5.0), self.grey(), 1.5);
            let f = Rect::from_min_size(pos2(inner.min.x + 34.0, y), vec2(inner.width() - 34.0, 10.0));
            self.p.rect_filled(f, 1.0, self.t.input);
            self.p.rect_stroke(f, 1.0, self.stroke(self.t.input_border), egui::StrokeKind::Inside);
        }
        inner
    }
    /// A printer at `c` (its centre).
    pub fn printer(&self, c: Pos2) {
        let body = Rect::from_center_size(c, vec2(34.0, 18.0));
        self.p.rect_filled(body, 2.0, self.t.hover);
        self.p.rect_stroke(body, 2.0, self.stroke(self.t.text), egui::StrokeKind::Inside);
        self.p.rect_stroke(Rect::from_center_size(c - vec2(0.0, 14.0), vec2(20.0, 10.0)), 1.0, self.stroke(self.t.text), egui::StrokeKind::Inside);
        let out = Rect::from_center_size(c + vec2(0.0, 15.0), vec2(20.0, 12.0));
        self.p.rect_filled(out, 1.0, Color32::WHITE);
        self.p.rect_stroke(out, 1.0, self.stroke(self.t.text), egui::StrokeKind::Inside);
    }
    /// A ribbon-like button with `text`, centred at `c`, drawn pressed when `on`.
    pub fn button(&self, c: Pos2, text: &str, on: bool) {
        let r = Rect::from_center_size(c, vec2(text.chars().count() as f32 * 6.0 + 16.0, 18.0));
        self.p.rect_filled(r, 3.0, if on { self.t.checked } else { self.t.menu });
        self.p.rect_stroke(r, 3.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        self.label(c, text, 8.0, self.t.text, Align2::CENTER_CENTER);
    }
    /// A margin box (dashed) inside a page area.
    pub fn dashed_rect(&self, r: Rect, c: Color32) {
        let step = 4.0;
        let mut x = r.min.x;
        while x < r.max.x {
            let x1 = (x + step * 0.6).min(r.max.x);
            self.line(pos2(x, r.min.y), pos2(x1, r.min.y), c, 1.0);
            self.line(pos2(x, r.max.y), pos2(x1, r.max.y), c, 1.0);
            x += step;
        }
        let mut y = r.min.y;
        while y < r.max.y {
            let y1 = (y + step * 0.6).min(r.max.y);
            self.line(pos2(r.min.x, y), pos2(r.min.x, y1), c, 1.0);
            self.line(pos2(r.max.x, y), pos2(r.max.x, y1), c, 1.0);
            y += step;
        }
    }
}
