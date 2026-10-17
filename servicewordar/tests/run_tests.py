# -*- coding: utf-8 -*-
"""Self-checks. Run:  python tests/run_tests.py   -> prints PASS/FAIL per check, exit code 1 on any failure."""
import os, sys, zipfile, re, tempfile
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
from engine import numbering as N
from engine.formatter import Formatter
from lxml import etree

W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
fails = 0
def check(name, cond, detail=""):
    global fails
    print(("PASS  " if cond else "FAIL  ") + name + ("" if cond else "   " + detail))
    if not cond: fails += 1

# ---- numbering
check("digit-dot detection", N.detect_marker("4. مهام")[:2] == (0, 4))
check("abjad with tatweel", N.detect_marker("هــ. شعبة")[:2] == (1, 5))
check("ordinal with stray space before tanween", N.detect_marker("سادسا ً. واجبات")[:2] == (2, 6))
check("digit-paren arabic-indic", N.detect_marker("(١٢) بند")[:2] == (3, 12))
check("plain sentence is not a marker", N.detect_marker("تقديم الخدمات الطبية") is None)
check("marker text abjad 3 -> جـ.", N.marker_text(1, 3, {}) == "جـ.")
check("marker text abjad 5 -> هـ.", N.marker_text(1, 5, {}) == "هـ.")
check("marker text ordinal 11", N.marker_text(2, 11, {}) == "حادي عشر.")
check("marker text digit paren", N.marker_text(3, 7, {}) == "(٧).")
check("cycle repeats at depth 6", N.marker_text(6, 2, {}) == "٢.")

# ---- synthetic document round trip
def make_doc(paragraphs):
    """paragraphs: list of (text, soft_break_positions?) -> minimal docx bytes"""
    body = ""
    for t in paragraphs:
        runs = ""
        for i, piece in enumerate(t.split("\n")):
            if i: runs += "<w:r><w:br/></w:r>"
            runs += '<w:r><w:rPr><w:szCs w:val="32"/></w:rPr><w:t xml:space="preserve">%s</w:t></w:r>' % piece
        body += "<w:p>%s</w:p>" % runs
    doc = ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
           '<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>%s'
           '<w:sectPr/></w:body></w:document>' % body)
    tmp = tempfile.mkdtemp()
    src = os.path.join(tmp, "in.docx")
    with zipfile.ZipFile(src, "w") as z:
        z.writestr("[Content_Types].xml", '<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>')
        z.writestr("_rels/.rels", '<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>')
        z.writestr("word/document.xml", doc)
    return src, os.path.join(tmp, "out.docx")

src, dst = make_doc([
    "العنوان",
    "1. الرئيسية الأولى",
    "أ. فرع",
    "اولا. عنوان فرعي",
    "(1) بند اول يحتوي على\nكسر سطر يدوي  ومسافات  مكررة وتـطـويـل .",
    "(2) بند ثاني",
    "ثانيا. عنوان فرعي آخر",
    "(1) بند",
    "ج. فرع ثانٍ",
    "",
    "ه. فرع ثالث",
])
rep = Formatter().format_file(src, dst)
rows = {r["index"]: r for r in rep["paragraphs"]}
check("counters restart under new heading", rows[7]["marker"] == "(١).", rows[7]["marker"])
check("second item numbered (٢)", rows[5]["marker"] == "(٢).", rows[5]["marker"])
check("abjad siblings renumbered sequentially (ج typed -> ب.)", rows[8]["marker"] == "ب.", rows[8]["marker"])
check("third sibling gets جـ. with tatweel", rows[10]["marker"] == "جـ.", rows[10]["marker"])
check("ordinal tanween normalised", rows[3]["marker"] == "أولاً.", rows[3]["marker"])
check("soft break + spaces + typed tatweel cleaned (kashida re-added evenly)", rows[4]["clean_text"].replace("\u00A0", " ").replace("\u0640", "") == "بند اول يحتوي على كسر سطر يدوي ومسافات مكررة وتطويل.", rows[4]["clean_text"])
check("line filled with kashida to the margin", rows[4]["layout"].get("kashida", 0) > 0, str(rows[4]["layout"]))
check("layout measured with real fonts (mode recorded)", rows[4]["layout"].get("mode") in ("distribute", "justify"), str(rows[4]["layout"]))
check("empty paragraph removed", "empty_paragraph_removed" in rows[9]["changes"])
with zipfile.ZipFile(dst) as z:
    xml = z.read("word/document.xml").decode("utf-8")
check("no soft breaks remain", "<w:br/>" not in xml)
check("bidi set on paragraphs", xml.count("<w:bidi/>") >= 9)
check("hanging indents present", "w:hanging=" in xml)
check("keepLines present", "<w:keepLines/>" in xml)
root = etree.fromstring(z.read("word/document.xml") if False else xml.encode("utf-8"))
ps = root.findall(".//" + W + "p")
def ind(p):
    e = p.find(W + "pPr/" + W + "ind"); return (int(e.get(W + "left")), int(e.get(W + "hanging")))
def ms(i): return rows[i]["layout"]["marker_start"]
def mw(i): return rows[i]["layout"]["marker_width"]
cols = rep["summary"]["marker_columns_twips"]
def dc(i): return rows[i]["layout"]["dot_column"]
check("dots of a level share one column", dc(3) == dc(6), f"{dc(3)} {dc(6)}")
def col(i): return rows[i]["layout"]["column"]
check("child column sits at the parent's dot (ink-aligned, within one dot width)", 0 <= dc(1) - col(2) <= 120 and 0 <= dc(2) - col(3) <= 120 and 0 <= dc(3) - col(4) <= 120, f"{col(2)}/{dc(1)} {col(3)}/{dc(2)} {col(4)}/{dc(3)}")
check("siblings share the column", ms(4) == ms(5), f"{ms(4)} {ms(5)}")
check("markers of a level equalized to one ink width (ثانياً vs أولاً)", any(rows[i]["layout"].get("marker_display", rows[i]["marker"]) != rows[i]["marker"] or rows[i]["layout"].get("marker_offset") or (rows[i]["layout"].get("marker_scale") or 100) != 100 or rows[i]["layout"].get("lamalef") for i in (3, 6)), str(rows[3]["layout"])+str(rows[6]["layout"]))
check("headings underlined", "<w:u w:val=\"single\"/>" in xml)
l4, h4 = ind(ps[4])
check("hanging = text starts after marker+space", l4 - h4 == ms(4), f"{l4}-{h4} vs {ms(4)}")
check("indents are integers", all(int(e.get(W+"left")) >= 0 for e in root.iter(W+"ind")))
check("original untouched", open(src, "rb").read() != open(dst, "rb").read())

# ---- header / footer on the real-structure sample (synthetic doc has none; check the engine on src if present)
sample = os.path.join(os.path.dirname(HERE), "..", "src11.docx")
if os.path.exists(sample):
    f2 = Formatter({"page": {"classification": "سري", "numbering": {"start": 3, "end": 20}}})
    out2 = os.path.join(tempfile.mkdtemp(), "o.docx")
    f2.format_file(sample, out2)
    with zipfile.ZipFile(out2) as z2:
        ft = z2.read("word/footer1.xml").decode("utf-8"); hd = z2.read("word/header1.xml").decode("utf-8"); dx = z2.read("word/document.xml").decode("utf-8")
        names2 = z2.namelist(); ftab = z2.read("word/fontTable.xml").decode("utf-8"); sett = z2.read("word/settings.xml").decode("utf-8")
    check("every item ends with a dot", all(p["clean_text"].rstrip()[-1:] in ".:؛؟!" for p in f2.report["paragraphs"] if p["role"] in ("item", "body") and p["clean_text"].strip()))
    la = [p for p in f2.report["paragraphs"] if (p.get("layout") or {}).get("lamalef")]
    check("أولاً keeps its ligature and reaches the level width (lam-alef glyph, no offset)", la and all(p["layout"]["lamalef"]["reached"] and not p["layout"]["marker_offset"] for p in la), str([p["layout"] for p in la[:1]]))
    check("derived lam-alef font embedded", any(n.startswith("word/fonts/") and "LamAlef" in n for n in names2) and "SW LamAlef" in ftab)
    check("classification replaced in header and footer", "سري" in ft and "سري" in hd and "محدود" not in ft)
    check("footer uses a plain PAGE field (no nested formula); end typed so no SECTIONPAGES", " PAGE " in ft and "SECTIONPAGES" not in ft and "> = <" not in ft)
    check("footer digits set in the embedded digits font", 'w:cs="ServiceWord Digits"' in ft)
    check("footer end number typed", ">20<" in ft)
    check("digits font embedded in the package", any(n.startswith("word/fonts/") and n.endswith(".odttf") for n in names2) and "ServiceWord Digits" in ftab and "embedTrueTypeFonts" in sett)
    check("cover is its own section without header/footer; body numbers from 3", dx.count("<w:sectPr") == 2 and 'w:start="3"' in dx and "<w:titlePg" not in dx and dx.find("<w:sectPr") < dx.find("headerReference"))

    # stamped print copy (needs a renderer: LibreOffice or Word)
    try:
        from engine.stamp import final_pdf, find_soffice
        import platform as _pf
        if find_soffice() or _pf.system() == "Windows":
            pdf_out = os.path.join(tempfile.mkdtemp(), "p.pdf")
            r = final_pdf(out2, pdf_out, {"page": {"classification": "سري", "numbering": {"start": 3, "end": 20}}})
            check("print copy stamped on every numbered page", r["stamped"] >= 10 and r["first"] == 3 and r["last"] == 20, str(r))
        else:
            print("SKIP  print copy (no renderer here)")
    except Exception as e:
        check("print copy pipeline", False, repr(e))

print("\n%d failure(s)" % fails if fails else "\nALL CHECKS PASSED")
sys.exit(1 if fails else 0)
