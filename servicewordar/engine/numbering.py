# -*- coding: utf-8 -*-
"""
Hierarchical Arabic staff-writing numbering.

Cycle of marker kinds (depth % 6):
  0  digit-dot      ١.
  1  abjad-dot      أ.   ب.   جـ.   د.   هـ.   و. ...
  2  ordinal-dot    أولاً.  ثانياً. ...
  3  digit-paren    (١)
  4  abjad-paren    (أ).
  5  ordinal-paren  (أولاً).
then the cycle repeats for deeper levels.
"""
import re

KINDS = ["digit_dot", "abjad_dot", "ordinal_dot", "digit_paren", "abjad_paren", "ordinal_paren"]

ABJAD = list("أبجدهوزحطيكلمنسعفصقرشتثخذضظغ")

ORDINALS = [
    "أولاً", "ثانياً", "ثالثاً", "رابعاً", "خامساً", "سادساً", "سابعاً", "ثامناً", "تاسعاً", "عاشراً",
    "حادي عشر", "ثاني عشر", "ثالث عشر", "رابع عشر", "خامس عشر", "سادس عشر", "سابع عشر", "ثامن عشر", "تاسع عشر",
    "عشرون", "حادي وعشرون", "ثاني وعشرون", "ثالث وعشرون", "رابع وعشرون", "خامس وعشرون", "سادس وعشرون",
    "سابع وعشرون", "ثامن وعشرون", "تاسع وعشرون", "ثلاثون",
]

WESTERN = "0123456789"
ARABIC_INDIC = "٠١٢٣٤٥٦٧٨٩"
_TO_AI = {ord(w): a for w, a in zip(WESTERN, ARABIC_INDIC)}
_TO_W = {ord(a): w for w, a in zip(WESTERN, ARABIC_INDIC)}
# Persian digits sometimes appear in documents
_TO_W.update({ord(a): w for w, a in zip(WESTERN, "۰۱۲۳۴۵۶۷۸۹")})

TATWEEL = "ـ"


def to_arabic_indic(s: str) -> str:
    return s.translate(_TO_AI)


def to_western(s: str) -> str:
    return s.translate(_TO_W)


def _norm_ordinal(word: str) -> str:
    """Normalize an ordinal word for matching: drop tanween/diacritics, spaces, tatweel; unify alef."""
    w = re.sub(r"[ً-ْٰ]", "", word)   # harakat & tanween
    w = w.replace(TATWEEL, "").replace(" ", "").replace(" ", "")
    w = re.sub("[إأآا]", "ا", w)
    w = w.replace("ى", "ي")
    return w


ORDINAL_INDEX = {_norm_ordinal(o): i + 1 for i, o in enumerate(ORDINALS)}


def _flex(word: str) -> str:
    """Regex for an ordinal word tolerant to diacritics, tatweel, alef variants and spacing."""
    out = []
    for ch in word:
        if ch == " ":
            out.append(r"\s*")
        elif ch in "اأإآ":
            out.append(r"[اأإآ][\u064B-\u0652\u0640 ]*")
        elif "\u064B" <= ch <= "\u0652":
            continue  # diacritics in the table are optional
        else:
            out.append(re.escape(ch) + r"[\u064B-\u0652\u0640 ]*")
    return "".join(out)


_ORD_ALT = "|".join(_flex(o) for o in sorted(ORDINALS, key=len, reverse=True))


def ordinal_number(matched: str) -> int:
    return ORDINAL_INDEX.get(_norm_ordinal(matched), 1)

# --- marker regexes applied to the *normalized* start of a paragraph ---
# A marker is: optional "(" , token , optional ")" , separator (. - : ـ) , whitespace
_DIGIT = r"[0-9٠-٩۰-۹]+"
_ABJAD_CHAR = "[" + "".join(ABJAD) + "اإآ" + "]"

RE_DIGIT_PAREN = re.compile(r"^\s*[\(（]\s*(" + _DIGIT + r")\s*[\)）]\s*[\.\-–:ـ]?\s*")
RE_DIGIT_DOT = re.compile(r"^\s*(" + _DIGIT + r")\s*[\.\-–:ـ]\s*")
RE_ABJAD_PAREN = re.compile(r"^\s*[\(（]\s*(" + _ABJAD_CHAR + r")\s*ـ*\s*[\)）]\s*[\.\-–:ـ]?\s*")
RE_ABJAD_DOT = re.compile(r"^\s*(" + _ABJAD_CHAR + r")\s*ـ+\s*[\.\-–:]\s*|^\s*(" + _ABJAD_CHAR + r")\s*[\.\-–:]\s*")
RE_ORD_PAREN = re.compile(r"^\s*[\(（]\s*(" + _ORD_ALT + r")\s*[\)）]\s*[\.\-–:ـ]?\s*")
RE_ORD_DOT = re.compile(r"^\s*(" + _ORD_ALT + r")\s*[\.\-–:ـ]\s*")


def detect_marker(text: str):
    """
    Return (kind_index, number, match_len_in_normalized_text) or None.
    The caller must pass the *normalized* paragraph text (see normalize_for_detection).
    """
    if not text:
        return None
    m = RE_DIGIT_PAREN.match(text)
    if m:
        return 3, int(to_western(m.group(1))), m.end()
    m = RE_ORD_PAREN.match(text)
    if m:
        return 5, ordinal_number(m.group(1)), m.end()
    m = RE_ABJAD_PAREN.match(text)
    if m:
        return 4, _abjad_index(m.group(1)), m.end()
    m = RE_ORD_DOT.match(text)
    if m:
        return 2, ordinal_number(m.group(1)), m.end()
    m = RE_DIGIT_DOT.match(text)
    if m:
        return 0, int(to_western(m.group(1))), m.end()
    m = RE_ABJAD_DOT.match(text)
    if m:
        ch = m.group(1) or m.group(2)
        return 1, _abjad_index(ch), m.end()
    return None


def _abjad_index(ch: str) -> int:
    ch = re.sub("[اإآ]", "أ", ch)
    return ABJAD.index(ch) + 1 if ch in ABJAD else 1


def normalize_for_detection(text: str) -> str:
    """Normalization used ONLY for marker detection. Keeps length-preserving ops out: we re-match on raw later."""
    return text


def ordinal_word(n: int) -> str:
    if 1 <= n <= len(ORDINALS):
        return ORDINALS[n - 1]
    # beyond table: fall back to digits
    return to_arabic_indic(str(n))


def abjad_letter(n: int, tatweel_letters) -> str:
    if n < 1:
        n = 1
    if n > len(ABJAD):
        # wrap with doubled letters: أأ, بب ...
        base = ABJAD[(n - 1) % len(ABJAD)]
        rep = (n - 1) // len(ABJAD) + 1
        letter = base * rep
    else:
        letter = ABJAD[n - 1]
    if letter and letter[-1] in tatweel_letters:
        letter = letter + TATWEEL
    return letter


def marker_text(depth: int, n: int, fmt_cfg: dict) -> str:
    """
    Build the marker string for a given depth and counter n.
    fmt_cfg['formats'] : list of 6 templates, one per kind, using {n}
    fmt_cfg['tatweel_letters'] : letters that get a trailing tatweel (default ج ه)
    """
    kind = depth % 6
    templates = fmt_cfg.get("formats") or DEFAULT_FORMATS
    tpl = templates[kind]
    tat = fmt_cfg.get("tatweel_letters", ["ج", "ه"])
    if kind in (0, 3):
        token = to_arabic_indic(str(n)) if fmt_cfg.get("arabic_indic_digits", True) else str(n)
    elif kind in (1, 4):
        token = abjad_letter(n, tat)
    else:
        token = ordinal_word(n)
    return tpl.replace("{n}", token)


DEFAULT_FORMATS = ["{n}.", "{n}.", "{n}.", "({n}).", "({n}).", "({n})."]

# approximate marker widths in twips (1/20 pt) per kind, used for hanging indents
DEFAULT_WIDTHS = [420, 420, 1000, 560, 640, 1150]
