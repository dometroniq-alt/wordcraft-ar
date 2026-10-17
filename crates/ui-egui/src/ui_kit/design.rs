//! Design tab thumbnails: themes (colours, fonts, spacing, effects) and the page background
//! (watermark, page colour, page borders).

use egui::{Align2, Color32, Rect, Stroke, pos2, vec2};

use super::Entry;
use super::draw::{Dir, Draw};

pub const ENTRIES: &[Entry] = &[
    Entry::new("design.theme", "Applies a matched set of colours, fonts and effects to the whole document at once.", |d| d.ds_theme()),
    Entry::new("design.themeColors", "Changes the set of colours used by headings, shapes and charts throughout the document.", |d| {
        d.ds_theme_colors()
    }),
    Entry::new("design.themeFonts", "Changes the pair of fonts used for headings and for body text throughout the document.", |d| d.ds_theme_fonts()),
    Entry::new("design.paragraphSpacing", "Sets, for the whole document, how much space lies between paragraphs and between lines.", |d| {
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
    }),
    Entry::new("design.effects", "Changes how shapes and pictures look: flat, with soft shadows, or with strong edges.", |d| d.ds_effects()),
    Entry::new("design.setDefault", "Makes the current theme, fonts and spacing the starting point for every new document.", |d| d.ds_set_default()),
    Entry::new("design.watermark", "Prints a faint word (such as DRAFT) diagonally behind the text of every page.", |d| d.ds_watermark()),
    Entry::new("design.pageColor", "Colours the background of every page instead of plain white.", |d| d.ds_page_color()),
    Entry::new("design.pageBorders", "Draws a frame around the edge of every page.", |d| d.ds_page_borders()),
];

impl Draw<'_> {
    /// Three colour swatches in a row at `y`, the first in accent.
    fn ds_swatches(&self, x: f32, y: f32) {
        for (i, c) in [self.accent(), self.green(), Color32::from_rgb(220, 150, 20)].iter().enumerate() {
            self.p.rect_filled(Rect::from_min_size(self.at(x + i as f32 * 16.0, y), vec2(12.0, 12.0)), 2.0, *c);
        }
    }

    /// Swatches plus a font sample beside a page whose heading and shape took them.
    fn ds_theme(&self) {
        self.ds_swatches(10.0, 18.0);
        self.glyph(self.at(32.0, 48.0), "Aa", 16.0, self.ink());
        self.arrow_right(self.at(62.0, 50.0), self.at(80.0, 50.0));
        let a = self.page(90.0, 8.0, 70.0, 88.0);
        self.heading_then_text(a, Dir::Rtl, 2, self.accent());
        self.shape(Rect::from_min_size(pos2(a.min.x + 10.0, a.min.y + 40.0), vec2(34.0, 20.0)));
    }

    /// Two colour sets: the page's heading and shape change from one to the other.
    fn ds_theme_colors(&self) {
        self.before_after(
            |d, a| {
                d.heading_then_text(a, Dir::Rtl, 2, d.grey());
                let r = Rect::from_min_size(pos2(a.min.x + 8.0, a.min.y + 40.0), vec2(34.0, 20.0));
                d.p.rect_filled(r, 4.0, d.grey().linear_multiply(0.4));
                d.p.rect_stroke(r, 4.0, Stroke::new(1.5, d.grey()), egui::StrokeKind::Inside);
            },
            |d, a| {
                d.heading_then_text(a, Dir::Rtl, 2, d.accent());
                d.shape(Rect::from_min_size(pos2(a.min.x + 8.0, a.min.y + 40.0), vec2(34.0, 20.0)));
            },
        );
        self.ds_swatches(66.0, 80.0);
    }

    /// A heading "Aa" and a body "Aa" in two styles on one page.
    fn ds_theme_fonts(&self) {
        let a = self.page_wide();
        self.glyph(pos2(a.max.x - 22.0, a.min.y + 14.0), "Aa", 18.0, self.accent());
        self.line(pos2(a.min.x, a.min.y + 14.0), pos2(a.max.x - 48.0, a.min.y + 14.0), self.accent(), 3.0);
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

    /// The current page's look, ticked, flowing into a stack of new pages.
    fn ds_set_default(&self) {
        let a = self.page(8.0, 8.0, 60.0, 80.0);
        self.heading_then_text(a, Dir::Rtl, 4, self.accent());
        self.check(self.at(62.0, 80.0));
        self.arrow_right(self.at(76.0, 48.0), self.at(94.0, 48.0));
        let top = self.page_stack(100.0, 8.0, 56.0, 68.0, 3);
        self.heading_then_text(top, Dir::Rtl, 3, self.accent());
    }

    /// A faint diagonal word behind the text.
    fn ds_watermark(&self) {
        let a = self.page_rl();
        let galley = self.p.layout_no_wrap("نموذج".to_string(), crate::theme::semibold(16.0), self.accent().linear_multiply(0.35));
        let shape = egui::epaint::TextShape::new(a.center(), galley, self.accent())
            .with_angle_and_anchor(-std::f32::consts::FRAC_PI_4, Align2::CENTER_CENTER);
        self.p.add(shape);
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

    /// An accent frame just inside the page edge, on two pages.
    fn ds_page_borders(&self) {
        for x in [26.0, 94.0] {
            let a = self.page(x, 8.0, 56.0, 88.0);
            self.p.rect_stroke(a.expand(4.0), 0.0, Stroke::new(2.0, self.accent()), egui::StrokeKind::Inside);
            self.paragraph(a.shrink2(vec2(2.0, 4.0)), Dir::Rtl, 5, self.grey());
        }
    }
}
