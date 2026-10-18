//! WordCraft AR: a wrapped right-to-left paragraph. egui wraps a shaped Arabic run by its
//! glyphs in visual order, so the first row gets the *end* of the sentence and mixed text
//! scatters; this widget breaks the text into lines itself, in reading order, and draws every
//! line in the order the renderer gets right ([`crate::i18n::rtl_display`]), aligned to the
//! right edge. English (no right-to-left character) is wrapped the ordinary way.

use egui::{Align2, Color32, FontId, Response, Sense, Ui, pos2, vec2};

use crate::i18n::{is_arabic_digit, rtl_display};
use wordcraft_fonts::is_rtl as is_rtl_char;

/// A paragraph of `text` wrapped to `max_width` in `font`, right-aligned when it holds
/// right-to-left text (every line starts at the right edge), left-aligned otherwise. Returns
/// the response of the area it took.
pub fn paragraph(ui: &mut Ui, text: &str, font: FontId, color: Color32, max_width: f32) -> Response {
    let rtl = text.chars().any(|c| is_rtl_char(c) || is_arabic_digit(c));
    if !rtl {
        return ui.add(egui::Label::new(egui::RichText::new(text).font(font).color(color)).wrap());
    }
    let lines = wrap(ui, text, &font, color, max_width);
    let galleys: Vec<_> = lines.iter().map(|l| ui.fonts_mut(|f| f.layout_no_wrap(rtl_display(l), font.clone(), color))).collect();
    let width = galleys.iter().map(|g| g.size().x).fold(0.0f32, f32::max).min(max_width).max(1.0);
    let row = galleys.first().map(|g| g.size().y).unwrap_or(font.size * 1.3);
    let height = row * galleys.len() as f32;
    let (rect, resp) = ui.allocate_exact_size(vec2(max_width.max(width), height), Sense::hover());
    let mut y = rect.min.y;
    for g in galleys {
        let x = rect.max.x - g.size().x;
        ui.painter().galley(pos2(x, y), g, color);
        y += row;
    }
    let _ = Align2::RIGHT_TOP;
    resp
}

/// `text` broken into lines of at most `max_width` (by words; a word longer than the width
/// stands alone), each line in logical order. Explicit newlines start new lines.
pub fn wrap(ui: &Ui, text: &str, font: &FontId, color: Color32, max_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for para in text.split('\n') {
        let mut line = String::new();
        for word in para.split(' ') {
            let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            let w = ui.fonts_mut(|f| f.layout_no_wrap(rtl_display(&candidate), font.clone(), color).size().x);
            if w <= max_width || line.is_empty() {
                line = candidate;
            } else {
                lines.push(std::mem::replace(&mut line, word.to_string()));
            }
        }
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_keep_reading_order_and_fit_the_width() {
        let ctx = egui::Context::default();
        let text = "يضع الخلايا التي قصصتها أو نسختها في الورقة بدءًا من الخلية المحددة. مثال: انسخ A1:A3 ثم انقر C1 والصق.";
        let mut out = ctx.run_ui(Default::default(), |ui| {
            let font = FontId::proportional(13.0);
            let lines = wrap(ui, text, &font, Color32::BLACK, 160.0);
            assert!(lines.len() >= 3, "{lines:?}");
            assert!(lines.first().is_some_and(|l| l.starts_with("يضع")), "the first line holds the start: {lines:?}");
            assert!(lines.last().is_some_and(|l| l.ends_with("والصق.")), "the last line holds the end: {lines:?}");
            assert_eq!(lines.join(" "), text, "nothing lost or doubled");
            for l in &lines {
                let w = ui.fonts_mut(|f| f.layout_no_wrap(l.clone(), font.clone(), Color32::BLACK).size().x);
                assert!(w <= 160.0 || !l.contains(' '), "{l}: {w}");
            }
            let r = paragraph(ui, text, font, Color32::BLACK, 160.0);
            assert!(r.rect.height() > 20.0);
        });
        out.textures_delta.clear();
    }
}
