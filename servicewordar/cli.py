# -*- coding: utf-8 -*-
r"""Command line:  python cli.py "C:\path\file.docx" [-o out.docx] [--rules rules.json]"""
import argparse, json, os, sys

# Windows consoles default to a legacy code page; the engine prints Arabic paths and messages.
for _stream in (sys.stdout, sys.stderr):
    if hasattr(_stream, "reconfigure"):
        try:
            _stream.reconfigure(encoding="utf-8", errors="replace")
        except (ValueError, OSError):
            pass
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from engine.formatter import format_docx
from engine.rules import DEFAULT_RULES, merge_rules

ap = argparse.ArgumentParser(description="منسّق كتابات الخدمة - سطر الأوامر")
ap.add_argument("src"); ap.add_argument("-o", "--out"); ap.add_argument("--rules", default=os.path.join(HERE, "rules.json"))
ap.add_argument("--pdf", action="store_true", help="also produce the stamped print copy (PDF)")
ap.add_argument("--no-auto-end", action="store_true", help="do not render once to learn the real page count for the footer")
a = ap.parse_args()
rules = {}
if os.path.exists(a.rules):
    rules = json.load(open(a.rules, encoding="utf-8"))
base, ext = os.path.splitext(a.src)
out = a.out or base + "_منسق" + ext
merged = merge_rules(DEFAULT_RULES, rules)
rep = format_docx(a.src, out, merged, report_path=os.path.splitext(out)[0] + "_تقرير.json")
num = (merged.get("page") or {}).get("numbering") or {}
if not a.no_auto_end and num.get("enabled", True) and not num.get("end"):
    # the end number of "( n - N )" is TYPED (Arabic-Indic), so it must be the real page count:
    # render once with LibreOffice/Word, count the pages, format again with N set.
    try:
        import subprocess, tempfile, re
        from engine.stamp import render_pdf, find_soffice
        if find_soffice():
            tmp = tempfile.mkdtemp(prefix="swar_end_")
            pdf = render_pdf(out, tmp)
            txt = subprocess.run(["pdfinfo", pdf], capture_output=True, text=True).stdout
            m = re.search(r"Pages:\s+(\d+)", txt)
            if m:
                total = int(m.group(1))
                start = int(num.get("start", 1) or 1)
                has_cover = any(p.get("role") == "cover" for p in rep["paragraphs"])
                end = start - 1 + total - (1 if has_cover else 0)
                merged["page"]["numbering"]["end"] = end
                rep = format_docx(a.src, out, merged, report_path=os.path.splitext(out)[0] + "_تقرير.json")
                print("آخر رقم صفحة في التذييل:", end)
    except Exception as exc:
        print("تعذّر حساب عدد الصفحات تلقائياً:", exc)
print("الناتج:", out)
for k, v in rep["summary"]["change_counts"].items():
    print(f"  {k}: {v}")
if a.pdf:
    from engine.stamp import final_pdf
    r = final_pdf(out, os.path.splitext(out)[0] + "_طباعة.pdf", merge_rules(DEFAULT_RULES, rules))
    print("PDF:", r["output"], f"({r['stamped']} صفحة مرقّمة، {r['first']}–{r['last']}، {r['renderer']})")
