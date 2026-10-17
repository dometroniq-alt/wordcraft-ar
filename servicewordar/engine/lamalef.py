# -*- coding: utf-8 -*-
"""
A wide lam-alef for the marker «أولاً»: the only ordinal marker without a kashida joint. Instead of
breaking the ligature (أولـــاً) or fattening the whole word, a small derived font is built from the
document's own font in which the lam-alef ligature is lengthened the calligraphic way — the lam's
sweep and the foot are stretched while the alef stem keeps its thickness — by `delta` em. The font
holds only ل ا ً and the ligature (with and without the fathatan baked in) and is embedded in the
.docx, so the marker still reads "أولاً" as plain text and falls back to the normal ligature if the
font is ever lost.
"""
from __future__ import annotations

import os

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.recordingPen import DecomposingRecordingPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTFont

try:
    import uharfbuzz as hb
except Exception:  # pragma: no cover
    hb = None

LAM, ALEF, FATHATAN = "ل", "ا", "ً"


def _shape(path, text):
    blob = hb.Blob.from_file_path(path)
    face = hb.Face(blob)
    font = hb.Font(face)
    buf = hb.Buffer()
    buf.add_str(text)
    buf.direction = "rtl"
    buf.script = "Arab"
    buf.guess_segment_properties()
    hb.shape(font, buf, {"kern": False})
    return face.upem, list(zip(buf.glyph_infos, buf.glyph_positions))


def _flatten(rec, steps=8):
    """Polyline approximation of recorded (decomposed) outline, for span detection."""
    polys, cur, start, last = [], [], None, None
    for op, args in rec:
        if op == "moveTo":
            if cur:
                polys.append(cur)
            cur = [args[0]]
            start = last = args[0]
        elif op == "lineTo":
            cur.append(args[0])
            last = args[0]
        elif op == "qCurveTo":
            pts = list(args)
            if pts[-1] is None:
                pts = pts[:-1]
            # implied on-curve points between consecutive off-curve points
            offs = pts[:-1]
            end = pts[-1]
            p0 = last
            for i, c in enumerate(offs):
                if i + 1 < len(offs):
                    n = offs[i + 1]
                    p1 = ((c[0] + n[0]) / 2.0, (c[1] + n[1]) / 2.0)
                else:
                    p1 = end
                for k in range(1, steps + 1):
                    t = k / steps
                    x = (1 - t) ** 2 * p0[0] + 2 * (1 - t) * t * c[0] + t * t * p1[0]
                    y = (1 - t) ** 2 * p0[1] + 2 * (1 - t) * t * c[1] + t * t * p1[1]
                    cur.append((x, y))
                p0 = p1
            last = end
        elif op == "curveTo":
            p0 = last
            c1, c2, end = args
            for k in range(1, steps + 1):
                t = k / steps
                x = (1 - t) ** 3 * p0[0] + 3 * (1 - t) ** 2 * t * c1[0] + 3 * (1 - t) * t * t * c2[0] + t ** 3 * end[0]
                y = (1 - t) ** 3 * p0[1] + 3 * (1 - t) ** 2 * t * c1[1] + 3 * (1 - t) * t * t * c2[1] + t ** 3 * end[1]
                cur.append((x, y))
            last = end
        elif op in ("closePath", "endPath"):
            if cur:
                polys.append(cur)
            cur = []
    if cur:
        polys.append(cur)
    return polys


def _spans_at(polys, y):
    xs = []
    for poly in polys:
        n = len(poly)
        for i in range(n):
            (x0, y0), (x1, y1) = poly[i], poly[(i + 1) % n]
            if (y0 <= y < y1) or (y1 <= y < y0):
                xs.append(x0 + (y - y0) * (x1 - x0) / (y1 - y0))
    xs.sort()
    return [(xs[i], xs[i + 1]) for i in range(0, len(xs) - 1, 2)]


def alef_stem_x(rec):
    """x of the left edge of the alef stem: the right-most filled span at several heights."""
    polys = _flatten(rec)
    ys = [p[1] for poly in polys for p in poly]
    y_min, y_max = min(ys), max(ys)
    cands = []
    for f in (0.55, 0.65, 0.75, 0.85):
        spans = _spans_at(polys, y_min + (y_max - y_min) * f)
        if spans:
            cands.append(spans[-1][0])
    return min(cands) if cands else None


def _transform_rec(rec, xa, delta, y_lo=None, y_hi=None, mode="shear"):
    """mode "shear": the alef and the foot move right by delta, the lam's head stays, so the lam's
    sweep lengthens (the calligraphic wide lam-alef); mode "scale": the whole lam part is stretched."""
    def tx(p):
        x, y = p
        if x >= xa:
            return (x + delta, y)
        if mode == "scale" or y_lo is None:
            return (x * (xa + delta) / xa if xa > 0 else x, y)
        if y <= y_lo:
            f = 1.0
        elif y >= y_hi:
            f = 0.0
        else:
            f = (y_hi - y) / (y_hi - y_lo)
        return (x + delta * f, y)
    out = []
    for op, args in rec:
        if op in ("moveTo", "lineTo"):
            out.append((op, (tx(args[0]),)))
        elif op == "qCurveTo":
            out.append((op, tuple(tx(a) if a is not None else None for a in args)))
        elif op == "curveTo":
            out.append((op, tuple(tx(a) for a in args)))
        else:
            out.append((op, args))
    return out


def _replay(rec, pen):
    for op, args in rec:
        getattr(pen, op)(*args)


def build(base_path: str, delta_em: float, out_path: str, family: str, mode: str = "scale",
          foot_share: float = 0.32, head_share: float = 0.92, over_alef_from_em: float = 0.3) -> dict:
    """Write the derived font. Returns metrics: advance of the wide ligature in font units, upem."""
    tt = TTFont(base_path)
    upem = tt["head"].unitsPerEm
    gs = tt.getGlyphSet()
    cmap = tt.getBestCmap()
    order = tt.getGlyphOrder()
    _, shaped = _shape(base_path, LAM + ALEF + FATHATAN)
    # HarfBuzz output is visual (LTR) order for RTL text: [mark?, ligature] ... find by advance
    lig = next(((i, p) for i, p in shaped if p.x_advance > 0), None)
    mark = next(((i, p) for i, p in shaped if p.x_advance == 0), None)
    if lig is None:
        raise RuntimeError("no lam-alef ligature in " + base_path)
    lig_name = order[lig[0].codepoint]
    lig_adv = lig[1].x_advance
    rec = DecomposingRecordingPen(gs)
    gs[lig_name].draw(rec)
    xa = alef_stem_x(rec.value)
    if xa is None or xa <= 0:
        raise RuntimeError("cannot locate the alef stem")
    delta = int(round(delta_em * upem))
    polys = _flatten(rec.value)
    ys = [pt[1] for poly in polys for pt in poly]
    y_min, y_max = min(ys), max(ys)
    y_lo = y_min + (y_max - y_min) * foot_share    # below: moves with the alef (the foot)
    y_hi = y_min + (y_max - y_min) * head_share    # above: stays (the lam's head)
    wide = _transform_rec(rec.value, xa, delta, y_lo, y_hi, mode)
    # where the fathatan sits over the ligature in the base font (GPOS), moved with the alef if it was
    # over the alef; written as a mark anchor so the mark keeps its place in the new font too
    anchor = None
    if mark is not None:
        pen_x = 0
        for inf, pos in shaped:
            if inf is mark[0]:
                break
            pen_x += pos.x_advance
        mx = pen_x + mark[1].x_offset
        my = mark[1].y_offset
        if delta_em >= over_alef_from_em:
            # a lengthened lam-alef carries the fathatan over the alef stem (as اً does), not over the
            # far end of the sweep
            wpolys = _flatten(wide)
            spans = _spans_at(wpolys, y_min + (y_max - y_min) * 0.7)
            if spans:
                stem_c = (spans[-1][0] + spans[-1][1]) / 2.0
                mname = order[mark[0].codepoint]
                mrec = DecomposingRecordingPen(gs)
                gs[mname].draw(mrec)
                mpolys = _flatten(mrec.value)
                mxs = [pt[0] for poly in mpolys for pt in poly]
                m_c = (min(mxs) + max(mxs)) / 2.0 if mxs else 0
                mx = stem_c - m_c
        elif mx >= xa:
            mx += delta
        elif mode == "scale":
            mx = mx * (xa + delta) / xa
        anchor = (int(round(mx)), int(round(my)))

    def glyph_from(rec_value):
        pen = TTGlyphPen(None)
        _replay(rec_value, pen)
        return pen.glyph()

    def copy_glyph(ch):
        name = cmap.get(ord(ch))
        if name is None:
            return None, 0
        r = DecomposingRecordingPen(gs)
        gs[name].draw(r)
        return glyph_from(r.value), tt["hmtx"][name][0]

    names = [".notdef", "space", "lam", "alef", "fathatan", "lamalef"]
    fb = FontBuilder(upem, isTTF=True)
    fb.setupGlyphOrder(names)
    fb.setupCharacterMap({0x20: "space", 0x644: "lam", 0x627: "alef", 0x64B: "fathatan"})
    glyphs, metrics = {}, {}
    glyphs[".notdef"] = TTGlyphPen(None).glyph()
    metrics[".notdef"] = (int(upem * 0.5), 0)
    glyphs["space"] = TTGlyphPen(None).glyph()
    metrics["space"] = (tt["hmtx"][cmap[0x20]][0] if 0x20 in cmap else int(upem * 0.25), 0)
    for ch, nm in ((LAM, "lam"), (ALEF, "alef"), (FATHATAN, "fathatan")):
        g, adv = copy_glyph(ch)
        glyphs[nm] = g if g is not None else TTGlyphPen(None).glyph()
        metrics[nm] = (adv, 0)
    glyphs["lamalef"] = glyph_from(wide)
    metrics["lamalef"] = (lig_adv + delta, 0)
    fb.setupGlyf(glyphs)
    for nm in names:
        g = glyphs[nm]
        try:
            g.recalcBounds(fb.font["glyf"])
            lsb = g.xMin
        except Exception:
            lsb = 0
        metrics[nm] = (metrics[nm][0], lsb)
    fb.setupHorizontalMetrics(metrics)
    asc = tt["hhea"].ascent
    desc = tt["hhea"].descent
    fb.setupHorizontalHeader(ascent=asc, descent=desc)
    os2 = tt["OS/2"]
    fb.setupOS2(sTypoAscender=os2.sTypoAscender, sTypoDescender=os2.sTypoDescender, sTypoLineGap=os2.sTypoLineGap,
                usWinAscent=os2.usWinAscent, usWinDescent=os2.usWinDescent, fsType=0,
                usWeightClass=os2.usWeightClass, ulUnicodeRange1=(1 << 13), achVendID="SWAR")
    fb.setupNameTable({"familyName": family, "styleName": "Regular", "fullName": family,
                       "psName": family.replace(" ", "") + "-Regular", "uniqueFontIdentifier": family + ";1.0",
                       "version": "Version 1.0"})
    fb.setupPost()
    fea = """
languagesystem DFLT dflt;
languagesystem arab dflt;
feature rlig {
    sub lam alef by lamalef;
} rlig;
feature liga {
    sub lam alef by lamalef;
} liga;
"""
    if anchor is not None:
        fea += """
markClass fathatan <anchor 0 0> @TOPMARK;
feature mark {
    pos base lamalef <anchor %d %d> mark @TOPMARK;
} mark;
""" % anchor
    fb.addOpenTypeFeatures(fea)
    fb.save(out_path)
    return {"upem": upem, "lig_advance": lig_adv, "delta": delta, "advance": lig_adv + delta, "xa": xa}
