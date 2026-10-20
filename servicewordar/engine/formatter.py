# -*- coding: utf-8 -*-
"""
ServiceWord-AR engine: formats Arabic military/administrative "staff writing" DOCX files.

What it does (deterministically, on a COPY of the file):
  1. Detects the hierarchy of every paragraph from its existing marker
     (typed "أ." / "أولاً." / "(1)" ... or a Word automatic list) and rebuilds the
     numbering with the requested forms (Arabic-Indic digits, abjad letters with
     final-form tatweel for جـ/هـ, ordinal words), restarting counters correctly.
  2. Cascades indents so every sub-level starts under the parent's text (after the dot).
  3. Removes manual line breaks inside paragraphs, duplicated spaces, typed tatweel
     elongation, spaces before punctuation.
  4. Sets paragraph direction RTL, justification with the requested kashida level,
     keep-lines-together / keep-with-next so phrases are not split across pages.
  5. Keeps the original fonts/sizes unless the rules say otherwise.
  6. Writes a JSON change report.

Everything is driven by a rules dict (see rules.py DEFAULT_RULES).
"""
from __future__ import annotations

import copy
import json
import os
import re
import shutil
import zipfile
from dataclasses import dataclass, field
from typing import List, Optional

from lxml import etree

from . import numbering as N
from .metrics import FontCatalog
from .rules import DEFAULT_RULES, merge_rules

W_NS = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
NSMAP = {"w": W_NS}


def w(tag: str) -> str:
    return "{%s}%s" % (W_NS, tag)


# Order of child elements inside w:pPr as required by the schema
SECT_ORDER = [
    "headerReference", "footerReference", "footnotePr", "endnotePr", "type", "pgSz", "pgMar", "paperSrc",
    "pgBorders", "lnNumType", "pgNumType", "cols", "formProt", "vAlign", "noEndnote", "titlePg",
    "textDirection", "bidi", "rtlGutter", "docGrid", "printerSettings", "sectPrChange",
]
PPR_ORDER = [
    "pStyle", "keepNext", "keepLines", "pageBreakBefore", "framePr", "widowControl", "numPr",
    "suppressLineNumbers", "pBdr", "shd", "tabs", "suppressAutoHyphens", "kinsoku", "wordWrap",
    "overflowPunct", "topLinePunct", "autoSpaceDE", "autoSpaceDN", "bidi", "adjustRightInd",
    "snapToGrid", "spacing", "ind", "contextualSpacing", "mirrorIndents", "suppressOverlap", "jc",
    "textDirection", "textAlignment", "textboxTightWrap", "outlineLvl", "divId", "cnfStyle",
    "rPr", "sectPr", "pPrChange",
]
RPR_ORDER = [
    "rStyle", "rFonts", "b", "bCs", "i", "iCs", "caps", "smallCaps", "strike", "dstrike", "outline",
    "shadow", "emboss", "imprint", "noProof", "snapToGrid", "vanish", "webHidden", "color", "spacing",
    "w", "kern", "position", "sz", "szCs", "highlight", "u", "effect", "bdr", "shd", "fitText",
    "vertAlign", "rtl", "cs", "em", "lang", "eastAsianLayout", "specVanish", "oMath",
]


def _set_child(parent, tag: str, order: List[str], attrs: Optional[dict] = None, remove=False):
    """Insert/replace/remove a single child element keeping schema order."""
    el = parent.find(w(tag))
    if remove:
        if el is not None:
            parent.remove(el)
        return None
    if el is None:
        el = etree.SubElement(parent, w(tag))
        # move to the right position
        idx = order.index(tag)
        parent.remove(el)
        pos = 0
        for i, child in enumerate(list(parent)):
            ltag = etree.QName(child).localname
            if ltag in order and order.index(ltag) < idx:
                pos = i + 1
        parent.insert(pos, el)
    for k in list(el.attrib):
        del el.attrib[k]
    if attrs:
        for k, v in attrs.items():
            el.set(w(k), str(v))
    return el


def _get_ppr(p):
    ppr = p.find(w("pPr"))
    if ppr is None:
        ppr = etree.Element(w("pPr"))
        p.insert(0, ppr)
    return ppr


def _get_rpr(r):
    rpr = r.find(w("rPr"))
    if rpr is None:
        rpr = etree.Element(w("rPr"))
        r.insert(0, rpr)
    return rpr


@dataclass
class Para:
    el: etree._Element
    index: int
    raw_text: str = ""
    kind: Optional[int] = None      # marker kind 0..5 or None
    number: Optional[int] = None    # original number
    depth: Optional[int] = None     # resolved depth
    role: str = "body"              # cover | main | section | heading | item | body | empty | pagebreak
    new_marker: str = ""
    changes: List[str] = field(default_factory=list)
    has_pagebreak: bool = False
    is_empty: bool = False
    layout: dict = field(default_factory=dict)
    marker_run: object = None


class Formatter:
    _catalog = None

    def __init__(self, rules: Optional[dict] = None, fonts: Optional[FontCatalog] = None):
        self.rules = merge_rules(DEFAULT_RULES, rules or {})
        self.report = {"paragraphs": [], "summary": {}}
        if fonts is None:
            if Formatter._catalog is None:
                Formatter._catalog = FontCatalog()
            fonts = Formatter._catalog
        self.fonts = fonts
        self.fonts.missing.clear()

    # ------------------------------------------------------------------ public
    def format_file(self, src: str, dst: str) -> dict:
        if os.path.abspath(src) == os.path.abspath(dst):
            raise ValueError("Destination must differ from the source (the original is never overwritten).")
        shutil.copyfile(src, dst)
        with zipfile.ZipFile(src) as zin:
            doc_xml = zin.read("word/document.xml")
            names = zin.namelist()
            others = {n: zin.read(n) for n in names if n != "word/document.xml"}
        root = etree.fromstring(doc_xml)
        self._root = root
        self._parts = others
        self._theme = self._load_theme_fonts(others.get("word/theme/theme1.xml"))
        self._style_sizes = self._load_style_sizes(others.get("word/styles.xml"))
        self._style_info = self._load_style_info(others.get("word/styles.xml"))
        self._process_document(root)
        self._apply_header_footer(root)
        out_xml = etree.tostring(root, xml_declaration=True, encoding="UTF-8", standalone=True)
        tmp = dst + ".tmp"
        with zipfile.ZipFile(tmp, "w", zipfile.ZIP_DEFLATED) as zout:
            zout.writestr("[Content_Types].xml", self._parts["[Content_Types].xml"])
            for n in names:
                if n == "word/document.xml":
                    zout.writestr(n, out_xml)
                elif n != "[Content_Types].xml" and n in self._parts:
                    zout.writestr(n, self._parts[n])
            for n in self._parts:
                if n not in names and n != "[Content_Types].xml":
                    zout.writestr(n, self._parts[n])
        os.replace(tmp, dst)
        self.report["summary"]["source"] = src
        self.report["summary"]["output"] = dst
        return self.report

    # ----------------------------------------------------------- font sizes
    @staticmethod
    def _load_style_sizes(styles_xml) -> dict:
        """styleId -> complex-script font size in pt (resolved through basedOn), plus '__default__'."""
        sizes = {"__default__": 11.0}
        if not styles_xml:
            return sizes
        st = etree.fromstring(styles_xml)
        dd = st.find(".//" + w("docDefaults") + "/" + w("rPrDefault") + "/" + w("rPr"))
        if dd is not None:
            sz = dd.find(w("szCs"))
            if sz is None:
                sz = dd.find(w("sz"))
            if sz is not None:
                sizes["__default__"] = int(sz.get(w("val"))) / 2
        raw = {}
        for style in st.findall(w("style")):
            sid = style.get(w("styleId"))
            based = style.find(w("basedOn"))
            rpr = style.find(w("rPr"))
            sz = None
            if rpr is not None:
                e = rpr.find(w("szCs"))
                if e is None:
                    e = rpr.find(w("sz"))
                if e is not None:
                    sz = int(e.get(w("val"))) / 2
            raw[sid] = (sz, based.get(w("val")) if based is not None else None)

        def resolve(sid, depth=0):
            if sid not in raw or depth > 20:
                return sizes["__default__"]
            sz, based = raw[sid]
            if sz is not None:
                return sz
            return resolve(based, depth + 1) if based else sizes["__default__"]

        for sid in raw:
            sizes[sid] = resolve(sid)
        return sizes

    def _para_size(self, p) -> float:
        """Complex-script size (pt) of the paragraph's first text run."""
        for r in p.iter(w("r")):
            if r.find(w("t")) is None:
                continue
            rpr = r.find(w("rPr"))
            if rpr is not None:
                e = rpr.find(w("szCs"))
                if e is None:
                    e = rpr.find(w("sz"))
                if e is not None:
                    return int(e.get(w("val"))) / 2
            break
        ppr = p.find(w("pPr"))
        if ppr is not None:
            rpr = ppr.find(w("rPr"))
            if rpr is not None:
                e = rpr.find(w("szCs"))
                if e is None:
                    e = rpr.find(w("sz"))
                if e is not None:
                    return int(e.get(w("val"))) / 2
            ps = ppr.find(w("pStyle"))
            if ps is not None:
                return self._style_sizes.get(ps.get(w("val")), self._style_sizes["__default__"])
        return self._style_sizes["__default__"]

    @staticmethod
    def _marker_em(marker: str) -> float:
        """Rough advance width of a marker string in em units (bold naskh-like fonts)."""
        total = 0.0
        for ch in marker:
            if ch in "اأإآل|":
                total += 0.28
            elif ch in "سشصضكط":
                total += 0.85
            elif ch in "جحخعغهمنقفيبتثى":
                total += 0.6
            elif ch in "دذرزو":
                total += 0.45
            elif ch == N.TATWEEL:
                total += 0.35
            elif ch in "().":
                total += 0.3
            elif ch == " ":
                total += 0.28
            elif "\u064B" <= ch <= "\u0652":
                total += 0.0
            elif ch.isdigit() or "\u0660" <= ch <= "\u0669":
                total += 0.55
            else:
                total += 0.6
        return total

    # --------------------------------------------------------------- pipeline
    def _process_document(self, root):
        body = root.find(w("body"))
        paras = []
        idx = 0
        for child in list(body):
            if etree.QName(child).localname == "p":
                paras.append(Para(el=child, index=idx))
                idx += 1
            elif etree.QName(child).localname == "tbl" and self.rules["include_tables"]:
                for p in child.iter(w("p")):
                    paras.append(Para(el=p, index=idx))
                    idx += 1

        for pa in paras:
            self._clean_runs(pa)
            pa.raw_text = self._text_of(pa.el)
            pa.is_empty = pa.raw_text.strip() == "" and not self._has_content_objects(pa.el)
            pa.has_pagebreak = any(
                br.get(w("type")) == "page" for br in pa.el.iter(w("br"))
            ) or pa.el.find(".//" + w("pageBreakBefore")) is not None

        self._apply_page_setup(root)
        self._detect_levels(paras)
        self._ensure_final_dot(paras)
        self._apply(paras)
        self._keep_groups(paras)
        self._fold_page_breaks(paras)
        self._remove_empties(paras, body)
        self._paras = paras
        self._finish_report(paras)

    # ---------------------------------------------------------------- helpers
    @staticmethod
    def _has_content_objects(p) -> bool:
        for tag in ("drawing", "pict", "object", "fldSimple", "sdt", "sectPr"):
            if p.find(".//" + w(tag)) is not None:
                return True
        return False

    @staticmethod
    def _text_of(p) -> str:
        parts = []
        for node in p.iter():
            ln = etree.QName(node).localname
            if ln == "t":
                parts.append(node.text or "")
            elif ln == "tab":
                parts.append("\t")
            elif ln == "br" and node.get(w("type")) != "page":
                parts.append("\n")
        return "".join(parts)

    # -------------------------------------------------------- text cleaning
    def _clean_runs(self, pa: Para):
        """Soft line breaks -> space, tatweel removal, whitespace normalisation (per run, then across runs)."""
        p = pa.el
        r_cfg = self.rules["cleanup"]
        # 1) soft breaks -> space
        if r_cfg["remove_soft_breaks"]:
            for br in list(p.iter(w("br"))):
                if br.get(w("type")) in (None, "textWrapping"):
                    parent = br.getparent()
                    t = etree.Element(w("t"))
                    t.text = " "
                    t.set("{http://www.w3.org/XML/1998/namespace}space", "preserve")
                    parent.replace(br, t)
                    pa.changes.append("soft_break_removed")
        # 2) per-run text normalisation
        ts = [t for t in p.iter(w("t"))]
        if not ts:
            return
        for t in ts:
            s = t.text or ""
            if r_cfg["remove_tatweel"] and N.TATWEEL in s:
                s2 = s.replace(N.TATWEEL, "")
                if s2 != s:
                    pa.changes.append("tatweel_removed")
                    s = s2
            s = s.replace(" ", " ").replace("\t", " ") if r_cfg["collapse_spaces"] else s
            t.text = s
        # 3) collapse whitespace across run boundaries
        if r_cfg["collapse_spaces"]:
            prev_space = True  # strip leading
            for t in ts:
                out = []
                for ch in (t.text or ""):
                    if ch == " ":
                        if prev_space:
                            continue
                        prev_space = True
                    else:
                        prev_space = False
                    out.append(ch)
                new = "".join(out)
                if new != (t.text or ""):
                    pa.changes.append("spaces_collapsed")
                t.text = new
            # strip trailing
            for t in reversed(ts):
                if t.text and t.text.endswith(" "):
                    t.text = t.text.rstrip(" ")
                    pa.changes.append("spaces_collapsed")
                if t.text:
                    break
        # 4) no space before closing punctuation ". : ، ؛ )"
        if r_cfg["fix_punctuation_spaces"]:
            full = "".join(t.text or "" for t in ts)
            if ")" in full and "(" not in full:
                full2 = re.sub(r"\)\s*([^()]+?)\s*\)", r"(\1)", full, count=1)
                if full2 != full:
                    self._rewrite_text_general(ts, full2)
                    full = full2
                    pa.changes.append("mirrored_paren_fixed")
            fixed = re.sub(r" +([\.،؛:\)\]])", r"\1", full)
            fixed = re.sub(r"([\(\[]) +", r"\1", fixed)
            if fixed != full:
                self._rewrite_text(ts, fixed)
                pa.changes.append("punctuation_spaces_fixed")
        # 5) digits inside the text
        if r_cfg["digits_in_text"] == "arabic_indic":
            for t in ts:
                s = t.text or ""
                s2 = N.to_arabic_indic(s)
                if s2 != s:
                    t.text = s2
                    pa.changes.append("digits_converted")
        for t in ts:
            if t.text and (t.text.startswith(" ") or t.text.endswith(" ")):
                t.set("{http://www.w3.org/XML/1998/namespace}space", "preserve")

    @staticmethod
    def _rewrite_text(ts, new_full: str):
        """Distribute a new full string over existing w:t elements keeping their lengths proportional
        (used only for deletions, so we walk the old string and drop chars)."""
        old_full = "".join(t.text or "" for t in ts)
        # simple diff: both strings differ only by deleted spaces; walk in lockstep
        i = j = 0
        pieces = []
        for t in ts:
            seg = t.text or ""
            out = []
            for ch in seg:
                if j < len(new_full) and new_full[j] == ch:
                    out.append(ch)
                    j += 1
                # else: this char was deleted
                i += 1
            pieces.append("".join(out))
        for t, s in zip(ts, pieces):
            t.text = s

    @staticmethod
    def _rewrite_text_general(ts, new_full: str):
        """Put new_full into the first w:t and empty the others (used for rare substitutions)."""
        for i, t in enumerate(ts):
            t.text = new_full if i == 0 else ""

    # ------------------------------------------------------- level detection
    def _detect_levels(self, paras: List[Para]):
        abstract_kinds = {}  # numId -> kind (from numbering.xml is not loaded here; infer from text later)
        prev_depth = -1
        clamp_map = {}   # typed marker kind -> depth it was re-mapped to
        seen_marker = False
        for pa in paras:
            if pa.is_empty:
                pa.role = "pagebreak" if pa.has_pagebreak else "empty"
                continue
            txt = pa.raw_text.strip()
            det = N.detect_marker(txt)
            num_pr = pa.el.find("./" + w("pPr") + "/" + w("numPr"))
            if det is None and num_pr is not None:
                # Word automatic list without typed marker: treat as "digit_paren" items (the common
                # case in these documents) unless rules say otherwise
                det = (self.rules["auto_list_kind"], None, 0)
            if det is None:
                # unnumbered paragraph
                if not seen_marker:
                    ppr = pa.el.find(w("pPr"))
                    jc = ppr.find(w("jc")) if ppr is not None else None
                    if jc is not None and jc.get(w("val")) == "center":
                        pa.role = "cover"
                        continue
                    pa.role = "cover"  # text before the first marker is front matter
                    continue
                pa.role = "body"
                pa.depth = prev_depth + 1
                continue
            kind, number, mlen = det
            seen_marker = True
            typed_kind = kind
            # a typed kind that was already re-mapped (see below) keeps its mapped depth while we stay
            # under the same ancestors, so siblings typed alike stay siblings
            if typed_kind in clamp_map:
                depth = clamp_map[typed_kind]
                kind = depth % 6
            else:
                # resolve depth: candidates kind + 6k, pick the largest <= prev_depth + 1
                cands = [kind + 6 * k for k in range(0, 4)]
                depth = None
                for c in cands:
                    if c <= prev_depth + 1:
                        depth = c
                if depth is None:
                    # the typed marker skips a level (e.g. «أولاً.» right under «١.» in a document cut
                    # out of a larger one): structure wins — a child is one level under its parent
                    depth = prev_depth + 1
                    kind = depth % 6
                    clamp_map[typed_kind] = depth
                    pa.changes.append("level_clamped_to_parent+1")
            # leaving a clamped region (a shallower marker) forgets the deeper re-mappings
            for k_, d_ in list(clamp_map.items()):
                if d_ > depth and k_ != typed_kind:
                    del clamp_map[k_]
            pa.kind, pa.number, pa.depth = kind, number, depth
            if depth == 0:
                pa.role = "main"
            elif depth == 1:
                pa.role = "section"
            else:
                pa.role = "item"  # refined below: becomes "heading" when it has deeper children
            pa._mlen = mlen
            prev_depth = depth

        # structural numbering: the shallowest level present in the document is level 1 (١.)
        if self.rules["numbering"].get("first_level_is_top", True):
            depths_present = [pa.depth for pa in paras if pa.kind is not None]
            if depths_present:
                shift = min(depths_present)
                if shift > 0:
                    for pa in paras:
                        if pa.depth is not None:
                            pa.depth = max(pa.depth - shift, 0)
                        if pa.kind is not None:
                            pa.kind = pa.depth % 6
                            pa.role = "main" if pa.depth == 0 else ("section" if pa.depth == 1 else "item")
                    self.report["summary"]["levels_shifted_up"] = shift

        # roles are structural, not positional: a marker paragraph followed by deeper content is a
        # title of its level (main / section / heading: underlined, kept with the next paragraph);
        # a marker paragraph with nothing under it is an item whatever its depth (justified, kashida,
        # final dot) — so "أولاً. … ثانياً. …" typed directly under a section are items, not sections.
        marked = [pa for pa in paras if pa.role not in ("empty", "pagebreak", "cover")]
        for i, pa in enumerate(marked):
            if pa.kind is None or pa.depth is None:
                continue
            nxt = marked[i + 1] if i + 1 < len(marked) else None
            has_children = nxt is not None and nxt.depth is not None and nxt.depth > pa.depth
            if has_children:
                pa.role = "main" if pa.depth == 0 else ("section" if pa.depth == 1 else "heading")
            else:
                pa.role = "item"

        # counters & new markers
        counters = {}
        last_depth = -1
        fmt_cfg = self.rules["numbering"]
        for pa in paras:
            if pa.depth is None or pa.kind is None:
                continue
            d = pa.depth
            # reset deeper counters when climbing up
            for k in list(counters):
                if k > d:
                    del counters[k]
            if d not in counters:
                start = 1
                if d == 0 and fmt_cfg.get("keep_top_level_start", True) and pa.number:
                    start = pa.number
                counters[d] = start
            else:
                counters[d] += 1
            pa.new_marker = N.marker_text(d, counters[d], fmt_cfg)
            last_depth = d

    # ------------------------------------------------------------- applying
    def _apply(self, paras: List[Para]):
        R = self.rules
        # ---------- pass A: paragraph/run properties and markers ----------
        prev_role = None
        for pa in paras:
            if pa.role in ("empty", "pagebreak"):
                continue
            keep_with_heading = (prev_role in ("heading", "section", "main")) and pa.role in ("item", "body")
            prev_role = pa.role
            p = pa.el
            ppr = _get_ppr(p)
            if ppr.find(w("numPr")) is not None:
                _set_child(ppr, "numPr", PPR_ORDER, remove=True)
                pa.changes.append("auto_numbering_replaced")
            _set_child(ppr, "bidi", PPR_ORDER)
            if pa.kind is not None:
                self._replace_marker(pa)
            role_cfg = R["roles"].get(pa.role, R["roles"]["body"])
            if pa.role != "cover":
                sp = role_cfg["spacing"]
                attrs = {}
                if sp.get("before") is not None:
                    attrs["before"] = int(sp["before"] * 20)
                if sp.get("after") is not None:
                    attrs["after"] = int(sp["after"] * 20)
                if sp.get("line") is not None:
                    attrs["line"] = int(sp["line"] * 240)
                    attrs["lineRule"] = "auto"
                if attrs:
                    _set_child(ppr, "spacing", PPR_ORDER, attrs)
                _set_child(ppr, "contextualSpacing", PPR_ORDER, remove=True)
                if role_cfg.get("keep_lines", True):
                    _set_child(ppr, "keepLines", PPR_ORDER)
                if role_cfg.get("keep_next", False) or (keep_with_heading and R["layout"].get("keep_first_item", True)):
                    _set_child(ppr, "keepNext", PPR_ORDER)
                else:
                    _set_child(ppr, "keepNext", PPR_ORDER, remove=True)
                _set_child(ppr, "widowControl", PPR_ORDER)
                if pa.role == "section" and R["layout"]["page_break_before_section"] == "always":
                    _set_child(ppr, "pageBreakBefore", PPR_ORDER)
            self._apply_run_props(pa, role_cfg)

        # ---------- pass B: equal-width markers per level (dots on one line), child at parent's dot ----------
        W_total = self._text_width_twips()
        base = int(R["layout"]["base_indent"])
        # 1) measure every marker in its own font; choose one width per depth that every marker can reach:
        #    stretchable words reach it with kashida, non-stretchable ones (أولاً, أ.) by widening the glyphs
        #    up to `marker_max_scale`, and the widest ones may be narrowed down to `marker_min_scale`.
        maxw, lo_reach, by_depth = {}, {}, {}
        L = R["layout"]
        for pa in paras:
            if pa.kind is None:
                continue
            fam, bold, size = self._run_font(pa.marker_run, pa.el)
            pa._mfont = (fam, bold, size)
            lead, core, tail = self._split_marker(pa.new_marker)
            # empty bearing before the marker's first visible glyph (shaped as drawn, RTL): the marker
            # cell starts that much earlier so its INK begins exactly on the column
            if lead:
                pf, pb, ps = self._punct_font(pa)
                eb = self.fonts.edge_bearings(lead, pf, ps, pb)
            else:
                eb = self.fonts.edge_bearings(core, fam, size, bold) if core else None
            pa._ink_lead = int(round(eb[0])) if eb else 0
            pa.layout["ink_lead"] = pa._ink_lead
            # widths below are INK widths: from the first visible glyph to the end of the dot cell
            mw = self._marker_width(pa) - pa._ink_lead
            pa.layout["marker_width"] = int(round(mw))
            d = pa.depth
            # markers are equalized per level AND per kind: the titles of a level (one font) and the
            # plain items of the same level (another font) each get their own width
            g = (d, "item" if pa.role == "item" else "head")
            pa._grp = g
            # two-word ordinals (حادي عشر …) do not set the level width: they are narrowed a little and
            # start earlier, into the empty space right of the column, so their dot stays on the line
            two_word = " " in core.strip()
            core_w = self._marker_width(pa, None, 100) - self._marker_width(pa, "", 100)
            if L.get("equalize_to_widest", False):
                # every marker of the level reaches the width of the WIDEST one (two-word ordinals included)
                maxw[g] = max(maxw.get(g, 0.0), mw)
                lo_reach[g] = maxw[g]
            elif not two_word:
                maxw[g] = max(maxw.get(g, 0.0), mw)
                # the narrowest width this marker can reach: only the word is narrowed, never its dot/parens
                lo_reach[g] = max(lo_reach.get(g, 0.0), mw - core_w * (1.0 - L.get("marker_min_scale", 0.85)))
            stretchable = self._best_joint(core, L.get("marker_kashida_lam_alef", False)) is not None
            if stretchable and L.get("marker_tatweel_fixed", True) and len(core) == 2 and core[1] == N.TATWEEL:
                stretchable = False   # «جـ» «هـ» keep their single tatweel; the rest is a gap before the dot
            lamalef = (not stretchable and pa.kind not in (0, 3) and L.get("marker_lamalef_glyph", True)
                       and re.search("ل[اأإآ]", core) is not None)
            pa._eq = {"mw": mw, "core_w": core_w, "stretchable": stretchable, "lamalef": lamalef,
                      "digits": pa.kind in (0, 3), "two_word": two_word}
            if lamalef:
                i = re.search("ل[اأإآ]", core).start()
                pa._eq["prefix"] = core[:i]
                pa._eq["lig"] = core[i:]
                pa._eq["w_prefix"] = self.fonts.width_twips(core[:i], fam, size, bold)
                pa._eq["w_lig"] = self.fonts.width_twips(core[i:], fam, size, bold)
            by_depth.setdefault(g, []).append(pa)
        # one width per level, chosen by cost: kashida widening is free, narrowing the widest ones
        # (down to marker_min_scale) costs, widening glyphs costs, lengthening أولاً's lam-alef costs;
        # a marker that cannot reach the width at all costs most (the level then aligns starts).
        tg, target, policy = {}, {}, {}
        for g, lst in by_depth.items():
            if g not in maxw:   # only two-word markers in this group: their widest sets the width
                maxw[g] = max(pa._eq["mw"] for pa in lst)
                lo_reach[g] = maxw[g]
            lo, hi = lo_reach[g], maxw[g]
            best = None
            steps = 24
            for k in range(steps + 1):
                t = lo + (hi - lo) * k / steps
                cost, unreachable = 0.0, 0
                for pa in lst:
                    c, ok = self._marker_cost(pa, t)
                    cost += c
                    unreachable += 0 if ok else 1
                if best is None or cost < best[0] - 1e-9 or (abs(cost - best[0]) <= 1e-9 and t > best[1]):
                    best = (cost, t, unreachable)
            tg[g] = best[1]
            policy[g] = "align_start" if best[2] else "align_dot"
        for g in tg:   # the level width used by children = its titles' width (items' if no titles)
            d = g[0]
            if g[1] == "head" or d not in target:
                target[d] = tg[g]
        self._level_policy = {g: v for g, v in policy.items()}
        policy = {f"{g[0]}/{g[1]}": v for g, v in policy.items()}
        self.report["summary"]["level_policy"] = {str(k): v for k, v in sorted(policy.items())}
        # 2) stretch the narrower markers to the widest one of their depth (kashida + glyph scaling)
        natural_items = bool(L.get("natural_item_markers", False))
        for pa in paras:
            if pa.kind is None:
                continue
            if natural_items and pa._grp[1] == "item":
                continue   # items keep their natural marker: it starts on the parent's dot line
            fam, bold, size = pa._mfont
            self._equalize_marker(pa, tg[pa._grp], fam, bold, size)
        self.report["summary"]["marker_columns_twips"] = {str(d): int(round(v)) for d, v in sorted(target.items())}
        # 3) indents: the child column starts where the INK of the parent's dot ends (not the glyph cell)
        depths = sorted(target)
        ink_inset = {}   # depth -> twips between the dot cell's far edge and the dot's ink
        for pa in paras:
            if pa.kind is None:
                continue
            d = pa.depth
            if d in ink_inset:
                continue
            fam, bold, size = pa._mfont
            lead, core, tail = self._split_marker(pa.new_marker)
            if tail:
                pf, pb, ps = self._punct_font(pa)
                eb = self.fonts.edge_bearings(tail, pf, ps, pb)
            else:
                eb = self.fonts.edge_bearings(core, fam, size, bold) if core else None
            ink_inset[d] = int(round(eb[1])) if eb else 0
        self.report["summary"]["dot_ink_inset_twips"] = {str(k): v for k, v in ink_inset.items()}
        col = {}
        for d in depths:
            prev = [x for x in depths if x < d]
            if not prev:
                # top level: the ink line sits just inside the margin, so that even the marker with the
                # widest empty bearing can start at the margin and still put its ink on the line
                lead_max = max([pa._ink_lead for pa in paras if pa.kind is not None and pa.depth == d] + [0])
                col[d] = int(round(base + lead_max))
            else:
                pd = prev[-1]
                # the parent's dot cell ends at col[pd] + target[pd]; its ink ends ink_inset earlier
                col[d] = int(round(col[pd] + target[pd] - ink_inset.get(pd, 0)))
        self.report["summary"]["dot_ink_inset_twips"] = {str(k): v for k, v in ink_inset.items()}
        for pa in paras:
            if pa.role in ("empty", "pagebreak", "cover"):
                continue
            p = pa.el
            ppr = _get_ppr(p)
            role_cfg = R["roles"].get(pa.role, R["roles"]["body"])
            d = pa.depth or 0
            if pa.kind is not None:
                fam, bold, size = pa._mfont
                sp = self.fonts.width_twips(" ", fam, size, bold)
                ms = col[d]
                own = tg[pa._grp]
                if pa.kind in (0, 3) and not R["layout"].get("equalize_digits", False):
                    own = self._marker_width(pa) - pa._ink_lead  # (١) keeps one space after it
                if natural_items and pa._grp[1] == "item":
                    own = self._marker_width(pa) - pa._ink_lead  # natural width: one space, then the text
                    pa.layout["marker_offset"] = 0
                hang_full = int(round(own + sp * R["layout"]["marker_space_factor"]))
                L = ms + hang_full
                # a marker that could not be stretched to the full width starts a little later so that
                # its dot still lands on the level's dot line
                off = int(pa.layout.get("marker_offset", 0)) - pa._ink_lead
                if ms + off < 0:   # the marker cannot start before the margin
                    off = -ms
                    pa.layout["marker_offset"] = off
                hang = hang_full - off
                _set_child(ppr, "ind", PPR_ORDER, {"left": L, "hanging": hang})
                tabs = _set_child(ppr, "tabs", PPR_ORDER)
                for c in list(tabs):
                    tabs.remove(c)
                tab = etree.SubElement(tabs, w("tab"))
                tab.set(w("val"), "left")
                tab.set(w("pos"), str(L))
                pa.layout.update({"column": ms, "marker_start": ms + off, "dot_column": ms + int(round(tg[pa._grp])), "hanging": hang, "left": L})
            else:  # body text under a heading: starts at the parent's dot column, no hanging
                pd = max([x for x in depths if x < d], default=None)
                L = base if pd is None else col[pd] + int(round(target[pd])) - ink_inset.get(pd, 0)
                _set_child(ppr, "ind", PPR_ORDER, {"left": L, "hanging": 0, "firstLine": 0})
                _set_child(ppr, "tabs", PPR_ORDER, remove=True)
                pa.layout.update({"left": L})
            # ---------- alignment / line balancing ----------
            jc = role_cfg["align"]
            if pa.role in ("item", "body") and R["layout"]["balance_lines"]:
                mode = self._balance_paragraph(pa, W_total - L)
                if mode == "distribute":
                    jc = "distribute"
            if jc in (None, "", "start"):
                _set_child(ppr, "jc", PPR_ORDER, remove=True)
            else:
                _set_child(ppr, "jc", PPR_ORDER, {"val": jc})
        self.report["summary"]["text_width_twips"] = W_total
        self.report["summary"]["font_substitutions"] = dict(self.fonts.missing)

    # ------------------------------------------------ marker punctuation
    @staticmethod
    def _split_marker(marker: str):
        lead = ""
        tail = ""
        core = marker
        while core and core[0] in "(":
            lead += core[0]
            core = core[1:]
        while core and core[-1] in ").":
            tail = core[-1] + tail
            core = core[:-1]
        return lead, core, tail

    def _punct_font(self, pa):
        """(family, bold, size) used for the marker's dot/parentheses.
        The dot is sized from the marker font's stroke weight (the ink width of its alef): the dot's
        diameter is made `dot_weight` x that stroke, so heavy display fonts get a dot that matches them."""
        fam, bold, size = self._run_font(pa.marker_run, pa.el)
        cfg = self.rules["numbering"]
        pf = cfg.get("dot_font") or fam
        if pa.kind in (0, 3) and not cfg.get("scale_digit_parens", False):
            return pf, bold, size
        stem = self.fonts.glyph_bbox_em("\u0627", fam, bold)
        period = self.fonts.glyph_bbox_em(".", pf, bold)
        ps = size
        if stem and period and period[0] > 0:
            want = float(cfg.get("dot_weight", 1.4)) * stem[0] * size   # desired dot diameter in pt
            ps = want / period[0]
            ps = max(size * 1.0, min(ps, size * 1.8))
        return pf, bold, round(ps, 1)

    def _style_punct_run(self, r, pa):
        pf, bold, ps = self._punct_font(pa)
        rpr = _get_rpr(r)
        rf = rpr.find(w("rFonts"))
        if rf is None:
            rf = _set_child(rpr, "rFonts", RPR_ORDER)
        for k in ("ascii", "hAnsi", "cs", "eastAsia"):
            rf.set(w(k), pf)
        for k in ("asciiTheme", "hAnsiTheme", "cstheme", "eastAsiaTheme"):
            if rf.get(w(k)) is not None:
                del rf.attrib[w(k)]
        _set_child(rpr, "sz", RPR_ORDER, {"val": int(round(ps * 2))})
        _set_child(rpr, "szCs", RPR_ORDER, {"val": int(round(ps * 2))})
        _set_child(rpr, "spacing", RPR_ORDER, {"val": 0})
        _set_child(rpr, "u", RPR_ORDER, remove=True)

    def _marker_width(self, pa, core_text=None, scale=100):
        """Width of the whole marker: punctuation in the dot font + core in the marker font (scaled)."""
        fam, bold, size = pa._mfont
        lead, core, tail = self._split_marker(pa.new_marker)
        if core_text is not None:
            core = core_text
        pf, pb, ps = self._punct_font(pa)
        wl = self.fonts.width_twips(lead, pf, ps, pb) if lead else 0.0
        wt = self.fonts.width_twips(tail, pf, ps, pb) if tail else 0.0
        wc = self.fonts.width_twips(core, fam, size, bold) * scale / 100.0
        return wl + wc + wt

    # ------------------------------------------------ marker width equalisation
    _DUAL = set("بتثجحخسشصضطظعغفقكلمنهيئپچژگکی")
    _RIGHT = set("اأإآدذرزوؤةىء")
    _MARKS = set("\u064B\u064C\u064D\u064E\u064F\u0650\u0651\u0652\u0670")

    def _joints(self, word: str, lam_alef: bool = False):
        """Indexes i such that a tatweel may be inserted after word[i] (between two joined letters).
        The lam-alef ligature is kept intact unless `lam_alef` (used for markers such as أولاً, where
        a kashida between ل and ا is the only way to stretch the word)."""
        out = []
        n = len(word)
        for i, ch in enumerate(word):
            if ch not in self._DUAL:
                continue
            # next base letter (skip diacritics)
            j = i + 1
            while j < n and word[j] in self._MARKS:
                j += 1
            if j >= n:
                continue
            nxt = word[j]
            if nxt == N.TATWEEL:
                out.append(j - 1)   # an existing tatweel (جـ, هـ) is itself a joint: more may follow
                continue
            if nxt in self._DUAL or nxt in self._RIGHT:
                if ch == "ل" and nxt in "اأإآ" and not lam_alef:
                    continue  # keep the lam-alef ligature intact
                out.append(j - 1)  # insert after the diacritics that follow ch
        return out

    def _gap_safe(self, word: str) -> bool:
        """True when letter spacing cannot open a joined stroke: no tatweel and no two letters that
        connect (a single letter, or unjoined letters like أ و)."""
        if N.TATWEEL in word:
            # «جـ» «هـ»: a letter with its conventional tatweel — the gap goes after the tatweel
            return len(word) == 2 and word[1] == N.TATWEEL
        bases = [ch for ch in word if ch not in self._MARKS]
        for a, b in zip(bases, bases[1:]):
            if a in self._DUAL and (b in self._DUAL or b in self._RIGHT):
                return False
        return True

    def _best_joint(self, word: str, lam_alef: bool = False):
        """One kashida position per word, by the classic (Microsoft) priority scheme:
        after Seen/Sad family > before final ة ه د > before final ا ط ل ك > before a medial Baa-family
        letter that precedes ر ز ي ى > before final و ع غ ق ف > before any other final letter > any joint;
        ties go toward the end of the word. Returns the index after which to insert, or None."""
        joints = self._joints(word, lam_alef)
        if not joints:
            return None
        # base letters (skip diacritics) with their indexes
        bases = [(i, ch) for i, ch in enumerate(word) if ch not in self._MARKS and ch != N.TATWEEL]
        idx_of = {i: k for k, (i, _) in enumerate(bases)}
        last = len(bases) - 1
        best = None
        for j in joints:
            # j = index of the char after which the tatweel goes (may be a diacritic); find its base letter
            b = j
            while b >= 0 and word[b] in self._MARKS:
                b -= 1
            k = idx_of.get(b)
            if k is None:
                continue
            if k + 1 > last:
                # a trailing tatweel (جـ, هـ): the only joint, always usable
                if best is None or (0, -j) < best[0]:
                    best = ((0, -j), j)
                continue
            ch = bases[k][1]
            nxt = bases[k + 1][1]
            nxt2 = bases[k + 2][1] if k + 2 <= last else ""
            is_final = (k + 1 == last)
            if self.rules["layout"].get("kashida_prefer_word_end", True):
                # the user's house style: the stretch goes before the last letter whenever possible
                if is_final:
                    pr = 1
                elif ch in "سشصض":
                    pr = 2
                elif nxt in "بتثنيئپ" and nxt2 in "رزيىئ":
                    pr = 5
                else:
                    pr = 8
            elif ch in "سشصض":
                pr = 2
            elif is_final and nxt in "ةهد":
                pr = 3
            elif is_final and nxt in "اأإآطلكگ":
                pr = 4
            elif nxt in "بتثنيئپ" and not is_final and nxt2 in "رزيىئ":
                pr = 5
            elif is_final and nxt in "وعغقف":
                pr = 6
            elif is_final:
                pr = 7
            else:
                pr = 8
            key = (pr, -j)  # same priority -> later position wins
            if best is None or key < best[0]:
                best = (key, j)
        return best[1] if best else None

    def _lamalef_plan(self, pa, target: float):
        """How أولاً reaches `target` (ink width) while staying in the font's own style: first the natural
        gaps between its unjoined letters open (أ | و | لا — the glyphs themselves untouched), then the
        lam-alef lengthens a little, then the gaps open further, then a mild glyph widening, then the
        lam-alef up to its hard cap. Returns dict(delta_tw, scale, spacing_tw, width, ok, cost)."""
        L = self.rules["layout"]
        fam, bold, size = pa._mfont
        em = size * 20.0
        e = pa._eq
        punct = e["mw"] - e["core_w"]
        sp_soft = L.get("lamalef_spacing_soft_em", 0.10) * em
        sp_max = L.get("lamalef_spacing_max_em", 0.22) * em
        d_soft = L.get("lamalef_extend_soft_em", 0.28) * em
        d_max = L.get("lamalef_extend_max_em", 0.55) * em
        s_max = L.get("lamalef_scale_max", 1.08)
        gaps = max(len(e["prefix"]), 1)          # spacing is added after every prefix letter
        base = e["w_prefix"] + e["w_lig"]
        need = target - punct                     # core ink width wanted
        delta, scale, sp = 0.0, 1.0, 0.0

        def width():
            return scale * (base + delta) + gaps * sp

        if need <= base:
            scale = max(need / base, L.get("marker_min_scale", 0.85))
        else:
            # 1) gaps up to the soft cap
            sp = min((need - base) / gaps, sp_soft)
            if width() < need - 1:
                # 2) lam-alef up to its soft cap
                delta = min(need - gaps * sp - base, d_soft)
            if width() < need - 1:
                # 3) gaps up to the hard cap
                sp = min((need - (base + delta)) / gaps, sp_max)
            if width() < need - 1:
                # 4) mild glyph widening
                scale = min(need / (base + delta + 0.0) if (base + delta) else 1.0, s_max)
                scale = min(max((need - gaps * sp) / (base + delta), 1.0), s_max)
            if width() < need - 1:
                # 5) lam-alef up to the hard cap
                delta = min((need - gaps * sp) / scale - base, d_max)
        wdt = width() + punct
        ok = abs(wdt - target) <= 3
        cost = ((sp / sp_max) ** 2 * 1.5 if sp_max else 0) + (delta / d_max) ** 2 * 1.5 + ((scale - 1.0) / 0.08) ** 2 * 2.0
        if not ok:
            cost += 50 + abs(wdt - target) / em
        return {"delta_tw": delta, "scale": scale, "spacing_tw": sp, "width": wdt, "ok": ok, "cost": cost}

    def _marker_cost(self, pa, t: float):
        """(cost, reachable) of making this marker `t` twips wide."""
        L = self.rules["layout"]
        e = pa._eq
        mw, core_w = e["mw"], max(e["core_w"], 1.0)
        if e["digits"] and not L.get("equalize_digits", False):
            return 0.0, True
        if e.get("two_word") and t < mw:
            return 0.0, True          # starts earlier instead; never drives the level width
        if e["lamalef"]:
            pl = self._lamalef_plan(pa, t)
            if pl["ok"] or not L.get("marker_dot_gap", True):
                return pl["cost"], pl["ok"]
            em = pa._mfont[2] * 20.0
            gap = max(t - pl["width"], 0.0)
            return pl["cost"] - 50 - abs(pl["width"] - t) / em + 1.5 + (gap / em) ** 2 * 2.0, True
        if t >= mw:
            if e["stretchable"]:
                em = pa._mfont[2] * 20.0
                return ((t - mw) / em) ** 2 * L.get("kashida_cost", 0.6), True   # long kashida is discouraged
            s = (t - mw) / core_w + 1.0
            smax = L.get("marker_max_scale", 1.15)
            if s <= smax + 1e-6:
                return ((s - 1.0) / 0.15) ** 2 * 1.5, True
            if L.get("marker_dot_gap", True) and self._gap_safe(self._split_marker(pa.new_marker)[1]):
                em = pa._mfont[2] * 20.0
                gap = t - mw - core_w * (smax - 1.0)
                return 1.5 + (gap / em) ** 2 * 2.0, True     # a gap before the dot, kept as small as possible
            return 50 + (s - smax), False
        s = 1.0 - (mw - t) / core_w
        if s >= L.get("marker_min_scale", 0.85) - 1e-6:
            return ((1.0 - s) / 0.15) ** 2 * L.get("narrow_cost", 0.25), True      # mild narrowing is nearly invisible
        return 50 + (L.get("marker_min_scale", 0.85) - s), False

    def _lamalef_font(self, fam: str, bold: bool, size: float, delta_tw: float) -> str:
        """Family name of the derived font with the lam-alef lengthened by delta (built, registered for
        measurement and embedded in the document once per distinct width)."""
        from . import embed, lamalef
        em = size * 20.0
        delta_em = round(delta_tw / em, 3)
        path, used_fam, used_bold = self.fonts.resolve(fam, bold, True)
        key = (path, delta_em)
        cache = getattr(self, "_lamalef_fonts", None)
        if cache is None:
            cache = self._lamalef_fonts = {}
        if key in cache:
            return cache[key]
        stem = re.sub(r"[^A-Za-z0-9]", "", os.path.splitext(os.path.basename(path))[0])[:10] or "Base"
        family = f"SW LamAlef {stem} {int(round(delta_em * 1000)):03d}"
        import tempfile
        outdir = getattr(self, "_fontdir", None) or tempfile.mkdtemp(prefix="swar_fonts_")
        self._fontdir = outdir
        out = os.path.join(outdir, family.replace(" ", "") + ".ttf")
        lamalef.build(path, delta_em, out, family, mode=self.rules["layout"].get("lamalef_mode", "scale"))
        self.fonts._register(out, override=True)
        embed.embed_font(self._parts, out, family, bold=False)
        cache[key] = family
        self.report["summary"].setdefault("lamalef_fonts", []).append(
            {"family": family, "base": os.path.basename(path), "delta_em": delta_em})
        return family

    def _apply_lamalef(self, pa, target: float, policy: str):
        """Write the أولاً plan into the runs: prefix run (أو) in the document font, lam-alef run in the
        derived font; glyph scale on both, letter spacing on the prefix only."""
        fam, bold, size = pa._mfont
        pl = self._lamalef_plan(pa, target)
        e = pa._eq
        lead, core, tail = self._split_marker(pa.new_marker)
        family = self._lamalef_font(fam, bold, size, pl["delta_tw"]) if pl["delta_tw"] > 0 else None
        scale = int(round(pl["scale"] * 100))
        sp = int(round(pl["spacing_tw"]))
        run = pa.marker_run
        t = run.find(w("t"))
        t.text = e["prefix"]
        rpr = _get_rpr(run)
        _set_child(rpr, "spacing", RPR_ORDER, {"val": sp})
        if scale != 100:
            _set_child(rpr, "w", RPR_ORDER, {"val": scale})
        else:
            _set_child(rpr, "w", RPR_ORDER, remove=True)
        # the lam-alef run
        r2 = copy.deepcopy(run)
        t2 = r2.find(w("t"))
        t2.text = e["lig"]
        rp2 = _get_rpr(r2)
        _set_child(rp2, "spacing", RPR_ORDER, {"val": 0})
        if family:
            rf = rp2.find(w("rFonts"))
            if rf is None:
                rf = _set_child(rp2, "rFonts", RPR_ORDER)
            for a in list(rf.attrib):
                del rf.attrib[a]
            for a in ("ascii", "hAnsi", "cs", "eastAsia"):
                rf.set(w(a), family)
            # the derived font is built from the bold face already: no synthetic bold on top
            _set_child(rp2, "b", RPR_ORDER, remove=True)
            _set_child(rp2, "bCs", RPR_ORDER, remove=True)
        run.addnext(r2)
        pa._lamalef_run = r2
        # real width with the derived font measured by HarfBuzz for the remainder
        w_lig = self.fonts.width_twips(e["lig"], family, size, False) if family else e["w_lig"]
        width = pl["scale"] * (e["w_prefix"] + w_lig) + max(len(e["prefix"]), 1) * sp + (e["mw"] - e["core_w"])
        remainder = target - width
        gap = 0
        if remainder > 2 and self.rules["layout"].get("marker_dot_gap", True):
            # still short: the rest is a small gap before the dot (a no-break space whose advance is
            # trimmed/extended with character spacing), so the dot stays on the level's line
            gap = int(round(remainder))
            sp_w = self.fonts.width_twips("\u00A0", fam, size, bold) or self.fonts.width_twips(" ", fam, size, bold)
            r3 = copy.deepcopy(run)
            t3 = r3.find(w("t"))
            t3.text = "\u00A0"
            t3.set("{http://www.w3.org/XML/1998/namespace}space", "preserve")
            rp3 = _get_rpr(r3)
            _set_child(rp3, "w", RPR_ORDER, remove=True)
            _set_child(rp3, "spacing", RPR_ORDER, {"val": int(round(gap - sp_w))})
            r2.addnext(r3)
            remainder = 0
        offset = int(round(remainder)) if abs(remainder) > 2 else 0
        if policy == "align_start":
            offset = 0
        pa.layout["marker_display"] = pa.new_marker
        pa.layout["marker_scale"] = scale
        pa.layout["marker_offset"] = offset
        pa.layout["marker_gap_twips"] = gap
        pa.layout["lamalef"] = {"delta_em": round(pl["delta_tw"] / (size * 20.0), 3), "font": family,
                                "spacing_twips": sp, "reached": pl["ok"] or bool(gap)}
        pa.changes.append("marker_lamalef_widened")

    def _equalize_marker(self, pa, target: float, fam: str, bold: bool, size: float):
        """Make this marker occupy `target` twips: kashida inside the word when it has a joint, otherwise
        widen the glyphs (horizontal scaling); markers wider than the target are narrowed; any remainder
        shifts the marker start so its dot still lands on the level's dot line. Punctuation is untouched."""
        marker = pa.new_marker
        if pa.kind in (0, 3) and not self.rules["layout"].get("equalize_digits", False):
            return  # (١) and (١٠): digits are not stretched; only the paren of 10+ shifts by one digit
        mw = self._marker_width(pa) - pa._ink_lead
        deficit = target - mw
        lead, core, tail = self._split_marker(marker)
        L = self.rules["layout"]
        policy = getattr(self, "_level_policy", {}).get(getattr(pa, "_grp", None), "align_dot")
        if getattr(pa, "_eq", None) and pa._eq.get("lamalef") and deficit > 4:
            self._apply_lamalef(pa, target, policy)
            return
        if abs(deficit) <= 4:
            return
        scale = 100
        new_core = core
        fixed_tatweel = L.get("marker_tatweel_fixed", True) and len(core) == 2 and core[1] == N.TATWEEL
        if deficit > 0:
            jbest = self._best_joint(core, L.get("marker_kashida_lam_alef", True)) if (L.get("equalize_with_kashida", True) and not fixed_tatweel) else None
            if jbest is not None:
                tw = self.fonts.width_twips(N.TATWEEL, fam, size, bold) or 1
                k = int(deficit // tw)
                if k > 0:
                    new_core = core[:jbest + 1] + N.TATWEEL * k + core[jbest + 1:]
            else:
                core_w = self.fonts.width_twips(core, fam, size, bold) or 1
                scale = int(round(min((core_w + deficit) / core_w, L.get("marker_max_scale", 1.15)) * 100))
        else:
            core_w = self.fonts.width_twips(core, fam, size, bold) or 1
            scale = int(round(max((core_w + deficit) / core_w, L.get("marker_min_scale", 0.85)) * 100))
        mw2 = self._marker_width(pa, new_core, scale) - pa._ink_lead
        if scale == 100 and target - mw2 > 2:
            core_w = self.fonts.width_twips(new_core, fam, size, bold) or 1
            scale = int(round(min((core_w + (target - mw2)) / core_w, L.get("marker_max_scale", 1.15)) * 100))
            mw2 = self._marker_width(pa, new_core, scale) - pa._ink_lead
        remainder = target - mw2
        # a marker that cannot be stretched (a single letter: أ. د. و.) takes the rest as a small gap
        # between the letter and its dot — the glyph is not distorted further, the dot stays on the
        # level's dot line and the letter starts at the parent's dot
        gap = 0
        if remainder > 2 and new_core == core and self._gap_safe(core) and L.get("marker_dot_gap", True):
            gap = int(round(remainder))
            mw2 += gap
            remainder = target - mw2
        t = pa.marker_run.find(w("t"))
        rpr = _get_rpr(pa.marker_run)
        # a style may carry character spacing (Word's Title style: -0.5pt); the marker must be measured
        # and drawn without it, so it is pinned to zero
        _set_child(rpr, "spacing", RPR_ORDER, {"val": 0})
        if gap and len(new_core) > 1:
            # the gap sits after the LAST glyph only (after the tatweel of «جـ»), so nothing inside the
            # marker opens: the last glyph goes to its own run carrying the spacing
            t.text = new_core[:-1]
            r2 = copy.deepcopy(pa.marker_run)
            r2.find(w("t")).text = new_core[-1]
            _set_child(_get_rpr(r2), "spacing", RPR_ORDER, {"val": gap})
            pa.marker_run.addnext(r2)
            pa._gap_run = r2
        else:
            t.text = new_core
            if gap:
                _set_child(rpr, "spacing", RPR_ORDER, {"val": gap})
        pa.layout["marker_gap_twips"] = gap
        if scale != 100:
            _set_child(rpr, "w", RPR_ORDER, {"val": scale})
        else:
            _set_child(rpr, "w", RPR_ORDER, remove=True)
        # remainder > 0: the marker is still narrower than the level width -> it starts later;
        # remainder < 0: still wider (clamped at marker_min_scale) -> it starts earlier (hanging grows)
        offset = int(round(remainder)) if abs(remainder) > 2 else 0
        if policy == "align_start":
            offset = 0   # a level with markers that cannot reach the width aligns the starts instead
        pa.layout["marker_display"] = lead + new_core + tail
        pa.layout["marker_scale"] = scale
        pa.layout["marker_offset"] = offset
        if new_core != core or scale != 100 or offset or gap:
            pa.changes.append("marker_equalized")

    # --------------------------------------------------------- measurement
    def _text_width_twips(self) -> int:
        sect = self._root.find(".//" + w("body") + "/" + w("sectPr"))
        if sect is None:
            return 9000
        pg = sect.find(w("pgSz"))
        mar = sect.find(w("pgMar"))
        width = int(pg.get(w("w"))) if pg is not None else 11906
        left = int(mar.get(w("left"))) if mar is not None else 1440
        right = int(mar.get(w("right"))) if mar is not None else 1440
        gutter = int(mar.get(w("gutter")) or 0) if mar is not None else 0
        return width - left - right - gutter

    def _run_font(self, r, p):
        """(family, bold, size_pt) effective for complex-script text in run r of paragraph p."""
        fam = bold = None
        size = None
        rpr = r.find(w("rPr")) if r is not None else None
        if rpr is not None:
            rf = rpr.find(w("rFonts"))
            if rf is not None:
                fam = rf.get(w("cs")) or self._theme_font(rf.get(w("cstheme")))
            b = rpr.find(w("bCs"))
            if b is None:
                b = rpr.find(w("b"))
            if b is not None:
                bold = b.get(w("val")) not in ("0", "false")
            e = rpr.find(w("szCs"))
            if e is None:
                e = rpr.find(w("sz"))
            if e is not None:
                size = int(e.get(w("val"))) / 2
        ppr = p.find(w("pPr"))
        sid = None
        if ppr is not None:
            ps = ppr.find(w("pStyle"))
            if ps is not None:
                sid = ps.get(w("val"))
        chain = self._style_chain(sid)
        for st in chain:
            if fam is None and st.get("fam"):
                fam = st["fam"]
            if bold is None and st.get("bold") is not None:
                bold = st["bold"]
            if size is None and st.get("size"):
                size = st["size"]
        dd = self._style_info.get("__defaults__", {})
        if fam is None:
            fam = dd.get("fam") or self._theme_font("minorBidi") or "Arial"
        if bold is None:
            bold = bool(dd.get("bold"))
        if size is None:
            size = dd.get("size") or 11.0
        return fam, bold, float(size)

    def _style_chain(self, sid):
        out = []
        seen = set()
        while sid and sid not in seen and sid in self._style_info:
            seen.add(sid)
            out.append(self._style_info[sid])
            sid = self._style_info[sid].get("based")
        return out

    def _theme_font(self, key):
        if not key:
            return None
        if key in ("majorBidi", "majorHAnsi", "majorAscii", "majorEastAsia"):
            return self._theme.get("major")
        if key in ("minorBidi", "minorHAnsi", "minorAscii", "minorEastAsia"):
            return self._theme.get("minor")
        return None

    @staticmethod
    def _load_theme_fonts(theme_xml) -> dict:
        out = {}
        if not theme_xml:
            return out
        A = "{http://schemas.openxmlformats.org/drawingml/2006/main}"
        root = etree.fromstring(theme_xml)
        for key in ("major", "minor"):
            el = root.find(".//" + A + key + "Font")
            if el is None:
                continue
            name = None
            for f in el.findall(A + "font"):
                if f.get("script") == "Arab" and f.get("typeface"):
                    name = f.get("typeface")
            if not name:
                cs = el.find(A + "cs")
                if cs is not None and cs.get("typeface"):
                    name = cs.get("typeface")
            if not name:
                lat = el.find(A + "latin")
                if lat is not None:
                    name = lat.get("typeface")
            out[key] = name
        return out

    def _load_style_info(self, styles_xml) -> dict:
        """styleId -> {fam, bold, size, based}; '__defaults__' from docDefaults."""
        info = {}
        if not styles_xml:
            return info
        st = etree.fromstring(styles_xml)
        dd = st.find(".//" + w("docDefaults") + "/" + w("rPrDefault") + "/" + w("rPr"))
        info["__defaults__"] = self._rpr_info(dd)
        for style in st.findall(w("style")):
            sid = style.get(w("styleId"))
            based = style.find(w("basedOn"))
            d = self._rpr_info(style.find(w("rPr")))
            d["based"] = based.get(w("val")) if based is not None else None
            info[sid] = d
        return info

    def _rpr_info(self, rpr):
        d = {"fam": None, "bold": None, "size": None}
        if rpr is None:
            return d
        rf = rpr.find(w("rFonts"))
        if rf is not None:
            d["fam"] = rf.get(w("cs")) or self._theme_font(rf.get(w("cstheme")))
        b = rpr.find(w("bCs"))
        if b is None:
            b = rpr.find(w("b"))
        if b is not None:
            d["bold"] = b.get(w("val")) not in ("0", "false")
        e = rpr.find(w("szCs"))
        if e is None:
            e = rpr.find(w("sz"))
        if e is not None:
            d["size"] = int(e.get(w("val"))) / 2
        return d

    # ------------------------------------------------------ line balancing
    def _body_words(self, pa: Para):
        """Words of the paragraph text (excluding the marker run) with widths, plus the space width.
        Returns (words, widths, space_width, char_offsets, t_elements)."""
        chars = []  # (char, t_element, fontinfo)
        mruns = getattr(pa, "marker_runs", None) or ([pa.marker_run] if getattr(pa, "marker_run", None) is not None else [])
        for r in pa.el.iter(w("r")):
            if any(r is m for m in mruns):
                continue
            fi = self._run_font(r, pa.el)
            for node in r:
                ln = etree.QName(node).localname
                if ln == "t":
                    for ch in (node.text or ""):
                        chars.append((ch, node, fi))
                elif ln == "tab":
                    chars.append((" ", None, fi))
        text = "".join(c for c, _, _ in chars)
        words, widths = [], []
        i = 0
        n = len(chars)
        starts = []
        while i < n:
            while i < n and chars[i][0] == " ":
                i += 1
            if i >= n:
                break
            j = i
            while j < n and chars[j][0] != " ":
                j += 1
            word = text[i:j]
            fam, bold, size = chars[i][2]
            words.append(word)
            widths.append(self.fonts.width_twips(word, fam, size, bold))
            starts.append(i)
            i = j
        if chars:
            fam, bold, size = chars[0][2]
            sw = self.fonts.width_twips(" ", fam, size, bold)
        else:
            sw = 0
        return words, widths, sw, starts, chars

    def _balance_paragraph(self, pa: Para, W_text: float) -> str:
        """Line breaks + kashida fill for one paragraph. Returns the alignment mode."""
        R = self.rules["layout"]
        words, ww, sw, starts, chars = self._body_words(pa)
        if not words or W_text <= 0:
            return "justify"
        C = W_text * (1.0 - R["line_safety"])
        total = sum(ww) + sw * (len(ww) - 1)

        def greedy():
            lines, cur, curw = [], [], 0.0
            for i, wd in enumerate(ww):
                add = wd if not cur else curw + sw + wd
                if cur and add > C:
                    lines.append(cur)
                    cur, curw = [i], wd
                else:
                    cur.append(i)
                    curw = add
            if cur:
                lines.append(cur)
            return lines
        g = greedy()
        k = len(g)
        info = {"lines": k, "width": round(W_text), "fill": round(total / W_text, 2)}
        if k == 2 and total <= W_text * R.get("tight_fit_max", 0.99):
            k = 1
            info["lines"] = 1
            info["tight"] = True
        n = len(ww)
        pref = [0.0]
        for x in ww:
            pref.append(pref[-1] + x)

        def lw(i, j):  # width of words i..j-1
            return pref[j] - pref[i] + sw * (j - i - 1)
        if k == 1:
            lines = [list(range(n))]
        else:
            # balanced partition into k lines minimising the widest line (DP over word boundaries)
            INF = float("inf")
            best = [[INF] * (n + 1) for _ in range(k + 1)]
            cut = [[-1] * (n + 1) for _ in range(k + 1)]
            best[0][0] = 0.0
            for lines_used in range(1, k + 1):
                for j in range(1, n + 1):
                    for i in range(j - 1, -1, -1):
                        width = lw(i, j)
                        if width > C:
                            break
                        cand = max(best[lines_used - 1][i], width)
                        if cand < best[lines_used][j]:
                            best[lines_used][j] = cand
                            cut[lines_used][j] = i
            if best[k][n] == INF:
                lines = g
            else:
                lines, j = [], n
                for lines_used in range(k, 0, -1):
                    i = cut[lines_used][j]
                    lines.append(list(range(i, j)))
                    j = i
                lines.reverse()
        fills = [lw(l[0], l[-1] + 1) / W_text for l in lines]
        stretch_min = R.get("stretch_min_fill", 0.0)
        fill_kashida = R.get("fill_lines_with_kashida", True)
        # every line is stretched to the margin (user rule), unless it is below the optional threshold
        mode = "distribute" if min(fills) >= stretch_min else "justify"
        info.update({"mode": mode, "fills": [round(f, 2) for f in fills]})
        pa.layout.update(info)
        if len(lines) > 1 and R["insert_breaks"]:
            break_word_idx = [l[-1] + 1 for l in lines[:-1]]
            self._insert_breaks(pa, [starts[i] for i in break_word_idx], chars)
            pa.changes.append(f"lines_balanced:{len(lines)}")
        if mode == "distribute" and fill_kashida:
            target = W_text * R.get("kashida_fill_target", 0.985)
            added = 0
            # process lines from the last to the first so earlier char indexes stay valid
            for l in reversed(lines):
                width = lw(l[0], l[-1] + 1)
                deficit = target - width
                if deficit <= 0:
                    continue
                # candidate joints of this line: (char_index_after_which, tatweel_width)
                joints = []
                one_per_word = R.get("kashida_one_per_word", True)
                for wi in l:
                    word = words[wi]
                    fam, bold, size = chars[starts[wi]][2]
                    tw = self.fonts.width_twips(N.TATWEEL, fam, size, bold) or 1
                    if one_per_word:
                        jb = self._best_joint(word)
                        if jb is not None:
                            joints.append((starts[wi] + jb, tw))
                    else:
                        for jpos in self._joints(word):
                            joints.append((starts[wi] + jpos, tw))
                if not joints:
                    continue
                # even distribution: how many tatweels in total, then spread over joints
                avg_tw = sum(t for _, t in joints) / len(joints)
                count = int(deficit // avg_tw)
                cap = R.get("kashida_max_per_joint", 0)
                if cap and one_per_word and count > cap * len(joints):
                    # a short line: one stretch per word would be a long ugly bar; use every joint
                    joints = []
                    for wi in l:
                        word = words[wi]
                        fam, bold, size = chars[starts[wi]][2]
                        tw = self.fonts.width_twips(N.TATWEEL, fam, size, bold) or 1
                        jb = self._best_joint(word)
                        if jb is not None:
                            joints.append((starts[wi] + jb, tw))
                        if len(word) >= 5:
                            # second, gentler stretch near the middle of longer words
                            others = [j for j in self._joints(word) if j != jb]
                            if others:
                                mid = min(others, key=lambda j: abs(j - len(word) / 2.0))
                                joints.append((starts[wi] + mid, tw))
                    if not joints:
                        continue
                    avg_tw = sum(t for _, t in joints) / len(joints)
                    count = int(deficit // avg_tw)
                if count <= 0:
                    continue
                per = [count // len(joints)] * len(joints)
                extra = count - sum(per)
                if extra:
                    step = len(joints) / extra
                    for e in range(extra):
                        per[int(e * step)] += 1
                for (pos, _), cnt in sorted(zip(joints, per), reverse=True):
                    if cnt:
                        self._insert_tatweel(chars, pos, cnt)
                        added += cnt
            if added:
                pa.changes.append(f"kashida_filled:{added}")
                pa.layout["kashida"] = added
        return mode

    @staticmethod
    def _insert_tatweel(chars, pos: int, count: int):
        """Insert `count` tatweel characters right after the character at index `pos` (in its w:t)."""
        t = chars[pos][1]
        if t is None:
            return
        idx = 0
        for q in range(pos - 1, -1, -1):
            if chars[q][1] is t:
                idx += 1
            else:
                break
        s = t.text or ""
        # idx counts chars of this t that precede pos in the table; the table may also hold a removed
        # space entry (soft-break conversion) which no longer exists in the text: clamp defensively
        idx = min(idx, max(len(s) - 1, 0))
        t.text = s[:idx + 1] + N.TATWEEL * count + s[idx + 1:]

    def _insert_breaks(self, pa: Para, char_positions, chars):
        """Replace the space just before each char position by a soft line break (w:br)."""
        for pos in sorted(char_positions, reverse=True):
            sp = pos - 1
            if sp < 0 or chars[sp][0] != " " or chars[sp][1] is None:
                continue
            t = chars[sp][1]
            # index of this space inside its w:t
            idx = 0
            for q in range(sp - 1, -1, -1):
                if chars[q][1] is t:
                    idx += 1
                else:
                    break
            s = t.text or ""
            left, right = s[:idx], s[idx + 1:]
            r = t.getparent()
            rpr = r.find(w("rPr"))
            t.text = left
            if left.endswith(" ") or left.startswith(" "):
                t.set("{http://www.w3.org/XML/1998/namespace}space", "preserve")
            # move any following siblings of t (rare) to the new run
            following = [c for c in r if list(r).index(c) > list(r).index(t)]
            br_run = etree.Element(w("r"))
            if rpr is not None:
                br_run.append(copy.deepcopy(rpr))
            etree.SubElement(br_run, w("br"))
            right_run = etree.Element(w("r"))
            if rpr is not None:
                right_run.append(copy.deepcopy(rpr))
            rt = etree.SubElement(right_run, w("t"))
            rt.text = right
            rt.set("{http://www.w3.org/XML/1998/namespace}space", "preserve")
            for c in following:
                r.remove(c)
                right_run.append(c)
            parent = r.getparent()
            i = list(parent).index(r)
            parent.insert(i + 1, br_run)
            parent.insert(i + 2, right_run)
            # keep the chars table consistent for earlier positions: entries after sp now belong to rt
            for q in range(sp + 1, len(chars)):
                if chars[q][1] is t:
                    chars[q] = (chars[q][0], rt, chars[q][2])

    @staticmethod
    def _glue_before_word(pa: Para, pos, chars):
        sp = pos - 1
        if sp < 0 or chars[sp][0] != " " or chars[sp][1] is None:
            return
        t = chars[sp][1]
        idx = 0
        for q in range(sp - 1, -1, -1):
            if chars[q][1] is t:
                idx += 1
            else:
                break
        s = t.text or ""
        t.text = s[:idx] + "\u00A0" + s[idx + 1:]

    def _replace_marker(self, pa: Para):
        """Delete the typed marker characters at the start of the paragraph and insert the new marker + tab."""
        p = pa.el
        ts = [t for t in p.iter(w("t"))]
        full = "".join(t.text or "" for t in ts)
        mlen = getattr(pa, "_mlen", 0)
        # recompute on the cleaned text to be safe
        det = N.detect_marker(full)
        mlen = det[2] if det else 0
        remaining = mlen
        for t in ts:
            if remaining <= 0:
                break
            s = t.text or ""
            cut = min(len(s), remaining)
            t.text = s[cut:]
            remaining -= cut
        # drop runs that became empty (but keep run props of the first run as the template)
        template_rpr = None
        for t in ts:
            r = t.getparent()
            if template_rpr is None:
                rp = r.find(w("rPr"))
                template_rpr = copy.deepcopy(rp) if rp is not None else None
            if (t.text or "") == "" and len([c for c in r if etree.QName(c).localname not in ("rPr",)]) == 1:
                r.getparent().remove(r)
        # strip leading spaces of the first remaining text
        ts = [t for t in p.iter(w("t"))]
        for t in ts:
            if t.text:
                t.text = t.text.lstrip(" ")
                if t.text:
                    break
        # build the marker as up to three runs: "(" | word/number | ")." — punctuation gets its own
        # font/size so the dot is clearly visible whatever the heading font draws for a period
        lead, core, tail = self._split_marker(pa.new_marker)

        def mkrun(text, punct=False):
            r = etree.Element(w("r"))
            if template_rpr is not None:
                r.append(copy.deepcopy(template_rpr))
            if punct:
                self._style_punct_run(r, pa)
            t = etree.SubElement(r, w("t"))
            t.text = text
            t.set("{http://www.w3.org/XML/1998/namespace}space", "preserve")
            return r
        core_run = mkrun(core)
        pa.marker_run = core_run          # punctuation sizing reads the core run's font/size
        runs = []
        if lead:
            runs.append(mkrun(lead, punct=True))
        runs.append(core_run)
        if tail:
            runs.append(mkrun(tail, punct=True))
        etree.SubElement(runs[-1], w("tab"))
        ppr = p.find(w("pPr"))
        pos = list(p).index(ppr) + 1 if ppr is not None else 0
        for i, r in enumerate(runs):
            p.insert(pos + i, r)
        pa.marker_run = core_run
        pa.marker_runs = runs
        old = full[:mlen].strip()
        if old != pa.new_marker:
            pa.changes.append(f"marker: '{old}' -> '{pa.new_marker}'")

    def _apply_run_props(self, pa: Para, role_cfg: dict):
        font = role_cfg.get("font")
        size = role_cfg.get("size")
        bold = role_cfg.get("bold")
        underline = role_cfg.get("underline")
        color = self.rules["fonts"].get("force_color")
        for r in pa.el.iter(w("r")):
            rpr = _get_rpr(r)
            if font:
                _set_child(rpr, "rFonts", RPR_ORDER, {"ascii": font, "hAnsi": font, "cs": font, "eastAsia": font})
                pa.changes.append("font_set")
            if size:
                _set_child(rpr, "sz", RPR_ORDER, {"val": int(size * 2)})
                _set_child(rpr, "szCs", RPR_ORDER, {"val": int(size * 2)})
                pa.changes.append("size_set")
            if bold is True:
                _set_child(rpr, "b", RPR_ORDER)
                _set_child(rpr, "bCs", RPR_ORDER)
            elif bold is False:
                _set_child(rpr, "b", RPR_ORDER, remove=True)
                _set_child(rpr, "bCs", RPR_ORDER, remove=True)
            if any(r is m for m in (getattr(pa, "marker_runs", None) or [getattr(pa, "marker_run", None)])):
                _set_child(rpr, "u", RPR_ORDER, remove=True)  # the number/letter is never underlined
            elif underline is True:
                _set_child(rpr, "u", RPR_ORDER, {"val": "single"})
            elif underline is False:
                _set_child(rpr, "u", RPR_ORDER, remove=True)
            if color:
                _set_child(rpr, "color", RPR_ORDER, {"val": color})
            # Arabic runs must be flagged rtl + complex script so Word uses the cs font/size
            _set_child(rpr, "rtl", RPR_ORDER)
            if self.rules["fonts"].get("pin_theme_fonts", True) and not font:
                rf = rpr.find(w("rFonts"))
                if rf is None or not rf.get(w("cs")):
                    fam, _, _ = self._run_font(r, pa.el)
                    if fam:
                        if rf is None:
                            rf = _set_child(rpr, "rFonts", RPR_ORDER)
                        rf.set(w("cs"), fam)
                        for k in ("cstheme",):
                            if rf.get(w(k)) is not None:
                                del rf.attrib[w(k)]

    def _protect_last_word(self, pa: Para):
        """Join the last two words with a no-break space so a single word never dangles on the last line."""
        ts = [t for t in pa.el.iter(w("t"))]
        # skip the marker run (first run containing new_marker) - it has a tab, not spaces
        body_ts = [t for t in ts if (t.text or "") != pa.new_marker]
        full = "".join(t.text or "" for t in body_ts)
        stripped = full.rstrip(" .،؛:")
        if stripped.count(" ") < 2:
            return
        last_space = stripped.rfind(" ")
        # don't glue if the last word is very long (would create a huge non-breaking block)
        if len(stripped) - last_space > 18:
            return
        # locate the w:t that contains that index
        pos = 0
        for t in body_ts:
            s = t.text or ""
            if pos <= last_space < pos + len(s):
                i = last_space - pos
                t.text = s[:i] + " " + s[i + 1:]
                pa.changes.append("orphan_word_protected")
                return
            pos += len(s)

    # ------------------------------------------------- header / footer
    CLASSIFICATIONS = ["عام", "محدود", "سري", "سري وشخصي", "سري للغاية"]

    def _apply_header_footer(self, root):
        pg = self.rules.get("page", {})
        cls = pg.get("classification")
        num = pg.get("numbering") or {}
        changed = {}
        # page numbering: the cover carries no number and the first content page reads `start`.
        # The cover becomes its own section (no header/footer) and the body section numbers from
        # `start` with a plain PAGE field — no pgNumType tricks and no nested fields, so Word and
        # LibreOffice agree.
        if num.get("enabled", True):
            start = int(num.get("start", 1) or 1)
            split = self._split_cover_section(root)
            if not self._has_cover():
                for sect in root.iter(w("sectPr")):
                    tp = sect.find(w("titlePg"))
                    if tp is not None:
                        sect.remove(tp)   # no cover page: the first page is numbered like the rest
            for sect in root.iter(w("sectPr")):
                if sect.getparent() is not None and etree.QName(sect.getparent()).localname == "pPr":
                    continue  # the cover section
                pn = sect.find(w("pgNumType"))
                if pn is None:
                    pn = _set_child(sect, "pgNumType", SECT_ORDER)
                pn.set(w("start"), str(start))
                if split:
                    tp = sect.find(w("titlePg"))
                    if tp is not None:
                        sect.remove(tp)
            changed["cover_section"] = split
        # digits font: Latin digits drawn with Arabic-Indic shapes, embedded in the file
        self._digits_font = None
        if num.get("enabled", True) and num.get("digits", "arabic_indic") == "arabic_indic":
            here = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
            ttf = os.path.join(here, "fonts", "ServiceWordDigits.ttf")
            if os.path.exists(ttf):
                try:
                    from . import embed
                    self._digits_font = embed.embed_font(self._parts, ttf, "ServiceWord Digits", bold=True)
                    changed["digits_font"] = "ServiceWord Digits (embedded)"
                except Exception as exc:  # never fail the whole run for the footer font
                    self.report.setdefault("warnings", []).append(f"digits font not embedded: {exc}")
        for name in list(self._parts):
            if not (name.startswith("word/header") or name.startswith("word/footer")) or not name.endswith(".xml"):
                continue
            part = etree.fromstring(self._parts[name])
            touched = False
            # classification word
            if cls:
                for t in part.iter(w("t")):
                    if (t.text or "").strip() in self.CLASSIFICATIONS and (t.text or "").strip() != cls:
                        t.text = cls
                        touched = True
                        changed["classification"] = cls
            # page number line: "( PAGE - END )"
            if name.startswith("word/footer") and num.get("enabled", True):
                for p in part.iter(w("p")):
                    instr = "".join(i.text or "" for i in p.iter(w("instrText")))
                    if "PAGE" in instr.upper():
                        self._rebuild_page_line(p, num)
                        touched = True
                        changed["page_numbers"] = f"({num.get('start', 1)} - {num.get('end') or 'NUMPAGES'})"
                        break
            if touched:
                self._parts[name] = etree.tostring(part, xml_declaration=True, encoding="UTF-8", standalone=True)
        if changed:
            self.report["summary"]["header_footer"] = changed

    def _rebuild_page_line(self, p, num):
        """Rewrite the footer paragraph as: ( PAGE - END ). PAGE/NUMPAGES are wrapped in a formula
        field that subtracts the unnumbered cover; the results are set in the embedded digits font
        (Latin digits drawn as ٠١٢…), so they read Arabic-Indic in Word, LibreOffice and print."""
        first_run = p.find(w("r"))
        rpr = copy.deepcopy(first_run.find(w("rPr"))) if first_run is not None and first_run.find(w("rPr")) is not None else None
        for child in list(p):
            if etree.QName(child).localname != "pPr":
                p.remove(child)
        digits = num.get("digits", "arabic_indic") == "arabic_indic"
        dfont = getattr(self, "_digits_font", None)
        k = getattr(self, "_page_offset", 0)

        def run(text=None, fld=None, instr=None, digits_font=False):
            r = etree.SubElement(p, w("r"))
            if rpr is not None:
                r.append(copy.deepcopy(rpr))
            if digits_font and dfont:
                rp = _get_rpr(r)
                rf = rp.find(w("rFonts"))
                if rf is None:
                    rf = _set_child(rp, "rFonts", RPR_ORDER)
                for a in list(rf.attrib):
                    del rf.attrib[a]
                for a in ("ascii", "hAnsi", "cs", "eastAsia"):
                    rf.set(w(a), dfont)
            if text is not None:
                t = etree.SubElement(r, w("t"))
                t.text = text
                t.set("{http://www.w3.org/XML/1998/namespace}space", "preserve")
            if fld is not None:
                etree.SubElement(r, w("fldChar")).set(w("fldCharType"), fld)
            if instr is not None:
                it = etree.SubElement(r, w("instrText"))
                it.text = instr
                it.set("{http://www.w3.org/XML/1998/namespace}space", "preserve")
            return r

        def conv(v):
            v = str(v)
            if dfont:
                return v                      # the font draws them Arabic-Indic
            return N.to_arabic_indic(v) if digits else v

        def field(code, cached):
            run(fld="begin", digits_font=True)
            run(instr=f" {code} ", digits_font=True)
            run(fld="separate", digits_font=True)
            run(text=cached, digits_font=True)
            run(fld="end", digits_font=True)

        start = int(num.get("start", 1) or 1)
        end = num.get("end")
        run(text="( ")
        field("PAGE", conv(start))
        run(text=" - ")
        if end:
            run(text=conv(int(end)), digits_font=True)
        else:
            # pages of the numbered section (Word); LibreOffice shows the cached estimate
            field("SECTIONPAGES", conv(self._estimate_pages()))
        run(text=" )")

    def _estimate_pages(self) -> int:
        """Cached value for the page-count field: the source file's own page count (docProps/app.xml)
        minus the cover, or 1. Word recomputes it on open."""
        try:
            app = self._parts.get("docProps/app.xml")
            if app:
                m = re.search(rb"<Pages>(\d+)</Pages>", app)
                if m:
                    n = int(m.group(1)) - (1 if self._has_cover() else 0)
                    return max(n, 1)
        except Exception:
            pass
        return 1

    def _has_cover(self) -> bool:
        return any(pa.role == "cover" for pa in getattr(self, "_paras", []))

    def _split_cover_section(self, root) -> bool:
        """Make the cover page its own section with no header or footer. Returns True when done."""
        paras = getattr(self, "_paras", [])
        covers = [pa for pa in paras if pa.role == "cover" and pa.el.getparent() is not None]
        if not covers:
            return False
        body_sect = root.find(w("body") + "/" + w("sectPr"))
        if body_sect is None:
            return False
        last = covers[-1]
        # the first paragraph with content after the cover
        after = [pa for pa in paras[paras.index(last) + 1:] if pa.el.getparent() is not None]
        first_body = next((pa for pa in after if pa.role not in ("empty", "pagebreak")), None)
        if first_body is None:
            return False
        # drop explicit page breaks between cover and body: the section break makes the new page
        for pa in after:
            if pa is first_body:
                break
            if pa.role == "pagebreak" and not pa.raw_text.strip():
                pa.el.getparent().remove(pa.el)
        for br in list(last.el.iter(w("br"))):
            if br.get(w("type")) == "page":
                br.getparent().remove(br)
        _set_child(_get_ppr(first_body.el), "pageBreakBefore", PPR_ORDER, remove=True)
        for br in list(first_body.el.iter(w("br"))):
            if br.get(w("type")) == "page":
                br.getparent().remove(br)
        ppr = _get_ppr(last.el)
        old = ppr.find(w("sectPr"))
        if old is not None:
            ppr.remove(old)
        sect = copy.deepcopy(body_sect)
        for c in list(sect):
            if etree.QName(c).localname in ("headerReference", "footerReference", "titlePg", "pgNumType", "type"):
                sect.remove(c)
        t = etree.Element(w("type"))
        t.set(w("val"), "nextPage")
        sect.insert(0, t)
        ppr.append(sect)
        last.changes.append("cover_section")
        return True

    # ------------------------------------------------------ page setup
    def _apply_page_setup(self, root):
        pg = self.rules.get("page", {})
        if pg.get("margins") in (None, "file"):
            return
        m = pg["margins"]
        for sect in root.iter(w("sectPr")):
            mar = sect.find(w("pgMar"))
            if mar is None:
                continue
            for side in ("right", "left", "top", "bottom"):
                v = m.get(side)
                if v is not None:
                    mar.set(w(side), str(int(round(float(v) / 2.54 * 1440))))
        self.report["summary"]["page_margins_cm"] = m

    # ------------------------------------------------------ page breaks
    # ------------------------------------------------------------ final dot
    _TERMINAL = ".:؛؟!،…\u06d4"

    def _ensure_final_dot(self, paras: List[Para]):
        """Every item/body paragraph ends with a dot: appended to the last text run when the text ends
        with a letter, digit or closing bracket (never after : ؛ ؟ ! or an existing dot)."""
        c = self.rules.get("cleanup", {})
        if not c.get("final_dot", True):
            return
        roles = set(c.get("final_dot_roles") or ["item", "body"])
        for pa in paras:
            if pa.role not in roles or pa.el.getparent() is None:
                continue
            ts = [t for t in pa.el.iter(w("t")) if t.text]
            if not ts:
                continue
            full = "".join(t.text for t in ts).rstrip()
            if not full or full[-1] in self._TERMINAL:
                continue
            # strip trailing spaces from the tail runs, then append the dot to the last non-empty run
            for t in reversed(ts):
                if t.text.strip():
                    t.text = t.text.rstrip() + "."
                    break
                t.text = ""
            pa.raw_text = self._text_of(pa.el)
            pa.changes.append("final_dot_added")

    # ------------------------------------------------- keep numbered groups together
    def _text_height_twips(self) -> int:
        sect = self._root.find(".//" + w("body") + "/" + w("sectPr"))
        if sect is None:
            return 12000
        pg = sect.find(w("pgSz"))
        mar = sect.find(w("pgMar"))
        h = int(pg.get(w("h"))) if pg is not None else 16838
        top = int(mar.get(w("top"))) if mar is not None else 1440
        bottom = int(mar.get(w("bottom"))) if mar is not None else 1440
        return h - top - bottom

    def _para_height_twips(self, pa) -> float:
        """Rough height of a formatted paragraph: lines x font line height + spacing before/after."""
        role_cfg = self.rules["roles"].get(pa.role, self.rules["roles"]["body"])
        sp = role_cfg.get("spacing") or {}
        r = pa.marker_run if pa.marker_run is not None else pa.el.find(w("r"))
        fam, bold, size = self._run_font(r, pa.el)
        if role_cfg.get("size"):
            size = float(role_cfg["size"])
        lines = int(pa.layout.get("lines") or 1)
        mult = float(sp.get("line") or 1.0)
        line_h = size * 20 * 1.22 * mult
        return lines * line_h + (sp.get("before") or 0) * 20 + (sp.get("after") or 0) * 20

    def _keep_groups(self, paras: List[Para]):
        """A heading and the numbered items under it stay on one page (keepNext chain) when the whole
        group is short enough (`keep_group_max_share` of the text height); longer groups only keep the
        heading with its first `keep_group_head_items` items and the last `keep_group_tail_items` items
        together, so a page never ends with a lone heading nor starts with a lone last item. Word still
        breaks a chain that cannot fit on any page (the worst case)."""
        L = self.rules["layout"]
        if not L.get("keep_groups", True):
            return
        share = float(L.get("keep_group_max_share", 0.5))
        head_n = int(L.get("keep_group_head_items", 2))
        tail_n = int(L.get("keep_group_tail_items", 2))
        H = self._text_height_twips() * share
        seq = [pa for pa in paras if pa.role not in ("empty", "pagebreak", "cover") and pa.el.getparent() is not None]
        groups = 0
        for i, pa in enumerate(seq):
            if pa.kind is None:
                continue
            d = pa.depth or 0
            j = i + 1
            sub = []
            while j < len(seq) and (seq[j].depth or 0) > d:
                sub.append(seq[j])
                j += 1
            if not sub:
                continue
            height = self._para_height_twips(pa) + sum(self._para_height_twips(q) for q in sub)
            chain = [pa] + sub
            if height <= H:
                keep = chain[:-1]                       # the whole group moves together
                pa.layout["group"] = "whole"
            else:
                keep = chain[:head_n] + chain[-tail_n - 1:-1]   # heading+first items, last items
                pa.layout["group"] = "partial"
            for q in keep:
                _set_child(_get_ppr(q.el), "keepNext", PPR_ORDER)
            pa.layout["group_height_twips"] = int(height)
            groups += 1
        self.report["summary"]["groups_kept"] = groups

    def _fold_page_breaks(self, paras: List[Para]):
        """A paragraph holding only a page break can spill onto a new page by itself and leave an empty
        page; replace it with pageBreakBefore on the next paragraph with content."""
        if not self.rules["layout"].get("fold_page_breaks", True):
            return
        for i, pa in enumerate(paras):
            if pa.role != "pagebreak" or pa.raw_text.strip():
                continue
            nxt = next((q for q in paras[i + 1:] if q.role not in ("empty", "pagebreak")), None)
            if nxt is None or nxt.el.getparent() is None:
                continue
            ppr = _get_ppr(nxt.el)
            _set_child(ppr, "pageBreakBefore", PPR_ORDER)
            parent = pa.el.getparent()
            if parent is not None and pa.el.find(w("pPr") + "/" + w("sectPr")) is None:
                parent.remove(pa.el)
                pa.changes.append("page_break_folded")

    # --------------------------------------------------------- empties
    def _remove_empties(self, paras: List[Para], body):
        if not self.rules["cleanup"]["remove_empty_paragraphs"]:
            return
        removed = 0
        first_marker = next((pa.index for pa in paras if pa.kind is not None), None)
        for pa in paras:
            if pa.role == "empty":
                if first_marker is not None and pa.index < first_marker:
                    continue  # front matter spacing above the cover title stays
                p = pa.el
                if p.find(w("pPr") + "/" + w("sectPr")) is not None:
                    continue
                parent = p.getparent()
                # keep one empty paragraph at the end of a table cell (schema requirement)
                if etree.QName(parent).localname == "tc" and len(parent.findall(w("p"))) == 1:
                    continue
                parent.remove(p)
                pa.changes.append("empty_paragraph_removed")
                removed += 1
        self.report["summary"]["empty_paragraphs_removed"] = removed

    # ------------------------------------------------------------ report
    def _finish_report(self, paras: List[Para]):
        counts = {}
        rows = []
        for pa in paras:
            for c in pa.changes:
                key = c.split(":")[0]
                counts[key] = counts.get(key, 0) + 1
            final = self._text_of(pa.el).replace("\t", " ").strip() if pa.el.getparent() is not None else ""
            disp = pa.layout.get("marker_display") or pa.new_marker
            if disp and final.startswith(disp):
                final = final[len(disp):].strip()
            elif pa.new_marker and final.startswith(pa.new_marker):
                final = final[len(pa.new_marker):].strip()
            rows.append({
                "index": pa.index,
                "role": pa.role,
                "depth": pa.depth,
                "marker": pa.new_marker,
                "text": (pa.raw_text.strip()[:160]),
                "clean_text": final,
                "layout": pa.layout,
                "changes": pa.changes,
            })
        self.report["paragraphs"] = rows
        self.report["summary"]["change_counts"] = counts
        self.report["summary"]["paragraphs"] = len(paras)


def format_docx(src: str, dst: str, rules: Optional[dict] = None, report_path: Optional[str] = None) -> dict:
    f = Formatter(rules)
    rep = f.format_file(src, dst)
    if report_path:
        with open(report_path, "w", encoding="utf-8") as fh:
            json.dump(rep, fh, ensure_ascii=False, indent=2)
    return rep
