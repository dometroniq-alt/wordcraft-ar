//! Kashida justification (WordCraft AR): Arabic lines that are distributed (Ctrl+Shift+J) or
//! aligned with Word's `lowKashida` / `mediumKashida` / `highKashida` are filled by lengthening
//! the joining stroke between two letters — one stretch per word, chosen the way military
//! service writing wants it: before the last letter when the word allows it («علــى», «الطبــي»),
//! else after a seen-family letter («مســتندات»), then the other classic places. The stretch is
//! drawn with the font's own tatweel glyph (U+0640), repeated to fill the gap; the text is untouched.

use crate::para::{ClKind, Cluster};

/// Letters that join on both sides.
const DUAL: &str = "بتثجحخسشصضطظعغفقكلمنهيئپچژگکی";
/// Letters that join only to the letter before them (right-joining).
const RIGHT: &str = "اأإآدذرزوؤةىء";
const ALEFS: &str = "اأإآ";
const TATWEEL: char = '\u{0640}';

fn is_mark(c: char) -> bool {
    matches!(c, '\u{064B}'..='\u{0652}' | '\u{0670}' | '\u{0610}'..='\u{061A}' | '\u{06D6}'..='\u{06ED}')
}

/// First base (non-mark) character of a cluster's text.
fn base_char(text: &str, c: &Cluster) -> Option<char> {
    text.get(c.start..c.end)?.chars().find(|ch| !is_mark(*ch))
}

/// A stretch drawn in a line: the cluster whose box grows (the tatweel fills the gap on the far
/// side of its letter, between it and the next letter of the word), and the tatweel to draw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KashidaFill {
    pub cluster: usize,
    /// Extra width added after the cluster's own advance, points.
    pub width: f32,
    /// Style (face) of the cluster: the tatweel comes from it.
    pub style: u32,
    /// Tatweel glyph id and advance at the style's size, points.
    pub gid: u32,
    pub adv: f32,
}

/// Clusters after which a kashida may be inserted: the cluster's letter joins the next letter of
/// the same word (both Arabic, the lam-alef ligature kept whole). Sorted by cluster index.
pub fn joints(text: &str, clusters: &[Cluster]) -> Vec<u32> {
    let mut out = Vec::new();
    for (k, c) in clusters.iter().enumerate() {
        if c.kind != ClKind::Text {
            continue;
        }
        let Some(next) = clusters.get(k + 1) else { break };
        if next.kind != ClKind::Text {
            continue;
        }
        let (Some(a), Some(b)) = (base_char(text, c), base_char(text, next)) else { continue };
        if b == TATWEEL {
            // An existing tatweel is itself a joint: more may follow it.
            if DUAL.contains(a) {
                out.push(k as u32);
            }
            continue;
        }
        if !DUAL.contains(a) || !(DUAL.contains(b) || RIGHT.contains(b)) {
            continue;
        }
        if a == 'ل' && ALEFS.contains(b) {
            continue; // lam-alef stays a ligature
        }
        out.push(k as u32);
    }
    out
}

/// Priority of inserting after cluster `k` (lower is better), the service-writing scheme:
/// before the last letter, else after س ش ص ض, else before a Baa-family letter followed by
/// ر ز ي, else anywhere.
fn priority(text: &str, clusters: &[Cluster], k: usize, word_end: usize) -> u8 {
    let ch = clusters.get(k).and_then(|c| base_char(text, c)).unwrap_or(' ');
    // base letters after k inside the word (skipping tatweel clusters)
    let mut rest = (k + 1..word_end).filter_map(|i| clusters.get(i).and_then(|c| base_char(text, c))).filter(|c| *c != TATWEEL);
    let nxt = rest.next().unwrap_or(' ');
    let nxt2 = rest.next();
    let is_final = nxt2.is_none();
    if is_final {
        1
    } else if "سشصض".contains(ch) {
        2
    } else if "بتثنيئپ".contains(nxt) && nxt2.is_some_and(|c| "رزيىئ".contains(c)) {
        5
    } else {
        8
    }
}

/// One joint per word of the line (clusters `c0..c1`): the best by priority, ties to the one
/// nearer the word's end. Returns cluster indices, sorted.
pub fn best_per_word(text: &str, clusters: &[Cluster], joints: &[u32], c0: usize, c1: usize) -> Vec<usize> {
    let mut out = Vec::new();
    let mut k = c0;
    while k < c1 {
        // next word: a run of Text clusters
        while k < c1 && clusters.get(k).is_some_and(|c| c.kind != ClKind::Text) {
            k += 1;
        }
        let start = k;
        while k < c1 && clusters.get(k).is_some_and(|c| c.kind == ClKind::Text) {
            k += 1;
        }
        let end = k;
        if end <= start {
            continue;
        }
        let mut best: Option<(u8, usize)> = None;
        for j in joints.iter().map(|j| *j as usize).filter(|j| *j >= start && *j + 1 < end) {
            let pr = priority(text, clusters, j, end);
            if best.is_none_or(|(bp, bj)| pr < bp || (pr == bp && j > bj)) {
                best = Some((pr, j));
            }
        }
        if let Some((_, j)) = best {
            out.push(j);
        }
    }
    out
}

/// Share of a line's slack that goes to kashida for an alignment (the rest widens spaces).
/// Word's three kashida levels lengthen the strokes more and the spaces less; distributed text
/// is all kashida. `None`: this alignment never inserts kashida.
pub fn share(align: wordcraft_doc::props::Align) -> Option<f32> {
    use wordcraft_doc::props::Align;
    match align {
        Align::Distribute | Align::HighKashida => Some(1.0),
        Align::MediumKashida => Some(0.75),
        Align::LowKashida => Some(0.5),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clusters(text: &str) -> Vec<Cluster> {
        let mut v = Vec::new();
        let mut i = 0;
        for ch in text.chars() {
            let n = ch.len_utf8();
            if is_mark(ch) && let Some(last) = v.last_mut() {
                let c: &mut Cluster = last;
                c.end += n;
                i += n;
                continue;
            }
            v.push(Cluster {
                start: i,
                end: i + n,
                adv: 5.0,
                kind: if ch == ' ' { ClKind::Space } else { ClKind::Text },
                style: 0,
                g0: 0,
                g1: 0,
                break_after: ch == ' ',
                obj_h: 0.0,
                obj_d: 0.0,
                dot: false,
            });
            i += n;
        }
        v
    }

    #[test]
    fn joints_follow_arabic_joining() {
        let t = "على الطبي";
        let cl = clusters(t);
        let j = joints(t, &cl);
        // ع-ل, ل-ى ; ا does not join forward; ل-ط, ط-ب, ب-ي
        assert_eq!(j, vec![0, 1, 5, 6, 7]);
        let t2 = "أولاً";
        assert!(joints(t2, &clusters(t2)).is_empty(), "lam-alef kept whole, و does not join");
    }

    #[test]
    fn one_stretch_per_word_before_the_last_letter() {
        let t = "على الطبي مستندات";
        let cl = clusters(t);
        let j = joints(t, &cl);
        let best = best_per_word(t, &cl, &j, 0, cl.len());
        // على: after ل (before ى); الطبي: after ب (before ي); مستندات: after س (seen family) since the
        // last joint ا-ت does not exist (ا does not join forward) -> before the last letter is د-ا? د is right-joining, so no; after س
        let letters: Vec<char> = best.iter().map(|k| base_char(t, &cl[*k]).unwrap()).collect();
        assert_eq!(letters, vec!['ل', 'ب', 'س']);
    }

    #[test]
    fn existing_tatweel_is_a_joint() {
        let t = "جـ";
        let cl = clusters(t);
        assert_eq!(joints(t, &cl), vec![0]);
    }
}
