# -*- coding: utf-8 -*-
"""
Build fonts/ServiceWordDigits.ttf: a tiny font whose Latin digits 0-9 are drawn with Arabic-Indic
shapes (٠١٢٣٤٥٦٧٨٩). Word's PAGE/NUMPAGES fields always produce Latin digits unless an unreliable
format switch is honoured; with this font on the field runs the page numbers *look* Arabic-Indic on
every machine, because the font is embedded in the .docx itself.
Source outlines: GNU FreeFont (FreeSerif Bold), GPLv3 with font exception — see fonts/LICENSE-FreeFont.
"""
import os
import sys

from fontTools import subset
from fontTools.ttLib import TTFont

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FAMILY = "ServiceWord Digits"


def find_source():
    for p in (sys.argv[1] if len(sys.argv) > 1 else "",
              "/usr/share/fonts/truetype/freefont/FreeSerifBold.ttf",
              os.path.join(HERE, "fonts", "FreeSerifBold.ttf")):
        if p and os.path.exists(p):
            return p
    raise SystemExit("FreeSerifBold.ttf not found")


def main():
    src = find_source()
    out = os.path.join(HERE, "fonts", "ServiceWordDigits.ttf")
    tt = TTFont(src)
    cmap = tt.getBestCmap()
    keep = [cmap[0x0660 + i] for i in range(10)] + [cmap[0x20], cmap[0x28], cmap[0x29], cmap[0x2D], ".notdef"]
    opts = subset.Options()
    opts.glyph_names = True
    opts.notdef_outline = True
    opts.name_IDs = ["*"]
    opts.name_legacy = True
    opts.layout_features = []
    s = subset.Subsetter(opts)
    s.populate(glyphs=keep)
    s.subset(tt)
    # remap: Latin digits -> Arabic-Indic outlines; keep ( ) - space; Arabic-Indic codepoints too
    new = {0x30 + i: cmap[0x0660 + i] for i in range(10)}
    new.update({0x0660 + i: cmap[0x0660 + i] for i in range(10)})
    new.update({0x20: cmap[0x20], 0x28: cmap[0x28], 0x29: cmap[0x29], 0x2D: cmap[0x2D]})
    for table in tt["cmap"].tables:
        table.cmap = dict(new)
    # names
    name = tt["name"]
    for rec in list(name.names):
        if rec.nameID in (1, 3, 4, 6, 16, 17, 2):
            name.removeNames(nameID=rec.nameID)
    for nid, val in ((1, FAMILY), (2, "Bold"), (3, "ServiceWord;Digits;1.0"), (4, FAMILY + " Bold"),
                     (6, "ServiceWordDigits-Bold"), (16, FAMILY), (17, "Bold")):
        name.setName(val, nid, 3, 1, 0x409)
        name.setName(val, nid, 1, 0, 0)
    tt["OS/2"].fsType = 0            # installable / embeddable
    tt["OS/2"].usWeightClass = 700
    tt["head"].macStyle |= 1
    tt.save(out)
    print("written", out, "glyphs:", len(tt.getGlyphOrder()))


if __name__ == "__main__":
    main()
