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

/// The icon a command's button shows (custom groups, the Quick Access Toolbar, the kit
/// gallery): the one its ribbon button uses. Usually that is the id's last part (`format.bold`
/// → `bold`); this table holds every button whose icon is named differently, and the dialog
/// buttons by key (`ui.dialog:replace`). `tests::every_entry_has_a_drawn_icon` keeps it complete:
/// a command not here and without an icon of its own name would draw a lettered tile.
pub fn icon_for(key: &str) -> &str {
    const ICONS: &[(&str, &str)] = &[
        ("arrange.wrap", "wrapText"),
        ("chart.editData", "chartData"),
        ("chart.type", "changeChartType"),
        ("design.effects", "effectsDesign"),
        ("design.theme", "themes"),
        ("design.paragraphSpacing", "paraSpacing"),
        ("design.themeColors", "colors"),
        ("design.themeFonts", "fonts"),
        ("developer.buildingBlock", "ccGallery"),
        ("developer.checkBox", "ccCheckBox"),
        ("developer.comboBox", "ccComboBox"),
        ("developer.datePicker", "ccDate"),
        ("developer.dropDown", "ccDropDown"),
        ("developer.picture", "ccPicture"),
        ("developer.plainText", "ccPlainText"),
        ("developer.repeatingSection", "ccRepeat"),
        ("developer.richText", "ccRichText"),
        ("draw.highlighter", "highlight"),
        ("edit.formatPainter", "painter"),
        ("edit.repeat", "redo"),
        ("file.new", "blankPage"),
        ("format.border", "charborder"),
        ("format.changeCase", "case"),
        ("format.color", "fontcolor"),
        ("format.growFont", "grow"),
        ("format.shrinkFont", "shrink"),
        ("format.strikethrough", "strike"),
        ("insert.crossReference", "crossRef"),
        ("insert.icon", "icons"),
        ("insert.shape", "shapes"),
        ("insert.signatureLine", "signature"),
        ("layout.break", "breaks"),
        ("mailings.envelopes", "envelope"),
        ("mailings.findRecipient", "find"),
        ("mailings.insertField", "mergeField"),
        ("mailings.start", "mailMerge"),
        ("outline.close", "closeOutline"),
        ("outline.collapse", "outlineCollapse"),
        ("outline.demote", "outlineDemote"),
        ("outline.demoteToBody", "outlineBody"),
        ("outline.expand", "outlineExpand"),
        ("outline.moveDown", "outlineDown"),
        ("outline.moveUp", "outlineUp"),
        ("outline.promote", "outlinePromote"),
        ("para.ltr", "textLtr"),
        ("para.rtl", "textRtl"),
        ("picture.border", "picture"),
        ("picture.color", "picture"),
        ("picture.corrections", "picture"),
        ("picture.iconColor", "icons"),
        ("picture.reset", "picture"),
        ("picture.style", "picture"),
        ("picture.transparency", "picture"),
        ("references.citationStyle", "citation"),
        ("references.notes", "showNotes"),
        ("references.updateAuthorities", "update"),
        ("references.updateFigures", "update"),
        ("references.updateIndex", "update"),
        ("references.updateToc", "updateTable"),
        ("review.changes", "reviewingPane"),
        ("review.previousChange", "prevChange"),
        ("review.previousComment", "prevComment"),
        ("select.all", "select"),
        ("service.applyList", "serviceList"),
        ("service.external", "serviceEngine"),
        ("service.kashida", "kashidaMedium"),
        ("service.pageNumbers", "servicePages"),
        ("service.renumber", "serviceRenumber"),
        ("shape.breakLink", "textBoxUnlink"),
        ("shape.effects", "shapeEffects"),
        ("shape.fill", "shading"),
        ("shape.outline", "pen"),
        ("table.deleteCells", "deleteTable"),
        ("table.distributeColumns", "distributeCols"),
        ("table.draw", "drawTable"),
        ("table.insertColumnLeft", "insertLeft"),
        ("table.insertColumnRight", "insertRight"),
        ("table.insertRowAbove", "insertAbove"),
        ("table.insertRowBelow", "insertBelow"),
        ("table.selectCell", "selectTable"),
        ("table.split", "splitCells"),
        ("table.style", "styles"),
        ("table.viewGridlines", "gridlines"),
        ("tools.customizeRibbon", "more"),
        ("tools.templates", "documentTemplate"),
        ("ui.changePicture", "picture"),
        ("ui.dialog:about", "help"),
        ("ui.dialog:commands", "macros"),
        ("ui.linkTextBox", "textBoxLink"),
        ("ui.zotero.addEditBibliography", "bibliography"),
        ("ui.zotero.addEditCitation", "citation"),
        ("ui.zotero.refresh", "update"),
        ("ui.zotero.removeCodes", "unlinkCitations"),
        ("ui.zotero.setDocPrefs", "docPrefs"),
        ("view.altTextPane", "altText"),
        ("view.commentsPane", "showComments"),
        ("view.marks", "pilcrow"),
    ];
    if let Some((_, icon)) = ICONS.iter().find(|(c, _)| *c == key) {
        return icon;
    }
    // A dialog button is named after its dialog (`ui.dialog:replace` → `replace`).
    if let Some(("ui.dialog", name)) = key.split_once(':') {
        return name;
    }
    let id = key.split(':').next().unwrap_or(key);
    ICONS.iter().find(|(c, _)| *c == id).map_or_else(|| id.rsplit('.').next().unwrap_or(id), |(_, icon)| *icon)
}

#[cfg(test)]
pub fn is_button_id(id: &str) -> bool {
    id.starts_with("ui.")
        || wordcraft_engine::cmd::registry().get(id).is_some()
        || wordcraft_engine::catalog::entries().iter().any(|(_, _, _, cmd)| *cmd == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_unique_and_explanations_are_sentences() {
        let mut seen = std::collections::HashSet::new();
        for e in entries() {
            assert!(seen.insert(e.key), "{}: listed twice", e.key);
            let id = e.key.split(':').next().unwrap_or("");
            assert!(is_button_id(id), "{}: unknown command", e.key);
            assert!(e.explain.ends_with('.') && e.explain.split_whitespace().count() >= 4, "{}: one full sentence", e.key);
        }
    }

    /// Every entry's command maps to an icon the set draws (no lettered tiles in the gallery, the
    /// Quick Access Toolbar or a custom group).
    #[test]
    fn every_entry_has_a_drawn_icon() {
        let mut missing = Vec::new();
        for e in entries() {
            let icon = icon_for(e.key);
            if !crate::icons::known(icon) {
                missing.push(format!("{} → {icon}", e.key));
            }
        }
        assert!(missing.is_empty(), "entries without an icon:\n{}", missing.join("\n"));
    }
}
