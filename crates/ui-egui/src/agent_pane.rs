//! WordCraft AR: the Assistant pane (Service tab › Assistant). Say what you want — «احذف الترقيم
//! الرئيسي», «شيل الأرقام», «اجعله غامق ووسّطه», "renumber" — and it is done, at once and in order:
//!
//! * as you type, a line under the log shows what the words became (`agent.interpret`), so you
//!   see what Enter will do before you press it;
//! * Enter (or «Do it») runs it (`agent.ask`) — each command is its own undo step;
//! * when it is not sure it asks «هل تقصد …؟» with Yes / No; a «نعم» teaches it, and the same
//!   words run at once next time (the memory is saved in the settings folder);
//! * a request it did not understand comes back with the nearest commands, one click each — the
//!   click teaches it too; after a request the likely next steps are offered as chips.
//!
//! There is no model and no waiting: the engine reads the request with local rules and a phrase
//! index in microseconds. The pane draws; the engine decides.

use egui::{Color32, Id, Key, Sense, Ui, vec2};
use serde_json::{Value, json};
use wordcraft_engine::cmd::agent::{ASK, Plan, SURE, interpret, memory_path};

use crate::WordApp;
use crate::theme::{Tokens, regular, semibold};

/// Requests that work as they are, shown until the first one is sent.
const EXAMPLES: [&str; 5] = ["شيل الترقيم الرئيسي", "الترقيم خربان", "خلي الكلام ثخين وبالنص", "درجة الكتمان سري", "وريني فهرس الترقيم"];

/// A command with its parameters, as the engine returned it.
#[derive(Clone)]
struct Step {
    id: String,
    params: Value,
    label: String,
}

/// One request and what came of it.
#[derive(Clone, Default)]
struct Turn {
    say: String,
    /// (command name, ok, error)
    done: Vec<(String, bool, String)>,
    /// The nearest commands when nothing was understood.
    suggestions: Vec<Step>,
    /// «هل تقصد …؟»: the first option's steps (the engine holds the rest).
    asking: Vec<Step>,
    next: Vec<Step>,
    learned: bool,
}

#[derive(Clone, Default)]
struct AgentState {
    input: String,
    log: Vec<Turn>,
    /// The best reading of `input`, kept while neither the words nor the document changed.
    plans: Option<(String, u64, Vec<Plan>)>,
    focus: bool,
    loaded: bool,
}

/// What a click in the log asks for.
enum Action {
    /// Send these words to the engine; show `shown` in the log.
    Ask { shown: String, text: String },
    /// Run a command; teach `teach` (the words that led here) when given.
    Run { step: Step, teach: Option<String> },
}

pub fn pane(app: &mut WordApp, ui: &mut Ui) {
    let sid = Id::new("agent_state");
    let mut st: AgentState = ui.data(|d| d.get_temp(sid)).unwrap_or_else(|| AgentState { focus: true, ..Default::default() });
    if crate::panes::header(ui, "Assistant") {
        let _ = app.run("view.agentPane", json!({"value": false}));
        return;
    }
    if !st.loaded {
        st.loaded = true;
        if let Some(path) = memory_path()
            && path.exists()
        {
            let _ = app.session.run("agent.load", &json!({"path": path.to_string_lossy()}));
        }
    }
    let t = Tokens::get(ui.ctx());
    let width = ui.available_width();
    // What the words became so far (shown above the box, so the box doesn't move as it changes).
    let rev = app.session.rev();
    let plans: Vec<Plan> = match &st.plans {
        Some((text, r, p)) if *text == st.input && *r == rev => p.clone(),
        _ => {
            let p = if st.input.trim().is_empty() { Vec::new() } else { interpret(&app.session, &st.input) };
            st.plans = Some((st.input.clone(), rev, p.clone()));
            p
        }
    };
    let best = plans.first().map(|p| p.score).unwrap_or(0.0);
    let shown: Vec<&Plan> = if best >= ASK { plans.iter().take(4).collect() } else { Vec::new() };

    // ---- the log
    let input_h = 30.0 + 22.0 + 20.0 * (shown.len() as f32) + 12.0;
    let log_h = (ui.available_height() - input_h - 10.0).max(60.0);
    let mut action: Option<Action> = None;
    let last = st.log.len().saturating_sub(1);
    egui::ScrollArea::vertical().max_height(log_h).auto_shrink([false, false]).stick_to_bottom(true).show(ui, |ui| {
        ui.set_width(width);
        if st.log.is_empty() {
            ui.add_space(6.0);
            crate::rtl_text::paragraph(
                ui,
                tl!("Tell the assistant what to do, in Arabic or English, and it is done at once. Several wishes in one sentence are done in order."),
                regular(12.5),
                t.text_dim,
                width - 4.0,
            );
            ui.add_space(8.0);
            ui.label(egui::RichText::new(tl!("Examples")).font(semibold(12.0)).color(t.text_dim));
            for ex in EXAMPLES {
                if chip(ui, &t, ex, width - 4.0).clicked() {
                    action = Some(Action::Ask { shown: ex.to_string(), text: ex.to_string() });
                }
            }
        }
        for (i, turn) in st.log.iter().enumerate() {
            ui.add_space(6.0);
            crate::rtl_text::paragraph(ui, &turn.say, semibold(13.0), t.text, width - 6.0);
            for (name, ok, err) in &turn.done {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(10.0, 14.0), Sense::hover());
                    let c = if *ok { Color32::from_rgb(40, 150, 80) } else { Color32::from_rgb(200, 60, 60) };
                    ui.painter().circle_filled(r.center(), 3.5, c);
                    let text = if *ok || err.is_empty() { name.clone() } else { format!("{name}: {err}") };
                    ui.label(egui::RichText::new(text).font(regular(12.5)).color(if *ok { t.text } else { c }));
                });
            }
            if turn.learned {
                ui.label(egui::RichText::new(tl!("Remembered for next time.")).font(regular(11.5)).color(t.text_dim));
            }
            if !turn.asking.is_empty() {
                crate::rtl_text::paragraph(ui, tl!("Did you mean:"), semibold(12.5), t.text, width - 6.0);
                for s in &turn.asking {
                    crate::rtl_text::paragraph(ui, &format!("• {}", s.label), regular(12.5), t.accent_text, width - 6.0);
                }
                // only the latest question is still open in the engine
                if i == last {
                    ui.horizontal(|ui| {
                        if ui.button(tl!("Yes")).clicked() {
                            action = Some(Action::Ask { shown: tl!("Yes").to_string(), text: "نعم".into() });
                        }
                        if ui.button(tl!("No")).clicked() {
                            action = Some(Action::Ask { shown: tl!("No").to_string(), text: "لا".into() });
                        }
                    });
                }
            }
            if !turn.suggestions.is_empty() {
                crate::rtl_text::paragraph(ui, tl!("I did not understand that. The nearest commands:"), regular(12.0), t.text_dim, width - 6.0);
                for s in &turn.suggestions {
                    if chip(ui, &t, &s.label, width - 4.0).clicked() {
                        action = Some(Action::Run { step: s.clone(), teach: Some(turn.say.clone()) });
                    }
                }
            }
            if i == last && !turn.next.is_empty() {
                ui.label(egui::RichText::new(tl!("Next:")).font(semibold(12.0)).color(t.text_dim));
                for s in &turn.next {
                    if chip(ui, &t, &s.label, width - 4.0).clicked() {
                        action = Some(Action::Run { step: s.clone(), teach: None });
                    }
                }
            }
        }
    });

    // ---- what Enter will do
    ui.add_space(4.0);
    if !shown.is_empty() {
        let head = if best >= SURE { tl!("It will do:") } else { tl!("Did you mean:") };
        ui.label(egui::RichText::new(head).font(semibold(12.0)).color(t.text_dim));
        for p in &shown {
            crate::rtl_text::paragraph(ui, &format!("• {}", crate::i18n::t(&p.label)), regular(12.5), t.accent_text, width - 6.0);
        }
    }

    // ---- the box
    ui.horizontal(|ui| {
        let w = (ui.available_width() - 62.0).max(80.0);
        let box_id = Id::new("agent_input");
        // Enter is read before the box sees it: the box gives up its focus on it.
        let enter = ui.memory(|m| m.has_focus(box_id)) && ui.input(|i| i.key_pressed(Key::Enter));
        let edit = egui::TextEdit::singleline(&mut st.input).hint_text(tl!("Tell the assistant what to do…")).desired_width(w).id(box_id);
        let resp = ui.add(edit);
        if st.focus {
            resp.request_focus();
            st.focus = false;
        }
        let go = ui.add_enabled(!st.input.trim().is_empty(), egui::Button::new(tl!("Do it"))).clicked();
        if (enter || go) && !st.input.trim().is_empty() {
            let say = std::mem::take(&mut st.input);
            action = Some(Action::Ask { shown: say.clone(), text: say });
            st.focus = true;
        }
    });

    if let Some(action) = action {
        let (say, reply, teach) = match action {
            Action::Ask { shown, text } => {
                let r = app.run("agent.ask", json!({"text": text}));
                (shown, r, None)
            }
            Action::Run { step, teach } => {
                let r = app.run(&step.id, step.params.clone());
                let name = step.label.clone();
                let reply = r.map(|_| json!({"done": [{"label": name, "ok": true}]}));
                (name, reply, teach.map(|text| (text, step)))
            }
        };
        let mut turn = turn(&say, reply);
        if let Some((text, step)) = teach
            && turn.done.iter().all(|d| d.1)
        {
            let _ = app.session.run("agent.teach", &json!({"text": text, "command": step.id, "params": step.params}));
            turn.learned = true;
        }
        if turn.learned
            && let Some(path) = memory_path()
        {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = app.session.run("agent.save", &json!({"path": path.to_string_lossy()}));
        }
        st.log.push(turn);
        if st.log.len() > 60 {
            st.log.remove(0);
        }
        st.plans = None;
    }
    ui.data_mut(|d| d.insert_temp(sid, st));
}

fn steps(v: Option<&Value>) -> Vec<Step> {
    v.and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|p| {
            let id = p.get("command").and_then(Value::as_str)?.to_string();
            let label = crate::i18n::t(p.get("label").and_then(Value::as_str).unwrap_or(&id)).to_string();
            Some(Step { id, params: p.get("params").cloned().unwrap_or_else(|| json!({})), label })
        })
        .collect()
}

/// A request and the engine's reply as a log entry.
fn turn(say: &str, reply: Result<Value, String>) -> Turn {
    let mut t = Turn { say: say.to_string(), ..Default::default() };
    match reply {
        Err(e) => t.done.push((String::new(), false, e)),
        Ok(v) => {
            for d in v.get("done").and_then(Value::as_array).into_iter().flatten() {
                let name = d.get("label").and_then(Value::as_str).unwrap_or("");
                let ok = d.get("ok").and_then(Value::as_bool).unwrap_or(false);
                let err = d.get("error").and_then(Value::as_str).unwrap_or("");
                t.done.push((crate::i18n::t(name).to_string(), ok, err.to_string()));
            }
            t.suggestions = steps(v.get("suggestions"));
            t.next = steps(v.get("next"));
            t.learned = v.get("learned").and_then(Value::as_bool).unwrap_or(false);
            if let Some(first) = v.pointer("/ask/options/0/plans") {
                t.asking = steps(Some(first));
            }
            if v.get("declined").and_then(Value::as_bool).unwrap_or(false) {
                t.done.clear();
            }
        }
    }
    t
}

/// A clickable line with a rounded outline.
fn chip(ui: &mut Ui, t: &Tokens, text: &str, width: f32) -> egui::Response {
    let galley = ui.fonts_mut(|f| f.layout_no_wrap(crate::i18n::rtl_display(text), regular(12.5), t.text));
    let size = vec2(width.max(40.0), galley.size().y + 8.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let fill = if resp.hovered() { t.hover } else { t.input };
    ui.painter().rect_filled(rect, 4.0, fill);
    ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
    let rtl = text.chars().any(wordcraft_fonts::is_rtl);
    let pos = if rtl { egui::pos2(rect.max.x - 8.0 - galley.size().x, rect.min.y + 4.0) } else { egui::pos2(rect.min.x + 8.0, rect.min.y + 4.0) };
    ui.painter().galley(pos, galley, t.text);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}
