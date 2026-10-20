//! WordCraft AR: the Numbering Outline pane (Service tab › Outline). The document's numbered items
//! (١. أ. أولاً. …) as a live tree of cards beside the page — an index that is also a workbench:
//!
//! * every card shows the item's marker, its words and (for a main item) a rendered preview of the
//!   item as it stands on the page; it follows every edit at once;
//! * click a card to go to that item (its words are selected, so the font bar above acts on it);
//!   double-click to edit the words in place;
//! * drag a card before, after or into another one to move it with its sub-items, and the whole list
//!   renumbers; drop it on the strip at the bottom to take it out of the numbering — it becomes an
//!   explanation under the item before it; drag an explanation into a card to number it;
//! * Remove level takes away every marker of a level and the levels below step up and renumber.
//!
//! All of it is `service.outline.*` commands of the engine (one undo step each), so scripts and MCP
//! do the same. The pane draws; it does not decide.

use std::collections::HashSet;
use std::sync::Arc;

use egui::{Align2, Color32, CornerRadius, FontId, Id, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use serde_json::{Value, json};
use wordcraft_doc::{Pos, StoryRef};
use wordcraft_engine::cmd::service_outline::{OutlineNode, outline};

use crate::WordApp;
use crate::i18n::rtl_display;
use crate::theme::{Tokens, regular, semibold};

/// Where a dragged card lands relative to the card under the pointer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Place {
    Before,
    After,
    Into,
    /// The strip at the bottom: out of the numbering.
    Detach,
}

#[derive(Clone)]
struct Drag {
    from: usize,
    numbered: bool,
    marker: String,
    text: String,
}

#[derive(Clone)]
struct PaneState {
    filter: String,
    show_body: bool,
    previews: bool,
    whole_level: bool,
    selected: Option<usize>,
    editing: Option<(usize, String, bool)>,
    collapsed: HashSet<String>,
    drag: Option<Drag>,
    /// Where each visible card was drawn this frame (block index, rectangle).
    rects: Vec<(usize, Rect)>,
    nodes: Option<(u64, Arc<Vec<OutlineNode>>)>,
}

impl Default for PaneState {
    fn default() -> Self {
        Self {
            filter: String::new(),
            show_body: true,
            previews: true,
            whole_level: false,
            selected: None,
            editing: None,
            collapsed: HashSet::new(),
            drag: None,
            rects: Vec::new(),
            nodes: None,
        }
    }
}

fn level_color(t: &Tokens, depth: usize) -> Color32 {
    [t.accent, t.green, t.orange, t.blue, t.red][depth % 5]
}

fn is_rtl_text(s: &str) -> bool {
    s.chars().any(|c| wordcraft_fonts::is_rtl(c) || crate::i18n::is_arabic_digit(c))
}

fn collapse_key(n: &OutlineNode) -> String {
    format!("{}:{}", n.depth, n.text)
}

/// The pane: header, font bar, tools, the cards, the summary.
pub fn pane(app: &mut WordApp, ui: &mut Ui) {
    let sid = Id::new("service_outline_state");
    let mut st: PaneState = ui.data(|d| d.get_temp(sid)).unwrap_or_default();
    if crate::panes::header(ui, "Numbering Outline") {
        let _ = app.run("view.serviceOutlinePane", json!({"value": false}));
        return;
    }
    let rev = app.session.rev();
    let nodes: Arc<Vec<OutlineNode>> = match &st.nodes {
        Some((r, n)) if *r == rev => n.clone(),
        _ => {
            let n = Arc::new(outline(&app.session));
            st.nodes = Some((rev, n.clone()));
            n
        }
    };
    // The item holding the caret follows the page.
    let caret_block = app.session.sel.focus.path.0.first().copied().unwrap_or(0) as usize;
    let current = nodes.iter().rev().find(|n| n.index <= caret_block).map(|n| n.index);

    font_bar(app, ui, &mut st, &nodes);
    ui.add_space(4.0);
    tools(app, ui, &mut st, &nodes);
    ui.add_space(4.0);

    let t = Tokens::get(ui.ctx());
    // Room for the summary line and, while dragging, the take-out strip.
    let dragging = st.drag.is_some();
    let footer_h = 26.0 + if dragging { 40.0 } else { 0.0 };
    let list_h = (ui.available_height() - footer_h).max(80.0);
    st.rects.clear();
    let mut drop_target: Option<(usize, Place, Rect)> = None;
    let pointer = ui.ctx().input(|i| i.pointer.interact_pos());
    egui::ScrollArea::vertical().max_height(list_h).auto_shrink([false, false]).show(ui, |ui| {
        if nodes.is_empty() {
            ui.add_space(8.0);
            crate::rtl_text::paragraph(
                ui,
                tl!("No numbered items yet. Type ١. أ. أولاً. in the document, or press Renumber below, and they appear here."),
                regular(12.5),
                t.text_dim,
                ui.available_width(),
            );
        }
        let width = ui.available_width();
        let q = st.filter.trim().to_lowercase();
        let mut hide_below: Option<usize> = None;
        for (k, n) in nodes.iter().enumerate() {
            // collapsed items hide what is under them
            if let Some(d) = hide_below {
                if !n.numbered || n.depth > d {
                    continue;
                }
                hide_below = None;
            }
            let collapsed = n.numbered && st.collapsed.contains(&collapse_key(n));
            if collapsed {
                hide_below = Some(n.depth);
            }
            if !n.numbered && !st.show_body {
                continue;
            }
            if !q.is_empty() && !(n.text.to_lowercase().contains(&q) || n.marker.contains(&q)) {
                continue;
            }
            let hidden: Vec<&OutlineNode> =
                if collapsed { nodes[k + 1..].iter().take_while(|m| !m.numbered || m.depth > n.depth).collect() } else { Vec::new() };
            let has_children = nodes.get(k + 1).is_some_and(|m| m.numbered && m.depth > n.depth);
            let card = Card {
                n,
                hidden: &hidden,
                collapsed,
                has_children,
                current: current == Some(n.index),
                selected: st.selected == Some(n.index),
                width,
            };
            let out = draw_card(app, ui, &mut st, &t, &card);
            st.rects.push((n.index, out.rect));
            if let Some(action) = out.action {
                run_action(app, &mut st, &nodes, n, action);
            }
            ui.add_space(3.0);
        }
        // Drag near the edge of the list: scroll it.
        if let (Some(p), true) = (pointer, dragging) {
            let view = ui.clip_rect();
            let speed = if p.y < view.top() + 28.0 {
                6.0
            } else if p.y > view.bottom() - 28.0 {
                -6.0
            } else {
                0.0
            };
            if speed != 0.0 {
                ui.scroll_with_delta(vec2(0.0, speed));
                ui.ctx().request_repaint();
            }
        }
    });

    // The strip a card is dropped on to leave the numbering.
    let mut strip = None;
    if dragging && st.drag.as_ref().is_some_and(|d| d.numbered) {
        ui.add_space(4.0);
        let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::hover());
        strip = Some(r);
        let over = pointer.is_some_and(|p| r.contains(p));
        ui.painter().rect_filled(r, CornerRadius::same(6), if over { t.orange.linear_multiply(0.25) } else { t.input });
        ui.painter().rect_stroke(r, CornerRadius::same(6), Stroke::new(1.0, if over { t.orange } else { t.border_strong }), StrokeKind::Inside);
        ui.painter().text(
            r.center(),
            Align2::CENTER_CENTER,
            tl!("Drop here to take it out of the numbering"),
            regular(12.0),
            if over { t.text } else { t.text_dim },
        );
    }

    // Where the pointer is while a card is held.
    if let (Some(drag), Some(p)) = (st.drag.clone(), pointer) {
        if let Some(r) = strip.filter(|r| r.contains(p)) {
            drop_target = Some((drag.from, Place::Detach, r));
        } else if let Some((idx, r)) = st.rects.iter().find(|(_, r)| r.contains(p)).copied()
            && let Some(n) = nodes.iter().find(|n| n.index == idx)
        {
            let src = nodes.iter().find(|m| m.index == drag.from);
            let inside = src.is_some_and(|s| idx >= s.index && idx <= s.end);
            if !inside {
                let f = (p.y - r.top()) / r.height().max(1.0);
                let place = if n.numbered {
                    if f < 0.28 {
                        Place::Before
                    } else if f > 0.72 {
                        Place::After
                    } else {
                        Place::Into
                    }
                } else if f < 0.5 {
                    Place::Before
                } else {
                    Place::After
                };
                drop_target = Some((idx, place, r));
            }
        }
        if let Some((_, place, r)) = drop_target {
            paint_drop(ui, &t, place, r);
        }
        ghost(ui, &t, &drag, p);
        ui.ctx().request_repaint();
    }
    if st.drag.is_some() {
        let released = ui.ctx().input(|i| i.pointer.any_released());
        let cancelled = ui.ctx().input(|i| i.key_pressed(egui::Key::Escape));
        if released || cancelled {
            if let (Some(drag), Some((to, place, _)), true) = (st.drag.clone(), drop_target, released && !cancelled) {
                drop_card(app, &mut st, &drag, to, place);
            }
            st.drag = None;
        }
    }

    summary(app, ui, &mut st, &nodes);
    ui.data_mut(|d| d.insert_temp(sid, st));
}

// ------------------------------------------------------------------------------ top bars

fn tool_button(ui: &mut Ui, icon: &str, tip: &str, checked: bool, enabled: bool) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let (r, resp) = ui.allocate_exact_size(vec2(24.0, 22.0), Sense::click());
    let hover = enabled && resp.hovered();
    let fill = if checked {
        t.checked
    } else if hover {
        t.hover
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(r, CornerRadius::same(4), fill);
    let c = if enabled { t.icon } else { t.text_disabled };
    crate::icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(16.0, 16.0)), icon, c, if enabled { t.accent } else { c });
    resp.on_hover_text(tip)
}

/// Font, size, bold and italic of the card that is selected (the page's selection is that card's
/// words), or of the whole level when "level" is on.
fn font_bar(app: &mut WordApp, ui: &mut Ui, st: &mut PaneState, nodes: &Arc<Vec<OutlineNode>>) {
    let state = app.session.run("format.state", &json!({})).unwrap_or_default();
    let flag = |k: &str| state.get(k).and_then(Value::as_bool).unwrap_or(false);
    let t = Tokens::get(ui.ctx());
    ui.horizontal(|ui| {
        let font = state.get("font").and_then(Value::as_str).unwrap_or("").to_string();
        let font = app.previews.font_shown(&font);
        let fams = app.previews.families();
        let prev = app.previews.font_preview_fn();
        let w = (ui.available_width() - 66.0).clamp(90.0, 170.0);
        if let Some(f) = crate::widgets::font_combo(ui, "outline_font", w, &font, &fams, Some(&*prev))
            && f != crate::widgets::ADD_FONT
            && f != crate::widgets::LOAD_SYSTEM_FONTS
        {
            let f = app.previews.font_stored(&f);
            apply(app, st, nodes, "format.font", json!({"name": f}));
        }
        let size =
            state.get("size").and_then(Value::as_f64).map(|s| if s.fract() == 0.0 { format!("{s:.0}") } else { format!("{s}") }).unwrap_or_default();
        let sizes: Vec<String> =
            wordcraft_engine::cmd::format::SIZES.iter().map(|s| if s.fract() == 0.0 { format!("{s:.0}") } else { s.to_string() }).collect();
        if let Some(v) = crate::widgets::combo(ui, "outline_size", 52.0, &size, &sizes, None)
            && let Ok(x) = v.trim().parse::<f64>()
        {
            apply(app, st, nodes, "format.size", json!({"size": x}));
        }
    });
    ui.horizontal(|ui| {
        if tool_button(ui, "bold", tl!("Bold"), flag("bold"), true).clicked() {
            apply(app, st, nodes, "format.bold", json!({}));
        }
        if tool_button(ui, "italic", tl!("Italic"), flag("italic"), true).clicked() {
            apply(app, st, nodes, "format.italic", json!({}));
        }
        if tool_button(ui, "underline", tl!("Underline"), flag("underline"), true).clicked() {
            apply(app, st, nodes, "format.underline", json!({}));
        }
        ui.add_space(6.0);
        // "Level": the change goes to every item of the selected card's level.
        let tip = tl!("Apply to every item of the selected card's level");
        if tool_button(ui, "multilevel", tip, st.whole_level, true).clicked() {
            st.whole_level = !st.whole_level;
        }
        let label = if st.whole_level { tl!("Whole level") } else { tl!("This item") };
        ui.label(egui::RichText::new(label).font(regular(11.5)).color(t.text_dim));
    });
}

/// Run a character-format command on the selected card, or on every item of its level.
fn apply(app: &mut WordApp, st: &PaneState, nodes: &Arc<Vec<OutlineNode>>, cmd: &str, params: Value) {
    let sel = st.selected.and_then(|i| nodes.iter().find(|n| n.index == i));
    match sel {
        Some(sel) if st.whole_level => {
            let like: Vec<&OutlineNode> = nodes.iter().filter(|n| n.depth == sel.depth && n.numbered == sel.numbered).collect();
            // a toggle (Bold) must go the same way on every item: decide on the selected one
            let state = app.session.run("format.state", &json!({})).unwrap_or_default();
            let flag = |k: &str| state.get(k).and_then(Value::as_bool).unwrap_or(false);
            let key = cmd.strip_prefix("format.").unwrap_or(cmd);
            let is_toggle = matches!(cmd, "format.bold" | "format.italic" | "format.underline");
            let want_on = !flag(key);
            for n in like {
                select_node(app, n);
                let now = app.session.run("format.state", &json!({})).unwrap_or_default().get(key).and_then(Value::as_bool).unwrap_or(false);
                if !is_toggle || now != want_on {
                    let _ = app.run(cmd, params.clone());
                }
            }
            select_node(app, sel);
        }
        Some(sel) => {
            // act on the card's words even if the page's selection moved since
            let (a, b) = app.session.sel.ordered();
            let para = wordcraft_doc::Path::top(sel.index);
            if a.path != para || b.path != para {
                select_node(app, sel);
            }
            let _ = app.run(cmd, params);
        }
        None => {
            let _ = app.run(cmd, params);
        }
    }
}

fn tools(app: &mut WordApp, ui: &mut Ui, st: &mut PaneState, nodes: &Arc<Vec<OutlineNode>>) {
    ui.horizontal(|ui| {
        let w = (ui.available_width() - 4.0 * 26.0 - 8.0).max(60.0);
        ui.add(egui::TextEdit::singleline(&mut st.filter).hint_text(tl!("Find an item")).desired_width(w));
        if tool_button(ui, "pilcrow", tl!("Show explanations under the items"), st.show_body, true).clicked() {
            st.show_body = !st.show_body;
        }
        if tool_button(ui, "preview", tl!("Show a preview of each main item"), st.previews, true).clicked() {
            st.previews = !st.previews;
        }
        let any = nodes.iter().any(|n| n.numbered);
        if tool_button(ui, "outlineCollapse", tl!("Collapse all items"), false, any).clicked() {
            st.collapsed = nodes
                .iter()
                .filter(|n| n.numbered && nodes.iter().any(|m| m.index > n.index && m.index <= n.end && m.numbered))
                .map(collapse_key)
                .collect();
        }
        if tool_button(ui, "outlineExpand", tl!("Expand all items"), false, any).clicked() {
            st.collapsed.clear();
        }
    });
    ui.horizontal(|ui| {
        // Remove a whole level: «١.» of every chapter, say; the levels below step up.
        let levels: Vec<(usize, String)> = {
            let mut v: Vec<(usize, String)> = Vec::new();
            for n in nodes.iter().filter(|n| n.typed) {
                if !v.iter().any(|(d, _)| *d == n.depth) {
                    v.push((n.depth, n.marker.clone()));
                }
            }
            v.sort_by_key(|(d, _)| *d);
            v
        };
        let label = tl!("Remove a level…");
        ui.add_enabled_ui(!levels.is_empty(), |ui| {
            ui.menu_button(label, |ui| {
                for (d, m) in &levels {
                    let text = crate::i18n::fmt(tl!("Level {n} ({marker})"), &[("n", &(d + 1).to_string()), ("marker", &rtl_display(m))]);
                    if ui.button(text).clicked() {
                        let _ = app.run("service.outline.removeLevel", json!({"depth": d}));
                        st.selected = None;
                        ui.close();
                    }
                }
            });
        });
    });
}

// ------------------------------------------------------------------------------ cards

struct Card<'a> {
    n: &'a OutlineNode,
    hidden: &'a [&'a OutlineNode],
    collapsed: bool,
    has_children: bool,
    current: bool,
    selected: bool,
    width: f32,
}

enum Action {
    Select,
    Edit,
    Toggle,
    Promote,
    Demote,
    Detach,
    Attach,
    Commit(String),
    Cancel,
    StartDrag,
}

struct CardOut {
    rect: Rect,
    action: Option<Action>,
}

/// Lines of `text` for a card: at most `max_rows`, the last cut with an ellipsis.
fn card_lines(ui: &Ui, text: &str, font: &FontId, color: Color32, width: f32, max_rows: usize) -> Vec<String> {
    let mut lines = crate::rtl_text::wrap(ui, text, font, color, width);
    if lines.len() > max_rows {
        lines.truncate(max_rows);
        if let Some(l) = lines.last_mut() {
            l.push('…');
        }
    }
    lines
}

const THUMB_H: f32 = 74.0;

/// The preview's height: a line of the page is about 10 pt in it, up to eight lines.
fn thumb_height(n: &OutlineNode) -> f32 {
    let blocks = n.end.min(n.index + 7) - n.index + 1;
    (12.0 + blocks as f32 * 10.0).min(THUMB_H)
}

fn draw_card(app: &mut WordApp, ui: &mut Ui, st: &mut PaneState, t: &Tokens, c: &Card) -> CardOut {
    let n = c.n;
    let rtl = crate::rtl::on() || is_rtl_text(&n.text) || is_rtl_text(&n.marker);
    let color = if n.numbered { level_color(t, n.depth) } else { t.text_dim };
    let indent =
        (n.depth.min(5) as f32) * 12.0 * if n.numbered { 1.0 } else { 0.0 } + if n.numbered { 0.0 } else { ((n.depth.min(5)) as f32) * 12.0 };
    let full = c.width.max(80.0);
    let pad = 7.0;
    let editing = st.editing.as_ref().is_some_and(|(i, _, _)| *i == n.index);
    let dragged = st.drag.as_ref().is_some_and(|d| d.from == n.index);

    // -- measure
    let chip_font = semibold(12.0);
    let marker_text = if n.numbered { n.marker.clone() } else { "¶".to_string() };
    let chip_galley = ui.painter().layout_no_wrap(rtl_display(&marker_text), chip_font, color);
    let chip_w = (chip_galley.size().x + 12.0).max(26.0);
    let actions_w = if c.selected || c.current { 3.0 * 20.0 } else { 0.0 };
    let text_w = (full - indent - chip_w - pad * 3.0 - 4.0 - 14.0 - actions_w).max(40.0);
    let font = if n.depth == 0 && n.numbered { semibold(13.0) } else { regular(12.5) };
    let text_color = if n.numbered { t.text } else { t.text_dim };
    let lines = if editing { Vec::new() } else { card_lines(ui, &n.text, &font, text_color, text_w, if n.numbered { 2 } else { 3 }) };
    let row_h = font.size * 1.4;
    let body_h = if editing { row_h + 8.0 } else { (lines.len().max(1) as f32) * row_h };
    let thumb = st.previews && n.numbered && n.depth == 0 && !c.collapsed;
    let skel_rows = c.hidden.len().min(4);
    let h = pad * 2.0
        + body_h.max(chip_galley.size().y + 6.0)
        + if thumb { thumb_height(n) + 6.0 } else { 0.0 }
        + if skel_rows > 0 { skel_rows as f32 * 8.0 + 4.0 } else { 0.0 };

    let (outer, _) = ui.allocate_exact_size(vec2(full, h), Sense::hover());
    let rect = if rtl {
        Rect::from_min_max(pos2(outer.min.x, outer.min.y), pos2(outer.max.x - indent, outer.max.y))
    } else {
        Rect::from_min_max(pos2(outer.min.x + indent, outer.min.y), outer.max)
    };
    let id = ui.id().with(("ol_card", n.index));
    let resp = ui.interact(rect, id, Sense::click_and_drag());
    let hovered = resp.hovered() && st.drag.is_none();

    // -- paint
    let fill = if dragged {
        t.input.linear_multiply(0.5)
    } else if c.selected {
        color.linear_multiply(0.16)
    } else if hovered {
        t.hover
    } else {
        t.input
    };
    let stroke = if c.selected {
        Stroke::new(1.5, color)
    } else if c.current {
        Stroke::new(1.0, color.linear_multiply(0.7))
    } else {
        Stroke::new(1.0, t.border)
    };
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(7), fill);
    p.rect_stroke(rect, CornerRadius::same(7), stroke, StrokeKind::Inside);
    // the level stripe on the start edge
    let stripe = if rtl {
        Rect::from_min_max(pos2(rect.max.x - 4.0, rect.min.y + 5.0), pos2(rect.max.x - 1.0, rect.max.y - 5.0))
    } else {
        Rect::from_min_max(pos2(rect.min.x + 1.0, rect.min.y + 5.0), pos2(rect.min.x + 4.0, rect.max.y - 5.0))
    };
    p.rect_filled(stripe, CornerRadius::same(2), color);

    let start = |x: f32| if rtl { rect.max.x - x } else { rect.min.x + x };
    let place_x = |x: f32, w: f32| if rtl { start(x) - w } else { start(x) };
    let top = rect.min.y + pad;
    // chip
    let chip = Rect::from_min_size(pos2(place_x(pad + 6.0, chip_w), top), vec2(chip_w, chip_galley.size().y + 6.0));
    p.rect_filled(chip, CornerRadius::same(5), color.linear_multiply(if n.numbered { 0.18 } else { 0.1 }));
    p.galley(pos2(chip.center().x - chip_galley.size().x / 2.0, chip.center().y - chip_galley.size().y / 2.0), chip_galley, color);
    // words
    let text_left = place_x(pad + 6.0 + chip_w + 7.0, text_w);
    let text_box = Rect::from_min_size(pos2(text_left, top - 1.0), vec2(text_w, body_h));
    let mut action = None;
    if editing {
        let (mut buf, mut focus) = st.editing.as_ref().map(|(_, b, f)| (b.clone(), *f)).unwrap_or_default();
        let edit = egui::TextEdit::singleline(&mut buf).font(font.clone()).desired_width(text_w).id(id.with("edit"));
        let r = ui.put(text_box, edit);
        if !focus {
            r.request_focus();
            focus = true;
        }
        let enter = r.lost_focus() && ui.ctx().input(|i| i.key_pressed(egui::Key::Enter));
        let esc = ui.ctx().input(|i| i.key_pressed(egui::Key::Escape));
        if esc {
            action = Some(Action::Cancel);
        } else if enter || (r.lost_focus() && !esc) {
            action = Some(Action::Commit(buf.clone()));
        }
        st.editing = Some((n.index, buf, focus));
    } else {
        let p = ui.painter();
        for (i, line) in lines.iter().enumerate() {
            let g = p.layout_no_wrap(rtl_display(line), font.clone(), text_color);
            let y = text_box.min.y + i as f32 * row_h + (row_h - g.size().y) / 2.0;
            let x = if rtl { text_box.max.x - g.size().x } else { text_box.min.x };
            p.galley(pos2(x, y), g, text_color);
        }
    }
    // wireframe bars for what a collapsed item hides
    if skel_rows > 0 {
        let base = rect.max.y - skel_rows as f32 * 8.0 - 2.0;
        for (i, m) in c.hidden.iter().take(skel_rows).enumerate() {
            let len = ((m.text.chars().count() as f32) * 3.0).clamp(30.0, text_w);
            let off = (m.depth.saturating_sub(n.depth)).min(4) as f32 * 8.0;
            let y = base + i as f32 * 8.0;
            let x0 = if rtl { text_box.max.x - off - len } else { text_box.min.x + off };
            let bar = Rect::from_min_size(pos2(x0, y), vec2(len, 3.5));
            ui.painter().rect_filled(
                bar,
                CornerRadius::same(2),
                if m.numbered { level_color(t, m.depth).linear_multiply(0.45) } else { t.border_strong },
            );
        }
    }
    // a rendered preview of a main item
    if thumb && ui.is_rect_visible(rect) {
        let tr = Rect::from_min_max(pos2(rect.min.x + pad + 4.0, rect.max.y - thumb_height(n) - pad), pos2(rect.max.x - pad - 4.0, rect.max.y - pad));
        item_preview(app, ui, t, n, tr);
    }
    // chevron for items that have items under them
    if n.numbered && c.has_children {
        let ch = Rect::from_center_size(pos2(if rtl { rect.min.x + 12.0 } else { rect.max.x - 12.0 }, top + 9.0), vec2(16.0, 16.0));
        let r = ui.interact(ch, id.with("chev"), Sense::click());
        crate::icons::paint(ui.painter(), ch, if c.collapsed { "chevronLeft" } else { "chevronDown" }, t.icon, t.accent);
        if r.clicked() {
            action = Some(Action::Toggle);
        }
    }
    // the small actions of the card that is selected or holds the caret
    if (c.selected || c.current) && !editing && st.drag.is_none() {
        let mut x = if rtl { rect.min.x + 28.0 } else { rect.max.x - 28.0 };
        let step = if rtl { 20.0 } else { -20.0 };
        let mut button = |icon: &str, tip: &str, act: Action, ui: &mut Ui, action: &mut Option<Action>| {
            let r = Rect::from_center_size(pos2(x, top + 9.0), vec2(18.0, 18.0));
            x += step;
            let rr = ui.interact(r, id.with(icon), Sense::click());
            if rr.hovered() {
                ui.painter().rect_filled(r, CornerRadius::same(4), t.pressed);
            }
            crate::icons::paint(ui.painter(), r.shrink(1.0), icon, t.icon, t.accent);
            if rr.on_hover_text(tip).clicked() {
                *action = Some(act);
            }
        };
        if n.typed {
            button("outlinePromote", tl!("Move out one level"), Action::Promote, ui, &mut action);
            button("outlineDemote", tl!("Move in one level"), Action::Demote, ui, &mut action);
            button("eraser", tl!("Take out of the numbering"), Action::Detach, ui, &mut action);
        } else {
            button("numbering", tl!("Put into the numbering"), Action::Attach, ui, &mut action);
        }
    }
    if action.is_none() {
        if resp.double_clicked() && !editing {
            action = Some(Action::Edit);
        } else if resp.clicked() && !editing {
            action = Some(Action::Select);
        } else if resp.drag_started() && !editing {
            action = Some(Action::StartDrag);
        }
    }
    CardOut { rect, action }
}

/// A rendered preview of an item with what is under it, as the page shows it.
fn item_preview(app: &mut WordApp, ui: &mut Ui, t: &Tokens, n: &OutlineNode, r: Rect) {
    let last = n.end.min(n.index + 7);
    let blocks: Vec<wordcraft_doc::Block> = (n.index..=last).filter_map(|i| app.session.doc.body.get(i).map(|b| (**b).clone())).collect();
    if blocks.is_empty() {
        return;
    }
    let sig = blocks.iter().filter_map(|b| b.as_para()).fold(blocks.len() as u64, |a, p| a.wrapping_mul(31).wrapping_add(p.rev));
    let ppp = ui.ctx().pixels_per_point();
    let (w, h) = (r.width() * 1.9, r.height() * 1.9);
    let key = format!("outline:{}:{sig:x}:{:.0}", n.index, ppp);
    let doc = &app.session.doc;
    let tex = app.previews.get_or(ui.ctx(), &key, || crate::previews::blocks_snippet(doc, blocks, w, h, ppp, 8.0));
    let p = ui.painter().with_clip_rect(r);
    p.rect_filled(r, CornerRadius::same(3), crate::previews::preview_tint(t, false));
    if let Some(tex) = tex {
        let size = tex.size_vec2() / ppp;
        let scale = (r.width() / size.x).min(1.0);
        let shown = size * scale;
        let img = Rect::from_min_size(r.min, shown);
        p.image(tex.id(), img, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    }
    ui.painter().rect_stroke(r, CornerRadius::same(3), Stroke::new(1.0, t.border_strong), StrokeKind::Inside);
}

// ------------------------------------------------------------------------------ actions

fn select_node(app: &mut WordApp, n: &OutlineNode) {
    let path = wordcraft_doc::Path::top(n.index);
    let len = app.session.doc.para(StoryRef::Body, &path).map(|p| p.len()).unwrap_or(0);
    let a = Pos::new(StoryRef::Body, path.clone(), 0);
    let b = Pos::new(StoryRef::Body, path, len);
    let _ = app.run("select.range", json!({"anchor": a, "focus": b}));
}

fn run_action(app: &mut WordApp, st: &mut PaneState, nodes: &Arc<Vec<OutlineNode>>, n: &OutlineNode, a: Action) {
    match a {
        Action::Select => {
            st.selected = Some(n.index);
            select_node(app, n);
            app.canvas.nav_to_caret = true;
            app.canvas.scroll_to_caret = false;
        }
        Action::Edit => {
            st.selected = Some(n.index);
            st.editing = Some((n.index, n.text.clone(), false));
        }
        Action::Toggle => {
            let k = collapse_key(n);
            if !st.collapsed.remove(&k) {
                st.collapsed.insert(k);
            }
        }
        Action::Promote => {
            let _ = app.run("service.outline.setLevel", json!({"index": n.index, "delta": -1}));
        }
        Action::Demote => {
            let _ = app.run("service.outline.setLevel", json!({"index": n.index, "delta": 1}));
        }
        Action::Detach => {
            let _ = app.run("service.outline.detach", json!({"index": n.index}));
        }
        Action::Attach => {
            let _ = app.run("service.outline.attach", json!({"index": n.index}));
        }
        Action::Commit(text) => {
            st.editing = None;
            if text.trim() != n.text {
                let _ = app.run("service.outline.editText", json!({"index": n.index, "text": text.trim()}));
            }
            app.canvas.want_focus = false;
        }
        Action::Cancel => st.editing = None,
        Action::StartDrag => {
            st.selected = Some(n.index);
            st.drag = Some(Drag { from: n.index, numbered: n.numbered, marker: n.marker.clone(), text: n.text.clone() });
        }
    }
    let _ = nodes;
}

fn drop_card(app: &mut WordApp, st: &mut PaneState, drag: &Drag, to: usize, place: Place) {
    st.selected = None;
    match place {
        Place::Detach => {
            let _ = app.run("service.outline.detach", json!({"index": drag.from}));
        }
        _ => {
            let p = match place {
                Place::Before => "before",
                Place::After => "after",
                _ => "into",
            };
            // a plain paragraph dropped into a card joins the numbering there
            let numbered = !drag.numbered && place == Place::Into;
            let _ = app.run("service.outline.move", json!({"from": drag.from, "to": to, "place": p, "numbered": numbered}));
        }
    }
}

fn paint_drop(ui: &Ui, t: &Tokens, place: Place, r: Rect) {
    let p = ui.painter();
    match place {
        Place::Before => {
            p.line_segment([pos2(r.min.x, r.min.y - 2.0), pos2(r.max.x, r.min.y - 2.0)], Stroke::new(3.0, t.accent));
        }
        Place::After => {
            p.line_segment([pos2(r.min.x, r.max.y + 2.0), pos2(r.max.x, r.max.y + 2.0)], Stroke::new(3.0, t.accent));
        }
        Place::Into => {
            p.rect_filled(r, CornerRadius::same(7), t.accent.linear_multiply(0.14));
            p.rect_stroke(r, CornerRadius::same(7), Stroke::new(2.0, t.accent), StrokeKind::Inside);
        }
        Place::Detach => {}
    }
}

/// The card under the pointer while it is dragged.
fn ghost(ui: &Ui, t: &Tokens, drag: &Drag, p: egui::Pos2) {
    egui::Area::new(Id::new("outline_ghost")).order(egui::Order::Tooltip).fixed_pos(p + vec2(10.0, 12.0)).interactable(false).show(ui.ctx(), |ui| {
        egui::Frame::NONE.fill(t.menu).stroke(Stroke::new(1.0, t.accent)).corner_radius(6).inner_margin(6).show(ui, |ui| {
            let text = format!("{} {}", drag.marker, drag.text);
            let shown: String = text.chars().take(40).collect();
            ui.label(egui::RichText::new(rtl_display(&shown)).font(semibold(12.5)).color(t.text));
        });
    });
}

fn summary(app: &mut WordApp, ui: &mut Ui, st: &mut PaneState, nodes: &Arc<Vec<OutlineNode>>) {
    let t = Tokens::get(ui.ctx());
    let items = nodes.iter().filter(|n| n.numbered).count();
    let notes = nodes.len() - items;
    ui.add_space(4.0);
    let text =
        crate::i18n::fmt(tl!("{items} numbered items · {notes} explanations"), &[("items", &items.to_string()), ("notes", &notes.to_string())]);
    ui.horizontal(|ui| {
        let w = (ui.available_width() - 90.0).max(60.0);
        crate::rtl_text::paragraph(ui, &text, regular(11.5), t.text_dim, w);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button(tl!("Renumber")).on_hover_text(tl!("Put every marker on the service cycle again")).clicked() {
                let _ = app.run("service.renumber", json!({}));
                st.selected = None;
            }
        });
    });
}
