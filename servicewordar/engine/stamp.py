# -*- coding: utf-8 -*-
"""
Final print copy: render the formatted DOCX to PDF with the real pagination (LibreOffice if present,
otherwise Microsoft Word through COM on Windows), then STAMP the page numbers on every page as static
text — "( page - last )" in Arabic-Indic digits — exactly one line above the classification word in the
footer, never over body text. The DOCX keeps Word fields for editing; the PDF is what gets printed.
"""
from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
import zipfile

from lxml import etree

from . import numbering as N

W_NS = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"


def w(tag):
    return "{%s}%s" % (W_NS, tag)


# ----------------------------------------------------------------- rendering
def find_soffice():
    for cand in ("soffice", "libreoffice"):
        p = shutil.which(cand)
        if p:
            return p
    if sys.platform.startswith("win"):
        for p in (r"C:\Program Files\LibreOffice\program\soffice.exe",
                  r"C:\Program Files (x86)\LibreOffice\program\soffice.exe"):
            if os.path.exists(p):
                return p
    return None


def render_pdf(docx_path: str, out_dir: str) -> str:
    """Return the path of the rendered PDF. Raises RuntimeError with a plain reason when no renderer."""
    os.makedirs(out_dir, exist_ok=True)
    soffice = find_soffice()
    base = os.path.splitext(os.path.basename(docx_path))[0]
    out_pdf = os.path.join(out_dir, base + ".pdf")
    if soffice:
        subprocess.run([soffice, "--headless", "--convert-to", "pdf", "--outdir", out_dir, docx_path],
                       check=True, capture_output=True, timeout=300)
        if os.path.exists(out_pdf):
            return out_pdf
    if sys.platform.startswith("win"):
        ps = (
            "$w = New-Object -ComObject Word.Application; $w.Visible = $false; "
            "$d = $w.Documents.Open('%s', $false, $true); "
            "$d.ExportAsFixedFormat('%s', 17); $d.Close(0); $w.Quit()"
            % (docx_path.replace("'", "''"), out_pdf.replace("'", "''"))
        )
        subprocess.run(["powershell", "-NoProfile", "-Command", ps], check=True, capture_output=True, timeout=300)
        if os.path.exists(out_pdf):
            return out_pdf
    raise RuntimeError("لا يوجد برنامج لتصيير الصفحات: ثبّت LibreOffice أو شغّل الأداة على جهاز فيه Microsoft Word.")


# ------------------------------------------------------- footer placeholder
def blank_footer_number(docx_path: str, out_path: str) -> dict:
    """Copy of the DOCX whose footer page-number paragraph is emptied (a non-breaking space keeps the
    line height) so the rendered pages carry no number — the stamp adds it. Returns footer facts."""
    info = {"classification": None, "footer_font": None, "footer_size": 12.0}
    with zipfile.ZipFile(docx_path) as zin:
        names = zin.namelist()
        data = {n: zin.read(n) for n in names}
    for n in names:
        if not (n.startswith("word/footer") and n.endswith(".xml")):
            continue
        root = etree.fromstring(data[n])
        for p in root.iter(w("p")):
            instr = "".join(i.text or "" for i in p.iter(w("instrText")))
            if "PAGE" in instr.upper():
                first = p.find(w("r"))
                rpr = first.find(w("rPr")) if first is not None else None
                if rpr is not None:
                    rf = rpr.find(w("rFonts"))
                    if rf is not None:
                        info["footer_font"] = rf.get(w("cs")) or rf.get(w("ascii"))
                    sz = rpr.find(w("szCs"))
                    if sz is None:
                        sz = rpr.find(w("sz"))
                    if sz is not None:
                        info["footer_size"] = int(sz.get(w("val"))) / 2
                for c in list(p):
                    if etree.QName(c).localname != "pPr":
                        p.remove(c)
                r = etree.SubElement(p, w("r"))
                if rpr is not None:
                    import copy
                    r.append(copy.deepcopy(rpr))
                t = etree.SubElement(r, w("t"))
                t.text = "\u00A0"
                t.set("{http://www.w3.org/XML/1998/namespace}space", "preserve")
            else:
                txt = "".join(t.text or "" for t in p.iter(w("t"))).strip()
                if txt in ("عام", "محدود", "سري", "سري وشخصي", "سري للغاية"):
                    info["classification"] = txt
        data[n] = etree.tostring(root, xml_declaration=True, encoding="UTF-8", standalone=True)
    with zipfile.ZipFile(out_path, "w", zipfile.ZIP_DEFLATED) as zout:
        for n in names:
            zout.writestr(n, data[n])
    return info


# ----------------------------------------------------------------- stamping
def _lowest_text_y(page):
    """Return (y_bottom, y_top, x_center) of the lowest text chunk on the page (the classification word)."""
    found = []

    def visitor(text, cm, tm, font_dict, font_size):
        if text and text.strip():
            x, y = tm[4], tm[5]
            found.append((y, x, font_size or 0, text))
    try:
        page.extract_text(visitor_text=visitor)
    except Exception:
        return None
    if not found:
        return None
    y, x, fs, text = min(found, key=lambda f: f[0])
    return y, fs, text


def stamp_pdf(pdf_in: str, pdf_out: str, start: int, end, font_path: str, size_pt: float,
              classification: str, skip_pages_without_footer=True, digits="arabic_indic") -> dict:
    from pypdf import PdfReader, PdfWriter
    from reportlab.pdfbase import pdfmetrics
    from reportlab.pdfbase.ttfonts import TTFont
    from reportlab.pdfgen import canvas
    import io

    pdfmetrics.registerFont(TTFont("StampFont", font_path))
    reader = PdfReader(pdf_in)
    writer = PdfWriter()
    total_pages = len(reader.pages)
    conv = N.to_arabic_indic if digits in ("arabic_indic", "unicode") else (lambda s: s)
    # pages that carry a footer (the cover does not)
    numbered = []
    for i, page in enumerate(reader.pages):
        low = _lowest_text_y(page)
        has_footer = False
        if low is not None:
            y, fs, text = low
            h = float(page.mediabox.height)
            has_footer = y < h * 0.12  # text in the bottom 12% of the page = footer
        numbered.append(has_footer or not skip_pages_without_footer)
    count = sum(1 for n in numbered if n)
    last = int(end) if end else (start + count - 1)
    stamped = 0
    n = start
    for i, page in enumerate(reader.pages):
        if numbered[i]:
            wdt = float(page.mediabox.width)
            hgt = float(page.mediabox.height)
            low = _lowest_text_y(page)
            if low is not None:
                y_class, fs, _ = low
                y = y_class + max(fs, size_pt) * 1.25   # one line above the classification word
            else:
                y = 40.0
            # visual order for a right-to-left reader: page number on the right, last page on the left
            label = f"( {conv(str(last))} - {conv(str(n))} )"
            buf = io.BytesIO()
            c = canvas.Canvas(buf, pagesize=(wdt, hgt), initialFontName="StampFont")
            c.setFont("StampFont", size_pt)
            c.drawCentredString(wdt / 2.0, y, label)
            c.save()
            buf.seek(0)
            overlay = PdfReader(buf).pages[0]
            page.merge_page(overlay)
            stamped += 1
            n += 1
        writer.add_page(page)
    with open(pdf_out, "wb") as fh:
        writer.write(fh)
    return {"pages": total_pages, "stamped": stamped, "first": start, "last": last}


def set_cached_pages(docx_path: str, last: int) -> bool:
    """Write the real page count into the cached result of the SECTIONPAGES/NUMPAGES field in the
    footer, so viewers that do not evaluate the field (LibreOffice) show the right number too."""
    import copy
    changed = False
    with zipfile.ZipFile(docx_path) as zin:
        names = zin.namelist()
        data = {n: zin.read(n) for n in names}
    for n in names:
        if not (n.startswith("word/footer") and n.endswith(".xml")):
            continue
        root = etree.fromstring(data[n])
        for p in root.iter(w("p")):
            runs = list(p.iter(w("r")))
            state = None
            for r in runs:
                it = r.find(w("instrText"))
                fc = r.find(w("fldChar"))
                if it is not None and ("SECTIONPAGES" in (it.text or "").upper() or "NUMPAGES" in (it.text or "").upper()):
                    state = "count"
                elif fc is not None and fc.get(w("fldCharType")) == "separate" and state == "count":
                    state = "result"
                elif state == "result":
                    t = r.find(w("t"))
                    if t is not None:
                        digits = any("\u0660" <= ch <= "\u0669" for ch in (t.text or ""))
                        t.text = N.to_arabic_indic(str(last)) if digits else str(last)
                        changed = True
                    state = None
                elif fc is not None and fc.get(w("fldCharType")) == "end":
                    state = None
        data[n] = etree.tostring(root, xml_declaration=True, encoding="UTF-8", standalone=True)
    if changed:
        tmp = docx_path + ".tmp"
        with zipfile.ZipFile(tmp, "w", zipfile.ZIP_DEFLATED) as zout:
            for n in names:
                zout.writestr(n, data[n])
        os.replace(tmp, docx_path)
    return changed


def final_pdf(docx_path: str, out_pdf: str, rules: dict, font_dirs=None) -> dict:
    """Whole pipeline: blank footer number -> render -> stamp. Returns a small report."""
    num = (rules.get("page") or {}).get("numbering") or {}
    cls = (rules.get("page") or {}).get("classification") or "محدود"
    tmpdir = tempfile.mkdtemp(prefix="swar_")
    blank = os.path.join(tmpdir, os.path.basename(docx_path))
    info = blank_footer_number(docx_path, blank)
    rendered = render_pdf(blank, tmpdir)
    # font for the digits: the footer's own font when its file is known, else Times New Roman from fonts/
    font_path = None
    here = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    for cand in ("timesbd.ttf", "times.ttf", "arialbd.ttf", "arial.ttf"):
        p = os.path.join(here, "fonts", cand)
        if os.path.exists(p):
            font_path = p
            break
    if font_path is None and sys.platform.startswith("win"):
        for cand in (r"C:\Windows\Fonts\timesbd.ttf", r"C:\Windows\Fonts\arialbd.ttf"):
            if os.path.exists(cand):
                font_path = cand
                break
    if font_path is None:   # Linux/macOS: any installed bold serif/sans that has the Arabic-Indic digits
        try:
            for fam in ("Times New Roman:bold", "Arial:bold", "Liberation Serif:bold", "DejaVu Sans:bold"):
                got = subprocess.run(["fc-match", "-f", "%{file}", fam], capture_output=True, text=True).stdout.strip()
                if got and os.path.exists(got):
                    font_path = got
                    break
        except Exception:
            pass
    if font_path is None:
        raise RuntimeError("لم أجد ملف خط لكتابة أرقام الصفحات (ضع timesbd.ttf في مجلد fonts).")
    rep = stamp_pdf(rendered, out_pdf, int(num.get("start", 1) or 1), num.get("end"), font_path,
                    float(info.get("footer_size") or 12.0), cls, digits=num.get("digits", "unicode"))
    rep["renderer"] = "LibreOffice" if find_soffice() else "Microsoft Word"
    rep["output"] = out_pdf
    # the formatted .docx gets the real count as the cached field value (Word recomputes it anyway)
    if not num.get("end"):
        try:
            rep["docx_page_count_written"] = set_cached_pages(docx_path, rep["last"])
        except Exception:
            rep["docx_page_count_written"] = False
    return rep
