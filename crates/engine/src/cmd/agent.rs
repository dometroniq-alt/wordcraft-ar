//! «المساعد» — WordCraft AR's built-in agent: say what you want, in Arabic (any dialect, Iraqi
//! included) or English, and it runs the commands that do it. It is deliberately *not* a language
//! model: it reads a request with local, deterministic layers in microseconds, offline, so a
//! request is done before the next frame and always the same way.
//!
//! 1. what the user taught it (`agent.teach`, `agent.train`, a confirmed «هل تقصد؟», a picked
//!    suggestion) — matched by trigram similarity, always first;
//! 2. the rule table below (stems of Arabic/English words → command + parameters);
//! 3. the same rules over the request in standard words (`agent_nlu::canonical`: dialect words,
//!    spelling and slips repaired);
//! 4. the phrase index (`agent_nlu::INTENTS`): the closest phrasing of a known request.
//!
//! How sure it is decides what happens: sure → it runs; middling → it asks «هل تقصد …؟» and the
//! next «نعم» / «لا» / a number answers; unsure → the nearest commands come back as suggestions.
//! The conversation has a context (`AgentMemory`): «مرة ثانية» repeats, «وبعدين» predicts the next
//! step from what usually follows the last one (seeded workflows + what the user's own sessions
//! taught). What was learned is saved by `agent.save` and survives restarts.
//!
//! * `agent.interpret` — the plans for a request, nothing run (read only);
//! * `agent.ask` — understand and do, or ask, with context;
//! * `agent.teach` / `agent.train` / `agent.forget` / `agent.save` / `agent.load` — the learning.
//!
//! A learned ranker (Laya, a fast classifier) can sit behind `candidates` as a re-ranker; the
//! local index is the first choice because it is exact, instant and needs no Python.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::agent_nlu::{self as nlu, INTENTS};
use super::service::CLASSIFICATIONS;
use crate::{CmdError, CmdResult, CommandSpec, Session, p};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("view.agentPane", "Assistant", "Service › Assistant", |s, v| {
            s.view.agent_pane = p::bool(v, "value").unwrap_or(!s.view.agent_pane);
            Ok(json!({"value": s.view.agent_pane}))
        })
        .params(r#"{"value"?: bool (default: toggle)} — the Assistant pane beside the page: say what to do, it is done"#)
        .pure(),
        CommandSpec::new("agent.interpret", "Understand a Request", "Assistant", |s, v| {
            let text = p::req_str(v, "text")?;
            Ok(json!({"plans": interpret(s, text)}))
        })
        .pure()
        .params(r#"{"text": string} → {"plans": [{"command", "params", "label", "score"}]}: what the assistant would run for a request in Arabic or English, nothing run"#),
        CommandSpec::new("agent.ask", "Ask the Assistant", "Assistant", ask)
            .params(r#"{"text": string} — understands the request (with the conversation's context) and runs the commands it names, in order; → {"done": [{"command", "label", "ok", "error"?}], "next": [plans], "learned"?: true} when it did them, {"ask": {"said", "options": [{"score", "plans"}]}} when it is not sure (answer «نعم» / «لا» / a number in the next ask), {"suggestions": [plans]} when it did not understand"#),
        CommandSpec::new("agent.teach", "Teach the Assistant", "Assistant", teach_cmd)
            .params(r#"{"text": string, "command": id, "params"?: object} — from now on this phrasing runs this command (more calls with the same text add steps)"#)
            .pure(),
        CommandSpec::new("agent.train", "Train the Assistant", "Assistant", train_cmd)
            .params(r#"{"examples": [{"text": string, "command": id, "params"?: object}]} → {"taught": n, "skipped": n}: many phrasings at once, e.g. a corpus of how people write requests"#)
            .pure(),
        CommandSpec::new("agent.forget", "Make the Assistant Forget", "Assistant", forget_cmd)
            .params(r#"{"text"?: string, "all"?: true} → {"forgotten": n}: drop what was taught for a phrasing, or everything with `all`"#)
            .pure(),
        CommandSpec::new("agent.save", "Save the Assistant's Memory", "Assistant", |s, v| {
            let path = p::req_str(v, "path")?;
            let json = serde_json::to_string(&s.agent).map_err(|e| CmdError::Failed(e.to_string()))?;
            crate::io::write_file(path, json.as_bytes()).map_err(CmdError::Failed)?;
            Ok(json!({"taught": s.agent.taught.len()}))
        })
        .params(r#"{"path": string} — writes what the assistant learned (taught phrasings, what follows what) to a file"#)
        .pure(),
        CommandSpec::new("agent.load", "Load the Assistant's Memory", "Assistant", |s, v| {
            let path = p::req_str(v, "path")?;
            let bytes = crate::io::read_file_capped(path, 16 << 20, "memory file").map_err(CmdError::Failed)?;
            let text = String::from_utf8(bytes).map_err(|e| CmdError::Failed(e.to_string()))?;
            let loaded: AgentMemory = serde_json::from_str(&text).map_err(|e| CmdError::Failed(e.to_string()))?;
            s.agent.taught = loaded.taught;
            s.agent.follow = loaded.follow;
            s.agent.params = loaded.params;
            s.agent.cap();
            Ok(json!({"taught": s.agent.taught.len()}))
        })
        .params(r#"{"path": string} — loads what `agent.save` wrote"#)
        .pure(),
    ]
}

/// Where the learned memory lives between runs: the app's settings folder.
pub fn memory_path() -> Option<std::path::PathBuf> {
    wordcraft_control_key::settings_dir().map(|d| d.join("agent-memory.json"))
}

/// At or above this a request runs; between `ASK` and this the assistant asks «هل تقصد؟».
pub const SURE: f32 = 0.85;
pub const ASK: f32 = 0.5;

/// What the user taught: a phrasing (in standard words) and the commands it stands for.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Taught {
    pub text: String,
    pub plans: Vec<(String, Value)>,
}

/// A question the assistant asked and is waiting for the answer to.
#[derive(Clone, Debug)]
pub struct Pending {
    pub said: String,
    pub options: Vec<Vec<Plan>>,
}

/// What the assistant knows beyond the rules: taught phrasings, what usually follows what, and the
/// conversation so far (not saved).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentMemory {
    pub taught: Vec<Taught>,
    /// command → the commands that followed it, with counts.
    pub follow: BTreeMap<String, BTreeMap<String, u32>>,
    /// command → the parameters it last ran with (for predicting the next step).
    pub params: BTreeMap<String, Value>,
    #[serde(skip)]
    pub pending: Option<Pending>,
    #[serde(skip)]
    pub last: Vec<Plan>,
}

impl AgentMemory {
    const MAX_TAUGHT: usize = 5000;
    const MAX_COMMANDS: usize = 600;

    /// Keep the memory bounded, whatever was loaded or taught.
    fn cap(&mut self) {
        self.taught.truncate(Self::MAX_TAUGHT);
        while self.follow.len() > Self::MAX_COMMANDS {
            if let Some(k) = self.follow.keys().next_back().cloned() {
                self.follow.remove(&k);
            }
        }
        for m in self.follow.values_mut() {
            while m.len() > 40 {
                if let Some(k) = m.keys().next_back().cloned() {
                    m.remove(&k);
                }
            }
        }
        while self.params.len() > Self::MAX_COMMANDS {
            if let Some(k) = self.params.keys().next_back().cloned() {
                self.params.remove(&k);
            }
        }
    }

    /// Remember that `said` means `plans`.
    pub fn teach(&mut self, said: &str, plans: &[(String, Value)]) {
        let key = nlu::canonical(said);
        if key.is_empty() || plans.is_empty() {
            return;
        }
        if let Some(t) = self.taught.iter_mut().find(|t| t.text == key) {
            t.plans = plans.to_vec();
        } else if self.taught.len() < Self::MAX_TAUGHT {
            self.taught.push(Taught { text: key, plans: plans.to_vec() });
        }
    }

    /// After `prev` the user did `next`.
    fn learn_step(&mut self, prev: &str, next: &str, params: &Value) {
        if prev.is_empty() || next.is_empty() {
            return;
        }
        *self.follow.entry(prev.to_string()).or_default().entry(next.to_string()).or_insert(0) += 1;
        self.params.insert(next.to_string(), params.clone());
        self.cap();
    }
}

/// Workflows that go together, before the user's own habits say otherwise.
const SEEDS: &[(&str, &str)] = &[
    ("service.renumber", "service.finalDot"),
    ("service.finalDot", "service.classification"),
    ("service.classification", "service.pageNumbers"),
    ("service.pageNumbers", "file.save"),
    ("service.outline.removeLevel", "service.renumber"),
    ("select.all", "format.bold"),
    ("format.bold", "para.alignCenter"),
    ("edit.undo", "edit.redo"),
];

/// One thing to do.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Plan {
    pub command: String,
    pub params: Value,
    /// The command's name (English; the UI translates it).
    pub label: String,
    /// 1.0 for a rule, less for a name search.
    pub score: f32,
}

/// Lower-case the text and fold what Arabic writers spell several ways: the marks (tashkeel,
/// tatweel) go, the alef forms become one, ى is ي, ة is ه; Arabic-Indic digits are digits.
pub fn norm(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        let c = match c {
            '\u{064B}'..='\u{065F}' | '\u{0670}' | '\u{0640}' => continue,
            'أ' | 'إ' | 'آ' | 'ٱ' => 'ا',
            'ى' => 'ي',
            'ة' => 'ه',
            'ؤ' => 'و',
            'گ' | 'چ' | 'ک' => 'ك',
            'پ' => 'ب',
            'ڤ' => 'ف',
            'ی' => 'ي',
            'ئ' => 'ي',
            '٠'..='٩' => char::from_digit(c as u32 - '٠' as u32, 10).unwrap_or(c),
            '۰'..='۹' => char::from_digit(c as u32 - '۰' as u32, 10).unwrap_or(c),
            '٫' => '.',
            '،' | '؛' | '؟' | ',' | ';' | '?' | '!' | ':' | '"' | '«' | '»' | '(' | ')' => ' ',
            c => c.to_lowercase().next().unwrap_or(c),
        };
        out.push(c);
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The first number in the text: digits, or a small Arabic number word.
fn number(t: &str) -> Option<f64> {
    let mut best: Option<(usize, f64)> = None;
    let mut cur = String::new();
    let mut start = 0usize;
    for (i, c) in t.char_indices().chain(std::iter::once((t.len(), ' '))) {
        if c.is_ascii_digit() || (c == '.' && !cur.is_empty()) {
            if cur.is_empty() {
                start = i;
            }
            cur.push(c);
        } else if !cur.is_empty() {
            if let Ok(n) = cur.trim_end_matches('.').parse::<f64>() {
                best = Some((start, n));
            }
            break;
        }
    }
    const WORDS: [(&str, f64); 14] = [
        ("واحد", 1.0),
        ("اثنين", 2.0),
        ("ثلاثه", 3.0),
        ("اربعه", 4.0),
        ("خمسه", 5.0),
        ("سته", 6.0),
        ("سبعه", 7.0),
        ("ثمانيه", 8.0),
        ("تسعه", 9.0),
        ("عشره", 10.0),
        ("عشرين", 20.0),
        ("ثلاثين", 30.0),
        ("اربعين", 40.0),
        ("خمسين", 50.0),
    ];
    let word = WORDS.iter().filter_map(|(w, n)| t.find(w).map(|i| (i, *n))).min_by_key(|(i, _)| *i);
    match (best, word) {
        (Some((i, n)), Some((j, m))) => Some(if i <= j { n } else { m }),
        (Some((_, n)), None) | (None, Some((_, n))) => Some(n),
        _ => None,
    }
}

/// The level a request names: «الرئيسي» 0, «الفرعي» 1, «الثالث» 2, a digit N → N−1.
fn level_named(t: &str) -> usize {
    let has = |ws: &[&str]| ws.iter().any(|w| t.contains(norm(w).as_str()));
    if has(&["رئيسي", "الاول", "اولي", "main", "top"]) {
        0
    } else if has(&["فرعي", "الثاني", "sub", "second"]) {
        1
    } else if has(&["الثالث", "third"]) {
        2
    } else if has(&["الرابع", "fourth"]) {
        3
    } else {
        number(t).map(|n| (n as usize).saturating_sub(1)).unwrap_or(0)
    }
}

const COLORS: [(&[&str], &str); 10] = [
    (&["احمر", "red"], "FF0000"),
    (&["ازرق", "blue"], "0000FF"),
    (&["اخضر", "green"], "008000"),
    (&["اسود", "black"], "000000"),
    (&["ابيض", "white"], "FFFFFF"),
    (&["اصفر", "yellow"], "FFC000"),
    (&["برتقالي", "orange"], "FF8000"),
    (&["رمادي", "gray", "grey"], "808080"),
    (&["بنفسجي", "purple"], "7030A0"),
    (&["بني", "brown"], "7F3F00"),
];

const FONTS: [(&[&str], &str); 8] = [
    (&["تايمز", "times"], "Times New Roman"),
    (&["اريال", "arial"], "Arial"),
    (&["تاهوما", "tahoma"], "Tahoma"),
    (&["كاليبري", "calibri"], "Calibri"),
    (&["ابتوس", "aptos"], "Aptos"),
    (&["سيجوي", "segoe"], "Segoe UI"),
    (&["كوفي", "kufi"], "Noto Kufi Arabic"),
    (&["نسخ", "naskh"], "Noto Naskh Arabic"),
];

/// The caret's paragraph, for the commands that act on an item.
fn caret_block(s: &Session) -> usize {
    s.sel.focus.path.0.first().copied().unwrap_or(0) as usize
}

/// The rule table: the plans for a request in the standard words, all of them sure.
fn rules(s: &Session, text: &str) -> Vec<Plan> {
    let t = norm(text);
    let mut plans: Vec<Plan> = Vec::new();
    let mut add = |command: &str, params: Value| {
        let label = s.registry.get(command).map(|c| c.label.to_string()).unwrap_or_default();
        plans.push(Plan { command: command.to_string(), params, label, score: 1.0 });
    };
    let has = |ws: &[&str]| ws.iter().any(|w| t.contains(norm(w).as_str()));
    let words: Vec<&str> = t.split(' ').collect();
    let word = |ws: &[&str]| ws.iter().any(|w| words.contains(&norm(w).as_str()));
    let numbering = has(&["ترقيم", "رقم", "ترقم"]) && !has(&["صفحه", "صفحات"]);
    let remove = has(&["احذف", "حذف", "ازل", "ازاله", "شيل", "الغ", "الغاء", "الغي", "امسح", "remove", "delete"]);
    let on = has(&["شغل", "فعل", "تفعيل", "شغّل", "فعّل"]) || word(&["on", "enable"]);
    let off = has(&["اطفي", "اطفئ", "عطل", "تعطيل", "ايقاف", "وقف", "اغلق"]) || word(&["off", "disable"]);
    let mut consumed_undo = false;

    // ---- the numbering
    if numbering && remove && has(&["مستوي", "رئيسي", "فرعي", "الاول", "الثاني", "الثالث", "level", "main", "sub"]) {
        add("service.outline.removeLevel", json!({"depth": level_named(&t)}));
        consumed_undo = true;
    } else if (numbering && has(&["اعد", "اعاده", "جدد", "صحح", "نسق", "رتب", "اصلح"])) || word(&["renumber"]) {
        if has(&["التحديد", "المحدد", "selection"]) {
            add("service.renumber", json!({"scope": "selection"}));
        } else {
            add("service.renumber", json!({}));
        }
    }
    if numbering && has(&["تلقائي", "auto"]) && (on || off) {
        add("service.autoNumber", json!({"value": on && !off}));
    }
    if has(&["فهرس", "لوحه الترقيم", "لائحه الترقيم", "outline"]) && has(&["افتح", "اظهر", "اعرض", "open", "show", "اغلق", "اخف", "close", "hide"])
    {
        let close = has(&["اغلق", "اخف", "close", "hide"]);
        add("view.serviceOutlinePane", json!({"value": !close}));
    }
    if has(&["نقطه ختاميه", "نقطه نهايه", "نقطه في النهايه", "final dot", "end with a dot"]) {
        add("service.finalDot", json!({}));
    }
    if has(&["كشيد", "kashida"]) {
        let v = if has(&["ايقاف", "بدون", "الغ", "off"]) {
            "off"
        } else if has(&["منخفض", "قليل", "خفيف", "low"]) {
            "low"
        } else if has(&["متوسط", "medium"]) {
            "medium"
        } else if has(&["عال", "كثير", "قوي", "high"]) {
            "high"
        } else {
            "distribute"
        };
        add("service.kashida", json!({"value": v}));
    }
    if has(&["كتمان", "تصنيف", "classification"]) {
        // the longest name first: «سري للغاية» before «سري»
        let mut names: Vec<(String, &str)> = CLASSIFICATIONS.iter().map(|c| (norm(c), *c)).collect();
        names.sort_by_key(|(n, _)| std::cmp::Reverse(n.len()));
        if let Some((_, name)) = names.into_iter().find(|(n, _)| t.contains(n.as_str())) {
            add("service.classification", json!({"value": name}));
        }
    }
    if has(&["ترقيم الصفحات", "رقم الصفحه", "ارقام الصفحات", "page number"]) {
        add("service.pageNumbers", json!({}));
    }
    if has(&["servicew", "serviceword", "المحرك الخارجي", "المنسق الخارجي"]) {
        add("service.external", json!({"pdf": has(&["pdf"])}));
    }
    // ---- an item of the numbering (the one the caret is in)
    let item = caret_block(s);
    if has(&["بند", "الفقره", "العنوان", "item"]) || numbering {
        if has(&["ارفع المستوي", "مستوي اعلي", "قدم", "promote", "ارفع البند"]) {
            add("service.outline.setLevel", json!({"index": item, "delta": -1}));
        } else if has(&["اخفض المستوي", "مستوي اسفل", "اجعله فرعي", "ادخل للداخل", "demote", "اخفض البند"])
        {
            add("service.outline.setLevel", json!({"index": item, "delta": 1}));
        }
        if has(&["اخرج من الترقيم", "فك الترقيم", "بدون ترقيم", "detach", "اخرجه من الترقيم"]) {
            add("service.outline.detach", json!({"index": item}));
        } else if has(&["ادخل في الترقيم", "ادخله في الترقيم", "رقم هذا", "رقمه", "attach"]) {
            add("service.outline.attach", json!({"index": item}));
        }
        if has(&["انسخ البند", "كرر البند", "نسخه", "duplicate", "copy item"]) {
            let children = !has(&["بدون فروع", "بدون الفروع", "بدون ضمنيات", "بدون ضمنياته", "without"]);
            add("service.outline.copy", json!({"from": item, "to": item, "place": "after", "children": children}));
        }
    }
    // ---- history, files, selection
    if !consumed_undo && (t.starts_with("تراجع") || word(&["undo"]) || has(&["الغ اخر", "ارجع خطوه", "الغاء اخر"])) {
        add("edit.undo", json!({}));
    }
    if has(&["اعاده التراجع", "اعد التراجع", "تقدم خطوه"]) || word(&["redo"]) {
        add("edit.redo", json!({}));
    }
    if has(&["حدد الكل", "اختر الكل", "تحديد الكل", "حدد كل شيء"]) || t.contains("select all") {
        add("select.all", json!({}));
    }
    if has(&["احفظ", "حفظ المستند"]) && !has(&["باسم"]) || word(&["save"]) {
        add("file.save", json!({}));
    }
    if has(&["فقره جديده", "سطر جديد", "اسطر جديد"]) || t.contains("new paragraph") {
        add("text.newParagraph", json!({}));
    }
    if has(&["فاصل صفحه", "صفحه جديده"]) || t.contains("page break") {
        add("text.pageBreak", json!({}));
    }
    // ---- character formatting
    let style_word = |w: &str| has(&[w]);
    if has(&["غامق", "عريض", "بولد"]) || word(&["bold"]) {
        add("format.bold", json!({}));
    }
    if has(&["مائل", "ماءل", "مايل", "ايتالك"]) || word(&["italic"]) {
        add("format.italic", json!({}));
    }
    if has(&["تسطير", "خط تحت", "تحته خط", "underline"]) {
        add("format.underline", json!({}));
    }
    if has(&["شطب", "يتوسطه خط", "strikethrough"]) {
        add("format.strikethrough", json!({}));
    }
    if has(&["حجم"])
        && let Some(n) = number(&t)
    {
        add("format.size", json!({"size": n}));
    } else if has(&["كبر الخط", "كبّر الخط", "اكبر", "تكبير", "bigger", "grow"]) {
        add("format.growFont", json!({}));
    } else if has(&["صغر الخط", "اصغر", "تصغير", "smaller", "shrink"]) {
        add("format.shrinkFont", json!({}));
    }
    if (has(&["لون"]) || word(&["color", "colour"]))
        && let Some((_, hex)) = COLORS.iter().find(|(names, _)| names.iter().any(|n| t.contains(n)))
    {
        add("format.color", json!({"color": hex}));
    }
    if (word(&["خط", "الخط", "font"]) || has(&["نوع الخط"])) && !has(&["حجم", "لون", "تكبير", "تصغير", "size", "color"]) && !style_word("خط تحت")
    {
        let named = FONTS.iter().find(|(names, _)| names.iter().any(|n| t.contains(n))).map(|(_, f)| f.to_string());
        let rest = || {
            // «خط X» / «الخط X» / «font X»: the name as typed, if it is one (a Latin word, or an
            // Arabic word after a plain «خط» that isn't a style or a size word)
            let toks: Vec<&str> = text.split([' ', '،']).filter(|w| !w.is_empty()).collect();
            toks.windows(2).find_map(|w| {
                let head = norm(w[0]);
                let next = w[1];
                let latin = next.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
                let arabic_ok = head == "خط"
                    && !["تحت", "عريض", "غامق", "مائل", "جديد", "جديده", "اكبر", "اصغر", "كبير", "صغير", "اخر"].contains(&norm(next).as_str());
                (matches!(head.as_str(), "خط" | "الخط" | "font") && (latin || arabic_ok)).then(|| next.to_string())
            })
        };
        if let Some(name) = named.or_else(rest) {
            add("format.font", json!({"name": name}));
        }
    }
    // ---- the paragraph
    if has(&["توسيط", "وسط", "center", "centre"]) {
        add("para.alignCenter", json!({}));
    } else if word(&["ضبط", "ضبطه", "ضبطها", "توزيع", "justify"]) || has(&["ضبط الفقره", "ضبط النص"]) {
        add("para.align", json!({"value": "justify"}));
    } else if has(&["محاذاه لليمين", "لليمين", "يمين", "right"]) {
        add("para.alignRight", json!({}));
    } else if has(&["محاذاه لليسار", "لليسار", "يسار", "left"]) {
        add("para.alignLeft", json!({}));
    }
    if has(&["تباعد", "مسافه بين الاسطر", "line spacing"]) {
        if has(&["مزدوج", "double"]) {
            add("para.double", json!({}));
        } else if has(&["مفرد", "single"]) {
            add("para.single", json!({}));
        } else if let Some(n) = number(&t) {
            add("para.lineSpacing", json!({"value": n}));
        }
    }
    // ---- typing
    if let Some(typed) = typed_text(text) {
        add("text.insert", json!({"text": typed}));
    }
    plans
}

/// One way to read a request, with how sure the reading is.
#[derive(Clone, Debug)]
pub struct Candidate {
    pub score: f32,
    pub plans: Vec<Plan>,
}

fn with_score(mut plans: Vec<Plan>, score: f32) -> Vec<Plan> {
    for p in &mut plans {
        p.score = score;
    }
    plans
}

/// The readings of a request, best first (see the module header for the layers).
pub fn candidates(s: &Session, text: &str) -> Vec<Candidate> {
    let t = norm(text);
    if t.is_empty() {
        return Vec::new();
    }
    let canon = nlu::canonical(text);
    let mut out: Vec<Candidate> = Vec::new();
    // 1. what the user taught
    let mut taught: Vec<(f32, &Taught)> =
        s.agent.taught.iter().map(|k| (nlu::similarity(&canon, &k.text), k)).filter(|(sim, _)| *sim >= 0.75).collect();
    taught.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    for (sim, k) in taught.into_iter().take(3) {
        let plans: Vec<Plan> = k
            .plans
            .iter()
            .filter_map(|(c, params)| {
                s.registry.get(c).map(|spec| Plan { command: c.clone(), params: params.clone(), label: spec.label.to_string(), score: 1.0 })
            })
            .collect();
        if !plans.is_empty() {
            out.push(Candidate { score: if sim >= 0.97 { 1.0 } else { sim.min(0.9) }, plans });
        }
    }
    // 2. the rules, as typed; 3. the rules over the standard words
    let exact = rules(s, text);
    if !exact.is_empty() {
        out.push(Candidate { score: 1.0, plans: exact });
    } else if canon != t {
        let plans = rules(s, &canon);
        if !plans.is_empty() {
            out.push(Candidate { score: 0.95, plans: with_score(plans, 0.95) });
        }
    }
    // 4. the closest phrasings of known requests
    if out.is_empty() {
        for (i, score) in nlu::rank(&canon).into_iter().take(4) {
            let Some(intent) = INTENTS.get(i) else { continue };
            let q = if intent.args { format!("{} {}", intent.canon, canon) } else { intent.canon.to_string() };
            let plans = rules(s, &q);
            if !plans.is_empty() {
                let score = score.min(0.92);
                out.push(Candidate { score, plans: with_score(plans, score) });
            }
        }
    }
    // 5. command names, for what nothing else knew
    if out.is_empty() {
        for p in by_name(s, &t) {
            out.push(Candidate { score: p.score, plans: vec![p] });
        }
    }
    out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// The plans for a request: the best reading (for the UI's live preview).
pub fn interpret(s: &Session, text: &str) -> Vec<Plan> {
    candidates(s, text).into_iter().next().map(|c| c.plans).unwrap_or_default()
}

/// «اكتب …» / «write …»: the words after the verb, as typed.
fn typed_text(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    for verb in ["اكتب ", "أكتب ", "ادخل النص ", "ادرج النص ", "write ", "type "] {
        if let Some(i) = lower.find(verb) {
            let rest = text.get(i + verb.len()..)?.trim().trim_matches(['"', '«', '»']).trim();
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    None
}

/// The commands whose English names share words with the request, for what the rules don't know.
fn by_name(s: &Session, t: &str) -> Vec<Plan> {
    let q: Vec<&str> = t.split(' ').filter(|w| w.len() > 2 && w.is_ascii()).collect();
    if q.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(f32, &crate::CommandSpec)> = s
        .registry
        .all()
        .iter()
        .filter_map(|c| {
            let label = c.label.to_lowercase();
            let hit = q.iter().filter(|w| label.split(' ').any(|l| l.starts_with(**w) || w.starts_with(l) && l.len() > 2)).count();
            (hit > 0).then(|| (hit as f32 / label.split(' ').count().max(q.len()) as f32, c))
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal).then(a.1.id.cmp(b.1.id)));
    scored
        .into_iter()
        .take(5)
        .map(|(score, c)| Plan { command: c.id.to_string(), params: json!({}), label: c.label.to_string(), score: (score * 0.6).min(0.6) })
        .collect()
}

fn is_one_of(t: &str, words: &[&str]) -> bool {
    let toks: Vec<&str> = t.split(' ').collect();
    toks.len() <= 3 && toks.iter().any(|w| words.contains(w))
}

fn is_yes(t: &str) -> bool {
    is_one_of(
        t,
        &[
            "نعم", "ايه", "اي", "ايوه", "ايوا", "اه", "اها", "صح", "صحيح", "تمام", "اكيد", "اجل", "yes", "y", "ok", "okay", "yep", "yeah", "sure",
            "اكو",
        ],
    )
}

fn is_no(t: &str) -> bool {
    is_one_of(t, &["لا", "كلا", "no", "nope", "n", "ما", "مو", "لاا"])
}

fn is_repeat(t: &str) -> bool {
    const REPEAT: &[&str] = &[
        "كرر",
        "كررها",
        "كرره",
        "مره ثانيه",
        "مره اخري",
        "كمان مره",
        "نفسه",
        "نفس الشي",
        "نفس الشيء",
        "اعدها",
        "هم مره",
        "وكمان",
        "كمان",
        "ثانيه",
        "again",
        "repeat",
        "do it again",
        "once more",
    ];
    REPEAT.contains(&t)
}

fn is_next(t: &str) -> bool {
    const NEXT: &[&str] =
        &["وبعدين", "وبعد", "وبعدها", "التالي", "ماذا بعد", "شنو بعد", "شنو بعدين", "الخطوه التاليه", "next", "what next", "what now", "then"];
    NEXT.contains(&t)
}

/// The steps likely to come after `last`: the user's own habits first, then the seeded workflows.
pub fn predict(s: &Session, last: &str) -> Vec<Plan> {
    let mut score: BTreeMap<&str, u32> = BTreeMap::new();
    for (a, b) in SEEDS {
        if *a == last {
            *score.entry(b).or_insert(0) += 1;
        }
    }
    if let Some(m) = s.agent.follow.get(last) {
        for (k, n) in m {
            *score.entry(k.as_str()).or_insert(0) += n.saturating_mul(2);
        }
    }
    let mut v: Vec<(&str, u32)> = score.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    v.into_iter()
        .filter_map(|(c, _)| {
            let spec = s.registry.get(c)?;
            let params = s.agent.params.get(c).cloned().unwrap_or_else(|| json!({}));
            Some(Plan { command: c.to_string(), params, label: spec.label.to_string(), score: 0.6 })
        })
        .take(3)
        .collect()
}

/// Run the plans in order; the first failure stops the rest. Remembers the run for the next
/// request's context and, when `learn` and everything worked, what `said` means.
fn run_plans(s: &mut Session, said: &str, plans: &[Plan], learn: bool) -> CmdResult {
    let mut done = Vec::new();
    let mut all_ok = true;
    let mut prev = s.agent.last.last().map(|p| p.command.clone()).unwrap_or_default();
    for plan in plans {
        match s.run(&plan.command, &plan.params) {
            Ok(_) => {
                done.push(json!({"command": plan.command, "label": plan.label, "ok": true}));
                // doing the same thing again is not a step to predict
                if prev != plan.command {
                    s.agent.learn_step(&prev, &plan.command, &plan.params);
                }
                prev = plan.command.clone();
            }
            Err(e) => {
                done.push(json!({"command": plan.command, "label": plan.label, "ok": false, "error": e.to_string()}));
                all_ok = false;
                break;
            }
        }
    }
    s.agent.last = plans.to_vec();
    let mut reply = json!({"done": done});
    if learn && all_ok {
        let steps: Vec<(String, Value)> = plans.iter().map(|p| (p.command.clone(), p.params.clone())).collect();
        s.agent.teach(said, &steps);
        reply["learned"] = json!(true);
    }
    if all_ok && let Some(last) = plans.last() {
        reply["next"] = json!(predict(s, &last.command));
    }
    Ok(reply)
}

fn question(said: &str, options: &[Vec<Plan>]) -> Value {
    json!({"done": [], "ask": {"said": said, "options": options.iter().map(|o| json!({"score": o.first().map(|p| p.score).unwrap_or(0.0), "plans": o})).collect::<Vec<_>>()}})
}

/// `agent.ask`.
fn ask(s: &mut Session, v: &Value) -> CmdResult {
    let text = p::req_str(v, "text")?;
    let t = nlu::canonical(text);
    let raw = norm(text);
    // the answer to a question
    if let Some(mut pending) = s.agent.pending.take() {
        if is_yes(&raw) {
            let plans = pending.options.first().cloned().unwrap_or_default();
            return run_plans(s, &pending.said, &plans, true);
        }
        if is_no(&raw) {
            pending.options.remove(0);
            if pending.options.is_empty() {
                return Ok(json!({"done": [], "declined": true}));
            }
            let reply = question(&pending.said, &pending.options);
            s.agent.pending = Some(pending);
            return Ok(reply);
        }
        if let Ok(k) = raw.parse::<usize>()
            && let Some(plans) = k.checked_sub(1).and_then(|i| pending.options.get(i)).cloned()
        {
            return run_plans(s, &pending.said, &plans, true);
        }
        // anything else is a new request
    }
    if is_repeat(&raw) && !s.agent.last.is_empty() {
        let plans = s.agent.last.clone();
        let mut r = run_plans(s, text, &plans, false)?;
        r["repeated"] = json!(true);
        return Ok(r);
    }
    if is_next(&raw) || is_next(&t) {
        let last = s.agent.last.last().map(|p| p.command.clone()).unwrap_or_default();
        return Ok(json!({"done": [], "next": predict(s, &last)}));
    }
    let cands = candidates(s, text);
    match cands.first() {
        Some(best) if best.score >= SURE => {
            let plans = best.plans.clone();
            run_plans(s, text, &plans, false)
        }
        Some(best) if best.score >= ASK => {
            let options: Vec<Vec<Plan>> = cands.iter().filter(|c| c.score >= ASK).take(3).map(|c| c.plans.clone()).collect();
            s.agent.pending = Some(Pending { said: text.to_string(), options: options.clone() });
            Ok(question(text, &options))
        }
        _ => {
            let suggestions: Vec<Plan> = cands.into_iter().flat_map(|c| c.plans).take(5).collect();
            Ok(json!({"done": [], "suggestions": suggestions}))
        }
    }
}

fn teach_one(s: &mut Session, text: &str, command: &str, params: Value, first: &mut Vec<String>) -> bool {
    if s.registry.get(command).is_none() || command.starts_with("agent.") {
        return false;
    }
    let key = nlu::canonical(text);
    if key.is_empty() {
        return false;
    }
    // the first example of a phrasing replaces what was taught, the next ones add steps
    if !first.contains(&key) {
        first.push(key.clone());
        s.agent.taught.retain(|t| t.text != key);
    }
    match s.agent.taught.iter_mut().find(|t| t.text == key) {
        Some(t) => t.plans.push((command.to_string(), params)),
        None => s.agent.taught.push(Taught { text: key, plans: vec![(command.to_string(), params)] }),
    }
    s.agent.cap();
    true
}

fn teach_cmd(s: &mut Session, v: &Value) -> CmdResult {
    let text = p::req_str(v, "text")?;
    let command = p::req_str(v, "command")?;
    let params = v.get("params").cloned().filter(Value::is_object).unwrap_or_else(|| json!({}));
    if !teach_one(s, text, command, params, &mut Vec::new()) {
        return Err(CmdError::Params(format!("cannot teach «{text}» → {command}")));
    }
    Ok(json!({"taught": s.agent.taught.len()}))
}

fn train_cmd(s: &mut Session, v: &Value) -> CmdResult {
    let examples = v.get("examples").and_then(Value::as_array).ok_or_else(|| CmdError::Params("examples must be a list".into()))?;
    let (mut taught, mut skipped) = (0usize, 0usize);
    let mut first: Vec<String> = Vec::new();
    for e in examples.iter().take(100_000) {
        let text = e.get("text").and_then(Value::as_str).unwrap_or("");
        let command = e.get("command").and_then(Value::as_str).unwrap_or("");
        let params = e.get("params").cloned().filter(Value::is_object).unwrap_or_else(|| json!({}));
        if teach_one(s, text, command, params, &mut first) {
            taught += 1;
        } else {
            skipped += 1;
        }
    }
    Ok(json!({"taught": taught, "skipped": skipped}))
}

fn forget_cmd(s: &mut Session, v: &Value) -> CmdResult {
    let before = s.agent.taught.len();
    if p::bool(v, "all").unwrap_or(false) {
        s.agent.taught.clear();
        s.agent.follow.clear();
        s.agent.params.clear();
    } else {
        let text = p::req_str(v, "text")?;
        let key = nlu::canonical(text);
        s.agent.taught.retain(|t| t.text != key);
    }
    Ok(json!({"forgotten": before - s.agent.taught.len()}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(text: &str) -> Session {
        Session::new(wordcraft_doc::Document::from_text(text))
    }

    fn cmds(s: &Session, say: &str) -> Vec<(String, Value)> {
        interpret(s, say).into_iter().map(|p| (p.command, p.params)).collect()
    }

    #[test]
    fn arabic_requests_become_commands() {
        let s = s("x");
        assert_eq!(cmds(&s, "احذف الترقيم الرئيسي"), vec![("service.outline.removeLevel".into(), json!({"depth": 0}))]);
        assert_eq!(cmds(&s, "أزل الترقيم الفرعي"), vec![("service.outline.removeLevel".into(), json!({"depth": 1}))]);
        assert_eq!(cmds(&s, "الغاء المستوى 3 من الترقيم"), vec![("service.outline.removeLevel".into(), json!({"depth": 2}))]);
        assert_eq!(cmds(&s, "أعد الترقيم"), vec![("service.renumber".into(), json!({}))]);
        assert_eq!(cmds(&s, "كشيدة عالية"), vec![("service.kashida".into(), json!({"value": "high"}))]);
        assert_eq!(cmds(&s, "درجة الكتمان سري للغاية"), vec![("service.classification".into(), json!({"value": "سري للغاية"}))]);
        assert_eq!(cmds(&s, "درجة الكتمان محدود"), vec![("service.classification".into(), json!({"value": "محدود"}))]);
        assert_eq!(cmds(&s, "افتح فهرس الترقيم"), vec![("view.serviceOutlinePane".into(), json!({"value": true}))]);
        assert_eq!(cmds(&s, "أغلق الفهرس"), vec![("view.serviceOutlinePane".into(), json!({"value": false}))]);
        assert_eq!(cmds(&s, "تراجع"), vec![("edit.undo".into(), json!({}))]);
    }

    #[test]
    fn several_wishes_in_one_sentence_run_in_order() {
        let s = s("x");
        let c: Vec<String> = cmds(&s, "اجعله غامق ومائل وحجم ١٦ ووسّطه").into_iter().map(|(c, _)| c).collect();
        assert_eq!(c, ["format.bold", "format.italic", "format.size", "para.alignCenter"]);
        let sz = cmds(&s, "حجم الخط ١٨");
        assert_eq!(sz[0], ("format.size".into(), json!({"size": 18.0})));
        let c = cmds(&s, "حدد الكل ثم لون احمر");
        assert_eq!(c[0].0, "select.all");
        assert_eq!(c[1], ("format.color".into(), json!({"color": "FF0000"})));
        assert_eq!(cmds(&s, "make it bold")[0].0, "format.bold");
    }

    #[test]
    fn typed_words_and_fonts_keep_their_spelling() {
        let s = s("x");
        assert_eq!(cmds(&s, "اكتب مرحباً بكم")[0], ("text.insert".into(), json!({"text": "مرحباً بكم"})));
        assert_eq!(cmds(&s, "خط تايمز")[0], ("format.font".into(), json!({"name": "Times New Roman"})));
        assert_eq!(cmds(&s, "اجعل الخط Garamond")[0], ("format.font".into(), json!({"name": "Garamond"})));
        // «خط تحت» is underline, «خط عريض» is bold, neither is a font
        assert!(cmds(&s, "ضع خط تحت النص").iter().all(|(c, _)| c != "format.font"));
        assert!(cmds(&s, "خط عريض").iter().all(|(c, _)| c != "format.font"));
    }

    #[test]
    fn what_the_rules_do_not_know_is_not_run_blindly() {
        let mut s = s("x");
        assert!(interpret(&s, "").is_empty());
        let plans = interpret(&s, "insert table");
        assert!(plans.iter().all(|p| p.score < SURE), "{plans:?}");
        // not sure: nothing runs; it asks or suggests
        let r = s.run("agent.ask", &json!({"text": "insert table"})).unwrap();
        assert!(r["done"].as_array().unwrap().is_empty());
        assert!(r.get("ask").is_some() || r.get("suggestions").is_some(), "{r}");
        let r = s.run("agent.ask", &json!({"text": "ما علاقة الجو بالموضوع"})).unwrap();
        assert!(r["done"].as_array().unwrap().is_empty());
    }

    #[test]
    fn asking_runs_the_plans_on_the_document() {
        let mut s = s("١. الأول\nأ. فرع\n٢. الثاني\nأ. فرع آخر");
        let r = s.run("agent.ask", &json!({"text": "احذف الترقيم الرئيسي"})).unwrap();
        assert_eq!(r["done"][0]["ok"], json!(true), "{r}");
        let t: Vec<String> = s.doc.body.iter().filter_map(|b| b.as_para()).map(|p| p.text.clone()).collect();
        assert!(t[1].starts_with("١."), "the sub-items became the main items: {t:?}");
        // a bad step stops the rest and says why
        let r = s.run("agent.ask", &json!({"text": "حدد الكل ثم غامق"})).unwrap();
        assert_eq!(r["done"].as_array().unwrap().len(), 2);
        assert!(s.run("agent.ask", &json!({})).is_err(), "text is required");
    }

    #[test]
    fn the_text_is_normalised_like_an_arabic_reader_would() {
        assert_eq!(norm("أَعِدْ الترقيمَ!"), "اعد الترقيم");
        assert_eq!(norm("حجم ١٦٫٥"), "حجم 16.5");
        assert_eq!(number(&norm("حجم الخط ١٦")), Some(16.0));
        assert_eq!(number("حجم عشره"), Some(10.0));
        assert_eq!(number("بلا ارقام"), None);
    }

    #[test]
    fn every_command_a_rule_names_exists() {
        let s = s("x");
        for say in [
            "احذف الترقيم الرئيسي",
            "أعد الترقيم",
            "شغل الترقيم التلقائي",
            "افتح الفهرس",
            "نقطة ختامية",
            "كشيدة",
            "درجة الكتمان سري",
            "ترقيم الصفحات",
            "ارفع المستوى",
            "ادخله في الترقيم",
            "انسخ البند",
            "تراجع",
            "اعادة التراجع",
            "حدد الكل",
            "احفظ",
            "فقرة جديدة",
            "فاصل صفحة",
            "غامق",
            "مائل",
            "تسطير",
            "شطب",
            "حجم 20",
            "كبر الخط",
            "صغر الخط",
            "لون ازرق",
            "خط تاهوما",
            "وسّط",
            "ضبط",
            "لليمين",
            "لليسار",
            "تباعد مزدوج",
            "تباعد 1.5",
            "اكتب مرحبا",
        ] {
            for p in interpret(&s, say) {
                assert!(s.registry.get(&p.command).is_some(), "{say}: {} is not a command", p.command);
            }
        }
    }

    #[test]
    fn hostile_text_is_fine() {
        let s = s("x");
        for t in ["\0", "ـــ", "9999999999999999999999 حجم", "حجم .", &"و".repeat(10_000), "حجم ...5.5.5"] {
            let _ = interpret(&s, t);
        }
    }

    fn doc() -> Session {
        s("١. الأول\nأ. فرع\n٢. الثاني\nأ. فرع آخر")
    }

    #[test]
    fn iraqi_dialect_and_slips_are_understood() {
        let s = s("x");
        for (say, want) in [
            ("شيل الأرقام الرئيسية", "service.outline.removeLevel"),
            ("لو سمحت خلي الكلام ثخين", "format.bold"),
            ("تكدر تخليه بالنص", "para.alignCenter"),
            ("غامك", "format.bold"),
            ("الترقيم خربان", "service.renumber"),
            ("الترقيم مو مضبوط", "service.renumber"),
            ("الخط صغير ما اشوفه", "format.growFont"),
            ("ردها", "edit.undo"),
            ("make the text bigger", "format.growFont"),
            ("خزن الشغل", "file.save"),
        ] {
            let c = candidates(&s, say);
            let best = c.first().unwrap_or_else(|| panic!("{say}: nothing"));
            assert!(best.plans.iter().any(|p| p.command == want), "{say}: {:?}", best.plans.iter().map(|p| &p.command).collect::<Vec<_>>());
            assert!(best.score >= ASK, "{say}: {}", best.score);
        }
        // the parameters survive the dialect: the level, the size, the colour
        assert_eq!(cmds(&s, "شيل الأرقام الفرعية")[0], ("service.outline.removeLevel".into(), json!({"depth": 1})));
        assert_eq!(cmds(&s, "خلي حجم الخط 22")[0], ("format.size".into(), json!({"size": 22.0})));
        assert_eq!(cmds(&s, "غير اللون الى ازرك")[0], ("format.color".into(), json!({"color": "0000FF"})));
    }

    #[test]
    fn a_middling_match_asks_and_the_answer_decides() {
        let mut s = doc();
        // a slip the repair can't be sure of: it asks first
        let r = s.run("agent.ask", &json!({"text": "الترقيم يخبط"})).unwrap();
        assert!(r.get("ask").is_some(), "{r}");
        assert_eq!(r["ask"]["options"][0]["plans"][0]["command"], json!("service.renumber"));
        assert!(r["done"].as_array().unwrap().is_empty());
        let r = s.run("agent.ask", &json!({"text": "نعم"})).unwrap();
        assert_eq!(r["done"][0]["ok"], json!(true), "{r}");
        assert_eq!(r["learned"], json!(true));
        // what it learned runs at once next time
        assert!(!s.agent.taught.is_empty());
        let r = s.run("agent.interpret", &json!({"text": "الترقيم يخبط"})).unwrap();
        assert_eq!(r["plans"][0]["score"], json!(1.0));
    }

    #[test]
    fn no_moves_to_the_next_option_or_gives_up() {
        let mut s = doc();
        s.agent.pending = Some(Pending {
            said: "x".into(),
            options: vec![
                vec![Plan { command: "edit.undo".into(), params: json!({}), label: "Undo".into(), score: 0.6 }],
                vec![Plan { command: "edit.redo".into(), params: json!({}), label: "Redo".into(), score: 0.55 }],
            ],
        });
        let r = s.run("agent.ask", &json!({"text": "لا"})).unwrap();
        assert_eq!(r["ask"]["options"].as_array().unwrap().len(), 1);
        let r = s.run("agent.ask", &json!({"text": "لا"})).unwrap();
        assert_eq!(r["declined"], json!(true));
        assert!(s.agent.pending.is_none());
        // a number picks an option
        s.agent.pending = Some(Pending {
            said: "x".into(),
            options: vec![
                vec![Plan { command: "edit.undo".into(), params: json!({}), label: "Undo".into(), score: 0.6 }],
                vec![Plan { command: "select.all".into(), params: json!({}), label: "Select All".into(), score: 0.55 }],
            ],
        });
        let r = s.run("agent.ask", &json!({"text": "2"})).unwrap();
        assert_eq!(r["done"][0]["command"], json!("select.all"));
    }

    #[test]
    fn context_repeat_and_what_comes_next() {
        let mut s = doc();
        s.run("agent.ask", &json!({"text": "أعد الترقيم"})).unwrap();
        let r = s.run("agent.ask", &json!({"text": "مرة ثانية"})).unwrap();
        assert_eq!(r["repeated"], json!(true));
        assert_eq!(r["done"][0]["command"], json!("service.renumber"));
        // after the renumbering the seeded workflow suggests the final dot
        let r = s.run("agent.ask", &json!({"text": "وبعدين"})).unwrap();
        assert_eq!(r["next"][0]["command"], json!("service.finalDot"));
        // a repeat with nothing before it is just a request
        let mut fresh = doc();
        let r = fresh.run("agent.ask", &json!({"text": "مرة ثانية"})).unwrap();
        assert!(r["done"].as_array().unwrap().is_empty());
    }

    #[test]
    fn habits_outrank_the_seeds() {
        let mut s = doc();
        for _ in 0..3 {
            s.agent.learn_step("service.renumber", "service.pageNumbers", &json!({}));
        }
        s.agent.last = vec![Plan { command: "service.renumber".into(), params: json!({}), label: String::new(), score: 1.0 }];
        let r = s.run("agent.ask", &json!({"text": "وبعدين"})).unwrap();
        assert_eq!(r["next"][0]["command"], json!("service.pageNumbers"), "{r}");
    }

    #[test]
    fn teaching_training_forgetting_and_saving() {
        let mut s = doc();
        s.run("agent.teach", &json!({"text": "رتبلي هالمستند", "command": "service.renumber"})).unwrap();
        assert_eq!(cmds(&s, "رتبلي هالمستند")[0].0, "service.renumber");
        assert!(s.run("agent.teach", &json!({"text": "x", "command": "no.such"})).is_err());
        assert!(s.run("agent.teach", &json!({"text": "x", "command": "agent.ask"})).is_err(), "no loops");
        let r = s
            .run(
                "agent.train",
                &json!({"examples": [
                    {"text": "زبطلي الارقام", "command": "service.renumber"},
                    {"text": "زبطلي الارقام", "command": "service.finalDot"},
                    {"text": "", "command": "service.renumber"},
                    {"text": "y", "command": "no.such"},
                ]}),
            )
            .unwrap();
        assert_eq!((r["taught"].as_u64(), r["skipped"].as_u64()), (Some(2), Some(2)));
        let two: Vec<String> = cmds(&s, "زبطلي الارقام").into_iter().map(|c| c.0).collect();
        assert_eq!(two, ["service.renumber", "service.finalDot"], "two steps in order");
        // save and load
        let path = std::env::temp_dir().join(format!("wc-agent-{}.json", std::process::id()));
        let path = path.to_string_lossy().to_string();
        s.run("agent.save", &json!({"path": path})).unwrap();
        let mut other = doc();
        let r = other.run("agent.load", &json!({"path": path})).unwrap();
        assert_eq!(r["taught"], json!(2));
        assert_eq!(cmds(&other, "رتبلي هالمستند")[0].0, "service.renumber");
        let _ = std::fs::remove_file(&path);
        assert!(other.run("agent.load", &json!({"path": "/no/such/file.json"})).is_err());
        // forgetting
        let r = other.run("agent.forget", &json!({"text": "رتبلي هالمستند"})).unwrap();
        assert_eq!(r["forgotten"], json!(1));
        other.run("agent.forget", &json!({"all": true})).unwrap();
        assert!(other.agent.taught.is_empty());
    }

    #[test]
    fn the_memory_stays_bounded() {
        let mut m = AgentMemory::default();
        for i in 0..(AgentMemory::MAX_COMMANDS + 50) {
            m.learn_step(&format!("a{i}"), &format!("b{i}"), &json!({}));
        }
        assert!(m.follow.len() <= AgentMemory::MAX_COMMANDS);
        assert!(m.params.len() <= AgentMemory::MAX_COMMANDS);
    }
}
