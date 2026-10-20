//! «فهرس الترقيم» — the numbering outline behind WordCraft AR's right-hand pane (and MCP / CLI).
//!
//! The outline is the document's typed service markers read as a tree: every numbered paragraph is
//! a node with its level, marker and text; an unnumbered paragraph under a marker is that item's
//! explanation (its "body"). Everything the pane lets a person do is a command here, so a script
//! can do it too, and each one is one undo step:
//!
//! * `service.outline` — the nodes (read only);
//! * `service.outline.move` — drag and drop: an item with its sub-items and explanations goes
//!   before, after or into another item, taking the level of its new place;
//! * `service.outline.setLevel` — promote / demote an item with its sub-items;
//! * `service.outline.detach` — the item loses its marker and becomes an explanation under the
//!   item before it; `service.outline.attach` does the reverse for a plain paragraph;
//! * `service.outline.removeLevel` — every marker of one level goes; the levels below step up and
//!   the whole list renumbers at once;
//! * `service.outline.editText` — the item's words, the marker kept.
//!
//! After each change every marker is rewritten to the cycle (١. أ. أولاً. …) so the numbers and the
//! levels are right immediately.

use std::collections::HashMap;

use serde::Serialize;
use serde_json::{Value, json};
use wordcraft_doc::{Block, Path, Pos, StoryRef};

use super::service::{Item, Kind, analyse, estimate_indent, like_at_depth, put_marker_sep};
use crate::{CmdError, CmdResult, CommandSpec, Session, p};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("service.outline", "Numbering Outline", "Service › Outline", |s, _| Ok(json!({"nodes": outline(s)}))).pure().params(
            r#"→ {"nodes": [{"index", "depth", "numbered", "marker", "text", "end"}]}: the document's numbered items as a tree in document order (`index` is the body block, `end` the last block of the item with its sub-items and explanations)"#,
        ),
        CommandSpec::new("view.serviceOutlinePane", "Numbering Outline", "Service › Outline", |s, v| {
            s.view.service_outline_pane = p::bool(v, "value").unwrap_or(!s.view.service_outline_pane);
            Ok(json!({"value": s.view.service_outline_pane}))
        })
        .params(r#"{"value"?: bool (default: toggle)} — the Numbering Outline pane beside the page"#)
        .pure(),
        CommandSpec::new("service.outline.move", "Move Numbered Item", "Service › Outline", move_node)
            .params(r#"{"from": block index, "to": block index, "place": "before|after|into", "numbered"?: bool} — the item with its sub-items goes there and takes the level of its new place; a plain paragraph dropped with `numbered` joins the numbering there; the list renumbers"#),
        CommandSpec::new("service.outline.setLevel", "Change Item Level", "Service › Outline", set_level)
            .params(r#"{"index": block index, "delta"?: -1|1, "depth"?: level} — promote or demote an item with its sub-items"#),
        CommandSpec::new("service.outline.detach", "Take Out of Numbering", "Service › Outline", detach)
            .params(r#"{"index": block index} — the marker goes; the paragraph is an explanation under the item before it"#),
        CommandSpec::new("service.outline.attach", "Put into Numbering", "Service › Outline", attach)
            .params(r#"{"index": block index, "depth"?: level (default: the item before it)} — a plain paragraph becomes a numbered item"#),
        CommandSpec::new("service.outline.removeLevel", "Remove Numbering Level", "Service › Outline", remove_level)
            .params(r#"{"depth": level (0 = the main level)} — every marker of that level goes, the levels below step up and renumber at once"#),
        CommandSpec::new("service.outline.editText", "Edit Numbered Item", "Service › Outline", edit_text)
            .params(r#"{"index": block index, "text": string} — the item's words after its marker"#),
    ]
}

/// One paragraph of the outline.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OutlineNode {
    /// The body block the paragraph is.
    pub index: usize,
    /// Level (0 = main). An explanation sits one level under the item above it.
    pub depth: usize,
    /// It starts with a service marker (typed, or a Word list item).
    pub numbered: bool,
    /// The marker is typed text (a Word list item's is not, so the pane can't rewrite it).
    pub typed: bool,
    /// The marker as typed («أ.»), empty for an unnumbered paragraph.
    pub marker: String,
    /// The paragraph's words after the marker.
    pub text: String,
    /// The marker and its separator, in bytes of the paragraph's text.
    pub marker_len: usize,
    /// The last body block of this item with its sub-items and explanations.
    pub end: usize,
}

/// Top-level body paragraph paths (the outline does not look into tables).
fn top_paths(s: &Session) -> Vec<Path> {
    s.doc.para_paths(StoryRef::Body).into_iter().filter(|p| p.0.len() == 1).collect()
}

/// The outline of the body, in document order.
pub fn outline(s: &Session) -> Vec<OutlineNode> {
    let paths = top_paths(s);
    let items = analyse(s, &paths, true);
    let by: HashMap<usize, &Item> = items.iter().map(|i| (i.path.last(), i)).collect();
    let mut nodes: Vec<OutlineNode> = Vec::new();
    for path in &paths {
        let Some(para) = s.doc.para(StoryRef::Body, path) else { continue };
        if para.text.trim().is_empty() {
            continue;
        }
        let index = path.last();
        let node = match by.get(&index) {
            Some(it) => {
                let len = it.marker.map(|m| m.2).unwrap_or(0).min(para.text.len());
                let marker_len = if para.text.is_char_boundary(len) { len } else { 0 };
                OutlineNode {
                    index,
                    depth: it.depth,
                    numbered: it.numbered,
                    typed: it.marker.is_some(),
                    marker: para.text.get(..marker_len).unwrap_or("").trim().to_string(),
                    text: para.text.get(marker_len..).unwrap_or("").trim().to_string(),
                    marker_len,
                    end: index,
                }
            }
            // text before the first marker: a preface, outside the numbering
            None => OutlineNode {
                index,
                depth: 0,
                numbered: false,
                typed: false,
                marker: String::new(),
                text: para.text.trim().to_string(),
                marker_len: 0,
                end: index,
            },
        };
        nodes.push(node);
    }
    let last_block = s.doc.body.len().saturating_sub(1);
    for k in 0..nodes.len() {
        if !nodes[k].numbered {
            continue;
        }
        let depth = nodes[k].depth;
        let end = nodes[k + 1..].iter().find(|n| n.numbered && n.depth <= depth).map(|n| n.index.saturating_sub(1)).unwrap_or(last_block);
        nodes[k].end = end.max(nodes[k].index);
    }
    nodes
}

fn find(nodes: &[OutlineNode], index: usize) -> Result<&OutlineNode, CmdError> {
    nodes.iter().find(|n| n.index == index).ok_or_else(|| CmdError::Params(format!("block {index} is not a numbered or explanatory paragraph")))
}

fn idx(v: &Value, key: &str) -> Result<usize, CmdError> {
    p::u64(v, key).map(|x| x as usize).ok_or_else(|| CmdError::Params(format!("`{key}` (a block index) is required")))
}

fn path_of(index: usize) -> Path {
    Path::top(index)
}

/// Copy the indents and the tab stop of the paragraph at `from` onto the one at `to`, so an item
/// that changed level lands on its new level's column.
fn copy_indent(s: &mut Session, from: &Path, to: &Path) -> Result<(), CmdError> {
    let Some(src) = s.doc.para(StoryRef::Body, from).map(|p| (p.props.indent_left, p.props.indent_first, p.props.tabs.clone())) else {
        return Ok(());
    };
    let para = s.doc.para_mut(StoryRef::Body, to)?;
    para.props.bidi = Some(true);
    para.props.indent_left = src.0;
    para.props.indent_first = src.1;
    para.props.tabs = src.2;
    para.touch();
    Ok(())
}

/// Put every numbered paragraph on the cycle: its marker of the kind its level has and the number
/// its place gives (the levels are those the markers' kinds make, the shallowest becoming ١.).
/// A paragraph whose level changed takes the indent of another at its new level.
fn relabel(s: &mut Session) -> Result<usize, CmdError> {
    let paths = top_paths(s);
    let items = analyse(s, &paths, true);
    let mut counters: Vec<u32> = vec![0; 64];
    let mut last_depth = 0usize;
    let mut edits = Vec::new();
    for (pos, it) in items.iter().enumerate() {
        if !it.numbered {
            continue;
        }
        if it.depth < last_depth {
            for c in counters.iter_mut().skip(it.depth + 1) {
                *c = 0;
            }
        }
        if let Some(c) = counters.get_mut(it.depth) {
            *c += 1;
        }
        last_depth = it.depth;
        let want = counters.get(it.depth).copied().unwrap_or(1);
        let want_kind = Kind::from_depth(it.depth);
        if let Some((kind, n, len)) = it.marker
            && (kind != want_kind || n != want)
        {
            edits.push((pos, len, want_kind, want, kind != want_kind));
        }
    }
    let changed = edits.len();
    for (pos, len, kind, n, kind_changed) in edits {
        let it = &items[pos];
        let like = if kind_changed { like_at_depth(&items, pos, it.depth) } else { None };
        put_marker_sep(s, &it.path, len, kind, n, like.as_ref(), None)?;
        if kind_changed && like.is_none() {
            estimate_indent(s, &it.path, it.depth)?;
        }
    }
    Ok(changed)
}

/// The shallowest level the paragraph at `at` (position in `nodes`) may take: one under the
/// numbered paragraph before it, or the top when there is none.
fn max_depth(nodes: &[OutlineNode], at: usize) -> usize {
    nodes[..at].iter().rev().find(|n| n.numbered).map(|n| n.depth + 1).unwrap_or(0)
}

fn reply(s: &mut Session, nodes_changed: usize) -> CmdResult {
    let mut r = super::sel_result(s)?;
    r["relabelled"] = json!(nodes_changed);
    r["nodes"] = json!(outline(s));
    Ok(r)
}

/// Rewrite the markers of the typed items in `span` (block indices) to level `delta` away from
/// where they are, then renumber everything.
fn shift_levels(s: &mut Session, nodes: &[OutlineNode], span: std::ops::RangeInclusive<usize>, delta: isize) -> Result<usize, CmdError> {
    for n in nodes.iter().filter(|n| n.typed && span.contains(&n.index)) {
        let depth = (n.depth as isize + delta).max(0) as usize;
        put_marker_sep(s, &path_of(n.index), n.marker_len, Kind::from_depth(depth), 1, None, None)?;
    }
    relabel(s)
}

/// After a change of levels: each moved or shifted item takes the indent of an item at its new
/// level (another one, or the engine's estimate when it is the only one).
fn align_indents(s: &mut Session, blocks: &[usize]) -> Result<(), CmdError> {
    let paths = top_paths(s);
    let items = analyse(s, &paths, true);
    for &b in blocks {
        let Some(pos) = items.iter().position(|i| i.path.last() == b) else { continue };
        let it = &items[pos];
        if !it.numbered || it.marker.is_none() {
            continue;
        }
        // another item of this level that is not one of those moved
        let like = items
            .iter()
            .enumerate()
            .filter(|(k, i)| *k != pos && i.numbered && i.marker.is_some() && i.depth == it.depth && !blocks.contains(&i.path.last()))
            .min_by_key(|(k, _)| k.abs_diff(pos))
            .map(|(_, i)| i.path.clone());
        match like {
            Some(l) => copy_indent(s, &l, &it.path)?,
            None => estimate_indent(s, &it.path, it.depth)?,
        }
    }
    Ok(())
}

/// `service.outline.move`.
fn move_node(s: &mut Session, v: &Value) -> CmdResult {
    let from = idx(v, "from")?;
    let to = idx(v, "to")?;
    let place = p::str(v, "place").unwrap_or("after");
    if !matches!(place, "before" | "after" | "into") {
        return Err(CmdError::Params("`place` is before, after or into".into()));
    }
    let nodes = outline(s);
    let a = find(&nodes, from)?.clone();
    let t = find(&nodes, to)?.clone();
    if t.index >= a.index && t.index <= a.end {
        return Err(CmdError::Params("an item cannot be moved into itself".into()));
    }
    let span_len = a.end - a.index + 1;
    if span_len >= s.doc.body.len() {
        return reply(s, 0);
    }
    let new_depth = if place == "into" { t.depth + 1 } else { t.depth };
    let delta = if a.numbered { new_depth as isize - a.depth as isize } else { 0 };
    let ins = if place == "before" { t.index } else { t.end + 1 };
    // take the blocks out, then put them in at the (shifted) place
    let mut blocks: Vec<Block> = Vec::with_capacity(span_len);
    for _ in 0..span_len {
        blocks.push(s.doc.remove_block(StoryRef::Body, &path_of(a.index))?);
    }
    let at = if ins > a.end { ins - span_len } else { ins };
    for (k, b) in blocks.into_iter().enumerate() {
        s.doc.insert_block(StoryRef::Body, &path_of(at + k), b)?;
    }
    let moved: Vec<usize> = (at..at + span_len).collect();
    if !a.numbered && p::bool(v, "numbered").unwrap_or(false) {
        // a plain paragraph dropped into the numbering: it is numbered at its new place's level
        let nodes = outline(s);
        let pos = nodes.iter().position(|n| n.index == at).ok_or_else(|| CmdError::Params("the moved paragraph is empty".into()))?;
        let depth = new_depth.min(max_depth(&nodes, pos));
        let items = analyse(s, &top_paths(s), true);
        let ipos = items.iter().position(|i| i.path.last() == at).unwrap_or(0);
        let like = like_at_depth(&items, ipos, depth);
        put_marker_sep(s, &path_of(at), 0, Kind::from_depth(depth), 1, like.as_ref(), None)?;
        if like.is_none() {
            estimate_indent(s, &path_of(at), depth)?;
        }
        let changed = relabel(s)?;
        return reply(s, changed.max(1));
    }
    let changed = if delta != 0 {
        let nodes = outline(s);
        // the moved items' levels, then the list
        let shifted = shift_levels(s, &nodes, at..=at + span_len - 1, delta)?;
        align_indents(s, &moved)?;
        shifted
    } else {
        relabel(s)?
    };
    reply(s, changed)
}

/// `service.outline.setLevel`.
fn set_level(s: &mut Session, v: &Value) -> CmdResult {
    let index = idx(v, "index")?;
    let nodes = outline(s);
    let at = nodes.iter().position(|n| n.index == index).ok_or_else(|| CmdError::Params(format!("block {index} is not in the outline")))?;
    let n = nodes[at].clone();
    if !n.typed {
        return Err(CmdError::Params("only an item with a typed marker changes level".into()));
    }
    let want = match (p::u64(v, "depth"), v.get("delta").and_then(Value::as_i64)) {
        (Some(d), _) => d as usize,
        (None, Some(d)) => (n.depth as i64 + d).max(0) as usize,
        _ => return Err(CmdError::Params("`delta` or `depth` is required".into())),
    };
    let top = max_depth(&nodes, at);
    if want > top {
        return Err(CmdError::Params(format!("this item can go down to level {top} at most (the item before it)")));
    }
    if want == n.depth {
        return reply(s, 0);
    }
    let delta = want as isize - n.depth as isize;
    let changed = shift_levels(s, &nodes, n.index..=n.end, delta)?;
    let blocks: Vec<usize> = (n.index..=n.end).collect();
    align_indents(s, &blocks)?;
    reply(s, changed.max(1))
}

/// Remove the typed marker of the paragraph at `index` and clear its marker indents.
fn strip_marker(s: &mut Session, node: &OutlineNode, parent_indent: Option<f32>) -> Result<(), CmdError> {
    let path = path_of(node.index);
    let a = Pos::new(StoryRef::Body, path.clone(), 0);
    let b = Pos::new(StoryRef::Body, path.clone(), node.marker_len);
    s.doc.delete_range(&a, &b)?;
    let para = s.doc.para_mut(StoryRef::Body, &path)?;
    para.props.indent_left = parent_indent;
    para.props.indent_first = parent_indent.map(|_| 0.0);
    para.props.tabs = None;
    para.touch();
    Ok(())
}

/// `service.outline.detach`.
fn detach(s: &mut Session, v: &Value) -> CmdResult {
    let index = idx(v, "index")?;
    let nodes = outline(s);
    let at = nodes.iter().position(|n| n.index == index).ok_or_else(|| CmdError::Params(format!("block {index} is not in the outline")))?;
    let n = nodes[at].clone();
    if !n.typed {
        return Err(CmdError::Params("this paragraph has no typed marker to take out".into()));
    }
    // it now explains the item before it: its text starts where that item's text does
    let parent_indent =
        nodes[..at].iter().rev().find(|p| p.numbered).and_then(|p| s.doc.para(StoryRef::Body, &path_of(p.index))).and_then(|p| p.props.indent_left);
    strip_marker(s, &n, parent_indent)?;
    let changed = relabel(s)?;
    reply(s, changed.max(1))
}

/// `service.outline.attach`.
fn attach(s: &mut Session, v: &Value) -> CmdResult {
    let index = idx(v, "index")?;
    let nodes = outline(s);
    let at = nodes.iter().position(|n| n.index == index).ok_or_else(|| CmdError::Params(format!("block {index} is not in the outline")))?;
    let n = nodes[at].clone();
    if n.numbered {
        return Err(CmdError::Params("this paragraph already has a marker".into()));
    }
    // by default a sibling of the numbered item before it
    let before = nodes[..at].iter().rev().find(|p| p.numbered);
    let depth = p::u64(v, "depth").map(|d| d as usize).unwrap_or_else(|| before.map(|b| b.depth).unwrap_or(0));
    let depth = depth.min(max_depth(&nodes, at));
    let paths = top_paths(s);
    let items = analyse(s, &paths, true);
    let pos = items.iter().position(|i| i.path.last() == index).unwrap_or(0);
    let like = like_at_depth(&items, pos, depth);
    put_marker_sep(s, &path_of(index), 0, Kind::from_depth(depth), 1, like.as_ref(), None)?;
    if like.is_none() {
        estimate_indent(s, &path_of(index), depth)?;
    }
    let changed = relabel(s)?;
    reply(s, changed.max(1))
}

/// `service.outline.removeLevel`.
fn remove_level(s: &mut Session, v: &Value) -> CmdResult {
    let depth = p::u64(v, "depth").ok_or_else(|| CmdError::Params("`depth` is required".into()))? as usize;
    let nodes = outline(s);
    let targets: Vec<OutlineNode> = nodes.iter().filter(|n| n.typed && n.depth == depth).cloned().collect();
    if targets.is_empty() {
        return Err(CmdError::Params(format!("there is no numbered item at level {depth}")));
    }
    for n in &targets {
        strip_marker(s, n, None)?;
    }
    let changed = relabel(s)?;
    // the levels that stepped up take the indent of their new level
    let blocks: Vec<usize> = nodes.iter().filter(|n| n.typed && n.depth > depth).map(|n| n.index).collect();
    align_indents(s, &blocks)?;
    reply(s, changed.max(targets.len()))
}

/// `service.outline.editText`.
fn edit_text(s: &mut Session, v: &Value) -> CmdResult {
    let index = idx(v, "index")?;
    let text = p::req_str(v, "text")?.replace(['\n', '\r'], " ");
    let nodes = outline(s);
    let n = find(&nodes, index)?.clone();
    let path = path_of(index);
    let (len, props) = {
        let para = s.doc.para(StoryRef::Body, &path).ok_or_else(|| CmdError::Params("no such paragraph".into()))?;
        // the formatting of the paragraph's words (the run after the marker), else its last run
        let mut off = 0usize;
        let mut found = None;
        for r in &para.runs {
            if off + r.len > n.marker_len && found.is_none() {
                found = Some(r.props.clone());
            }
            off += r.len;
        }
        (para.len(), found.or_else(|| para.runs.last().map(|r| r.props.clone())).unwrap_or_default())
    };
    let a = Pos::new(StoryRef::Body, path.clone(), n.marker_len);
    if len > n.marker_len {
        s.doc.delete_range(&a, &Pos::new(StoryRef::Body, path.clone(), len))?;
    }
    if !text.is_empty() {
        s.doc.insert_text(&a, &text, &props)?;
    }
    reply(s, 0)
}

#[cfg(test)]
mod tests {
    use super::super::service::marker_text;
    use super::*;

    fn doc(text: &str) -> Session {
        Session::new(wordcraft_doc::Document::from_text(text))
    }

    fn texts(s: &Session) -> Vec<String> {
        s.doc.body.iter().filter_map(|b| b.as_para()).map(|p| p.text.clone()).collect()
    }

    fn markers(s: &Session) -> Vec<String> {
        outline(s).iter().map(|n| format!("{} {}", n.marker, n.text)).collect()
    }

    const SAMPLE: &str = "١. الأول\nشرح الأول\nأ. فرع أول\nب. فرع ثان\n٢. الثاني\nأ. فرع ثالث\n٣. الثالث";

    #[test]
    fn the_outline_reads_levels_markers_and_explanations() {
        let s = doc(SAMPLE);
        let o = outline(&s);
        assert_eq!(o.len(), 7);
        assert_eq!((o[0].depth, o[0].numbered, o[0].marker.as_str(), o[0].text.as_str()), (0, true, "١.", "الأول"));
        assert_eq!((o[1].depth, o[1].numbered), (1, false), "an explanation sits one level under its item");
        assert_eq!((o[2].depth, o[2].marker.as_str()), (1, "أ."));
        assert_eq!(o[0].end, 3, "the first item holds its explanation and both sub-items");
        assert_eq!(o[4].end, 5);
        assert_eq!(o[6].end, 6);
    }

    #[test]
    fn removing_the_main_level_steps_the_sub_levels_up_and_renumbers_at_once() {
        let mut s = doc(SAMPLE);
        s.run("service.outline.removeLevel", &json!({"depth": 0})).unwrap();
        let t = texts(&s);
        assert_eq!(t[0], "الأول", "the main markers are gone, the words stay");
        assert!(t[2].starts_with("١."), "the first sub-item is now the main level: {t:?}");
        assert!(t[3].starts_with("٢."), "{t:?}");
        assert!(t[5].starts_with("٣."), "the next sub-item counts on: {t:?}");
        // one undo brings the numbering back
        s.undo();
        assert!(texts(&s)[0].starts_with("١."));
    }

    #[test]
    fn detaching_makes_an_explanation_and_the_list_renumbers() {
        let mut s = doc(SAMPLE);
        // «٢. الثاني» (block 4) leaves the numbering
        s.run("service.outline.detach", &json!({"index": 4})).unwrap();
        let t = texts(&s);
        assert_eq!(t[4], "الثاني");
        assert!(t[6].starts_with("٢."), "«٣.» became «٢.»: {t:?}");
        let o = outline(&s);
        let n = o.iter().find(|n| n.index == 4).unwrap();
        assert!(!n.numbered && n.depth >= 1, "an explanation under the item before it: {n:?}");
    }

    #[test]
    fn attaching_a_plain_paragraph_numbers_it_after_the_item_before() {
        let mut s = doc("١. الأول\nأ. فرع\nنص حر\n٢. الثاني");
        s.run("service.outline.attach", &json!({"index": 2})).unwrap();
        let t = texts(&s);
        assert!(t[2].starts_with("ب."), "a sibling of «أ.»: {t:?}");
        assert!(t[3].starts_with("٢."), "{t:?}");
    }

    #[test]
    fn demoting_and_promoting_an_item_with_its_sub_items() {
        let mut s = doc("١. الأول\n٢. الثاني\nأ. تحت الثاني\n٣. الثالث");
        // «٢.» goes under «١.»: it becomes «أ.» and its sub-item «أولاً.»
        s.run("service.outline.setLevel", &json!({"index": 1, "delta": 1})).unwrap();
        let t = texts(&s);
        assert!(t[1].starts_with("أ."), "{t:?}");
        assert!(t[2].starts_with("أولاً."), "{t:?}");
        assert!(t[3].starts_with("٢."), "the next main item counts on from ١.: {t:?}");
        s.run("service.outline.setLevel", &json!({"index": 1, "delta": -1})).unwrap();
        let t = texts(&s);
        assert!(t[1].starts_with("٢."), "{t:?}");
        assert!(t[2].starts_with("أ."), "{t:?}");
        assert!(t[3].starts_with("٣."), "{t:?}");
        // the first item cannot go further down; the top cannot go further up
        assert!(s.run("service.outline.setLevel", &json!({"index": 0, "delta": 1})).is_err());
        assert!(s.run("service.outline.setLevel", &json!({"index": 0, "delta": -1})).is_ok(), "already at the top: nothing changes");
    }

    #[test]
    fn dragging_an_item_before_another_renumbers_the_siblings() {
        let mut s = doc(SAMPLE);
        // «٣. الثالث» (block 6) goes before «٢. الثاني» (block 4)
        s.run("service.outline.move", &json!({"from": 6, "to": 4, "place": "before"})).unwrap();
        let t = texts(&s);
        assert_eq!(t[4].split_whitespace().last(), Some("الثالث"), "{t:?}");
        assert!(t[4].starts_with("٢."), "{t:?}");
        assert!(t[5].starts_with("٣.") && t[5].contains("الثاني"), "{t:?}");
        assert!(t[6].starts_with("أ."), "its sub-item moved with it: {t:?}");
    }

    #[test]
    fn dragging_an_item_into_another_makes_it_a_sub_item() {
        let mut s = doc(SAMPLE);
        // «٣. الثالث» (block 6) goes into «٢. الثاني» (block 4): after its sub-item, one level down
        s.run("service.outline.move", &json!({"from": 6, "to": 4, "place": "into"})).unwrap();
        let t = texts(&s);
        assert!(t[6].starts_with("ب.") && t[6].contains("الثالث"), "{t:?}");
        // an item with sub-items keeps its sub-items one level below it
        let mut s = doc(SAMPLE);
        s.run("service.outline.move", &json!({"from": 4, "to": 0, "place": "into"})).unwrap();
        let t = texts(&s);
        let o = outline(&s);
        assert_eq!(o.iter().filter(|n| n.depth == 0 && n.numbered).count(), 2, "{t:?}");
        assert!(t.iter().any(|x| x.starts_with("أولاً.") && x.contains("فرع ثالث")), "the moved item's sub-item went down two levels: {t:?}");
    }

    #[test]
    fn an_item_cannot_move_into_itself() {
        let mut s = doc(SAMPLE);
        assert!(s.run("service.outline.move", &json!({"from": 0, "to": 2, "place": "into"})).is_err());
        assert_eq!(texts(&s).len(), 7);
    }

    #[test]
    fn dragging_an_explanation_moves_just_that_paragraph() {
        let mut s = doc(SAMPLE);
        s.run("service.outline.move", &json!({"from": 1, "to": 4, "place": "after"})).unwrap();
        let t = texts(&s);
        assert_eq!(t.len(), 7);
        let at = t.iter().position(|x| x == "شرح الأول").unwrap();
        assert!(at > 4, "after «الثاني»: {t:?}");
    }

    #[test]
    fn a_plain_phrase_dragged_into_the_numbering_becomes_an_item() {
        let mut s = doc("١. الأول\nأ. فرع\n٢. الثاني\nجملة حرة");
        // «جملة حرة» (block 3) goes after «١.»'s sub-item «أ.» as a sibling
        s.run("service.outline.move", &json!({"from": 3, "to": 1, "place": "after", "numbered": true})).unwrap();
        let t = texts(&s);
        assert!(t[2].starts_with("ب.") && t[2].contains("جملة حرة"), "{t:?}");
        assert!(t[3].starts_with("٢."), "{t:?}");
    }

    #[test]
    fn editing_an_item_keeps_its_marker() {
        let mut s = doc(SAMPLE);
        s.run("service.outline.editText", &json!({"index": 2, "text": "فرع معدّل"})).unwrap();
        assert!(texts(&s)[2].starts_with("أ."));
        assert!(texts(&s)[2].ends_with("فرع معدّل"));
        assert_eq!(markers(&s)[2], "أ. فرع معدّل");
    }

    #[test]
    fn hostile_indexes_are_errors_not_panics() {
        let mut s = doc(SAMPLE);
        for id in ["move", "setLevel", "detach", "attach", "editText", "removeLevel"] {
            for bad in [
                json!({}),
                json!({"index": 999, "from": 999, "to": 999, "depth": 99, "delta": 1, "text": "x", "place": "into"}),
                json!({"index": -1}),
            ] {
                let _ = s.run(&format!("service.outline.{id}"), &bad);
            }
        }
        assert!(!texts(&s).is_empty());
        let _ = marker_text(0, 1);
    }
}
