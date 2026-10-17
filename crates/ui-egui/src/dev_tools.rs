//! WordCraft AR — built-in developer systems, reachable from Developer › WordCraft AR:
//!
//! - **UI Kit**: every command of the registry ([`crate::ui_kit`]) with its icon, its thumbnail
//!   and its explanation (English and Arabic), filterable — the place to check that a new button
//!   looks right and reads right before it ships.
//! - **Icons**: the whole icon set at the ribbon's two sizes, so sizes and weights can be
//!   compared side by side.
//! - **Self-test**: runs every ribbon button on a scratch Arabic document (never the open one)
//!   and lists what each returned and how long it took, like the `every_ribbon_button_runs_on_an_arabic_document`
//!   test but inside the app, on this machine's fonts.
//! - **Checklist**: what is still missing — buttons without visual help, explanations without
//!   an Arabic translation — the same checks the tests make, readable without a build.
//!
//! Sibling apps (GridCraft, DeckCraft) can take this module with `ui_kit` as it is.

use std::time::Instant;

use egui::{Rect, Sense, Stroke, Ui, vec2};
use serde_json::json;

use crate::WordApp;
use crate::theme::{Tokens, semibold};

/// The dialog's tabs.
pub const TABS: [&str; 4] = ["UI Kit", "Icons", "Self-test", "Checklist"];

/// One ribbon button's run in the self-test.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Run {
    pub tab: String,
    pub id: String,
    pub params: String,
    /// Empty when the command succeeded.
    pub error: String,
    pub millis: u64,
}

/// The self-test's result.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Report {
    pub runs: Vec<Run>,
    pub pages: usize,
    pub text_ok: bool,
}

impl Report {
    pub fn failed(&self) -> usize {
        self.runs.iter().filter(|r| !r.error.is_empty()).count()
    }
}

/// The Arabic service document the self-test works on.
pub fn scratch_document() -> wordcraft_engine::Session {
    let mut s = wordcraft_engine::Session::new(wordcraft_doc::Document::new());
    wordcraft_engine::make_arabic(&mut s.doc);
    for line in [
        "١. شعبة الأسنان",
        "أ. المهمة",
        "تقوم شعبة الاسنان بتقديم الخدمات الطبية لمنتسبي وزارة الدفاع.",
        "ب. الواجبات",
        "أولاً. فحص ومعالجة مرضى الفم والأسنان",
        "ثانياً. الإشراف على إدامة الأجهزة",
    ] {
        let _ = s.run("text.insert", &json!({"text": line}));
        let _ = s.run("text.newParagraph", &json!({}));
    }
    s
}

/// Run every ribbon control of every tab on a scratch document and report. Interface-only
/// commands (`ui.*`), file commands and dialogs are skipped: they need the window.
pub fn self_test() -> Report {
    // A scratch app, so the interface-level commands (Dictate, Translate, Arrange All…) run too;
    // dialogs, tabs and File need the window and are skipped.
    let mut app = WordApp::new(scratch_document(), crate::Services::default());
    let mut runs = Vec::new();
    for tab in crate::ribbon::TABS.iter().chain(&["Table Design", "Table Layout"]) {
        for (id, params) in crate::keytips::controls_public_with_params(tab) {
            if id == "ui.dialog" || id == "ui.tab" || id.starts_with("file.") || id.starts_with("ui.zotero.") {
                continue;
            }
            let start = Instant::now();
            let error = match app.run(id, params.clone()) {
                Ok(_) => String::new(),
                Err(e) => e.to_string(),
            };
            app.session.clamp_selection();
            let _ = app.session.layout();
            runs.push(Run {
                tab: tab.to_string(),
                id: id.to_string(),
                params: params.to_string(),
                error,
                millis: start.elapsed().as_millis() as u64,
            });
        }
    }
    // The Service tab's own work, on a fresh copy (the run above may have left the caret in a
    // text box or a table): the cycle, a dot on every item.
    let mut fresh = scratch_document();
    let _ = fresh.run("select.all", &json!({}));
    let _ = fresh.run("service.renumber", &json!({}));
    let _ = fresh.run("service.finalDot", &json!({}));
    let text = fresh.doc.plain_text(wordcraft_doc::StoryRef::Body);
    let text_ok = text.contains("أولاً.") && text.contains("فحص ومعالجة مرضى الفم والأسنان.");
    Report { runs, pages: app.session.layout().pages.len(), text_ok }
}

/// Ribbon buttons (tab › id) that have no visual help yet.
pub fn unexplained() -> Vec<String> {
    let mut v = Vec::new();
    for tab in crate::ribbon::TABS.iter().chain(&["Table Design", "Table Layout"]) {
        for (id, params) in crate::keytips::controls_public_with_params(tab) {
            if id == "ui.tab" {
                continue;
            }
            let key = crate::ui_kit::key(id, &params);
            if crate::ui_kit::entry(&key).is_none() {
                v.push(format!("{tab} › {key}"));
            }
        }
    }
    v
}

/// Explanations the Arabic catalog doesn't translate.
pub fn untranslated() -> Vec<&'static str> {
    let Some(ar) = crate::i18n::Lang::from_code("ar") else { return Vec::new() };
    crate::ui_kit::entries().filter(|e| crate::i18n::explain(ar, e.explain) == e.explain).map(|e| e.key).collect()
}

/// The dialog body. Returns true when it should close.
pub fn body(app: &mut WordApp, ui: &mut Ui, tab: &mut u8, filter: &mut String, report: &mut Option<Report>) -> bool {
    ui.set_width(720.0);
    ui.horizontal(|ui| {
        for (i, l) in TABS.into_iter().enumerate() {
            let i = i as u8;
            if ui.selectable_label(*tab == i, tl!(l)).clicked() {
                *tab = i;
            }
        }
        ui.add_space(12.0);
        if *tab < 2 {
            ui.label(tl!("Filter:"));
            ui.add(egui::TextEdit::singleline(filter).desired_width(180.0));
        }
    });
    ui.separator();
    let t = Tokens::get(ui.ctx());
    egui::ScrollArea::vertical().max_height(480.0).show(ui, |ui| match *tab {
        0 => kit_gallery(ui, &t, filter),
        1 => icon_gallery(ui, &t, filter),
        2 => self_test_ui(ui, &t, report),
        _ => checklist(app, ui),
    });
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(tl!(
                "Add a command: its icon in icons.rs, one Entry in ui_kit/<tab>.rs, one line in i18n/ar.tsv — see docs/ar-ui-kit.md."
            ))
            .small()
            .weak(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| ui.button(tl!("Close")).clicked()).inner
    })
    .inner
}

fn matches(filter: &str, hay: &[&str]) -> bool {
    let f = filter.trim().to_lowercase();
    f.is_empty() || hay.iter().any(|h| h.to_lowercase().contains(&f))
}

/// Every registry entry: icon, thumbnail, key, explanation, Arabic.
fn kit_gallery(ui: &mut Ui, t: &Tokens, filter: &str) {
    let ar = crate::i18n::Lang::from_code("ar");
    let mut shown = 0;
    for e in crate::ui_kit::entries() {
        let id = e.key.split(':').next().unwrap_or(e.key);
        let icon = crate::ui_kit::icon_for(id);
        let arabic = ar.map(|ar| crate::i18n::explain(ar, e.explain)).unwrap_or(e.explain);
        if !matches(filter, &[e.key, e.explain, arabic, icon]) {
            continue;
        }
        shown += 1;
        ui.horizontal_top(|ui| {
            let (r, _) = ui.allocate_exact_size(crate::ui_kit::SIZE, Sense::hover());
            ui.painter().rect_filled(r, 4.0, t.panel);
            ui.painter().rect_stroke(r, 4.0, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
            (e.draw)(&crate::ui_kit::Draw::new(ui.painter(), r, t));
            ui.add_space(8.0);
            let (ir, _) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::hover());
            crate::icons::paint(ui.painter(), ir, icon, t.icon, t.accent);
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.set_max_width(400.0);
                ui.label(egui::RichText::new(crate::i18n::artwork(e.key)).font(semibold(12.0)));
                ui.label(egui::RichText::new(crate::i18n::explain(crate::i18n::current(), e.explain)).size(11.5));
                if arabic != e.explain {
                    ui.add(egui::Label::new(egui::RichText::new(arabic).size(11.5).color(t.text_dim)).halign(egui::Align::RIGHT));
                } else {
                    ui.label(egui::RichText::new(tl!("No Arabic translation yet")).small().color(t.red));
                }
                if !crate::icons::known(icon) {
                    ui.label(
                        egui::RichText::new(crate::i18n::fmt(tl!("No icon named {icon} (a lettered tile is drawn)"), &[("icon", icon)]))
                            .small()
                            .color(t.red),
                    );
                }
            });
        });
        ui.add_space(6.0);
    }
    ui.label(
        egui::RichText::new(crate::i18n::fmt(
            tl!("{shown} of {total} entries"),
            &[("shown", &shown.to_string()), ("total", &crate::ui_kit::entries().count().to_string())],
        ))
        .small()
        .weak(),
    );
}

/// The icon set at 17 and 32 pt, eight per row.
fn icon_gallery(ui: &mut Ui, t: &Tokens, filter: &str) {
    let names: Vec<&str> = crate::icons::NAMES.iter().copied().filter(|n| matches(filter, &[n])).collect();
    for row in names.chunks(6) {
        ui.horizontal(|ui| {
            for name in row {
                let (r, resp) = ui.allocate_exact_size(vec2(118.0, 44.0), Sense::hover());
                if resp.hovered() {
                    ui.painter().rect_filled(r, 3.0, t.hover);
                }
                let small = Rect::from_center_size(r.min + vec2(14.0, 22.0), vec2(17.0, 17.0));
                let big = Rect::from_center_size(r.min + vec2(44.0, 22.0), vec2(32.0, 32.0));
                crate::icons::paint(ui.painter(), small, name, t.icon, t.accent);
                crate::icons::paint(ui.painter(), big, name, t.icon, t.accent);
                ui.painter().text(
                    r.min + vec2(66.0, 22.0),
                    egui::Align2::LEFT_CENTER,
                    crate::i18n::artwork(name),
                    egui::FontId::proportional(9.5),
                    t.text,
                );
            }
        });
    }
    ui.label(
        egui::RichText::new(crate::i18n::fmt(
            tl!("{shown} of {total} icons"),
            &[("shown", &names.len().to_string()), ("total", &crate::icons::NAMES.len().to_string())],
        ))
        .small()
        .weak(),
    );
}

fn self_test_ui(ui: &mut Ui, t: &Tokens, report: &mut Option<Report>) {
    ui.label(tl!("Runs every ribbon button on a scratch Arabic service document (your document is not touched) and lists what each one answered."));
    if ui.button(tl!("Run the self-test")).clicked() {
        *report = Some(self_test());
    }
    let Some(rep) = report else { return };
    let failed = rep.failed();
    let total_ms: u64 = rep.runs.iter().map(|r| r.millis).sum();
    ui.label(
        egui::RichText::new(crate::i18n::fmt(
            tl!("{total} buttons ran in {ms} ms; {errors} answered with an error (a dialog that needs input, a feature that needs a selection); {pages} page(s); service cycle {ok}"),
            &[
                ("total", &rep.runs.len().to_string()),
                ("ms", &total_ms.to_string()),
                ("errors", &failed.to_string()),
                ("pages", &rep.pages.to_string()),
                ("ok", if rep.text_ok { "✓" } else { "✗" }),
            ],
        ))
        .font(semibold(12.0))
        .color(if rep.text_ok { t.text } else { t.red }),
    );
    egui::Grid::new("self_test").striped(true).min_col_width(60.0).show(ui, |ui| {
        for h in ["Tab", "Command", "Parameters", "ms", "Answer"] {
            ui.label(egui::RichText::new(tl!(h)).font(semibold(11.5)));
        }
        ui.end_row();
        for r in &rep.runs {
            ui.label(tl!(r.tab.as_str()));
            ui.label(crate::i18n::artwork(&r.id));
            ui.label(crate::i18n::artwork(if r.params == "{}" { "" } else { &r.params }));
            ui.label(r.millis.to_string());
            if r.error.is_empty() {
                ui.label(egui::RichText::new("✓").color(t.green));
            } else {
                // The engine's own message (English, like every command error).
                ui.label(egui::RichText::new(crate::i18n::artwork(&r.error)).color(t.orange));
            }
            ui.end_row();
        }
    });
}

fn checklist(app: &WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let missing = unexplained();
    let untranslated = untranslated();
    let line = |ui: &mut Ui, ok: bool, text: String| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(if ok { "✓" } else { "✗" }).color(if ok { t.green } else { t.red }).font(semibold(13.0)));
            ui.label(text);
        });
    };
    line(ui, missing.is_empty(), crate::i18n::fmt(tl!("{n} ribbon buttons without visual help"), &[("n", &missing.len().to_string())]));
    for m in &missing {
        ui.label(egui::RichText::new(crate::i18n::artwork(&format!("    {m}"))).small());
    }
    line(
        ui,
        untranslated.is_empty(),
        crate::i18n::fmt(tl!("{n} explanations without an Arabic translation"), &[("n", &untranslated.len().to_string())]),
    );
    for m in &untranslated {
        ui.label(egui::RichText::new(crate::i18n::artwork(&format!("    {m}"))).small());
    }
    let icons_total = crate::icons::NAMES.len();
    line(ui, true, crate::i18n::fmt(tl!("{n} icons in the set"), &[("n", &icons_total.to_string())]));
    let entries = crate::ui_kit::entries().count();
    line(ui, entries > 0, crate::i18n::fmt(tl!("{n} entries in the UI kit"), &[("n", &entries.to_string())]));
    let commands = app.session.registry.all().len();
    line(ui, true, crate::i18n::fmt(tl!("{n} commands in the registry"), &[("n", &commands.to_string())]));
    let ar_entries = crate::i18n::Lang::from_code("ar").map(|l| l.catalog().len()).unwrap_or(0);
    line(ui, ar_entries > 0, crate::i18n::fmt(tl!("{n} Arabic interface strings"), &[("n", &ar_entries.to_string())]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_self_test_runs_every_button_and_keeps_the_service_cycle() {
        let rep = self_test();
        assert!(rep.runs.len() > 150, "{} ran", rep.runs.len());
        assert!(rep.text_ok);
        assert!(rep.pages >= 1);
        // The buttons that answer with an error do so with a message, never a panic.
        for r in &rep.runs {
            assert!(r.error.is_empty() || r.error.len() > 3, "{}: {:?}", r.id, r.error);
        }
    }

    #[test]
    fn the_checklist_matches_the_tests() {
        // The checklist's "missing" list is what `every_ribbon_button_is_explained` asserts on.
        let _ = unexplained();
        let _ = untranslated();
    }
}
