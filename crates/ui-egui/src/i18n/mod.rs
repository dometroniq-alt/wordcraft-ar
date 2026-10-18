//! Interface languages (#8). Strings in the code stay English and are the lookup keys; a catalog
//! per language (`*.tsv`, format in `zh-hans.tsv`) maps them to display text when the UI is drawn.
//! Command ids, document text, file names, the control channel, the CLI and MCP never see
//! translated text, so scripts and agents are unaffected. A string without a translation shows in
//! English, so coverage can grow one entry at a time.
//!
//! The `language` setting is `auto` by default: the UI follows the operating system's language
//! (Windows display language, macOS preferred languages, `LANG`/`LC_*` elsewhere) when there is a
//! catalog for it, and English otherwise. File ▸ Options ▸ Interface language picks one by hand.
//!
//! This is the system PdfCraft and PhotoCraft use (`pdfcraft/crates/ui-egui/src/i18n`), so the
//! catalog format and habits carry over between the Crafting Apps.
//!
//! # Adding a language
//! 1. Add `xx.tsv` next to `zh-hans.tsv` (copy its header; translate from the *meaning* of the
//!    English text, clean-room: never from Microsoft Word's or another product's localisation).
//! 2. Add one row to [`LANGUAGES`]. Set its `editing_lang` only when `wordcraft-proof` can proof
//!    the language (Polish: `pl-PL`): new documents are then written in it.
//!
//! The Options dropdown, the system-language match and the catalog tests pick it up from there.
//!
//! # Looking strings up
//! - [`tl!`](crate::tl) / [`t`]: a string in the current language. [`tr`]: in a given one.
//! - [`tc`] / [`trc`]: worded for a context (`preview`), for text that must fit a tight spot.
//! - [`location`]: a ribbon location such as `Home › Font`, segment by segment.
//! - [`fmt`]: fill `{name}` placeholders after a lookup; translators may reorder them.

mod catalog;

use std::cell::Cell;
use std::sync::OnceLock;

use catalog::Catalog;

/// The `language` setting that follows the system.
pub const AUTO: &str = "auto";

/// One supported interface language.
pub struct LangInfo {
    /// BCP 47 code, lowercase (`ja`, `zh-hans`). Also the `language` setting's value.
    pub code: &'static str,
    /// The language's name in itself, shown in the Options dropdown.
    pub name: &'static str,
    /// Catalog file contents (empty for the built-in English).
    pub source: &'static str,
    /// Which CJK interface face comes first (Chinese text wants the Chinese face).
    pub prefer_hans: bool,
    /// The language (BCP 47) new blank documents are written in with this interface, for
    /// languages WordCraft can proof; `None` keeps WordCraft's default, US English.
    pub editing_lang: Option<&'static str>,
    catalog: OnceLock<Catalog>,
}

/// The registry. English first: it is the fallback and the source language.
pub static LANGUAGES: [LangInfo; 24] = [
    LangInfo { code: "en", name: "English", source: "", prefer_hans: false, editing_lang: None, catalog: OnceLock::new() },
    // WordCraft AR: Arabic interface, picked by an Arabic system language or in Options.
    LangInfo { code: "ar", name: "العربية", source: include_str!("ar.tsv"), prefer_hans: false, editing_lang: None, catalog: OnceLock::new() },
    // Simplified Chinese; `zh`, `zh-CN`, `zh-SG` and `zh-Hans-*` resolve here (see `candidates`).
    LangInfo {
        code: "zh-hans",
        name: "简体中文",
        source: include_str!("zh-hans.tsv"),
        prefer_hans: true,
        editing_lang: None,
        catalog: OnceLock::new(),
    },
    // Traditional Chinese (Taiwan vocabulary); `zh-TW`, `zh-HK`, `zh-MO` and `zh-Hant-*` resolve here.
    LangInfo {
        code: "zh-hant",
        name: "繁體中文",
        source: include_str!("zh-hant.tsv"),
        prefer_hans: true,
        editing_lang: None,
        catalog: OnceLock::new(),
    },
    LangInfo { code: "ja", name: "日本語", source: include_str!("ja.tsv"), prefer_hans: false, editing_lang: None, catalog: OnceLock::new() },
    LangInfo {
        code: "ko",
        name: "한국어",
        source: include_str!("ko.tsv"),
        prefer_hans: false, // 일본어처럼 CJK지만 한자 우선순위는 필요 없으므로 false
        editing_lang: None, // 현재 한국어 맞춤법(proofing) 지원이 없으므로 None
        catalog: OnceLock::new(),
    },
    LangInfo {
        code: "uk", name: "Українська", source: include_str!("uk.tsv"), prefer_hans: false, editing_lang: None, catalog: OnceLock::new()
    },
    // Spanish, neutral across Spain and Latin America; `es-ES`, `es-MX`, `es-419` … resolve here.
    LangInfo { code: "es", name: "Español", source: include_str!("es.tsv"), prefer_hans: false, editing_lang: None, catalog: OnceLock::new() },
    // Brazilian Portuguese; `pt-BR` and `pt-BR-*` resolve here. Plain `pt` and `pt-PT` have no
    // catalog yet (the European vocabulary differs), so they stay in English.
    LangInfo {
        code: "pt-br",
        name: "Português (Brasil)",
        source: include_str!("pt-br.tsv"),
        prefer_hans: false,
        editing_lang: Some("pt-BR"),
        catalog: OnceLock::new(),
    },
    // Serbian, Cyrillic script (the default per BCP 47 when no script is given); `sr`, `sr-RS`,
    // `sr-Cyrl-*` resolve here.
    LangInfo { code: "sr", name: "Српски", source: include_str!("sr.tsv"), prefer_hans: false, editing_lang: None, catalog: OnceLock::new() },
    // Serbian, Latin script; `sr-Latn-*` resolves here (see `candidates`'s generic prefix
    // matching — no special-casing needed, unlike Chinese's script-by-region fallback).
    LangInfo {
        code: "sr-latn",
        name: "Srpski (latinica)",
        source: include_str!("sr-latn.tsv"),
        prefer_hans: false,
        editing_lang: None,
        catalog: OnceLock::new(),
    },
    // Polish; `pl`, `pl-PL` and `pl_PL.UTF-8` resolve here. WordCraft proofs Polish (spelling,
    // grammar, hyphenation), so new documents are written in Polish with this interface.
    LangInfo {
        code: "pl",
        name: "Polski",
        source: include_str!("pl.tsv"),
        prefer_hans: false,
        editing_lang: Some("pl-PL"),
        catalog: OnceLock::new(),
    },
    LangInfo {
        code: "cs",
        name: "Čeština",
        source: include_str!("cs.tsv"),
        prefer_hans: false,
        editing_lang: Some("cs-CZ"),
        catalog: OnceLock::new(),
    },
    LangInfo { code: "et", name: "Eesti", source: include_str!("et.tsv"), prefer_hans: false, editing_lang: None, catalog: OnceLock::new() },
    // German, neutral across Germany, Austria and Switzerland; `de`, `de-DE`, `de-AT`, `de-CH` …
    // resolve here.
    LangInfo { code: "de", name: "Deutsch", source: include_str!("de.tsv"), prefer_hans: false, editing_lang: None, catalog: OnceLock::new() },
    // Russian; `ru`, `ru-RU`, `ru_RU.UTF-8` and other regions resolve here.
    LangInfo { code: "ru", name: "Русский", source: include_str!("ru.tsv"), prefer_hans: false, editing_lang: None, catalog: OnceLock::new() },
    LangInfo {
        code: "tr",
        name: "Türkçe",
        source: include_str!("tr.tsv"),
        prefer_hans: false,
        editing_lang: Some("tr-TR"),
        catalog: OnceLock::new(),
    },
    // Italian; `it-IT`, `it-CH` and `it-*` resolve here. New documents are written in
    // Italian: `wordcraft-proof` checks it (`ProofLang::It`).
    LangInfo {
        code: "it",
        name: "Italiano",
        source: include_str!("it.tsv"),
        prefer_hans: false,
        editing_lang: Some("it-IT"),
        catalog: OnceLock::new(),
    },
    // Hungarian; `hu`, `hu-HU` and `hu_HU.UTF-8` resolve here.
    LangInfo { code: "hu", name: "Magyar", source: include_str!("hu.tsv"), prefer_hans: false, editing_lang: None, catalog: OnceLock::new() },
    // French; `fr`, `fr-FR`, `fr_CA` and other regions resolve here.
    LangInfo { code: "fr", name: "Français", source: include_str!("fr.tsv"), prefer_hans: false, editing_lang: None, catalog: OnceLock::new() },
    // Bulgarian; `bg`, `bg-BG` and `bg_BG.UTF-8` resolve here.
    LangInfo {
        code: "bg", name: "Български", source: include_str!("bg.tsv"), prefer_hans: false, editing_lang: None, catalog: OnceLock::new()
    },
    // Vietnamese; `vi`, `vi-VN` and `vi_VN.UTF-8` resolve here.
    LangInfo { code: "vi", name: "Tiếng Việt", source: include_str!("vi.tsv"), prefer_hans: false, editing_lang: None, catalog: OnceLock::new() },
    // Swedish; sv, sv-SE, sv-FI and sv_SE.UTF-8 resolve here.
    LangInfo { code: "sv", name: "Svenska", source: include_str!("sv.tsv"), prefer_hans: false, editing_lang: None, catalog: OnceLock::new() },
    // Croatian; `hr`, `hr-HR` and `hr_HR.UTF-8` resolve here.
    LangInfo { code: "hr", name: "Hrvatski", source: include_str!("hr.tsv"), prefer_hans: false, editing_lang: None, catalog: OnceLock::new() },
];

impl LangInfo {
    fn catalog(&self) -> &Catalog {
        self.catalog.get_or_init(|| {
            let (catalog, errors) = Catalog::parse_with(self.source, self.code == "ar");
            for error in errors {
                log::warn!("{} interface catalog: {error}; that entry shows in English", self.code);
            }
            catalog
        })
    }
}

/// A language the UI can be shown in (a handle into [`LANGUAGES`]).
#[derive(Clone, Copy)]
pub struct Lang(&'static LangInfo);

impl std::fmt::Debug for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lang({})", self.0.code)
    }
}

impl PartialEq for Lang {
    fn eq(&self, other: &Self) -> bool {
        self.0.code == other.0.code
    }
}

impl Eq for Lang {}

impl Lang {
    pub const EN: Lang = Lang(&LANGUAGES[0]);

    pub fn code(self) -> &'static str {
        self.0.code
    }

    pub fn name(self) -> &'static str {
        self.0.name
    }

    pub fn prefers_hans(self) -> bool {
        self.0.prefer_hans
    }

    pub fn prefers_korean(self) -> bool {
        self.0.code == "ko"
    }

    /// WordCraft AR: the Arabic interface (its font order puts the Arabic face first).
    pub fn is_arabic(self) -> bool {
        self.0.code == "ar"
    }

    /// The language new blank documents are written in with this interface ([`LangInfo::editing_lang`]).
    pub fn editing_lang(self) -> Option<&'static str> {
        self.0.editing_lang
    }

    /// Is the interface written in CJK script (its own name is), so it needs a CJK face?
    pub fn uses_cjk(self) -> bool {
        self.0.name.chars().any(|c| ('\u{2E80}'..='\u{9FFF}').contains(&c) || ('\u{AC00}'..='\u{D7AF}').contains(&c))
    }

    /// The CJK interface text the fonts must cover with the Chinese (`hans`) or other CJK fallback
    /// order: the catalogs of the CJK languages using that order, and every language's own name (the
    /// Options list). The interface fonts check the embedded faces against it (#486).
    pub fn cjk_font_text(hans: bool) -> (Vec<&'static str>, String) {
        let catalogs = Lang::all().filter(|l| l.uses_cjk() && l.prefers_hans() == hans).map(|l| l.0.source).collect();
        (catalogs, Lang::all().map(Lang::name).collect::<Vec<_>>().join(" "))
    }

    /// The CJK interface catalogs using the Korean or Chinese writing-system preference.
    pub fn cjk_font_text_for(hans: bool, korean: bool) -> (Vec<&'static str>, String) {
        let catalogs = Lang::all().filter(|l| l.uses_cjk() && l.prefers_hans() == hans && l.prefers_korean() == korean).map(|l| l.0.source).collect();
        (catalogs, Lang::all().map(Lang::name).collect::<Vec<_>>().join(" "))
    }

    /// A language by its exact code (any case).
    pub fn from_code(code: &str) -> Option<Lang> {
        LANGUAGES.iter().find(|l| l.code.eq_ignore_ascii_case(code)).map(Lang)
    }

    /// Resolve the `language` setting: a language code, or [`AUTO`] (and anything unknown, e.g. a
    /// code from a newer version) to follow the system.
    pub fn from_pref(pref: &str) -> Lang {
        Lang::from_code(pref).unwrap_or_else(system_lang)
    }

    /// Every registered language.
    pub fn all() -> impl Iterator<Item = Lang> {
        LANGUAGES.iter().map(Lang)
    }

    pub fn catalog(self) -> &'static Catalog {
        self.0.catalog()
    }
}

/// The canonical `language` setting for user input: [`AUTO`] or a registered code (any case).
/// `None` for anything else, so callers keep the current setting.
pub fn normalize_pref(pref: &str) -> Option<&'static str> {
    if pref.eq_ignore_ascii_case(AUTO) {
        return Some(AUTO);
    }
    Lang::from_code(pref).map(Lang::code)
}

/// Candidate codes for a locale tag, most specific first: `zh_TW.UTF-8` → `zh-tw`, `zh-hant`, `zh`.
fn candidates(tag: &str) -> Vec<String> {
    let base = tag.split(['.', '@']).next().unwrap_or("").replace('_', "-").to_ascii_lowercase();
    let parts: Vec<&str> = base.split('-').filter(|p| !p.is_empty()).collect();
    let Some(&primary) = parts.first() else { return Vec::new() };
    let mut out = Vec::new();
    for n in (1..=parts.len()).rev() {
        out.push(parts.get(..n).unwrap_or_default().join("-"));
    }
    if primary == "zh" && !parts.iter().any(|p| matches!(*p, "hans" | "hant")) {
        // Chinese by region when no script is given.
        let script = if parts.iter().any(|p| matches!(*p, "tw" | "hk" | "mo")) { "zh-hant" } else { "zh-hans" };
        out.insert(out.len().saturating_sub(1), script.to_string());
    }
    out
}

/// The registered language for a locale tag such as `ja_JP.UTF-8` or `zh-Hant-TW`; `None` when
/// there is no catalog for it. `C`/`POSIX` mean English.
pub fn lang_from_tag(tag: &str) -> Option<Lang> {
    let cands = candidates(tag);
    if matches!(cands.first().map(String::as_str), Some("c" | "posix")) {
        return Some(Lang::EN);
    }
    cands.iter().find_map(|c| Lang::from_code(c))
}

/// The first of the system's preferred languages WordCraft has, in the user's order: someone
/// who prefers French, then Japanese, gets Japanese rather than English.
pub fn first_supported<'a>(tags: impl IntoIterator<Item = &'a str>) -> Option<Lang> {
    tags.into_iter().find_map(lang_from_tag)
}

/// The system language (looked up once). English when it can't be determined or isn't supported.
pub fn system_lang() -> Lang {
    // Tests drive the UI by its English labels whatever the machine's language is.
    if cfg!(test) {
        return Lang::EN;
    }
    static SYSTEM: OnceLock<Lang> = OnceLock::new();
    *SYSTEM.get_or_init(detect_system_lang)
}

fn detect_system_lang() -> Lang {
    // An explicit override, then the platform's preferred UI languages (Windows display language,
    // macOS preferred languages, `LC_ALL`/`LC_MESSAGES`/`LANG` on Unix, the browser on the web).
    if let Some(l) = std::env::var("WORDCRAFT_LANGUAGE").ok().and_then(|v| lang_from_tag(&v)) {
        return l;
    }
    let tags: Vec<String> = sys_locale::get_locales().take(16).collect();
    // WordCraft AR: the interface follows the system language (an English Windows keeps the
    // English interface, as its owner prefers; an Arabic Windows gets the Arabic one). The
    // document's text, numerals and kashida are Arabic regardless. File › Options › Interface
    // language switches at any time.
    first_supported(tags.iter().map(String::as_str)).unwrap_or(Lang::EN)
}

/// The system region (#838): it picks the paper File › New starts on, the default measurement
/// unit and the decimal separator, independently of the interface language (an English
/// interface in Germany still gets A4 and centimetres). `en-US` in tests.
pub fn system_region() -> wordcraft_engine::Region {
    if cfg!(test) {
        return wordcraft_engine::Region::default();
    }
    // The environment's locale variables (Unix), else the platform's first preferred locale (on
    // macOS it carries the region, as in `en-PL`; on the web it is the browser's language).
    wordcraft_engine::Region::from_locale_vars(|name| std::env::var(name).ok(), sys_locale::get_locale)
}

thread_local! {
    // Per thread: parallel test harnesses must not change each other's language.
    static CURRENT: Cell<Lang> = const { Cell::new(Lang::EN) };
}

/// Set the language the UI is drawn in (the app does this every frame from its setting).
pub fn set_current(lang: Lang) {
    CURRENT.set(lang);
}

/// The language the UI is drawn in.
pub fn current() -> Lang {
    CURRENT.get()
}

/// Does `lang` have a catalog entry for this string? (English never does: it is the source.)
pub fn has(lang: Lang, s: &str) -> bool {
    lang.catalog().plain(s).is_some()
}

/// `s` in the current language ([`tr`] with [`current`]).
pub fn t(s: &str) -> &str {
    tr(current(), s)
}

/// `s` in the current language as worded for `context` (one of `catalog::CONTEXTS`), falling
/// back to the plain translation and then to English.
pub fn tc<'a>(context: &str, s: &'a str) -> &'a str {
    trc(current(), context, s)
}

/// [`tc`] in a given language.
pub fn trc<'a>(lang: Lang, context: &str, s: &'a str) -> &'a str {
    #[cfg(test)]
    if pseudo::on() {
        return pseudo::mark(s);
    }
    lang.catalog().in_context(context, s).unwrap_or(s)
}

/// Lowercase translated interface labels and UI search queries in the current language.
/// Turkish has two distinct I pairs: `İ`/`i` and `I`/`ı`. Unicode's default casing turns `İ`
/// into `i` plus a combining dot, which would prevent `italik` from finding `İtalik`.
/// Command IDs and document text keep their own existing casing semantics.
pub fn lowercase(s: &str) -> String {
    if current().code() != "tr" {
        return s.to_lowercase();
    }
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'İ' => out.push('i'),
            'I' => out.push('ı'),
            _ => out.extend(c.to_lowercase()),
        }
    }
    out
}

/// A visual-help explanation ([`crate::ui_kit`]) in `lang`: translated where the catalog has
/// it (English and Arabic ship complete; other catalogs may add theirs), English otherwise. The
/// translation audit doesn't hold every catalog to these sentences.
pub fn explain(lang: Lang, s: &str) -> &str {
    #[cfg(test)]
    if pseudo::on() {
        return pseudo::mark_quiet(s);
    }
    lang.catalog().logical(s).unwrap_or(s)
}

/// Text that is part of a drawing (a thumbnail's sample words, a command id in the kit
/// gallery): never translated, and not interface text for the translation audit.
pub fn artwork(s: &str) -> &str {
    #[cfg(test)]
    if pseudo::on() {
        return pseudo::mark_quiet(s);
    }
    s
}

/// `s` in `lang`; strings without a translation come back unchanged.
pub fn tr(lang: Lang, s: &str) -> &str {
    #[cfg(test)]
    if pseudo::on() {
        return pseudo::mark(s);
    }
    lang.catalog().plain(s).unwrap_or(s)
}

/// A style's name as the interface shows it: built-in styles (`Normal`, `Heading 2`, `TOC 3`) in
/// the current language, styles the author made exactly as named. Display only: the document,
/// commands and saved files keep the English name.
pub fn style_name(style: &wordcraft_doc::Style) -> String {
    if style.builtin { builtin_style_name(&style.name) } else { style.name.clone() }
}

/// A built-in style's English name in the current language; numbered families go through one
/// template each (`Heading {n}`), and names without a translation stay English.
pub fn builtin_style_name(name: &str) -> String {
    let direct = t(name);
    if direct != name {
        return direct.to_string();
    }
    for (prefix, template) in [("Heading ", "Heading {n}"), ("TOC ", "TOC {n}"), ("Index ", "Index {n}")] {
        if let Some(n) = name.strip_prefix(prefix)
            && !n.is_empty()
            && n.chars().all(|c| c.is_ascii_digit())
        {
            return fmt(t(template), &[("n", n)]);
        }
    }
    name.to_string()
}

/// A ribbon location (`Home › Font`) in the current language, segment by segment.
pub fn location(loc: &str) -> String {
    loc.split(" › ").map(t).collect::<Vec<_>>().join(" › ")
}

/// A label with an action after a fixed English prefix, e.g. `Undo Typing`: both parts translated
/// where the catalog has them.
pub fn prefixed(prefix: &str, rest: &str) -> String {
    if rest.is_empty() {
        return t(prefix).to_string();
    }
    fmt(t(&format!("{prefix} {{action}}")), &[("action", t(rest))])
}

/// Fill `{name}` placeholders in one pass. Unknown placeholders stay as written, and inserted
/// values are never read as templates again, so a file name containing `{n}` stays intact.
pub fn fmt(template: &str, args: &[(&str, &str)]) -> String {
    let out = fill(template, args);
    if current().is_arabic() { rtl_display(&out) } else { out }
}

/// WordCraft AR: `s` with its runs in the order egui draws a right-to-left label correctly.
///
/// egui places a label's font runs left to right in logical order and shapes each run on its
/// own, so an Arabic run comes out right to left while a number or Latin word next to it stays
/// where it was: "الصفحة 1 من 2" would show the words right to left but the sequence
/// *word, 1, word, 2* read left to right. UAX #9's rule L2 gives the visual order; applied to
/// the runs of one embedding level (not to characters, which the shaper orders within a run)
/// it yields a string that egui draws as the bidi algorithm would. Each line of a multi-line
/// label is a paragraph of its own. Strings of one run come back unchanged.
pub fn rtl_display(s: &str) -> String {
    if !s.chars().any(|c| wordcraft_fonts::is_rtl(c) || is_arabic_digit(c)) {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    for (i, line) in s.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&rtl_display_line(line));
    }
    out
}

fn rtl_display_line(line: &str) -> String {
    let levels = wordcraft_layout::para::bidi_levels(line, true);
    if levels.len() != line.len() {
        return line.to_string();
    }
    // Runs of one level, as (start byte, level).
    let mut runs: Vec<(usize, u8)> = Vec::new();
    for (i, _) in line.char_indices() {
        let level = levels.get(i).copied().unwrap_or(1);
        if runs.last().is_none_or(|last| last.1 != level) {
            runs.push((i, level));
        }
    }
    if runs.len() < 2 {
        return line.to_string();
    }
    let run_levels: Vec<u8> = runs.iter().map(|r| r.1).collect();
    let mut out = String::with_capacity(line.len());
    for k in wordcraft_layout::para::visual_order(&run_levels) {
        let Some(&(start, level)) = runs.get(k) else { continue };
        let end = runs.get(k + 1).map(|r| r.0).unwrap_or(line.len());
        let Some(piece) = line.get(start..end) else { continue };
        if level % 2 == 0 {
            // Arabic-Indic digits share the Arabic face with the letters around them, so the
            // shaper reverses them with that run; written backwards, they read right.
            if piece.chars().all(|c| is_arabic_digit(c) || c == '٫' || c == '٬') {
                out.extend(piece.chars().rev());
            } else {
                out.push_str(piece);
            }
        } else {
            out.push_str(&rtl_run_for_faces(piece));
        }
    }
    out
}

/// One right-to-left bidi run as egui will draw it. The Arabic interface face has the Arabic
/// letters, the space and a few marks (`.,:!-«»`), so a bracket, a quote, a question mark or a
/// slash in Arabic text comes from the Latin face: epaint then cuts the run into one font run
/// per face and lays those out left to right, undoing the right-to-left order of the pieces.
/// Emitting the pieces in reverse order (the Arabic ones unchanged: the shaper reverses them)
/// restores it; a bracket drawn by the Latin face is mirrored by hand, as the shaper would
/// have in a right-to-left run.
fn rtl_run_for_faces(run: &str) -> String {
    let mut pieces: Vec<(bool, String)> = Vec::new();
    for c in run.chars() {
        let arabic = in_arabic_face(c);
        match pieces.last_mut() {
            Some((a, s)) if *a == arabic => s.push(c),
            _ => pieces.push((arabic, c.to_string())),
        }
    }
    if pieces.len() < 2 {
        return run.to_string();
    }
    let mut out = String::with_capacity(run.len());
    for (arabic, s) in pieces.iter().rev() {
        if *arabic {
            out.push_str(s);
        } else {
            out.extend(s.chars().rev().map(mirror));
        }
    }
    out
}

/// A character the bundled Arabic interface face draws (so it stays in the Arabic run).
fn in_arabic_face(c: char) -> bool {
    wordcraft_fonts::is_rtl(c) || is_arabic_digit(c) || matches!(c, ' ' | '.' | ',' | ':' | '!' | '-' | '«' | '»' | '\u{a0}' | '\u{2010}')
}

/// An Arabic-Indic or extended Arabic-Indic digit.
pub fn is_arabic_digit(c: char) -> bool {
    matches!(c as u32, 0x0660..=0x0669 | 0x06F0..=0x06F9)
}

/// The mirrored form of a paired bracket (what the shaper draws for it in right-to-left text).
fn mirror(c: char) -> char {
    match c {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        other => other,
    }
}

/// Fill `{name}` placeholders of `template` from `args`; unknown names stay as written.
fn fill(template: &str, args: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some((before, after_open)) = rest.split_once('{') {
        out.push_str(before);
        let Some((name, after_close)) = after_open.split_once('}') else {
            out.push('{');
            out.push_str(after_open);
            return out;
        };
        match args.iter().find(|(key, _)| *key == name) {
            Some((_, value)) => out.push_str(value),
            None => {
                out.push('{');
                out.push_str(name);
                out.push('}');
            }
        }
        rest = after_close;
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod audit;
#[cfg(test)]
pub(crate) mod pseudo;
#[cfg(test)]
mod tests;
