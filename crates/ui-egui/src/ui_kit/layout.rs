//! Layout tab thumbnails: page setup (margins, orientation, size, columns, breaks, line numbers,
//! hyphenation, text direction) and arranging pictures and shapes on the page.

use egui::{Align2, Rect, Shape, Stroke, pos2, vec2};

use super::Entry;
use super::draw::{Dir, Draw};

pub const ENTRIES: &[Entry] = &[
    Entry::new("layout.margins", "Sets how much blank space is left around the text on every page.", |d| d.lay_margins()),
    Entry::new("layout.orientation", "Turns the page sideways (wider than tall) or upright again.", |d| d.lay_orientation()),
    Entry::new("layout.size", "Picks the paper size the pages are laid out for (A4, Letter…).", |d| d.lay_paper_size()),
    Entry::new("layout.columns", "Splits the text into two or more columns side by side, like a newspaper.", |d| d.lay_columns()),
    Entry::new("layout.break", "Starts a new page, column or section at the caret instead of pressing Enter many times.", |d| d.lay_breaks()),
    Entry::new("layout.lineNumbers", "Prints a small number in the margin beside every line of text.", |d| d.lay_line_numbers()),
    Entry::new("layout.hyphenation", "Breaks long words at a line end with a hyphen so the lines fill out evenly.", |d| d.lay_hyphenation()),
    Entry::new("layout.textDirection", "Turns the text to run top to bottom in the selected cells, text box or section.", |d| d.lay_text_direction()),
    Entry::new("arrange.position", "Moves the selected picture or shape to a set spot on the page, such as a corner or the middle.", |d| {
        d.lay_position()
    }),
    Entry::new("arrange.wrap", "Chooses how the text flows around the selected picture: around it, above and below, or behind it.", |d| d.lay_wrap()),
    Entry::new("arrange.bringForward", "Moves the selected shape one step up so it shows on top of the shapes it overlaps.", |d| {
        d.lay_stack_order(true)
    }),
    Entry::new("arrange.sendBackward", "Moves the selected shape one step down so the shapes it overlaps show on top of it.", |d| {
        d.lay_stack_order(false)
    }),
    Entry::new("arrange.selectionPane", "Opens a list of every picture, shape and text box on the page so you can select or hide each one.", |d| {
        d.lay_selection_pane()
    }),
    Entry::new("arrange.align", "Lines up the selected shapes with each other or with the edges of the page.", |d| d.lay_align()),
    Entry::new("arrange.group", "Joins the selected shapes into one object that moves and resizes together.", |d| d.lay_group()),
    Entry::new("arrange.rotate", "Turns the selected picture or shape a quarter turn, or flips it over.", |d| d.lay_rotate()),
];

impl Draw<'_> {
    /// A page whose text area is outlined with a dashed box, the margins measured.
    fn lay_margins(&self) {
        let page = Rect::from_min_size(self.at(50.0, 6.0), vec2(76.0, 92.0));
        let area = self.page(50.0, 6.0, 76.0, 92.0);
        let inner = area.shrink2(vec2(4.0, 6.0));
        self.dashed_rect(inner, self.accent());
        self.paragraph(inner.shrink(3.0), Dir::Ltr, 5, self.grey());
        // The left margin and the top margin, measured.
        self.measure_h(pos2(page.min.x + 1.0, inner.center().y), pos2(inner.min.x, inner.center().y));
        self.measure_v(pos2(inner.center().x, page.min.y + 1.0), pos2(inner.center().x, inner.min.y));
    }

    /// An upright page becomes a sideways one.
    fn lay_orientation(&self) {
        let before = self.page(10.0, 16.0, 52.0, 72.0);
        self.paragraph(before, Dir::Ltr, 5, self.grey());
        self.arrow_right(self.at(68.0, 52.0), self.at(84.0, 52.0));
        let after = self.page(90.0, 26.0, 80.0, 52.0);
        self.paragraph(after, Dir::Ltr, 3, self.grey());
    }

    /// A page with its paper name on it.
    fn lay_paper_size(&self) {
        let area = self.page_lr();
        self.badge(area.center(), "A4", self.accent());
        self.measure_h(pos2(area.min.x - 8.0, area.max.y + 4.0), pos2(area.max.x + 8.0, area.max.y + 4.0));
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

    /// Text stops halfway down one page at a dashed cut and goes on at the top of the next.
    fn lay_breaks(&self) {
        let first = self.page(14.0, 8.0, 66.0, 88.0);
        self.text_lines(first, false, &[50.0, 46.0, 48.0, 30.0], 10.0, self.grey());
        let y = first.min.y + 42.0;
        let cut = Rect::from_min_max(pos2(first.min.x - 4.0, y), pos2(first.max.x + 4.0, y));
        self.dashed_rect(cut, self.accent());
        self.arrow_right(self.at(84.0, 52.0), self.at(92.0, 52.0));
        let second = self.page(96.0, 8.0, 66.0, 88.0);
        self.text_lines(second, false, &[50.0, 46.0, 48.0, 44.0, 28.0], 10.0, self.accent());
    }

    /// A number beside every line.
    fn lay_line_numbers(&self) {
        let area = self.page_center();
        let text = Rect::from_min_max(pos2(area.min.x + 12.0, area.min.y), area.max);
        self.paragraph(text, Dir::Ltr, 7, self.grey());
        for (i, n) in ["1", "2", "3", "4", "5", "6", "7"].iter().enumerate() {
            let y = text.min.y + 4.0 + i as f32 * 10.0;
            self.label(pos2(area.min.x + 6.0, y), n, 6.0, self.accent(), Align2::RIGHT_CENTER);
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

    /// A picture moves from the middle of the page to its top-right corner; the text flows round it.
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
        self.picture(pic);
        self.arrow_right(was.right_top() + vec2(2.0, -2.0), pic.left_bottom() + vec2(-2.0, 2.0));
    }

    /// Text runs down both sides of a picture.
    fn lay_wrap(&self) {
        let area = self.page_center();
        let pic = Rect::from_center_size(area.center(), vec2(26.0, 24.0));
        let side = 14.0;
        self.text_lines(area, false, &[60.0, 56.0], 7.0, self.grey());
        let left = Rect::from_min_max(pos2(area.min.x, area.min.y + 14.0), pos2(pic.min.x - 3.0, pic.max.y));
        let right = Rect::from_min_max(pos2(pic.max.x + 3.0, area.min.y + 14.0), pos2(area.max.x, pic.max.y));
        self.text_lines(left, false, &[side, side, side, side], 7.0, self.accent());
        self.text_lines(right, true, &[side, side, side, side], 7.0, self.accent());
        self.text_lines(Rect::from_min_max(pos2(area.min.x, area.min.y + 44.0), area.max), false, &[60.0, 58.0, 36.0], 7.0, self.grey());
        self.picture(pic);
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

    /// A page with a picture and a shape, and the pane that lists them.
    fn lay_selection_pane(&self) {
        let area = self.page(10.0, 8.0, 84.0, 88.0);
        self.text_lines(area, false, &[60.0, 56.0], 7.0, self.grey());
        self.picture(Rect::from_min_size(area.min + vec2(0.0, 16.0), vec2(30.0, 22.0)));
        self.shape(Rect::from_min_size(area.min + vec2(38.0, 20.0), vec2(28.0, 18.0)));
        self.text_lines(Rect::from_min_max(pos2(area.min.x, area.min.y + 46.0), area.max), false, &[64.0, 60.0, 40.0], 7.0, self.grey());
        let pane = self.pane(true, 66.0, 3);
        // The first row (the picture) is the selected one.
        let row = Rect::from_min_size(pos2(pane.min.x + 3.0, pane.min.y + 15.0), vec2(pane.width() - 6.0, 10.0));
        self.selection(row);
        self.line(pos2(row.min.x + 3.0, row.center().y), pos2(row.max.x - 12.0, row.center().y), self.accent(), 1.5);
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

    /// Two shapes, each with its own handles, become one framed object with one set of handles.
    fn lay_group(&self) {
        let a_box = |a: Rect| Rect::from_min_size(a.min + vec2(6.0, 10.0), vec2(26.0, 20.0));
        let b_box = |a: Rect| Rect::from_min_size(a.min + vec2(22.0, 36.0), vec2(26.0, 20.0));
        self.before_after(
            |d, a| {
                for r in [a_box(a), b_box(a)] {
                    d.shape(r);
                    d.handles(r);
                }
            },
            |d, a| {
                let frame = a_box(a).union(b_box(a)).expand(3.0);
                d.dashed_rect(frame, d.accent());
                d.shape(a_box(a));
                d.shape(b_box(a));
                d.handles(frame);
            },
        );
    }

    /// A picture with a curved arrow around it.
    fn lay_rotate(&self) {
        let c = self.at(88.0, 54.0);
        self.picture(Rect::from_center_size(c, vec2(44.0, 34.0)));
        let r = 36.0;
        let n = 16;
        let (a0, a1) = (-2.2_f32, -0.35_f32);
        let pts: Vec<_> = (0..=n).map(|i| a0 + (a1 - a0) * i as f32 / n as f32).map(|ang| c + vec2(ang.cos(), ang.sin()) * r).collect();
        self.p.add(Shape::line(pts, Stroke::new(2.0, self.accent())));
        let tip = c + vec2(a1.cos(), a1.sin()) * r;
        let tangent = vec2(-a1.sin(), a1.cos());
        let back = tip - tangent * 7.0;
        let side = vec2(tangent.y, -tangent.x) * 4.5;
        self.p.add(Shape::convex_polygon(vec![tip + tangent * 2.0, back + side, back - side], self.accent(), Stroke::NONE));
    }
}
