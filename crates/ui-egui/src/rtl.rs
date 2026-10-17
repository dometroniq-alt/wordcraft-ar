//! WordCraft AR: the interface is laid out right-to-left when its language is Arabic, as Word's
//! Arabic interface is — the ribbon's groups run from the right (Clipboard at the far right),
//! the tabs start at the right edge, a small button's icon stands at its right end, the status
//! bar's page count sits at the right and the zoom at the left.
//!
//! This module is the single source of truth for the direction: layout code asks it, never the
//! language itself. When the interface is not Arabic every helper is exactly egui's
//! left-to-right default (the `Ui` helpers reproduce `ui.horizontal`, `ui.horizontal_top`,
//! `ui.horizontal_centered` and `ui.vertical` to the pixel), so every other language is laid
//! out as before.

use egui::{Align, Align2, InnerResponse, Layout, Pos2, Rect, Ui, pos2, vec2};

/// Is the interface laid out right-to-left (its language is Arabic)?
pub fn on() -> bool {
    crate::i18n::current().is_arabic()
}

/// A row from the start edge: right-to-left when [`on`], else left-to-right.
pub fn row(valign: Align) -> Layout {
    if on() { Layout::right_to_left(valign) } else { Layout::left_to_right(valign) }
}

/// A row from the start edge with its items at the top.
pub fn row_top() -> Layout {
    row(Align::Min)
}

/// A row from the start edge with its items vertically centred.
pub fn row_center() -> Layout {
    row(Align::Center)
}

/// A row from the end edge, the opposite direction: from the right in a left-to-right interface,
/// from the left when [`on`]. For the controls that sit at the end of a bar (the title bar's
/// search box and account, the tab strip's Share, the status bar's zoom).
pub fn row_from_end(valign: Align) -> Layout {
    if on() { Layout::left_to_right(valign) } else { Layout::right_to_left(valign) }
}

/// A column whose items are aligned to the start edge (`ui.vertical`'s layout mirrored).
pub fn column() -> Layout {
    Layout::top_down(align_start())
}

/// The start edge: right when [`on`], else left.
pub fn align_start() -> Align {
    if on() { Align::RIGHT } else { Align::LEFT }
}

/// The end edge: left when [`on`], else right.
pub fn align_end() -> Align {
    if on() { Align::LEFT } else { Align::RIGHT }
}

/// Text anchored at the start edge, vertically centred (`LEFT_CENTER` or `RIGHT_CENTER`).
pub fn start_center() -> Align2 {
    Align2([align_start(), Align::Center])
}

/// Text anchored at the start edge and the top (`LEFT_TOP` or `RIGHT_TOP`).
pub fn start_top() -> Align2 {
    Align2([align_start(), Align::Min])
}

/// The x `inset` points in from `r`'s start edge.
pub fn start_x(r: Rect, inset: f32) -> f32 {
    if on() { r.max.x - inset } else { r.min.x + inset }
}

/// The x `inset` points in from `r`'s end edge.
pub fn end_x(r: Rect, inset: f32) -> f32 {
    if on() { r.min.x + inset } else { r.max.x - inset }
}

/// `rect` mirrored horizontally within `within` when [`on`]; `rect` itself otherwise.
pub fn mirror_x(rect: Rect, within: Rect) -> Rect {
    if !on() {
        return rect;
    }
    let left = within.min.x + (within.max.x - rect.max.x);
    let right = within.max.x - (rect.min.x - within.min.x);
    Rect::from_min_max(pos2(left, rect.min.y), pos2(right, rect.max.y))
}

/// Where the next item of a row goes: its left edge in a left-to-right row, its right edge when
/// the row runs right-to-left (egui's cursor rect keeps that side as its `max.x`).
pub fn cursor_x(ui: &Ui) -> f32 {
    let c = ui.cursor();
    if ui.layout().main_dir() == egui::Direction::RightToLeft { c.max.x } else { c.min.x }
}

/// The top corner where the next item of a row goes ([`cursor_x`] at the cursor's top).
pub fn cursor_start(ui: &Ui) -> Pos2 {
    pos2(cursor_x(ui), ui.cursor().min.y)
}

/// The direction a row inside `ui` runs: right-to-left when [`on`], or when egui itself would
/// prefer it there (a column aligned to the right), else left-to-right — `ui.horizontal`'s rule
/// plus the Arabic interface.
fn row_in(ui: &Ui, valign: Align) -> Layout {
    if on() || ui.layout().prefer_right_to_left() { Layout::right_to_left(valign) } else { Layout::left_to_right(valign) }
}

/// `ui.horizontal`, mirrored when [`on`].
pub fn horizontal<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let size = vec2(ui.available_size_before_wrap().x, ui.spacing().interact_size.y);
    let layout = row_in(ui, Align::Center);
    ui.allocate_ui_with_layout(size, layout, add)
}

/// `ui.horizontal_top`, mirrored when [`on`].
pub fn horizontal_top<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let size = ui.available_size_before_wrap();
    let layout = row_in(ui, Align::Min);
    ui.allocate_ui_with_layout(size, layout, add)
}

/// `ui.horizontal_centered`, mirrored when [`on`].
pub fn horizontal_centered<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let size = ui.available_size_before_wrap();
    let layout = row_in(ui, Align::Center);
    ui.allocate_ui_with_layout(size, layout, add)
}

/// `ui.vertical`, its items aligned to the start edge when [`on`].
pub fn vertical<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    ui.with_layout(column(), add)
}

/// A menu item with a shortcut: the text at the start edge, the shortcut (weak) at the end edge.
/// Menu popups keep egui's left-to-right column (egui's `Grid`, which several ribbon menus hold,
/// can't lay out in a right-aligned one), so a right-to-left item puts its text at the right end
/// of the button through the button's own parts instead of the column's alignment.
pub fn menu_item<'a>(text: impl egui::IntoAtoms<'a>, shortcut: &str) -> egui::Button<'a> {
    if on() {
        // From the left: the shortcut, a stretch, then the text, which so ends at the right edge.
        egui::Button::new(egui::RichText::new(shortcut).weak()).right_text(text)
    } else {
        egui::Button::new(text).shortcut_text(shortcut)
    }
}

/// A menu item with a check mark's slot before its text: the mark at the start edge, the text
/// after it (at the right end, the mark rightmost, when [`on`]).
pub fn check_item<'a>(mark: egui::RichText, text: &'a str) -> egui::Button<'a> {
    if on() { egui::Button::new("").right_text((text, mark)) } else { egui::Button::new((mark, text)) }
}

/// An island laid out left-to-right inside a right-to-left row or column, for what egui lays out
/// left-to-right only (`Grid`): as wide as it was last frame, anchored at the row's cursor (its
/// right end). In a left-to-right interface, or a left-to-right `ui`, it is `add` itself. The
/// first frame, with no width known yet, is discarded like a `Grid`'s own first frame.
pub fn ltr_island<R>(ui: &mut Ui, id_salt: impl egui::AsIdSalt, add: impl FnOnce(&mut Ui) -> R) -> R {
    if !ui.layout().prefer_right_to_left() {
        return add(ui);
    }
    let id = ui.id().with(id_salt).with("ltr_island");
    let width = ui.data(|d| d.get_temp::<f32>(id));
    if width.is_none() && ui.is_visible() {
        ui.ctx().request_discard("rtl::ltr_island first frame");
    }
    let avail = ui.available_rect_before_wrap();
    let rect = Rect::from_min_max(pos2(avail.max.x - width.unwrap_or(0.0), avail.min.y), avail.max);
    let inner = ui.scope_builder(egui::UiBuilder::new().max_rect(rect).layout(Layout::top_down(Align::Min)), add);
    ui.data_mut(|d| d.insert_temp(id, inner.response.rect.width()));
    inner.inner
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::WordApp;
    use crate::i18n::{Lang, set_current};

    /// Every text drawn in one frame of the whole app at 2560×900 (the audit's screen), with the
    /// top-left corner of each.
    fn drawn_texts(ctx: &egui::Context, app: &mut WordApp) -> Vec<(String, egui::Pos2)> {
        fn texts(shape: &egui::Shape, out: &mut Vec<(String, egui::Pos2)>) {
            match shape {
                egui::Shape::Text(t) => out.push((t.galley.text().to_string(), t.pos)),
                egui::Shape::Vec(v) => v.iter().for_each(|s| texts(s, out)),
                _ => {}
            }
        }
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(2560.0, 900.0));
        let mut drawn = Vec::new();
        // Fonts install over the first frames; the ribbon's right-to-left strip settles in one.
        for n in 0..4 {
            let input = egui::RawInput { screen_rect: Some(screen), time: Some(f64::from(n) * 0.1), ..Default::default() };
            let mut out = ctx.run_ui(input, |ui| {
                app.logic(ui.ctx());
                app.ui(ui);
            });
            drawn.clear();
            for s in &out.shapes {
                texts(&s.shape, &mut drawn);
            }
            out.textures_delta.clear();
        }
        drawn
    }

    fn x_of(drawn: &[(String, egui::Pos2)], text: &str) -> f32 {
        let hits: Vec<f32> = drawn.iter().filter(|(t, _)| t == text).map(|(_, p)| p.x).collect();
        assert!(!hits.is_empty(), "{text:?} was not drawn; drawn: {:?}", drawn.iter().map(|(t, _)| t).collect::<Vec<_>>());
        hits.into_iter().fold(f32::NAN, f32::min)
    }

    fn app_in(code: &str) -> (egui::Context, WordApp) {
        let ctx = egui::Context::default();
        let mut app = WordApp::new(wordcraft_engine::Session::new(wordcraft_engine::sample::sample_document()), Default::default());
        app.run("ui.language", json!({"value": code})).unwrap();
        let _ = app.run("ui.tab", json!({"tab": "Home"}));
        (ctx, app)
    }

    /// Arabic interface: the tab strip starts at the right (Home right of Insert) and the ribbon's
    /// groups run right-to-left (Clipboard right of Font), as in Word's Arabic interface.
    #[test]
    fn arabic_interface_is_laid_out_right_to_left() {
        let (ctx, mut app) = app_in("ar");
        let drawn = drawn_texts(&ctx, &mut app);
        assert!(super::on(), "the interface language is Arabic while the app is drawn");
        let (home, insert) = (x_of(&drawn, "الرئيسية"), x_of(&drawn, "إدراج"));
        assert!(home > insert, "Home tab at x={home} must be right of Insert at x={insert}");
        let (clipboard, font) = (x_of(&drawn, "الحافظة"), x_of(&drawn, "الخط"));
        assert!(clipboard > font, "Clipboard group label at x={clipboard} must be right of Font at x={font}");
        // The first group starts at the ribbon's right edge: well into the right half of the window.
        assert!(clipboard > 2560.0 / 2.0, "Clipboard group label at x={clipboard} sits at the far right");
        set_current(Lang::EN);
        assert!(!super::on());
    }

    /// English (and every other language): left-to-right, unchanged.
    #[test]
    fn english_interface_stays_left_to_right() {
        let (ctx, mut app) = app_in("en");
        let drawn = drawn_texts(&ctx, &mut app);
        assert!(!super::on());
        let (home, insert) = (x_of(&drawn, "Home"), x_of(&drawn, "Insert"));
        assert!(home < insert, "Home tab at x={home} must be left of Insert at x={insert}");
        let (clipboard, font) = (x_of(&drawn, "Clipboard"), x_of(&drawn, "Font"));
        assert!(clipboard < font, "Clipboard group label at x={clipboard} must be left of Font at x={font}");
        assert!(clipboard < 2560.0 / 2.0, "Clipboard group label at x={clipboard} sits at the far left");
        set_current(Lang::EN);
    }

    /// The helpers are egui's own layouts in a left-to-right interface and their mirror images
    /// in Arabic.
    #[test]
    fn helpers_follow_the_interface_language() {
        set_current(Lang::EN);
        assert_eq!(super::row_top(), egui::Layout::left_to_right(egui::Align::Min));
        assert_eq!(super::row_from_end(egui::Align::Center), egui::Layout::right_to_left(egui::Align::Center));
        assert_eq!(super::column(), egui::Layout::top_down(egui::Align::Min));
        assert_eq!(super::align_start(), egui::Align::LEFT);
        let within = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(100.0, 10.0));
        let r = egui::Rect::from_min_max(egui::pos2(10.0, 0.0), egui::pos2(30.0, 10.0));
        assert_eq!(super::mirror_x(r, within), r);
        assert_eq!(super::start_x(r, 4.0), 14.0);
        assert_eq!(super::end_x(r, 4.0), 26.0);
        set_current(Lang::from_code("ar").unwrap());
        assert!(super::on());
        assert_eq!(super::row_top(), egui::Layout::right_to_left(egui::Align::Min));
        assert_eq!(super::row_from_end(egui::Align::Center), egui::Layout::left_to_right(egui::Align::Center));
        assert_eq!(super::column(), egui::Layout::top_down(egui::Align::Max));
        assert_eq!(super::align_start(), egui::Align::RIGHT);
        assert_eq!(super::start_center(), egui::Align2::RIGHT_CENTER);
        assert_eq!(super::mirror_x(r, within), egui::Rect::from_min_max(egui::pos2(70.0, 0.0), egui::pos2(90.0, 10.0)));
        assert_eq!(super::start_x(r, 4.0), 26.0);
        assert_eq!(super::end_x(r, 4.0), 14.0);
        set_current(Lang::EN);
    }
}
