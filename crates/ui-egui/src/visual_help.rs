//! WordCraft AR — visual help for ribbon buttons: a thumbnail of what a command does and a
//! one-sentence explanation, shown in the button's tooltip ([`crate::widgets`]). The content
//! lives in the UI kit ([`crate::ui_kit`]): one registry of command → icon → thumbnail →
//! explanation, a file per tab. This module is the thin face the tooltips call.

use egui::{Painter, Rect};
use serde_json::Value;

use crate::theme::Tokens;
use crate::ui_kit;

/// Thumbnail size, in points.
pub const SIZE: egui::Vec2 = ui_kit::SIZE;

/// The visual-help key of a command and its parameters ([`ui_kit::key`]).
pub fn key(id: &str, params: &Value) -> String {
    ui_kit::key(id, params)
}

/// What a command does, in one plain sentence (English source; the catalogs translate it).
pub fn explanation(key: &str) -> Option<&'static str> {
    ui_kit::entry(key).map(|e| e.explain)
}

/// Draw the thumbnail for `key` into `r`; false when there is none (the tooltip then shows text
/// only).
pub fn thumbnail(p: &Painter, r: Rect, key: &str, t: &Tokens) -> bool {
    match ui_kit::entry(key) {
        Some(e) => {
            (e.draw)(&ui_kit::Draw::new(p, r, t));
            true
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn every_explained_command_has_a_thumbnail_and_is_registered() {
        let reg = wordcraft_engine::cmd::registry();
        let ctx = egui::Context::default();
        ctx.set_fonts(crate::theme::font_definitions(false, false));
        let mut out = ctx.run_ui(Default::default(), |ui| {
            let t = Tokens::get(ui.ctx());
            for e in ui_kit::entries() {
                let (key, text) = (e.key, e.explain);
                let id = key.split(':').next().unwrap_or("");
                assert!(reg.get(id).is_some() || id.starts_with("ui."), "{key}: unknown command");
                assert!(!text.trim().is_empty() && text.ends_with('.'), "{key}: one full sentence");
                let r = Rect::from_min_size(Pos2::ZERO, SIZE);
                assert!(thumbnail(ui.painter(), r, key, &t), "{key}: no thumbnail");
                assert_eq!(explanation(key), Some(text));
            }
            assert!(!thumbnail(ui.painter(), Rect::from_min_size(Pos2::ZERO, SIZE), "no.such", &Tokens::get(ui.ctx())));
        });
        out.textures_delta.clear();
    }

    #[test]
    fn keys_pick_the_variant() {
        assert_eq!(key("service.kashida", &serde_json::json!({"value": "low"})), "service.kashida:low");
        assert_eq!(key("service.renumber", &serde_json::json!({"scope": "selection"})), "service.renumber:selection");
        assert_eq!(key("service.renumber", &serde_json::json!({})), "service.renumber");
        assert_eq!(key("service.external", &serde_json::json!({"pdf": true})), "service.external:pdf");
        assert_eq!(key("service.external", &serde_json::json!({"pdf": false})), "service.external");
        assert_eq!(key("mailings.next", &serde_json::json!({})), "mailings.next");
    }

    /// "Check every button": every control of every ribbon tab runs on an Arabic service
    /// document without a panic (an error is a valid answer: a dialog needs input, a feature
    /// needs a selection), the document still lays out, and the Service buttons do their job.
    #[test]
    fn every_ribbon_button_runs_on_an_arabic_document() {
        let mut app = crate::WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        app.run("file.new", serde_json::json!({})).unwrap();
        for line in [
            "١. شعبة الأسنان",
            "أ. المهمة",
            "تقوم شعبة الاسنان بتقديم الخدمات الطبية لمنتسبي وزارة الدفاع.",
            "ب. الواجبات",
            "أولاً. فحص ومعالجة مرضى الفم والأسنان",
            "ثانياً. الإشراف على إدامة الأجهزة",
        ] {
            app.run("text.insert", serde_json::json!({"text": line})).unwrap();
            app.run("text.newParagraph", serde_json::json!({})).unwrap();
        }
        let mut ran = 0;
        for tab in crate::ribbon::TABS.iter().chain(&["Table Design", "Table Layout"]) {
            for (id, params) in crate::keytips::controls_public_with_params(tab) {
                if id == "ui.dialog" || id == "ui.tab" || id.starts_with("file.") || id == "ui.zotero.addEditCitation" {
                    continue;
                }
                let _ = app.run(id, params);
                app.session.clamp_selection();
                let _ = app.session.layout();
                ran += 1;
            }
        }
        assert!(ran > 150, "{ran} controls ran");
        // The Service tab's work is in the document: the cycle, a dot on every item, the stamp.
        app.run("select.all", serde_json::json!({})).unwrap();
        app.run("service.renumber", serde_json::json!({})).unwrap();
        app.run("service.finalDot", serde_json::json!({})).unwrap();
        app.run("service.classification", serde_json::json!({"value": "محدود"})).unwrap();
        app.run("service.pageNumbers", serde_json::json!({})).unwrap();
        let text = app.session.doc.plain_text(wordcraft_doc::StoryRef::Body);
        assert!(text.contains("أولاً.") && text.contains("فحص ومعالجة مرضى الفم والأسنان."), "{text}");
        assert!(!app.session.layout().pages.is_empty());
    }

    /// Every Service and Mailings ribbon control has visual help.
    #[test]
    fn service_and_mailings_buttons_are_all_explained() {
        for tab in ["Service", "Mailings"] {
            for c in crate::keytips::controls_public_with_params(tab) {
                if c.0 == "ui.dialog" || c.0 == "ui.tab" {
                    continue;
                }
                let k = key(c.0, &c.1);
                assert!(explanation(&k).is_some(), "{tab} › {}: no visual help ({k})", c.0);
            }
        }
    }
}
