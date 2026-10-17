//! WordCraft AR — a contact sheet of the UI kit: every thumbnail with its key and icon, or the
//! whole icon set, rendered offscreen (wgpu) to a PNG for review.
//!
//! `cargo run -p wordcraft-ui-egui --example kit_sheet -- thumbs out.png [filter]`
//! `cargo run -p wordcraft-ui-egui --example kit_sheet -- icons out.png`

use egui::{Rect, Sense, Stroke, vec2};
use wordcraft_ui_egui::theme::Tokens;
use wordcraft_ui_egui::{icons, ui_kit};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let what = args.get(1).cloned().unwrap_or_else(|| "thumbs".into());
    let out = args.get(2).cloned().unwrap_or_else(|| "kit.png".into());
    let filter = args.get(3).cloned().unwrap_or_default();
    let cols = 10usize;
    let cell = vec2(ui_kit::SIZE.x + 12.0, ui_kit::SIZE.y + 30.0);
    let n =
        if what == "icons" { icons::NAMES.len() } else { ui_kit::entries().filter(|e| filter.is_empty() || e.key.contains(filter.as_str())).count() };
    let (w, h) = if what == "icons" {
        (8.0 * 120.0 + 16.0, (n as f32 / 8.0).ceil() * 48.0 + 16.0)
    } else {
        (cols as f32 * cell.x + 16.0, (n as f32 / cols as f32).ceil() * cell.y + 16.0)
    };
    let mut frame = 0u32;
    let mut harness = egui_kittest::Harness::builder().with_size(vec2(w, h)).with_pixels_per_point(2.0).wgpu().build_ui(move |ui| {
        frame += 1;
        if frame == 1 {
            ui.ctx().set_fonts(wordcraft_ui_egui::theme::font_definitions(false, false));
            ui.ctx().set_theme(egui::Theme::Light);
            return;
        }
        let t = Tokens::get(ui.ctx());
        ui.painter().rect_filled(ui.max_rect(), 0.0, t.menu);
        if what == "icons" {
            for (i, name) in icons::NAMES.iter().enumerate() {
                let (col, row) = (i % 8, i / 8);
                let r = Rect::from_min_size(ui.max_rect().min + vec2(8.0 + col as f32 * 120.0, 8.0 + row as f32 * 48.0), vec2(118.0, 44.0));
                let small = Rect::from_center_size(r.min + vec2(14.0, 22.0), vec2(17.0, 17.0));
                let big = Rect::from_center_size(r.min + vec2(44.0, 22.0), vec2(32.0, 32.0));
                icons::paint(ui.painter(), small, name, t.icon, t.accent);
                icons::paint(ui.painter(), big, name, t.icon, t.accent);
                ui.painter().text(r.min + vec2(66.0, 22.0), egui::Align2::LEFT_CENTER, *name, egui::FontId::proportional(9.0), t.text);
            }
            return;
        }
        for (i, e) in ui_kit::entries().filter(|e| filter.is_empty() || e.key.contains(filter.as_str())).enumerate() {
            let (col, row) = (i % cols, i / cols);
            let origin = ui.max_rect().min + vec2(8.0 + col as f32 * cell.x, 8.0 + row as f32 * cell.y);
            let r = Rect::from_min_size(origin, ui_kit::SIZE);
            ui.painter().rect_filled(r, 4.0, t.panel);
            ui.painter().rect_stroke(r, 4.0, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
            (e.draw)(&ui_kit::Draw::new(ui.painter(), r, &t));
            let ic = Rect::from_min_size(origin + vec2(0.0, ui_kit::SIZE.y + 4.0), vec2(20.0, 20.0));
            icons::paint(ui.painter(), ic, ui_kit::icon_for(e.key), t.icon, t.accent);
            ui.painter().text(origin + vec2(24.0, ui_kit::SIZE.y + 14.0), egui::Align2::LEFT_CENTER, e.key, egui::FontId::proportional(9.5), t.text);
            let _ = ui.allocate_rect(r, Sense::hover());
        }
    });
    harness.input_mut().max_texture_side = Some(8192);
    for _ in 0..3 {
        harness.step();
    }
    match harness.render() {
        Ok(img) => {
            img.save(&out).expect("save png");
            println!("{out}: {n} items");
        }
        Err(e) => eprintln!("render failed: {e}"),
    }
}
