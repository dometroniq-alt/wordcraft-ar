# -*- coding: utf-8 -*-
"""Default rules. Everything here can be overridden from rules.json or from the UI."""
import copy

DEFAULT_RULES = {
    "include_tables": False,
    # kind used for Word automatic lists that carry no typed marker: 3 = "(١)"
    "auto_list_kind": 3,
    "numbering": {
        # templates per kind: digit_dot, abjad_dot, ordinal_dot, digit_paren, abjad_paren, ordinal_paren
        "formats": ["{n}.", "{n}.", "{n}.", "({n}).", "({n}).", "({n})."],
        "arabic_indic_digits": True,
        "tatweel_letters": ["ج", "ه"],
        # hanging width per kind in twips; null = automatic (widest marker of the level x its font size)
        "marker_widths": None,
        "marker_gap_em": 0.3,          # gap between the marker and the text, in em
        "marker_width_factor": 1.05,    # safety factor for wide/bold fonts
        "keep_top_level_start": False,
        "first_level_is_top": True,
        "dot_font": "Times New Roman",  # the dot / parentheses of every marker are drawn in this font
        "dot_weight": 1.4,              # dot diameter = 1.4 x the marker font's stroke width (sized automatically)
        "scale_digit_parens": False,    # keep (١) parentheses at the digit size     # the shallowest level in the file becomes level 1 (١.) even if typed as أ.
    },
    # page margins in cm; "file" keeps the document's own margins. Presets offered by the UI:
    #   narrow: 1.27 all sides, tight: right/left 1.27 keep top/bottom, normal: 2.54 all sides
    "page": {
        "margins": {"right": 1.27, "left": 1.27, "top": None, "bottom": None},
        # security classification word in the header and footer: عام | محدود | سري | سري وشخصي | سري للغاية
        "classification": "محدود",
        # footer page line "( PAGE - END )": start = number of the first content page, end = last page
        # number typed (None = automatic NUMPAGES field); digits in Arabic-Indic via the field switch
        "numbering": {"enabled": True, "start": 1, "end": None, "digits": "arabic_indic"},
    },
    "layout": {
        "base_indent": 0,                       # twips from the right margin for the top level
        "page_break_before_section": "keep",    # keep | always
        "marker_space_factor": 1.0,             # gap after the marker, in spaces of the marker's font
        "balance_lines": True,                  # measure with the real fonts and balance line breaks
        "insert_breaks": True,                  # write the balanced breaks as soft line breaks
        "line_safety": 0.04,                    # keep 4% of the line free so Word never re-wraps
        "stretch_min_fill": 0.0,                # 0 = every line (even one word) is stretched to the margin
        "fill_lines_with_kashida": True,        # write the kashida into the text (same look in Word, LibreOffice, WordCraft)
        "kashida_fill_target": 0.985,
        "kashida_one_per_word": True,           # one stretch per word
        "kashida_prefer_word_end": True,        # the stretch goes before the last letter (مستنداـت → مستنداـت, علــى, الطبــي)           # fill lines with kashida up to this share; Word fills the rest
        "tight_fit_max": 0.99,                  # text up to 99% of the line stays on one line
        "tight_fit_min": 0.965,                 # a line fuller than this is justified normally (no distribute)
        "keep_first_item": True,
        "fold_page_breaks": True,               # lone page-break paragraphs become "page break before" on the next paragraph
        "equalize_markers": True,               # all markers of a level get the width of the widest one
        "equalize_with_kashida": True,          # stretch marker words with kashida first, then letter spacing
        "marker_max_scale": 1.15,               # a marker without a kashida joint (أ. د.) may be widened up to 115%
        "marker_dot_gap": True,
        "marker_tatweel_fixed": True,           # «جـ» «هـ» keep one conventional tatweel (no longer kashida); the rest is the gap
        "kashida_cost": 0.6,                    # cost weight of a long kashida in a marker (per em², squared)
        "narrow_cost": 0.25,                    # cost weight of narrowing a marker down to marker_min_scale                 # …and takes the rest as a small gap before its dot (dots stay on one line)
        "marker_min_scale": 0.85,               # the widest marker of a level may be narrowed down to 85%
        "equalize_digits": False,               # digit markers (١) keep their natural width
        # a heading and its numbered items stay on one page when the group is at most this share of
        # the text height; longer groups keep only heading+first items and the last items together
        "marker_kashida_lam_alef": False,       # alternative: kashida between ل and ا (أولـــاً); off = keep the ligature
        # أولاً keeps its ligature and reaches the level width with a lengthened lam-alef glyph (a tiny
        # font derived from the document's font, embedded in the file), plus mild glyph scaling and a
        # little letter spacing — in this order and within these ranges:
        "marker_lamalef_glyph": True,
        "lamalef_spacing_soft_em": 0.10,         # 1) open the natural gaps (أ|و|لا) a little, each
        "lamalef_extend_soft_em": 0.28,          # 2) lengthen the lam-alef (the font's own sweep, stretched)
        "lamalef_spacing_max_em": 0.22,          # 3) gaps up to this
        "lamalef_scale_max": 1.08,               # 4) very mild glyph widening
        "lamalef_extend_max_em": 0.55,           # 5) lam-alef never beyond this
        "lamalef_mode": "scale",                 # scale = stretch the lam's sweep and foot; shear = move the foot only
        "keep_groups": True,
        "keep_group_max_share": 0.5,
        "keep_group_head_items": 2,
        "keep_group_tail_items": 2,
    },
    "cleanup": {
        "remove_soft_breaks": True,
        "remove_tatweel": True,
        "collapse_spaces": True,
        "fix_punctuation_spaces": True,
        "digits_in_text": "arabic_indic",       # arabic_indic | keep
        "remove_empty_paragraphs": True,
        "prevent_orphan_word": True,
        "final_dot": True,                      # every item/body paragraph ends with a dot
        "final_dot_roles": ["item", "body"],
    },
    "fonts": {
        "force_color": None,                    # e.g. "000000" to force black
        "pin_theme_fonts": True,                # write the resolved theme font (e.g. Arial) explicitly on runs
    },
    # per-role formatting. font/size/bold/underline = None  -> keep the original
    # align: start (= right in RTL) | both | lowKashida | mediumKashida | highKashida | distribute | center
    "roles": {
        "cover":   {"font": None, "size": None, "bold": None, "underline": None, "align": "center",
                    "spacing": {"before": 0, "after": 12, "line": 1.0}, "keep_lines": True, "keep_next": True},
        "main":    {"font": None, "size": None, "bold": None, "underline": True, "align": "start",
                    "spacing": {"before": 12, "after": 6, "line": 1.0}, "keep_lines": True, "keep_next": True},
        "section": {"font": None, "size": None, "bold": None, "underline": True, "align": "start",
                    "spacing": {"before": 12, "after": 6, "line": 1.0}, "keep_lines": True, "keep_next": True},
        "heading": {"font": None, "size": None, "bold": None, "underline": True, "align": "start",
                    "spacing": {"before": 6, "after": 3, "line": 1.0}, "keep_lines": True, "keep_next": True},
        "item":    {"font": None, "size": None, "bold": None, "underline": None, "align": "lowKashida",
                    "spacing": {"before": 0, "after": 3, "line": 1.1}, "keep_lines": True, "keep_next": False},
        "body":    {"font": None, "size": None, "bold": None, "underline": None, "align": "lowKashida",
                    "spacing": {"before": 0, "after": 3, "line": 1.1}, "keep_lines": True, "keep_next": False},
    },
}


def merge_rules(base: dict, override: dict) -> dict:
    out = copy.deepcopy(base)

    def _merge(a, b):
        for k, v in b.items():
            if isinstance(v, dict) and isinstance(a.get(k), dict):
                _merge(a[k], v)
            else:
                a[k] = v
    _merge(out, override or {})
    return out
