//! Parser and lookup table for translation catalogs (`*.tsv`, format documented in `zh-hans.tsv`).
//! The format is PdfCraft's. Most entries are plain (empty context); a few use one of
//! [`CONTEXTS`] where the same English needs different wording in one place.
//!
//! Validation is strict (unknown escapes, unknown `@` contexts, placeholder, ellipsis and plural
//! form mismatches, duplicates), but a bad line is only skipped and reported: the rest of the
//! catalog still loads and the skipped string shows in English. The tests insist the bundled
//! catalogs have no errors at all.

use std::collections::{HashMap, HashSet};

/// A catalog entry as read from the file.
#[derive(Debug, PartialEq, Eq)]
pub struct Entry {
    pub context: String,
    pub source: String,
    pub translation: String,
}

/// The contexts a catalog entry may name, and where each applies.
/// - `preview`: sample text inside a rendered preview (the Design tab's style-set tiles), which
///   has to fit a small tile; it falls back to the plain entry.
/// - `template`: the name of a built-in document template (Home and New), which may need other
///   wording than the same English word elsewhere (the "Letter" template and the "Letter" paper
///   size); it falls back to the plain entry.
pub const CONTEXTS: &[&str] = &["preview", "template"];

/// A parsed catalog. Values are owned for the life of the process (catalogs are built once).
#[derive(Debug, Default)]
pub struct Catalog {
    /// English source → translation (the hot path, looked up every frame).
    plain: HashMap<String, String>,
    /// (context, English source) → translation, for the few entries that need one.
    contextual: HashMap<(String, String), String>,
    /// English source → the translation as written (logical order), for text that is wrapped
    /// into lines before it is put in the renderer's order (`rtl_text::paragraph`).
    logical: HashMap<String, String>,
}

fn unescape(text: &str) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some(other) => return Err(format!("unknown escape \\{other}; use \\n, \\t or \\\\")),
            None => return Err("trailing backslash; use \\\\ for a literal backslash".into()),
        }
    }
    Ok(out)
}

/// `{name}` placeholders of a template, sorted (translators may reorder them).
pub fn placeholders(text: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut rest = text;
    while let Some((_, after_open)) = rest.split_once('{') {
        let Some((name, after_close)) = after_open.split_once('}') else { break };
        names.push(name);
        rest = after_close;
    }
    names.sort_unstable();
    names
}

/// Check one entry against its English source.
fn validate(entry: &Entry) -> Result<(), String> {
    let Entry { context, source, translation } = entry;
    if !context.is_empty() && !CONTEXTS.contains(&context.as_str()) {
        return Err(format!("unknown context {context:?}; use one of {CONTEXTS:?} or leave the first column empty"));
    }
    if placeholders(source) != placeholders(translation) {
        return Err("placeholders differ from the English source".into());
    }
    if source.ends_with('…') != translation.ends_with('…') {
        return Err("trailing ellipsis differs from the English source".into());
    }
    Ok(())
}

fn parse_line(line: &str) -> Result<Entry, String> {
    let mut columns = line.split('\t');
    let (Some(context), Some(source), Some(translation), None) = (columns.next(), columns.next(), columns.next(), columns.next()) else {
        return Err("expected `context<TAB>source<TAB>translation`".into());
    };
    let entry = Entry { context: unescape(context)?, source: unescape(source)?, translation: unescape(translation)? };
    if entry.source.is_empty() || entry.translation.is_empty() {
        return Err("source and translation must be nonempty".into());
    }
    validate(&entry)?;
    Ok(entry)
}

/// Read the entries of a catalog file. Malformed lines and duplicates (the first entry wins) are
/// returned as errors and skipped, so a bad translation never breaks the UI.
pub fn parse_entries(text: &str) -> (Vec<Entry>, Vec<String>) {
    let mut entries = Vec::new();
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let line_no = index.saturating_add(1);
        match parse_line(line) {
            Ok(entry) if !seen.insert((entry.context.clone(), entry.source.clone())) => {
                errors.push(format!("line {line_no}: duplicate key {:?} / {:?}", entry.context, entry.source));
            }
            Ok(entry) => entries.push(entry),
            Err(error) => errors.push(format!("line {line_no}: {error}")),
        }
    }
    (entries, errors)
}

impl Catalog {
    /// Build a catalog from its valid entries; the errors of the skipped lines come back too.
    /// With `rtl`, each translation without placeholders is stored in the run order egui draws
    /// right-to-left text in ([`super::rtl_display`]); entries with placeholders are reordered by
    /// [`super::fmt`] once they are filled.
    pub fn parse_with(text: &str, rtl: bool) -> (Catalog, Vec<String>) {
        let (entries, errors) = parse_entries(text);
        let mut catalog = Catalog::default();
        for e in entries {
            let translation = if rtl && placeholders(&e.translation).is_empty() { super::rtl_display(&e.translation) } else { e.translation.clone() };
            if e.context.is_empty() {
                if translation != e.translation {
                    catalog.logical.insert(e.source.clone(), e.translation);
                }
                catalog.plain.insert(e.source, translation);
            } else {
                catalog.contextual.insert((e.context, e.source), translation);
            }
        }
        (catalog, errors)
    }

    /// How many plain entries the catalog holds.
    pub fn len(&self) -> usize {
        self.plain.len()
    }
    pub fn is_empty(&self) -> bool {
        self.plain.is_empty()
    }
    pub fn plain(&self, s: &str) -> Option<&str> {
        self.plain.get(s).map(String::as_str)
    }

    /// The translation of `s` as written in the catalog (logical character order), for a
    /// paragraph that is wrapped first and ordered line by line.
    pub fn logical(&self, s: &str) -> Option<&str> {
        self.logical.get(s).map(String::as_str).or_else(|| self.plain(s))
    }

    /// The entry for `s` in `context`, else the plain one.
    pub fn in_context(&self, context: &str, s: &str) -> Option<&str> {
        self.contextual.get(&(context.to_string(), s.to_string())).map(String::as_str).or_else(|| self.plain(s))
    }
}
