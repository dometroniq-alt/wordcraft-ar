//! WordCraft AR's UI kit: **one place** that maps every command to what the user sees of it —
//! its icon, its thumbnail (a drawn picture of what the button does) and its one-sentence
//! explanation. The ribbon's tooltips, the Developer tab's gallery and the tests all read this
//! registry, and a sibling app (GridCraft, DeckCraft) can take the module as it is and fill its
//! own entries.
//!
//! Layout:
//! - [`draw`]: the drawing primitives every thumbnail is made of (pages, lines, arrows, objects).
//! - One file per ribbon tab (`home`, `insert`, …) holding that tab's `ENTRIES`.
//! - [`entries`] chains them; [`entry`] finds one by key; [`icon_for`] names a command's icon.
//!
//! Keys are command ids, plus `:variant` when one button runs the command with a parameter
//! that changes what it does (`service.kashida:low`, `service.external:pdf`), see [`key`].

use serde_json::Value;

pub mod draw;
pub use draw::{Dir, Draw, SIZE};

pub mod design;
pub mod draw_tab;
pub mod home;
pub mod insert;
pub mod layout;
pub mod mailings;
pub mod misc;
pub mod references;
pub mod review;
pub mod service;
pub mod table;
pub mod view;

/// One command as the user sees it: the key, what it does in one plain sentence (English
/// source; the Arabic catalog translates it) and how to draw it.
pub struct Entry {
    pub key: &'static str,
    pub explain: &'static str,
    pub draw: fn(&Draw),
}

impl Entry {
    pub const fn new(key: &'static str, explain: &'static str, draw: fn(&Draw)) -> Self {
        Self { key, explain, draw }
    }
}

/// Every entry of every tab, in ribbon order.
pub fn entries() -> impl Iterator<Item = &'static Entry> {
    [
        home::ENTRIES,
        insert::ENTRIES,
        draw_tab::ENTRIES,
        design::ENTRIES,
        layout::ENTRIES,
        references::ENTRIES,
        mailings::ENTRIES,
        review::ENTRIES,
        view::ENTRIES,
        service::ENTRIES,
        table::ENTRIES,
        misc::ENTRIES,
    ]
    .into_iter()
    .flatten()
}

/// The entry for a key (`id` or `id:variant`).
pub fn entry(key: &str) -> Option<&'static Entry> {
    entries().find(|e| e.key == key)
}

/// The visual-help key of a command and its parameters: the id, plus the parameter that picks
/// the variant a button shows (`:low` for the kashida level, `:selection` for Selection Only,
/// `:pdf` for the engine's PDF variant, `:kind` values for list and border kinds).
pub fn key(id: &str, params: &Value) -> String {
    let variant = match id {
        "service.kashida" => params.get("value").and_then(Value::as_str),
        "service.renumber" => params.get("scope").and_then(Value::as_str).filter(|s| *s == "selection"),
        "service.external" => params.get("pdf").and_then(Value::as_bool).filter(|b| *b).map(|_| "pdf"),
        // A ribbon button that opens a named dialog (Replace, Word Count, Zoom…).
        "ui.dialog" => params.get("name").and_then(Value::as_str),
        _ => None,
    };
    match variant {
        Some(v) => format!("{id}:{v}"),
        None => id.to_string(),
    }
}

/// The icon a command's button shows outside its own ribbon group (custom groups, the Quick
/// Access Toolbar, the kit gallery): the one its ribbon button uses, which is usually named
/// after the id's last part. Names the icon set doesn't have draw a lettered tile
/// ([`crate::icons::paint`]).
pub fn icon_for(id: &str) -> &str {
    const ICONS: &[(&str, &str)] = &[
        ("arrange.wrap", "wrapText"),
        ("draw.highlighter", "highlight"),
        ("format.border", "charborder"),
        ("format.color", "fontcolor"),
        ("format.growFont", "grow"),
        ("format.shrinkFont", "shrink"),
        ("format.strikethrough", "strike"),
        ("format.highlight", "highlight"),
        ("insert.signatureLine", "signature"),
        ("mailings.envelopes", "envelope"),
        ("mailings.insertField", "mergeField"),
        ("para.ltr", "textLtr"),
        ("para.rtl", "textRtl"),
        ("para.bullets", "bullets"),
        ("para.numbering", "numbering"),
        ("table.insertRowAbove", "insertAbove"),
        ("view.commentsPane", "showComments"),
        ("view.marks", "pilcrow"),
        ("edit.repeat", "redo"),
        ("file.new", "blankPage"),
        ("tools.customizeRibbon", "more"),
        ("service.renumber", "serviceRenumber"),
        ("service.applyList", "serviceList"),
        ("service.pageNumbers", "servicePages"),
        ("service.external", "serviceEngine"),
        ("service.kashida", "kashidaMedium"),
    ];
    ICONS.iter().find(|(c, _)| *c == id).map_or_else(|| id.rsplit('.').next().unwrap_or(id), |(_, icon)| *icon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_unique_and_explanations_are_sentences() {
        let mut seen = std::collections::HashSet::new();
        let reg = wordcraft_engine::cmd::registry();
        for e in entries() {
            assert!(seen.insert(e.key), "{}: listed twice", e.key);
            let id = e.key.split(':').next().unwrap_or("");
            assert!(reg.get(id).is_some() || id.starts_with("ui."), "{}: unknown command", e.key);
            assert!(e.explain.ends_with('.') && e.explain.split_whitespace().count() >= 4, "{}: one full sentence", e.key);
        }
    }
}
