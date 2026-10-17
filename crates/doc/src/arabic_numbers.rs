//! Arabic list number formats (WordCraft AR): the abjad letters of military service writing
//! (أ، ب، جـ، د، هـ…), the alphabetical letters (أ، ب، ت، ث…), Arabic-Indic digits and the
//! ordinal words (أولاً، ثانياً، … حادي عشر…). All 1-based; 0 gives an empty string.

const ABJAD: [char; 28] = [
    'أ', 'ب', 'ج', 'د', 'ه', 'و', 'ز', 'ح', 'ط', 'ي', 'ك', 'ل', 'م', 'ن', 'س', 'ع', 'ف', 'ص', 'ق', 'ر', 'ش', 'ت', 'ث', 'خ', 'ذ', 'ض', 'ظ', 'غ',
];
const ALPHA: [char; 28] = [
    'أ', 'ب', 'ت', 'ث', 'ج', 'ح', 'خ', 'د', 'ذ', 'ر', 'ز', 'س', 'ش', 'ص', 'ض', 'ط', 'ظ', 'ع', 'غ', 'ف', 'ق', 'ك', 'ل', 'م', 'ن', 'ه', 'و', 'ي',
];
/// Tatweel (kashida) written after ج and ه in service-writing markers: «جـ.» «هـ.».
pub const TATWEEL: char = '\u{0640}';
const ORDINALS: [&str; 30] = [
    "أولاً", "ثانياً", "ثالثاً", "رابعاً", "خامساً", "سادساً", "سابعاً", "ثامناً", "تاسعاً", "عاشراً", "حادي عشر", "ثاني عشر", "ثالث عشر",
    "رابع عشر", "خامس عشر", "سادس عشر", "سابع عشر", "ثامن عشر", "تاسع عشر", "عشرون", "حادي وعشرون", "ثاني وعشرون", "ثالث وعشرون",
    "رابع وعشرون", "خامس وعشرون", "سادس وعشرون", "سابع وعشرون", "ثامن وعشرون", "تاسع وعشرون", "ثلاثون",
];

fn letters(table: &[char; 28], n: u32, tatweel: bool) -> String {
    if n == 0 {
        return String::new();
    }
    let i = ((n - 1) % 28) as usize;
    let reps = ((n - 1) / 28 + 1).min(30) as usize;
    let c = table.get(i).copied().unwrap_or('أ');
    let mut s: String = std::iter::repeat_n(c, reps).collect();
    if tatweel && matches!(c, 'ج' | 'ه') {
        s.push(TATWEEL);
    }
    s
}

/// Abjad letters as service writing numbers them: أ، ب، جـ، د، هـ، و، ز، ح، ط، ي، ك … غ, then doubled
/// (أأ، بب…). ج and ه carry a tatweel so the marker reads «جـ.» «هـ.».
pub fn abjad(n: u32) -> String {
    letters(&ABJAD, n, true)
}

/// Alphabetical letters: أ، ب، ت، ث، ج، ح، خ، د … ي, then doubled.
pub fn alpha(n: u32) -> String {
    letters(&ALPHA, n, false)
}

/// Arabic-Indic digits (U+0660–U+0669).
pub fn arabic_indic(n: u32) -> String {
    n.to_string().chars().map(|c| c.to_digit(10).and_then(|d| char::from_u32(0x0660 + d)).unwrap_or(c)).collect()
}

/// Ordinal words: أولاً، ثانياً، … عاشراً، حادي عشر، … ثلاثون; past thirty the Arabic-Indic number.
pub fn ordinal(n: u32) -> String {
    match n.checked_sub(1).and_then(|i| ORDINALS.get(i as usize)) {
        Some(s) => (*s).to_string(),
        None if n == 0 => String::new(),
        None => arabic_indic(n),
    }
}

/// Is this list level Arabic? (`lang` is the level run's complex-script language tag, e.g. `ar-IQ`.)
pub fn is_arabic_lang(lang: Option<&str>) -> bool {
    lang.is_some_and(|l| {
        let l = l.to_ascii_lowercase();
        l.starts_with("ar") || l.starts_with("fa") || l.starts_with("ur") || l.starts_with("ps")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abjad_follows_service_writing() {
        assert_eq!(abjad(1), "أ");
        assert_eq!(abjad(3), "جـ");
        assert_eq!(abjad(5), "هـ");
        assert_eq!(abjad(10), "ي");
        assert_eq!(abjad(11), "ك");
        assert_eq!(abjad(28), "غ");
        assert_eq!(abjad(29), "أأ");
        assert_eq!(abjad(0), "");
        assert!(!abjad(u32::MAX).is_empty());
    }

    #[test]
    fn alpha_is_alphabetical() {
        assert_eq!(alpha(3), "ت");
        assert_eq!(alpha(5), "ج");
        assert_eq!(alpha(26), "ه", "no tatweel in the alphabetical order");
    }

    #[test]
    fn digits_and_ordinals() {
        assert_eq!(arabic_indic(2026), "٢٠٢٦");
        assert_eq!(ordinal(1), "أولاً");
        assert_eq!(ordinal(11), "حادي عشر");
        assert_eq!(ordinal(30), "ثلاثون");
        assert_eq!(ordinal(31), "٣١");
        assert_eq!(ordinal(0), "");
        assert!(is_arabic_lang(Some("ar-IQ")) && !is_arabic_lang(Some("en-US")) && !is_arabic_lang(None));
    }
}
