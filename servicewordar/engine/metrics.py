# -*- coding: utf-8 -*-
"""
Real text measurement with the document's own fonts (HarfBuzz shaping via uharfbuzz).

The catalog scans font folders (Windows system + user fonts, the tool's own `fonts/` folder,
Linux dirs) and maps family name + bold -> file. Widths are returned in twips (1/20 pt).
When a family is missing, the nearest fallback is used and the substitution is recorded in
`missing` so the report can say so (never silently).
"""
from __future__ import annotations

import os
import sys
from functools import lru_cache

try:
    import uharfbuzz as hb
    from fontTools.ttLib import TTFont
    HAVE_HB = True
except Exception:  # pragma: no cover
    HAVE_HB = False

FALLBACKS = {
    # family -> ordered alternatives when the file is not available
    "arial": ["Arial", "Liberation Sans", "Noto Naskh Arabic", "DejaVu Sans"],
    "times new roman": ["Times New Roman", "Liberation Serif", "Noto Naskh Arabic", "DejaVu Serif"],
    "calibri": ["Calibri", "Carlito", "Arial", "Noto Naskh Arabic"],
    "simplified arabic": ["Simplified Arabic", "Arial", "Noto Naskh Arabic"],
    "traditional arabic": ["Traditional Arabic", "Noto Naskh Arabic", "Arial"],
    "tahoma": ["Tahoma", "DejaVu Sans", "Arial"],
}
GENERIC_FALLBACK = ["Arial", "Noto Naskh Arabic", "Amiri", "DejaVu Sans", "FreeSerif", "FreeSans"]


def default_font_dirs():
    dirs = []
    here = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    dirs.append(os.path.join(here, "fonts"))
    if sys.platform.startswith("win"):
        dirs.append(os.path.join(os.environ.get("WINDIR", r"C:\Windows"), "Fonts"))
        la = os.environ.get("LOCALAPPDATA")
        if la:
            dirs.append(os.path.join(la, "Microsoft", "Windows", "Fonts"))
    else:
        dirs += ["/usr/share/fonts", "/usr/local/share/fonts", os.path.expanduser("~/.fonts"),
                 os.path.expanduser("~/.local/share/fonts")]
    return [d for d in dirs if os.path.isdir(d)]


class FontCatalog:
    def __init__(self, dirs=None):
        self.dirs = dirs or default_font_dirs()
        self.files = {}      # (family_lower, bold) -> path
        self.arabic = {}     # (family_lower, bold) -> has Arabic glyphs
        self.missing = {}    # requested family -> used family
        self._hb = {}
        self._bbox = {}
        if HAVE_HB:
            self._scan()

    # ------------------------------------------------------------ scanning
    def _scan(self):
        for d in self.dirs:
            for root, _, names in os.walk(d):
                for n in names:
                    if not n.lower().endswith((".ttf", ".otf")):
                        continue
                    p = os.path.join(root, n)
                    try:
                        self._register(p)
                    except Exception:
                        pass

    def _register(self, path, override=False):
        tt = TTFont(path, lazy=True, fontNumber=0)
        name = tt["name"]
        fam = None
        sub = ""
        for rec in name.names:
            if rec.nameID == 16 and fam is None:
                fam = rec.toUnicode()
            if rec.nameID == 17 and not sub:
                sub = rec.toUnicode()
        if fam is None:
            for rec in name.names:
                if rec.nameID == 1:
                    fam = rec.toUnicode()
                    break
            for rec in name.names:
                if rec.nameID == 2:
                    sub = rec.toUnicode()
                    break
        if not fam:
            return
        bold = False
        try:
            bold = bool(tt["OS/2"].usWeightClass >= 600) or ("bold" in sub.lower())
        except Exception:
            bold = "bold" in sub.lower()
        italic = "italic" in sub.lower() or "oblique" in sub.lower()
        if italic or "condensed" in sub.lower() or "narrow" in sub.lower() or "light" in sub.lower():
            return
        key = (fam.lower(), bold)
        try:
            cmap = tt.getBestCmap() or {}
            arabic = 0x0628 in cmap or 0x0644 in cmap
        except Exception:
            arabic = False
        # keep the first registered (tool's own fonts dir has priority by scan order)
        if key not in self.files or override:
            self.files[key] = path
            self.arabic[key] = arabic
            self._hb.pop(self.files.get(key), None)
            self._width_cached.cache_clear()
        tt.close()

    # ---------------------------------------------------------- resolution
    def resolve(self, family: str, bold: bool, need_arabic: bool = True):
        fam = (family or "Arial").strip()
        for b in (bold, not bold):
            key = (fam.lower(), b)
            if key in self.files and (not need_arabic or self.arabic.get(key)):
                return self.files[key], fam, b
        for alt in FALLBACKS.get(fam.lower(), []) + GENERIC_FALLBACK:
            for b in (bold, not bold):
                k = (alt.lower(), b)
                if k in self.files and (not need_arabic or self.arabic.get(k)):
                    self.missing[fam] = alt
                    return self.files[k], alt, b
        # last resort: any font with Arabic glyphs
        for k, path in self.files.items():
            if self.arabic.get(k) and k[1] == bold:
                self.missing[fam] = k[0]
                return path, k[0], bold
        return None, fam, bold

    def _font(self, path):
        f = self._hb.get(path)
        if f is None:
            blob = hb.Blob.from_file_path(path)
            face = hb.Face(blob)
            font = hb.Font(face)
            f = (face, font)
            self._hb[path] = f
        return f

    # ---------------------------------------------------------- measuring
    def width_twips(self, text: str, family: str, size_pt: float, bold: bool) -> float:
        """Advance width of `text` shaped as one run. Falls back to a crude estimate without HarfBuzz."""
        if not text:
            return 0.0
        if not HAVE_HB:
            return self._estimate(text, size_pt)
        need_arabic = any("\u0600" <= ch <= "\u06FF" for ch in text)
        path, _, _ = self.resolve(family, bold, need_arabic)
        if path is None:
            return self._estimate(text, size_pt)
        return self._width_cached(path, text, float(size_pt))

    @lru_cache(maxsize=65536)
    def _width_cached(self, path, text, size_pt):
        face, font = self._font(path)
        buf = hb.Buffer()
        buf.add_str(text)
        buf.guess_segment_properties()
        # Word does not apply pair kerning by default; keep required Arabic shaping only
        hb.shape(font, buf, {"kern": False})
        adv = sum(p.x_advance for p in buf.glyph_positions)
        return adv / face.upem * size_pt * 20.0

    def glyph_bbox_em(self, ch: str, family: str, bold: bool):
        """(width, height) of a glyph's ink box in em units, or None."""
        if not HAVE_HB:
            return None
        path, _, _ = self.resolve(family, bold, "\u0600" <= ch <= "\u06FF")
        if path is None:
            return None
        key = (path, ch)
        if key in self._bbox:
            return self._bbox[key]
        try:
            from fontTools.pens.boundsPen import BoundsPen
            tt = TTFont(path)
            gs = tt.getGlyphSet()
            g = tt.getBestCmap().get(ord(ch))
            res = None
            if g:
                bp = BoundsPen(gs)
                gs[g].draw(bp)
                if bp.bounds:
                    x0, y0, x1, y1 = bp.bounds
                    u = tt["head"].unitsPerEm
                    res = ((x1 - x0) / u, (y1 - y0) / u)
            tt.close()
        except Exception:
            res = None
        self._bbox[key] = res
        return res

    def glyph_metrics(self, ch: str, family: str, bold: bool):
        """(advance, left side bearing, ink width) of one glyph in em units; None when unknown."""
        if not HAVE_HB:
            return None
        path, _, _ = self.resolve(family, bold, "\u0600" <= ch <= "\u06FF")
        if path is None:
            return None
        key = ("m", path, ch)
        if key in self._bbox:
            return self._bbox[key]
        res = None
        try:
            from fontTools.pens.boundsPen import BoundsPen
            tt = TTFont(path)
            g = tt.getBestCmap().get(ord(ch))
            if g:
                adv, lsb = tt["hmtx"][g]
                gs = tt.getGlyphSet()
                bp = BoundsPen(gs)
                gs[g].draw(bp)
                u = tt["head"].unitsPerEm
                if bp.bounds:
                    x0, y0, x1, y1 = bp.bounds
                    res = (adv / u, x0 / u, (x1 - x0) / u)
                else:
                    res = (adv / u, 0.0, 0.0)
            tt.close()
        except Exception:
            res = None
        self._bbox[key] = res
        return res

    def edge_bearings(self, text: str, family: str, size_pt: float, bold: bool, rtl: bool = True):
        """(right-edge bearing, left-edge bearing) in twips of `text` shaped as one RTL/LTR run: the
        empty space between the run's cell edges and its visible ink. Shaped with HarfBuzz, so joined
        forms (initial ث, final ى) are measured as drawn. None without HarfBuzz."""
        if not HAVE_HB or not text:
            return None
        need_arabic = any("\u0600" <= ch <= "\u06FF" for ch in text)
        path, _, _ = self.resolve(family, bold, need_arabic)
        if path is None:
            return None
        face, font = self._font(path)
        buf = hb.Buffer()
        buf.add_str(text)
        buf.guess_segment_properties()
        if rtl:   # markers live in RTL paragraphs: "(" is mirrored, neutrals take the paragraph direction
            buf.direction = "rtl"
        hb.shape(font, buf, {"kern": False})
        infos, pos = buf.glyph_infos, buf.glyph_positions
        if not infos:
            return None
        k = size_pt * 20.0 / face.upem
        x = 0.0
        ink_min, ink_max = None, None
        for inf, p in zip(infos, pos):
            ext = font.get_glyph_extents(inf.codepoint)
            if ext is not None and ext.width:
                x0 = x + p.x_offset + ext.x_bearing
                x1 = x0 + ext.width
                ink_min = x0 if ink_min is None else min(ink_min, x0)
                ink_max = x1 if ink_max is None else max(ink_max, x1)
            x += p.x_advance
        if ink_min is None:
            return (0.0, 0.0)
        return (max(x - ink_max, 0.0) * k, max(ink_min, 0.0) * k)

    def glyph_gaps(self, text: str, family: str, size_pt: float, bold: bool) -> int:
        """Number of places where renderers add letter spacing: base glyphs (non-zero advance) minus one."""
        if not HAVE_HB or not text:
            return max(len(text) - 1, 1)
        path, _, _ = self.resolve(family, bold, True)
        if path is None:
            return max(len(text) - 1, 1)
        face, font = self._font(path)
        buf = hb.Buffer()
        buf.add_str(text)
        buf.guess_segment_properties()
        hb.shape(font, buf, {"kern": False})
        bases = sum(1 for p in buf.glyph_positions if p.x_advance > 0)
        return max(bases - 1, 1)

    @staticmethod
    def _estimate(text, size_pt):
        em = 0.0
        for ch in text:
            if ch in "اأإآل|":
                em += 0.28
            elif ch == " ":
                em += 0.28
            elif "\u064B" <= ch <= "\u0652":
                em += 0.0
            else:
                em += 0.58
        return em * size_pt * 20.0
