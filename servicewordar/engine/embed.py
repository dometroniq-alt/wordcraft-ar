# -*- coding: utf-8 -*-
"""Embed a TrueType font inside the .docx (obfuscated .odttf part + fontTable entry), so that the
document shows it on any machine without installing anything. Used for the page-number digits font."""
from __future__ import annotations

import os
import re
import uuid

from lxml import etree

W_NS = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
R_NS = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
PKG_REL_NS = "http://schemas.openxmlformats.org/package/2006/relationships"
CT_NS = "http://schemas.openxmlformats.org/package/2006/content-types"
FONT_REL_TYPE = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/font"
ODTTF_CT = "application/vnd.openxmlformats-officedocument.obfuscatedFont"


def w(tag):
    return "{%s}%s" % (W_NS, tag)


def obfuscate(data: bytes, guid: str) -> bytes:
    """ECMA-376 Part 2 §… font obfuscation: XOR the first 32 bytes with the GUID bytes in reverse."""
    hexs = guid.strip("{}").replace("-", "")
    key = bytes.fromhex(hexs)[::-1]
    head = bytes(b ^ key[i % 16] for i, b in enumerate(data[:32]))
    return head + data[32:]


def embed_font(parts: dict, ttf_path: str, family: str, bold: bool = True) -> str:
    """Add `ttf_path` to the package `parts` (dict name -> bytes) under `family`. Idempotent."""
    ft_name = "word/fontTable.xml"
    ft = etree.fromstring(parts[ft_name]) if ft_name in parts else etree.Element(w("fonts"), nsmap={"w": W_NS, "r": R_NS})
    for f in ft.findall(w("font")):
        if f.get(w("name")) == family and (f.find(w("embedRegular")) is not None or f.find(w("embedBold")) is not None):
            return family  # already embedded
    # relationships of the font table
    rels_name = "word/_rels/fontTable.xml.rels"
    rels = etree.fromstring(parts[rels_name]) if rels_name in parts else etree.Element("{%s}Relationships" % PKG_REL_NS, nsmap={None: PKG_REL_NS})
    existing = {r.get("Id") for r in rels}
    n = 1
    while f"rIdFont{n}" in existing:
        n += 1
    rid = f"rIdFont{n}"
    part = f"word/fonts/{os.path.splitext(os.path.basename(ttf_path))[0]}.odttf"
    k = 1
    while part in parts:
        part = f"word/fonts/font{k}.odttf"
        k += 1
    guid = "{%s}" % str(uuid.uuid4()).upper()
    with open(ttf_path, "rb") as fh:
        parts[part] = obfuscate(fh.read(), guid)
    rel = etree.SubElement(rels, "{%s}Relationship" % PKG_REL_NS)
    rel.set("Id", rid)
    rel.set("Type", FONT_REL_TYPE)
    rel.set("Target", part[len("word/"):])
    parts[rels_name] = etree.tostring(rels, xml_declaration=True, encoding="UTF-8", standalone=True)
    # font entry
    font = None
    for f in ft.findall(w("font")):
        if f.get(w("name")) == family:
            font = f
    if font is None:
        font = etree.SubElement(ft, w("font"))
        font.set(w("name"), family)
        etree.SubElement(font, w("charset")).set(w("val"), "00")
        etree.SubElement(font, w("family")).set(w("val"), "roman")
        etree.SubElement(font, w("pitch")).set(w("val"), "variable")
    emb = etree.SubElement(font, w("embedBold") if bold else w("embedRegular"))
    emb.set("{%s}id" % R_NS, rid)
    emb.set(w("fontKey"), guid)
    if "r" not in (ft.nsmap or {}):
        # lxml cannot add a namespace declaration to an existing root; rebuild the root with it
        new = etree.Element(ft.tag, nsmap={**(ft.nsmap or {}), "r": R_NS})
        for c in ft:
            new.append(c)
        ft = new
    parts[ft_name] = etree.tostring(ft, xml_declaration=True, encoding="UTF-8", standalone=True)
    # content type for .odttf
    ct_name = "[Content_Types].xml"
    ct = etree.fromstring(parts[ct_name])
    if not any(d.get("Extension") == "odttf" for d in ct.findall("{%s}Default" % CT_NS)):
        d = etree.SubElement(ct, "{%s}Default" % CT_NS)
        d.set("Extension", "odttf")
        d.set("ContentType", ODTTF_CT)
        # Defaults must precede Overrides
        ct.remove(d)
        ct.insert(0, d)
    parts[ct_name] = etree.tostring(ct, xml_declaration=True, encoding="UTF-8", standalone=True)
    # settings: tell Word the file carries embedded fonts (subset, system fonts excluded on re-save)
    st_name = "word/settings.xml"
    if st_name in parts:
        st = etree.fromstring(parts[st_name])
        if st.find(w("embedTrueTypeFonts")) is None:
            # these three are the first children of w:settings in schema order after writeProtection/view/zoom…
            # Word tolerates them at the top of the element
            idx = 0
            for i, c in enumerate(list(st)):
                if etree.QName(c).localname in ("writeProtection", "view", "zoom", "removePersonalInformation",
                                                 "removeDateAndTime", "doNotDisplayPageBoundaries",
                                                 "displayBackgroundShape", "printPostScriptOverText",
                                                 "printFractionalCharacterWidth", "printFormsData", "embedTrueTypeFonts"):
                    idx = i + 1
            e1 = etree.Element(w("embedTrueTypeFonts"))
            e2 = etree.Element(w("embedSystemFonts"))
            e2.set(w("val"), "0")
            e3 = etree.Element(w("saveSubsetFonts"))
            for j, e in enumerate((e1, e2, e3)):
                st.insert(idx + j, e)
            parts[st_name] = etree.tostring(st, xml_declaration=True, encoding="UTF-8", standalone=True)
    return family
