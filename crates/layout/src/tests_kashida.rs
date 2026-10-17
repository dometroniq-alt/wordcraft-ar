//! Kashida justification (WordCraft AR): distributed and kashida-aligned Arabic paragraphs fill
//! their lines by lengthening one joining stroke per word; the text is untouched and the
//! stretches are drawn with the font's tatweel.

use super::*;
use wordcraft_doc::props::{Align, ParaProps};
use wordcraft_doc::{Pos, StoryRef};

fn lay(doc: &Document) -> DocLayout {
    let mut c = LayoutCache::new();
    layout(doc, &mut c, &LayoutOptions::default())
}

fn arabic_doc(text: &str, align: Align) -> Document {
    let mut d = Document::from_text(text);
    let at = Pos::body(0, 0);
    d.format_paragraphs(&at, &at, &|p: &mut ParaProps| {
        p.bidi = Some(true);
        p.align = Some(align);
    })
    .unwrap();
    d
}

fn first_para(l: &DocLayout) -> &ParaLayout {
    for p in &l.pages {
        for it in &p.items {
            if let Placed::Lines { story: StoryRef::Body, path, para, .. } = it
                && path.0 == vec![0]
            {
                return para;
            }
        }
    }
    panic!("paragraph not laid out");
}

#[test]
fn distributed_arabic_line_is_filled_with_kashida() {
    let text = "الإشراف على الأعمال";
    let l = lay(&arabic_doc(text, Align::Distribute));
    let pl = first_para(&l);
    let line = &pl.lines[0];
    assert!(!pl.kashida_joints.is_empty(), "the words have joining strokes");
    assert!(!line.kashida.is_empty(), "a distributed line gets kashida");
    // One stretch per word that has a joint (الإشراف, على, الأعمال).
    assert_eq!(line.kashida.len(), 3, "{:?}", line.kashida);
    for kf in &line.kashida {
        assert!(kf.width > 0.0 && kf.adv > 0.0 && kf.gid != 0, "{kf:?}");
    }
    // The line fills its width: the last x reaches the right edge (within rounding).
    let lo = line.vis.iter().map(|v| v.x).fold(f32::MAX, f32::min);
    let hi = line.vis.iter().map(|v| v.x + v.w).fold(f32::MIN, f32::max);
    assert!(
        (hi - lo) >= (line.right - line.left) - 1.0,
        "filled: {lo}..{hi} in {}..{} fills {:?} vis {:?}",
        line.left,
        line.right,
        line.kashida,
        line.vis
    );
    // The stretch sits before the last letter of «على» (after ل).
    let k = line
        .kashida
        .iter()
        .find(|f| text.get(pl.clusters[f.cluster].start..).is_some_and(|s| s.starts_with('ل') && s.starts_with("لى")))
        .map(|f| f.cluster);
    assert!(k.is_some(), "علــى stretches before its last letter: {:?}", line.kashida);
}

#[test]
fn justify_both_does_not_insert_kashida_but_low_kashida_does() {
    let text = "الإشراف على الأعمال كلها في المستوصف وفي الوحدات المسندة إليه من جميع الجهات والأقسام الطبية والإدارية والفنية في كل الأوقات والظروف";
    let l = lay(&arabic_doc(text, Align::Justify));
    let pl = first_para(&l);
    assert!(pl.lines.len() >= 2, "wraps");
    assert!(pl.lines[0].kashida.is_empty(), "plain justification widens spaces only");
    let l = lay(&arabic_doc(text, Align::LowKashida));
    let pl = first_para(&l);
    assert!(!pl.lines[0].kashida.is_empty(), "low kashida lengthens strokes");
    let last = pl.lines.last().unwrap();
    assert!(last.kashida.is_empty(), "the last line of a kashida-justified paragraph is not filled");
}

#[test]
fn kashida_draws_tatweel_glyphs_in_the_gap() {
    let text = "الإشراف على الأعمال";
    let d = arabic_doc(text, Align::Distribute);
    let l = lay(&d);
    let items = crate::display::page_display(&d, &l.pages[0], &crate::display::DisplayOptions::default());
    let tatweel = items
        .iter()
        .filter(|it| matches!(it, crate::display::Draw::Glyphs { text, .. } if text.chars().all(|c| c == '\u{0640}') && !text.is_empty()))
        .count();
    assert!(tatweel >= 3, "one tatweel run per stretch: {tatweel}");
}
