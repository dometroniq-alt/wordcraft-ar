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
a = ap.parse_args()
rules = {}
if os.path.exists(a.rules):
    rules = json.load(open(a.rules, encoding="utf-8"))
base, ext = os.path.splitext(a.src)
out = a.out or base + "_منسق" + ext
rep = format_docx(a.src, out, merge_rules(DEFAULT_RULES, rules), report_path=os.path.splitext(out)[0] + "_تقرير.json")
print("الناتج:", out)
for k, v in rep["summary"]["change_counts"].items():
    print(f"  {k}: {v}")
if a.pdf:
    from engine.stamp import final_pdf
    r = final_pdf(out, os.path.splitext(out)[0] + "_طباعة.pdf", merge_rules(DEFAULT_RULES, rules))
    print("PDF:", r["output"], f"({r['stamped']} صفحة مرقّمة، {r['first']}–{r['last']}، {r['renderer']})")
