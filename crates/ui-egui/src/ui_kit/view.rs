//! View tab thumbnails: the document views, zoom levels, dark mode, windows and macros. Every
//! scene is a page (or a window) drawn with the shared kit; the part the button changes is in
//! the accent colour.

use egui::{Align2, Color32, Rect, Stroke, pos2, vec2};

use super::Entry;
use super::draw::{Dir, Draw, SIZE};

pub const ENTRIES: &[Entry] = &[
    Entry::new("view.readMode", "Shows the document as two facing pages for reading, with the ribbon hidden.", |d| d.vw_read_mode()),
    Entry::new("view.printLayout", "Shows every page as it will print, with its margins, on a grey background.", |d| d.vw_print_layout()),
    Entry::new("view.webLayout", "Spreads the text across the whole window, like a web page, without page edges.", |d| d.vw_web_layout()),
    Entry::new("view.outline", "Shows only the headings, stepped in by level, so you can reorder the document.", |d| d.vw_outline()),
    Entry::new("view.draft", "Shows the text alone, without page edges, pictures or headers, for fast typing.", |d| d.vw_draft()),
    Entry::new("view.focus", "Hides everything except the page, on a dark background, so you can concentrate.", |d| d.vw_focus()),
    Entry::new("view.immersive", "Shows the text in large, widely spaced lines that are easier to read.", |d| d.vw_immersive()),
    Entry::new("view.vertical", "Scrolls through the pages up and down, one below the other.", |d| d.vw_vertical()),
    Entry::new("view.sideToSide", "Lays the pages out in a row and flips through them sideways, like a book.", |d| d.vw_side_to_side()),
    Entry::new("ui.dialog:zoom", "Opens a box where you choose how large the page appears on screen.", |d| d.vw_zoom_dialog()),
    Entry::new("view.zoom100", "Shows the page at its real printed size on screen.", |d| d.vw_zoom_100()),
    Entry::new("view.zoomIn", "Makes the page appear larger on screen, a step at a time.", |d| d.vw_zoom_step(true)),
    Entry::new("view.zoomOut", "Makes the page appear smaller on screen, a step at a time.", |d| d.vw_zoom_step(false)),
    Entry::new("view.onePage", "Shrinks the view so one whole page fits in the window.", |d| d.vw_fit_pages(1)),
    Entry::new("view.multiplePages", "Shrinks the view so several pages fit in the window side by side.", |d| d.vw_fit_pages(3)),
    Entry::new("view.pageWidth", "Enlarges the view so the page is exactly as wide as the window.", |d| d.vw_page_width()),
    Entry::new("view.darkMode", "Switches the page between white with dark text and dark with light text.", |d| d.vw_switch_modes()),
    Entry::new("view.newWindow", "Opens the same document in a second window; what you change shows in both.", |d| d.vw_new_window()),
    Entry::new("view.arrangeAll", "Places all the open windows side by side so you can see every one.", |d| d.vw_arrange_all()),
    Entry::new("view.split", "Divides the window into two parts that scroll separately through the same document.", |d| d.vw_split()),
    Entry::new("view.fullScreen", "Makes the window fill the whole screen, hiding the title bar and the ribbon.", |d| d.vw_full_screen()),
    Entry::new("tools.macros", "Lists the recorded macros so you can run, edit or delete one.", |d| d.vw_macros()),
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

    fn vw_read_mode(&self) {
        // No ribbon: just two facing pages, edge to edge.
        let l = self.page(20.0, 8.0, 66.0, 88.0);
        let r = self.page(90.0, 8.0, 66.0, 88.0);
        self.paragraph(l, Dir::Ltr, 7, self.grey());
        self.paragraph(r, Dir::Ltr, 7, self.grey());
        self.arrow_left(self.at(84.0, 100.0), self.at(76.0, 100.0));
        self.arrow_right(self.at(92.0, 100.0), self.at(100.0, 100.0));
    }

    fn vw_print_layout(&self) {
        self.p.rect_filled(self.r, 2.0, self.t.canvas);
        let a = self.page_center();
        self.dashed_rect(a.expand(3.0), self.accent());
        self.paragraph(a, Dir::Ltr, 7, self.grey());
    }

    fn vw_web_layout(&self) {
        self.p.rect_filled(self.r, 2.0, Color32::WHITE);
        let a = Rect::from_min_max(self.at(10.0, 10.0), self.at(SIZE.x - 10.0, SIZE.y - 10.0));
        self.heading_then_text(a, Dir::Ltr, 7, self.accent());
    }

    fn vw_outline(&self) {
        let a = self.page_center();
        let rows: [(f32, &str, f32); 7] =
            [(0.0, "–", 3.0), (1.0, "–", 2.0), (2.0, "+", 1.5), (2.0, "+", 1.5), (1.0, "–", 2.0), (2.0, "+", 1.5), (0.0, "+", 3.0)];
        for (i, (level, mark, w)) in rows.iter().enumerate() {
            let y = a.min.y + 5.0 + i as f32 * 12.0;
            let x = a.min.x + level * 9.0;
            self.label(pos2(x + 2.0, y), mark, 8.0, self.accent(), Align2::CENTER_CENTER);
            self.line(pos2(x + 8.0, y), pos2(a.max.x - level * 4.0, y), if *w > 2.0 { self.ink() } else { self.grey() }, *w);
        }
    }

    fn vw_draft(&self) {
        self.p.rect_filled(self.r, 2.0, Color32::WHITE);
        let a = Rect::from_min_max(self.at(16.0, 8.0), self.at(SIZE.x - 16.0, SIZE.y - 8.0));
        self.paragraph(Rect::from_min_size(a.min, vec2(a.width(), 44.0)), Dir::Ltr, 4, self.grey());
        // A page break: a dotted line across, no page edge.
        let y = a.min.y + 50.0;
        let mut x = a.min.x;
        while x < a.max.x {
            self.line(pos2(x, y), pos2((x + 3.0).min(a.max.x), y), self.accent(), 1.0);
            x += 6.0;
        }
        self.paragraph(Rect::from_min_size(pos2(a.min.x, y + 4.0), vec2(a.width(), 40.0)), Dir::Ltr, 3, self.grey());
    }

    fn vw_focus(&self) {
        self.vw_dark_fill(self.r);
        let a = self.page_center();
        self.paragraph(a, Dir::Ltr, 7, self.grey());
    }

    fn vw_immersive(&self) {
        let a = self.page_wide();
        // Thicker lines, far apart: the big, airy reading font.
        for (i, w) in [100.0, 96.0, 104.0, 60.0].iter().enumerate() {
            let y = a.min.y + 6.0 + i as f32 * 18.0;
            self.line(pos2(a.min.x, y), pos2(a.min.x + w, y), self.ink(), 3.0);
        }
    }

    fn vw_vertical(&self) {
        for y in [6.0, 56.0] {
            let a = self.page(62.0, y, 52.0, 42.0);
            self.paragraph(a, Dir::Ltr, 3, self.grey());
        }
        self.measure_v(self.at(130.0, 12.0), self.at(130.0, 92.0));
    }

    fn vw_side_to_side(&self) {
        self.pages_row(3);
        self.measure_h(self.at(30.0, 86.0), self.at(146.0, 86.0));
    }

    fn vw_zoom_dialog(&self) {
        let inner = self.dialog("Zoom", 2);
        self.label(pos2(inner.max.x - 2.0, inner.max.y - 8.0), "%", 9.0, self.accent(), Align2::RIGHT_CENTER);
        self.magnifier(pos2(inner.min.x + 10.0, inner.max.y - 12.0));
    }

    fn vw_zoom_100(&self) {
        let a = self.page_lr();
        self.badge(pos2(a.max.x + 4.0, a.max.y - 6.0), "100%", self.accent());
    }

    fn vw_zoom_step(&self, bigger: bool) {
        let small = self.page(14.0, 30.0, 36.0, 44.0);
        self.paragraph(small, Dir::Ltr, 3, self.grey());
        let big = self.page(98.0, 10.0, 66.0, 84.0);
        self.paragraph(big, Dir::Ltr, 5, self.grey());
        if bigger {
            self.arrow_right(self.at(58.0, 52.0), self.at(90.0, 52.0));
        } else {
            self.arrow_left(self.at(90.0, 52.0), self.at(58.0, 52.0));
        }
        let c = self.at(74.0, 28.0);
        self.magnifier(c);
        self.glyph(c, if bigger { "+" } else { "−" }, 12.0, self.accent());
    }

    fn vw_fit_pages(&self, n: usize) {
        let area = self.vw_window(8.0, 4.0, SIZE.x - 16.0, SIZE.y - 8.0);
        let n = n.clamp(1, 4);
        let w = 34.0;
        let total = n as f32 * (w + 8.0) - 8.0;
        let x0 = area.center().x - total / 2.0 - self.r.min.x;
        for i in 0..n {
            let a = self.page(x0 + i as f32 * (w + 8.0), area.min.y - self.r.min.y + 12.0, w, 54.0);
            self.paragraph(a, Dir::Ltr, 4, self.grey());
        }
    }

    fn vw_page_width(&self) {
        let area = self.vw_window(8.0, 4.0, SIZE.x - 16.0, SIZE.y - 8.0);
        self.measure_h(pos2(area.min.x + 6.0, area.min.y + 6.0), pos2(area.max.x - 6.0, area.min.y + 6.0));
        let a = self.page(14.0, area.min.y - self.r.min.y + 12.0, SIZE.x - 28.0, 72.0);
        self.paragraph(a, Dir::Ltr, 5, self.grey());
    }

    fn vw_switch_modes(&self) {
        let a = self.page_center();
        let outer = a.expand(8.0);
        let right = Rect::from_min_max(pos2(outer.center().x, outer.min.y), outer.max);
        self.vw_dark_fill(right);
        let light = Color32::from_rgb(210, 210, 215);
        for i in 0..7 {
            let y = a.min.y + 4.0 + i as f32 * 10.0;
            let w = if i == 6 { 0.55 } else { 0.95 };
            let x1 = a.min.x + a.width() * w;
            let mid = outer.center().x;
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
        let moon = pos2(a.max.x - 8.0, a.max.y - 2.0);
        self.p.circle_filled(moon, 6.0, self.accent());
        self.p.circle_filled(moon + vec2(3.0, -2.0), 5.0, DARK);
    }

    fn vw_new_window(&self) {
        let back = self.vw_window(16.0, 8.0, 100.0, 70.0);
        self.vw_mini_page(back);
        let front = self.vw_window(60.0, 28.0, 100.0, 70.0);
        self.vw_mini_page(front);
        let whole = Rect::from_min_max(pos2(front.min.x, front.min.y - 10.0), front.max);
        self.p.rect_stroke(whole, 2.0, Stroke::new(1.5, self.accent()), egui::StrokeKind::Inside);
    }

    /// Two windows side by side (Arrange All).
    fn vw_arrange_all(&self) {
        let l = self.vw_window(6.0, 8.0, 80.0, 88.0);
        self.vw_mini_page(l);
        let r = self.vw_window(90.0, 8.0, 80.0, 88.0);
        self.vw_mini_page(r);
    }

    fn vw_split(&self) {
        let a = self.vw_window(28.0, 4.0, 120.0, 96.0);
        let top = Rect::from_min_max(a.min, pos2(a.max.x, a.center().y - 3.0));
        let bottom = Rect::from_min_max(pos2(a.min.x, a.center().y + 3.0), a.max);
        self.p.rect_filled(top.shrink(4.0), 1.0, Color32::WHITE);
        self.p.rect_filled(bottom.shrink(4.0), 1.0, Color32::WHITE);
        self.paragraph(top.shrink(8.0), Dir::Ltr, 3, self.grey());
        self.paragraph(bottom.shrink(8.0), Dir::Ltr, 3, self.grey());
        self.line(pos2(a.min.x, a.center().y), pos2(a.max.x, a.center().y), self.accent(), 4.0);
    }

    fn vw_full_screen(&self) {
        let a = self.page(4.0, 4.0, SIZE.x - 8.0, SIZE.y - 8.0);
        self.paragraph(a.shrink2(vec2(24.0, 8.0)), Dir::Ltr, 6, self.grey());
        let o = a.expand(2.0);
        let c = self.accent();
        for (corner, dx, dy) in [(o.left_top(), 1.0, 1.0), (o.right_top(), -1.0, 1.0), (o.left_bottom(), 1.0, -1.0), (o.right_bottom(), -1.0, -1.0)] {
            let inner = corner + vec2(dx * 12.0, dy * 12.0);
            self.line(inner, corner, c, 1.5);
            self.line(corner, corner + vec2(dx * 7.0, 0.0), c, 1.5);
            self.line(corner, corner + vec2(0.0, dy * 7.0), c, 1.5);
        }
    }

    fn vw_macros(&self) {
        let inner = self.dialog("Macros", 0);
        let rows = 3;
        for i in 0..rows {
            let y = inner.min.y + 2.0 + i as f32 * 12.0;
            let row = Rect::from_min_size(pos2(inner.min.x, y - 4.0), vec2(inner.width() - 30.0, 10.0));
            if i == 0 {
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
