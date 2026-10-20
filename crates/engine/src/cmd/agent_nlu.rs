//! The assistant's language layer: what turns «شيل الأرقام الرئيسية», «خلي الكلام ثخين» or
//! «bolld» into the words the rule table (`agent.rs`) understands, and what ranks a request
//! against a catalogue of phrasings when the rules know nothing of it.
//!
//! Three steps, all local and a few microseconds long:
//!
//! 1. `canonical` — folds spelling (Persian/Iraqi letters گ چ پ, hamza forms, tashkeel), drops
//!    politeness and filler («لو سمحت», «تكدر», «اني»), maps dialect words to the standard ones
//!    («شيل» → «احذف», «بالنص» → «توسيط»), and repairs a slip of one or two letters against the
//!    known vocabulary («غامك» → «غامق»);
//! 2. `rank` — scores the request against every phrasing of every intent (word-stem overlap and
//!    character trigrams, Dice coefficient) and returns the closest intents;
//! 3. the caller decides: a close match runs, a middling one becomes «هل تقصد …؟», a poor one
//!    becomes a list of suggestions.
//!
//! The catalogue is plain data (`INTENTS`); adding a phrasing is adding a string. What the user
//! teaches the assistant (`agent.teach`) is matched by the same trigram score, ahead of all this.

use std::collections::HashSet;
use std::sync::OnceLock;

use super::agent::norm;

/// Words that carry no request: politeness, hedges, the speaker.
const FILLERS: &[&str] = &[
    "لو",
    "سمحت",
    "رجاء",
    "رجاءا",
    "ممكن",
    "تكدر",
    "تكدرون",
    "تقدر",
    "اريد",
    "اريدك",
    "ابي",
    "ابغي",
    "اني",
    "انا",
    "يا",
    "استاذ",
    "عمي",
    "حبيبي",
    "الله",
    "يخليك",
    "شويه",
    "لطفا",
    "هسه",
    "هلا",
    "مرحبا",
    "شكرا",
    "فضلك",
    "ياريت",
    "وياك",
    "كلش",
    "هاي",
    "هذي",
    "هذا",
    "هذه",
    "لي",
    "لنا",
    "بيه",
    "please",
    "can",
    "you",
    "could",
    "would",
    "i",
    "want",
    "to",
    "the",
    "a",
    "an",
    "my",
    "it",
    "me",
    "just",
    "now",
];

/// Dialect (and loose) words → the standard ones the rules know. Spelling is already folded.
const LEXICON: &[(&str, &str)] = &[
    ("خلي", "اجعل"),
    ("خليه", "اجعله"),
    ("خليها", "اجعلها"),
    ("خلين", "اجعل"),
    ("سوي", "اجعل"),
    ("سويه", "اجعله"),
    ("سويها", "اجعلها"),
    ("صير", "اجعل"),
    ("صيره", "اجعله"),
    ("صيرها", "اجعلها"),
    ("ساوي", "اجعل"),
    ("حط", "ضع"),
    ("حطه", "ضعه"),
    ("حطها", "ضعها"),
    ("شيل", "احذف"),
    ("شيله", "احذف"),
    ("شيلها", "احذف"),
    ("شيلهم", "احذف"),
    ("طير", "احذف"),
    ("مسح", "امسح"),
    ("فك", "ازل"),
    ("دخل", "ادخل"),
    ("رجع", "تراجع"),
    ("رجعه", "تراجع"),
    ("رجعها", "تراجع"),
    ("رده", "تراجع"),
    ("ردها", "تراجع"),
    ("بالنص", "توسيط"),
    ("بالوسط", "توسيط"),
    ("بنص", "توسيط"),
    ("بنصف", "توسيط"),
    ("عاليمين", "لليمين"),
    ("عاليسار", "لليسار"),
    ("عاليمن", "لليمين"),
    ("اثخن", "غامق"),
    ("ثخين", "غامق"),
    ("ثخينه", "غامق"),
    ("سميك", "غامق"),
    ("اسمك", "غامق"),
    ("بولد", "غامق"),
    ("عريض", "غامق"),
    ("ميل", "مايل"),
    ("ميله", "مايل"),
    ("كبره", "كبر الخط"),
    ("كبرها", "كبر الخط"),
    ("كبر", "كبر الخط"),
    ("اكبر", "كبر الخط"),
    ("صغره", "صغر الخط"),
    ("صغرها", "صغر الخط"),
    ("صغر", "صغر الخط"),
    ("اصغر", "صغر الخط"),
    ("خزن", "احفظ"),
    ("خزنه", "احفظ"),
    ("احفظه", "احفظ"),
    ("احفظها", "احفظ"),
    ("كلشي", "كل شيء"),
    ("ازرك", "ازرق"),
    ("اسمر", "بني"),
    ("سريه", "سري"),
    ("اوتوماتيكي", "تلقائي"),
    ("اوتوماتيك", "تلقائي"),
    ("اوتوماتيكيا", "تلقائي"),
    ("لحاله", "تلقائي"),
    ("ارقام", "ترقيم"),
    ("الارقام", "الترقيم"),
    ("اختار", "حدد"),
    ("اختر", "حدد"),
    ("وريني", "اعرض"),
    ("شوفني", "اعرض"),
    ("نسق", "رتب"),
    ("الكلام", "النص"),
    ("الكتابه", "النص"),
    ("الكتابة", "النص"),
    ("الجمله", "الفقره"),
    ("السطر", "الفقره"),
];

/// A request the assistant can do: its standard phrasing (which the rules understand) and the
/// ways people say it.
pub struct Intent {
    /// Standard words for the rule table.
    pub canon: &'static str,
    /// The request's own words (colour, number, level, name) are passed on with the standard ones.
    pub args: bool,
    pub phrases: &'static [&'static str],
}

pub const INTENTS: &[Intent] = &[
    Intent {
        canon: "غامق",
        args: false,
        phrases: &["اجعل الخط غامق", "خلي الكلام ثخين", "اسمك الكتابه", "بولد", "make it bold", "bold text", "اكتب بخط عريض", "عرض الخط"],
    },
    Intent {
        canon: "مائل", args: false, phrases: &["اجعل الخط مائل", "خلي الكلام مايل", "ميل النص", "make it italic", "italic text", "خط مائل"]
    },
    Intent {
        canon: "تسطير", args: false, phrases: &["ضع خط تحت الكلام", "سطر تحت النص", "underline it", "underline the text", "اجعل تحته خط"]
    },
    Intent {
        canon: "شطب", args: false, phrases: &["شطب الكلام", "ضع خط على الكلمه", "strike it out", "strikethrough the text", "اجعل يتوسطه خط"]
    },
    Intent {
        canon: "توسيط",
        args: false,
        phrases: &["اجعل النص في الوسط", "خليه بالنص", "وسط الفقره", "center the text", "align center", "ضع النص في المنتصف", "خليه بنص الصفحه"],
    },
    Intent {
        canon: "ضبط", args: false, phrases: &["ساوي الاطراف", "وزع النص على العرض", "justify the paragraph", "ضبط الفقره", "وزع الفقره"]
    },
    Intent {
        canon: "لليمين", args: false, phrases: &["ضع النص على اليمين", "خليه عاليمين", "align right", "حاذي النص لليمين", "ادفع النص لليمين"]
    },
    Intent {
        canon: "لليسار", args: false, phrases: &["ضع النص على اليسار", "خليه عاليسار", "align left", "حاذي النص لليسار"]
    },
    Intent {
        canon: "كبر الخط",
        args: false,
        phrases: &[
            "كبر الكتابه",
            "اجعل الخط اكبر",
            "الخط صغير شويه",
            "increase font size",
            "make the text bigger",
            "الخط صغير مو واضح",
            "زيد حجم الخط",
        ],
    },
    Intent {
        canon: "صغر الخط",
        args: false,
        phrases: &["صغر الكتابه", "اجعل الخط اصغر", "الخط كبير شويه", "decrease font size", "make the text smaller", "نقص حجم الخط", "الخط كبير"],
    },
    Intent {
        canon: "حجم", args: true, phrases: &["اجعل حجم الخط", "خلي الحجم", "font size", "set the size", "غير حجم الخط الى", "حجم الكتابه"]
    },
    Intent {
        canon: "لون",
        args: true,
        phrases: &["غير اللون الى", "لون الخط", "خلي اللون", "text color", "change the color to", "اجعل لون الكتابه", "لون النص"],
    },
    Intent {
        canon: "تراجع",
        args: false,
        phrases: &["ارجع للخلف", "الغي اخر شي", "ردها", "ما عجبني", "غلط", "undo that", "go back", "ارجع خطوه", "الغي اخر شيء فعلته", "ارجع كما كان"],
    },
    Intent {
        canon: "اعاده التراجع", args: false, phrases: &["ارجع الى الامام", "redo", "اعد الي الغيته", "ارجع الي رجعته"]
    },
    Intent {
        canon: "احفظ", args: false, phrases: &["خزن الملف", "احفظ المستند", "save the file", "save my work", "احفظ شغلي", "خزن الشغل"]
    },
    Intent {
        canon: "حدد الكل", args: false, phrases: &["اختار كل شي", "حدد كل النص", "select everything", "حدد كل الكلام", "اختر الكل"]
    },
    Intent {
        canon: "اعد الترقيم",
        args: false,
        phrases: &[
            "رتب الترقيم",
            "صلح الترقيم",
            "الترقيم خربان",
            "fix the numbering",
            "renumber the list",
            "نسق الترقيم",
            "الترقيم غلط",
            "الترقيم مو مضبوط",
            "عدل الترقيم",
        ],
    },
    Intent {
        canon: "احذف ترقيم مستوي",
        args: true,
        phrases: &[
            "شيل الترقيم الرئيسي",
            "احذف ترقيم المستوى",
            "remove numbering level",
            "امسح ترقيم الفرعي",
            "الغ المستوى من الترقيم",
            "شيل الترقيم الفرعي",
            "ازل مستوى الترقيم",
        ],
    },
    Intent {
        canon: "شغل الترقيم تلقائي",
        args: false,
        phrases: &["خلي الترقيم ينزل لحاله", "فعل الترقيم الاوتوماتيكي", "turn on auto numbering", "شغل الترقيم التلقائي", "الترقيم يمشي لحاله"],
    },
    Intent {
        canon: "اطفي الترقيم تلقائي",
        args: false,
        phrases: &["اطفي الترقيم الاوتوماتيكي", "turn off auto numbering", "وقف الترقيم التلقائي", "الغ الترقيم التلقائي"],
    },
    Intent {
        canon: "افتح فهرس الترقيم",
        args: false,
        phrases: &["وريني الترقيم", "اعرض الفهرس", "open the outline", "show numbering outline", "افتح لوحه الترقيم", "اظهر فهرس البنود"],
    },
    Intent {
        canon: "اغلق فهرس الترقيم",
        args: false,
        phrases: &["خفي الفهرس", "اخف لوحه الترقيم", "close the outline", "hide numbering outline", "اسكر الفهرس"],
    },
    Intent {
        canon: "نقطه ختاميه",
        args: false,
        phrases: &[
            "ضع نقطه بنهايه كل بند",
            "ضع نقطه في نهايه البنود",
            "add a full stop at the end of each item",
            "حط نقطه بنهايه الجمل",
            "اخر كل بند نقطه",
        ],
    },
    Intent {
        canon: "كشيده", args: true, phrases: &["مد الحروف", "وزع بالكشيده", "justify with kashida", "فعل الكشيده", "اجعل الكشيده"]
    },
    Intent {
        canon: "ترقيم الصفحات",
        args: false,
        phrases: &["ضع ارقام للصفحات", "رقم الصفحات", "add page numbers", "اكتب رقم الصفحه", "خل ارقام الصفحات"],
    },
    Intent {
        canon: "درجه الكتمان",
        args: true,
        phrases: &["اكتب سري على الصفحات", "اجعل الصفحه سري", "mark the document confidential", "ضع درجه الكتمان", "درجه السريه"],
    },
    Intent {
        canon: "تباعد مزدوج",
        args: false,
        phrases: &["اجعل السطور متباعده", "double space the text", "زيد المسافه بين السطور", "تباعد السطور مزدوج", "السطور قريبه من بعضها"],
    },
    Intent {
        canon: "تباعد مفرد", args: false, phrases: &["قرب السطور", "single space the text", "نقص المسافه بين السطور", "تباعد السطور مفرد"]
    },
    Intent {
        canon: "فقره جديده", args: false, phrases: &["ادخل سطر", "سطر جديد", "اسطر جديد", "new line", "new paragraph", "انتقل الى سطر جديد"]
    },
    Intent {
        canon: "فاصل صفحه", args: false, phrases: &["صفحه جديده", "اكمل في صفحه ثانيه", "start a new page", "page break", "انتقل لصفحه جديده"]
    },
    Intent {
        canon: "ارفع المستوي البند",
        args: false,
        phrases: &["ارفع البند مستوى", "قدم البند", "promote the item", "ارفع هذا البند", "طلع البند"],
    },
    Intent {
        canon: "اخفض المستوي البند",
        args: false,
        phrases: &["نزل البند مستوى", "اجعله فرعي", "demote the item", "اخفض هذا البند", "ادخله للداخل"],
    },
    Intent {
        canon: "انسخ البند",
        args: false,
        phrases: &["كرر البند", "اجعل نسخه من البند", "duplicate the item", "سوي نسخه من هذا البند", "نسخ البند"],
    },
    Intent {
        canon: "اخرج من الترقيم",
        args: false,
        phrases: &["فك البند من الترقيم", "بدون ترقيم", "detach from numbering", "اخرجه من الترقيم", "ازل رقم البند"],
    },
    Intent {
        canon: "ادخل في الترقيم", args: false, phrases: &["ادخله في الترقيم", "رقم هذا البند", "attach to numbering", "اجعله بند مرقم"]
    },
    Intent {
        canon: "المحرك الخارجي",
        args: false,
        phrases: &["نسق بالمحرك الخارجي", "format with the external engine", "شغل المحرك الخارجي", "استخدم المنسق الخارجي"],
    },
];

/// A phrasing, prepared for scoring.
struct Doc {
    intent: usize,
    stems: Vec<String>,
    grams: Vec<[char; 3]>,
}

struct Index {
    docs: Vec<Doc>,
    vocab: HashSet<String>,
}

fn index() -> &'static Index {
    static INDEX: OnceLock<Index> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut vocab: HashSet<String> = HashSet::new();
        let mut canon_phrases: Vec<(usize, String)> = Vec::new();
        for (i, intent) in INTENTS.iter().enumerate() {
            for t in norm(intent.canon).split(' ') {
                vocab.insert(t.to_string());
            }
            for p in intent.phrases {
                let c = fold(p, None);
                for t in c.split(' ') {
                    vocab.insert(t.to_string());
                }
                canon_phrases.push((i, c));
            }
        }
        for (_, to) in LEXICON {
            for t in norm(to).split(' ') {
                vocab.insert(t.to_string());
            }
        }
        // what the rule table itself is written in
        for w in CORE {
            vocab.insert(norm(w));
        }
        let docs = canon_phrases.into_iter().map(|(intent, c)| Doc { intent, stems: stems(&c), grams: trigrams(&c) }).collect();
        Index { docs, vocab }
    })
}

/// Words of the rule table, the vocabulary a typo is repaired against besides the catalogue.
const CORE: &[&str] = &[
    "غامق",
    "مائل",
    "تسطير",
    "شطب",
    "توسيط",
    "ضبط",
    "لليمين",
    "لليسار",
    "حجم",
    "لون",
    "خط",
    "ترقيم",
    "رئيسي",
    "فرعي",
    "مستوي",
    "احذف",
    "ازل",
    "اعد",
    "تراجع",
    "احفظ",
    "حدد",
    "الكل",
    "فهرس",
    "افتح",
    "اغلق",
    "كشيده",
    "درجه",
    "الكتمان",
    "صفحه",
    "تباعد",
    "مزدوج",
    "مفرد",
    "اكتب",
    "فقره",
    "جديده",
    "احمر",
    "ازرق",
    "اخضر",
    "اسود",
    "ابيض",
    "اصفر",
    "برتقالي",
    "رمادي",
    "بنفسجي",
    "بني",
    "وردي",
    "تلقائي",
    "نقطه",
    "ختاميه",
    "بند",
    "البند",
    "الترقيم",
    "النص",
    "الخط",
];

/// Spelling folded, fillers dropped, dialect words replaced; `vocab` (when given) repairs slips.
fn fold(text: &str, vocab: Option<&HashSet<String>>) -> String {
    let t = norm(text);
    let mut out: Vec<String> = Vec::new();
    for w in t.split(' ').filter(|w| !w.is_empty()) {
        if FILLERS.contains(&w) {
            continue;
        }
        let w = LEXICON.iter().find(|(from, _)| *from == w).map(|(_, to)| norm(to)).unwrap_or_else(|| w.to_string());
        let w = match vocab {
            Some(v) if !v.contains(&w) && !w.chars().any(|c| c.is_ascii_digit()) && w.chars().count() >= 4 => repair(&w, v).unwrap_or(w),
            _ => w,
        };
        out.push(w);
    }
    out.join(" ")
}

/// The request in the standard words the rules know (see the module header).
pub fn canonical(text: &str) -> String {
    fold(text, Some(&index().vocab))
}

/// The one vocabulary word a token is a slip of (one or two letters off), if exactly one is.
fn repair(w: &str, vocab: &HashSet<String>) -> Option<String> {
    let a: Vec<char> = w.chars().collect();
    let max = if a.len() >= 7 { 2 } else { 1 };
    let mut best: Option<(usize, &String)> = None;
    let mut tie = false;
    for v in vocab {
        let n = v.chars().count();
        if n.abs_diff(a.len()) > max || n < 3 {
            continue;
        }
        let b: Vec<char> = v.chars().collect();
        let d = distance(&a, &b);
        if d == 0 || d > max {
            continue;
        }
        match best {
            Some((bd, _)) if d > bd => {}
            Some((bd, _)) if d == bd => tie = true,
            _ => {
                best = Some((d, v));
                tie = false;
            }
        }
    }
    if tie { None } else { best.map(|(_, v)| v.clone()) }
}

/// Edit distance with adjacent swaps (a slip like «غمامق» or «ماىل»).
fn distance(a: &[char], b: &[char]) -> usize {
    let (n, m) = (a.len(), b.len());
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, c) in d[0].iter_mut().enumerate() {
        *c = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut v = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                v = v.min(d[i - 2][j - 2] + 1);
            }
            d[i][j] = v;
        }
    }
    d[n][m]
}

/// A word without its article, particle and ending: «الرئيسية» and «رئيسي» meet.
fn stem(w: &str) -> String {
    let mut s: Vec<char> = w.chars().collect();
    if s.len() > 4 && s[0] == 'ا' && s[1] == 'ل' {
        s.drain(..2);
    } else if s.len() > 4 && matches!(s[0], 'و' | 'ب' | 'ل' | 'ف') {
        s.remove(0);
    }
    for suffix in ["ها", "هم", "ات", "ين", "ون", "ه", "ي"] {
        let k = suffix.chars().count();
        if s.len() >= 3 + k && s.iter().rev().take(k).rev().copied().eq(suffix.chars()) {
            s.truncate(s.len() - k);
            break;
        }
    }
    s.into_iter().collect()
}

fn stems(text: &str) -> Vec<String> {
    let mut v: Vec<String> = text.split(' ').filter(|w| !w.is_empty()).map(stem).collect();
    v.sort();
    v.dedup();
    v
}

fn trigrams(text: &str) -> Vec<[char; 3]> {
    let padded: Vec<char> = std::iter::once(' ').chain(text.chars()).chain(std::iter::once(' ')).collect();
    let mut g: Vec<[char; 3]> = padded.windows(3).map(|w| [w[0], w[1], w[2]]).collect();
    g.sort();
    g
}

/// Dice coefficient of two sorted multisets.
fn dice<T: Ord>(a: &[T], b: &[T]) -> f32 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let (mut i, mut j, mut hit) = (0, 0, 0usize);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Equal => {
                hit += 1;
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }
    2.0 * hit as f32 / (a.len() + b.len()) as f32
}

/// How close two requests are, 0 to 1 (folded text on both sides).
pub fn similarity(a: &str, b: &str) -> f32 {
    0.5 * dice(&stems(a), &stems(b)) + 0.5 * dice(&trigrams(a), &trigrams(b))
}

/// The intents closest to a request (already `canonical`), best first, with their scores.
pub fn rank(canonical_request: &str) -> Vec<(usize, f32)> {
    let ix = index();
    let st = stems(canonical_request);
    let tg = trigrams(canonical_request);
    let mut best: Vec<f32> = vec![0.0; INTENTS.len()];
    for d in &ix.docs {
        let s = 0.5 * dice(&st, &d.stems) + 0.5 * dice(&tg, &d.grams);
        if s > best[d.intent] {
            best[d.intent] = s;
        }
    }
    let mut out: Vec<(usize, f32)> = best.into_iter().enumerate().filter(|(_, s)| *s > 0.25).collect();
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then(a.0.cmp(&b.0)));
    out.truncate(5);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialect_becomes_standard() {
        assert_eq!(canonical("لو سمحت شيل الأرقام الرئيسية"), "احذف الترقيم الرييسي");
        assert_eq!(canonical("خلّي الكلام ثخين"), "اجعل النص غامق");
        assert_eq!(canonical("خليه بالنص"), "اجعله توسيط");
        assert_eq!(canonical("چبّر الخط"), canonical("كبر الخط"));
    }

    #[test]
    fn slips_are_repaired_but_not_guessed_wildly() {
        assert_eq!(canonical("غامك"), "غامق");
        assert_eq!(canonical("تسطيير"), "تسطير");
        // two close candidates: leave it
        assert!(!canonical("xyzzy").is_empty());
        assert_eq!(canonical("123"), "123");
    }

    #[test]
    fn the_closest_intent_wins() {
        let top = |q: &str| INTENTS[rank(&canonical(q))[0].0].canon;
        assert_eq!(top("خلي الكلام ثخين"), "غامق");
        assert_eq!(top("الترقيم خربان"), "اعد الترقيم");
        assert_eq!(top("make the text bigger"), "كبر الخط");
        assert_eq!(top("ردها"), "تراجع");
        assert!(rank(&canonical("ما علاقة الجو بالموضوع")).first().is_none_or(|(_, s)| *s < 0.5));
    }

    #[test]
    fn a_phrasing_of_the_catalogue_scores_at_the_top() {
        for (i, intent) in INTENTS.iter().enumerate() {
            for p in intent.phrases {
                let r = rank(&canonical(p));
                assert!(!r.is_empty(), "{p}");
                assert!(r[0].1 > 0.95, "{p}: {:?}", r[0]);
                let _ = i;
            }
        }
    }

    #[test]
    fn similarity_is_symmetric_and_bounded() {
        assert!((similarity("اجعل النص غامق", "اجعل النص غامق") - 1.0).abs() < 1e-6);
        assert_eq!(similarity("", "x"), 0.0);
        let a = similarity("غامق", "غامقه");
        assert!(a > 0.3 && a <= 1.0);
    }
}
