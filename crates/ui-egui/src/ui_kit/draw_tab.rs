//! Draw tab thumbnails: ink pens on the page, selecting and erasing ink, converting it and
//! playing it back. Ink is a wavy stroke; the pen kind changes its width and texture.

use egui::{Color32, Pos2, Rect, Shape, Stroke, pos2, vec2};

use super::Entry;
use super::draw::Draw;

pub const ENTRIES: &[Entry] = &[
    Entry::new("draw.select", "Puts the pen down and returns to the normal pointer for selecting text and objects.", |d| {
        d.dr_ink_page(false);
        d.pointer(d.at(112.0, 44.0));
    }),
    Entry::new("draw.lasso", "Draws a free loop around ink strokes to select everything inside it.", |d| d.dr_lasso()),
    Entry::new("draw.eraser", "Rubs out the ink strokes you drag over.", |d| d.dr_eraser()),
    Entry::new("draw.pen", "Draws on the page with a crisp ink line in the chosen colour and thickness.", |d| {
        d.page_rl();
        d.dr_ink_wave(d.at(42.0, 44.0), 92.0, 8.0, 2.5, d.accent());
        d.dr_pen_tip(d.at(134.0, 40.0), d.accent());
    }),
    Entry::new("draw.pencil", "Draws on the page with a soft, grainy line, like a pencil.", |d| {
        d.page_rl();
        d.dr_pencil_ink(d.at(42.0, 44.0), 92.0, 8.0);
        d.dr_pen_tip(d.at(134.0, 40.0), d.grey());
    }),
    Entry::new("draw.highlighter", "Draws a wide see-through stroke over the text, like a highlighter pen.", |d| {
        d.page_rl();
        d.dr_ink_wave(d.at(48.0, 36.0), 80.0, 3.0, 9.0, d.yellow().linear_multiply(0.6));
        d.dr_pen_tip(d.at(134.0, 32.0), d.yellow());
    }),
    Entry::new("draw.addPen", "Adds another pen with its own colour and thickness to the row of pens.", |d| d.dr_add_pen()),
    Entry::new("draw.inkToShape", "Turns a shape you draw by hand into a clean shape with straight edges.", |d| d.dr_ink_to_shape()),
    Entry::new("insert.canvas", "Inserts a drawing area on the page that keeps its shapes and ink together as one object.", |d| d.dr_canvas()),
    Entry::new("draw.inkToMath", "Turns a formula you write by hand into a typed, editable equation.", |d| d.dr_ink_to_math()),
    Entry::new("draw.replay", "Plays your ink strokes back in the order you drew them.", |d| d.dr_replay()),
];

impl Draw<'_> {
    /// A wavy ink stroke from `from`, `len` long, `amp` high, `w` wide, in `c`.
    fn dr_ink_wave(&self, from: Pos2, len: f32, amp: f32, w: f32, c: Color32) {
        let n = 24;
        let pts: Vec<Pos2> =
            (0..=n).map(|i| from + vec2(len * i as f32 / n as f32, amp * (i as f32 / n as f32 * std::f32::consts::TAU).sin())).collect();
        self.p.add(Shape::line(pts, Stroke::new(w, c)));
    }

    /// The same wave as short grey dashes: a pencil's grain.
    fn dr_pencil_ink(&self, from: Pos2, len: f32, amp: f32) {
        let n = 36;
        for i in 0..n {
            if i % 3 == 2 {
                continue;
            }
            let t0 = i as f32 / n as f32;
            let t1 = (i as f32 + 0.8) / n as f32;
            let a = from + vec2(len * t0, amp * (t0 * std::f32::consts::TAU).sin());
            let b = from + vec2(len * t1, amp * (t1 * std::f32::consts::TAU).sin());
            self.line(a, b, self.grey(), 1.5);
        }
    }

    /// A pen body (a short thick bar) ending in a tip at `tip`.
    fn dr_pen_tip(&self, tip: Pos2, c: Color32) {
        self.line(tip + vec2(6.0, -18.0), tip + vec2(2.0, -6.0), c, 4.0);
        self.line(tip + vec2(2.0, -6.0), tip, self.ink(), 1.5);
    }

    /// A page with two ink strokes on it; the strokes in accent (or grey when `faint`).
    fn dr_ink_page(&self, faint: bool) {
        let a = self.page_rl();
        let c = if faint { self.grey() } else { self.accent() };
        self.dr_ink_wave(pos2(a.min.x + 4.0, a.min.y + 30.0), 48.0, 6.0, 2.0, c);
        self.dr_ink_wave(pos2(a.min.x + 10.0, a.min.y + 54.0), 36.0, 5.0, 2.0, c);
    }

    /// A dashed loop around the ink strokes.
    fn dr_lasso(&self) {
        self.dr_ink_page(false);
        let c = self.at(88.0, 50.0);
        let n = 40;
        for i in 0..n {
            if i % 2 == 1 {
                continue;
            }
            let t0 = i as f32 / n as f32 * std::f32::consts::TAU;
            let t1 = (i as f32 + 1.0) / n as f32 * std::f32::consts::TAU;
            let r = vec2(34.0, 24.0);
            let a = c + vec2(t0.cos() * r.x, t0.sin() * r.y);
            let b = c + vec2(t1.cos() * r.x, t1.sin() * r.y);
            self.line(a, b, self.ink(), 1.2);
        }
        self.pointer(self.at(124.0, 70.0));
    }

    /// A stroke with a gap rubbed out of it and the eraser block at the gap.
    fn dr_eraser(&self) {
        let a = self.page_rl();
        let from = pos2(a.min.x + 4.0, a.min.y + 34.0);
        let n = 24;
        let pts: Vec<Pos2> =
            (0..=n).map(|i| from + vec2(56.0 * i as f32 / n as f32, 7.0 * (i as f32 / n as f32 * std::f32::consts::TAU).sin())).collect();
        for part in [pts.get(..10), pts.get(16..)].into_iter().flatten() {
            self.p.add(Shape::line(part.to_vec(), Stroke::new(2.5, self.accent())));
        }
        let e = Rect::from_center_size(pos2(from.x + 56.0 * 0.5, from.y - 10.0), vec2(16.0, 10.0));
        self.p.rect_filled(e, 2.0, Color32::from_rgb(230, 150, 160));
        self.p.rect_stroke(e, 2.0, self.stroke(self.ink()), egui::StrokeKind::Inside);
        self.p.rect_filled(Rect::from_min_max(e.min, pos2(e.max.x, e.min.y + 4.0)), 2.0, self.t.text_dim);
    }

    /// A row of pens of different colours and widths, with a + badge for the new one.
    fn dr_add_pen(&self) {
        let colours = [self.accent(), self.red(), self.green(), Color32::from_rgb(120, 60, 180)];
        for (i, c) in colours.iter().enumerate() {
            let x = 30.0 + i as f32 * 26.0;
            let w = 2.0 + i as f32 * 1.5;
            self.line(self.at(x, 20.0), self.at(x, 56.0), *c, w);
            self.line(self.at(x, 56.0), self.at(x, 66.0), self.ink(), 1.5);
        }
        self.dr_ink_wave(self.at(20.0, 86.0), 100.0, 5.0, 2.0, self.grey());
        let c = self.at(142.0, 40.0);
        self.p.circle_filled(c, 12.0, self.accent());
        self.line(c + vec2(-6.0, 0.0), c + vec2(6.0, 0.0), Color32::WHITE, 2.5);
        self.line(c + vec2(0.0, -6.0), c + vec2(0.0, 6.0), Color32::WHITE, 2.5);
    }

    /// A wobbly hand-drawn rectangle becomes a clean one.
    fn dr_ink_to_shape(&self) {
        self.before_after(
            |d, a| {
                let r = Rect::from_center_size(a.center(), vec2(40.0, 30.0));
                let corners = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()];
                let mut pts = Vec::new();
                for w in corners.windows(2) {
                    if let (Some(p0), Some(p1)) = (w.first(), w.get(1)) {
                        for i in 0..8 {
                            let t = i as f32 / 8.0;
                            let wob = ((i as f32 * 1.7).sin()) * 1.8;
                            pts.push(*p0 + (*p1 - *p0) * t + vec2(wob, -wob));
                        }
                    }
                }
                d.p.add(Shape::line(pts, Stroke::new(2.0, d.accent())));
            },
            |d, a| d.shape(Rect::from_center_size(a.center(), vec2(40.0, 30.0))),
        );
    }

    /// A dashed canvas on the page holding a shape and an ink stroke.
    fn dr_canvas(&self) {
        let a = self.page_center();
        self.text_lines(a, true, &[56.0, 50.0], 10.0, self.grey());
        let r = Rect::from_min_max(pos2(a.min.x, a.min.y + 22.0), pos2(a.max.x, a.min.y + 62.0));
        self.p.rect_filled(r, 0.0, self.t.hover.linear_multiply(0.6));
        self.dashed_rect(r, self.accent());
        self.shape(Rect::from_min_size(r.min + vec2(5.0, 6.0), vec2(20.0, 14.0)));
        self.dr_ink_wave(r.min + vec2(28.0, 26.0), 26.0, 4.0, 2.0, self.accent());
        self.handles(r);
    }

    /// A stroke drawn up to its pen, with a play triangle.
    fn dr_replay(&self) {
        let a = self.page_rl();
        self.dr_ink_wave(pos2(a.min.x + 4.0, a.min.y + 30.0), 48.0, 6.0, 2.0, self.accent());
        self.dr_pencil_ink(pos2(a.min.x + 8.0, a.min.y + 56.0), 40.0, 5.0);
        let c = self.at(140.0, 56.0);
        self.p.circle_filled(c, 13.0, self.accent());
        let tri = vec![c + vec2(-4.0, -7.0), c + vec2(7.0, 0.0), c + vec2(-4.0, 7.0)];
        self.p.add(Shape::convex_polygon(tri, Color32::WHITE, Stroke::NONE));
    }

    /// A handwritten "x²" (wobbly ink) becomes a typed equation in a box.
    fn dr_ink_to_math(&self) {
        self.before_after(
            |d, a| {
                let c = a.center();
                // A hand-drawn x: two wobbly diagonals.
                for (p0, p1) in [(c + vec2(-14.0, -8.0), c + vec2(-2.0, 8.0)), (c + vec2(-2.0, -8.0), c + vec2(-14.0, 8.0))] {
                    let pts: Vec<Pos2> = (0..=8)
                        .map(|i| {
                            let t = i as f32 / 8.0;
                            p0 + (p1 - p0) * t + vec2(((i as f32) * 2.1).sin() * 1.2, 0.0)
                        })
                        .collect();
                    d.p.add(Shape::line(pts, Stroke::new(2.0, d.accent())));
                }
                // A small hand-drawn 2, raised.
                let o = c + vec2(6.0, -10.0);
                let pts = vec![o, o + vec2(5.0, -1.0), o + vec2(5.0, 3.0), o + vec2(0.0, 7.0), o + vec2(6.0, 7.0)];
                d.p.add(Shape::line(pts, Stroke::new(2.0, d.accent())));
            },
            |d, a| {
                let r = Rect::from_center_size(a.center(), vec2(40.0, 24.0));
                d.p.rect_filled(r, 2.0, Color32::WHITE);
                d.p.rect_stroke(r, 2.0, d.stroke(d.t.border), egui::StrokeKind::Inside);
                d.glyph(r.center() + vec2(-6.0, 1.0), "x", 16.0, d.ink());
                d.glyph(r.center() + vec2(5.0, -5.0), "2", 9.0, d.accent());
            },
        );
    }
}
