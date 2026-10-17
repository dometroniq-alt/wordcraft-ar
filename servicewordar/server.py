# -*- coding: utf-8 -*-
"""
ServiceWord-AR local server. Standard library only (plus lxml for the engine).
Runs fully offline:  python server.py  ->  http://127.0.0.1:8765
"""
import io
import json
import mimetypes
import os
import platform
import re
import sys
import threading
import time
import traceback
import urllib.parse
import webbrowser
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

from engine.formatter import Formatter  # noqa: E402
from engine.stamp import final_pdf  # noqa: E402
from engine.rules import DEFAULT_RULES, merge_rules  # noqa: E402

RULES_PATH = os.path.join(HERE, "rules.json")
OUTPUT_DIR = os.path.join(HERE, "output")
UI_DIR = os.path.join(HERE, "ui")
HOST, PORT = "127.0.0.1", 8765
OUT_SUFFIX = "_منسق"


def load_rules() -> dict:
    if os.path.exists(RULES_PATH):
        with open(RULES_PATH, "r", encoding="utf-8") as fh:
            return merge_rules(DEFAULT_RULES, json.load(fh))
    return merge_rules(DEFAULT_RULES, {})


def save_rules(rules: dict):
    with open(RULES_PATH, "w", encoding="utf-8") as fh:
        json.dump(rules, fh, ensure_ascii=False, indent=2)


def list_fonts():
    names = set()
    try:
        if platform.system() == "Windows":
            import winreg
            for hive, key in ((winreg.HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts"),
                              (winreg.HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts")):
                try:
                    with winreg.OpenKey(hive, key) as k:
                        i = 0
                        while True:
                            try:
                                name, _, _ = winreg.EnumValue(k, i)
                            except OSError:
                                break
                            i += 1
                            name = re.sub(r"\s*\((TrueType|OpenType|All res)\)\s*$", "", name)
                            name = re.sub(r"\s+(Bold|Italic|Bold Italic|Regular|Light|Black|Medium|SemiBold|ExtraBold)$", "", name)
                            names.add(name.strip())
                except OSError:
                    pass
        else:
            import subprocess
            out = subprocess.run(["fc-list", ":", "family"], capture_output=True, text=True).stdout
            for line in out.splitlines():
                for fam in line.split(","):
                    names.add(fam.strip())
    except Exception:
        pass
    common = ["Arial", "Times New Roman", "Tahoma", "Traditional Arabic", "Simplified Arabic", "Sakkal Majalla",
              "PT Bold Heading", "Calibri", "Segoe UI", "Noto Naskh Arabic", "Amiri"]
    for c in common:
        names.add(c)
    return sorted(names, key=lambda s: s.lower())


def parse_multipart(content_type: str, body: bytes) -> dict:
    """Minimal multipart/form-data parser (the cgi module was removed in Python 3.13).
    Returns {field: {"filename": str|None, "data": bytes}}."""
    m = re.search(r'boundary="?([^";]+)"?', content_type)
    if not m:
        return {}
    boundary = m.group(1).encode()
    parts = body.split(b"--" + boundary)
    out = {}
    for part in parts[1:]:
        if part.strip() in (b"", b"--"):
            continue
        if part.startswith(b"\r\n"):
            part = part[2:]
        head, _, data = part.partition(b"\r\n\r\n")
        if data.endswith(b"\r\n"):
            data = data[:-2]
        disp = ""
        for line in head.decode("utf-8", "replace").split("\r\n"):
            if line.lower().startswith("content-disposition"):
                disp = line
        name = re.search(r'name="([^"]*)"', disp)
        fname = re.search(r'filename="([^"]*)"', disp)
        if not name:
            continue
        out[name.group(1)] = {"filename": fname.group(1) if fname else None, "data": data}
    return out


def output_path_for(src_path: str, out_dir: str = None) -> str:
    base, ext = os.path.splitext(os.path.basename(src_path))
    folder = out_dir or os.path.dirname(src_path) or OUTPUT_DIR
    os.makedirs(folder, exist_ok=True)
    cand = os.path.join(folder, f"{base}{OUT_SUFFIX}{ext}")
    n = 2
    while os.path.exists(cand):
        cand = os.path.join(folder, f"{base}{OUT_SUFFIX}{n}{ext}")
        n += 1
    return cand


def run_format(src_path: str, rules: dict, out_dir: str = None) -> dict:
    dst = output_path_for(src_path, out_dir)
    f = Formatter(rules)
    rep = f.format_file(src_path, dst)
    rep_path = os.path.splitext(dst)[0] + "_تقرير.json"
    with open(rep_path, "w", encoding="utf-8") as fh:
        json.dump(rep, fh, ensure_ascii=False, indent=2)
    rep["summary"]["report"] = rep_path
    rep["summary"]["output_name"] = os.path.basename(dst)
    return rep


class Handler(BaseHTTPRequestHandler):
    server_version = "ServiceWordAR/1.0"

    def log_message(self, fmt, *args):  # quieter console
        sys.stderr.write("%s - %s\n" % (time.strftime("%H:%M:%S"), fmt % args))

    # ------------------------------------------------------------ helpers
    def _json(self, obj, status=200):
        data = json.dumps(obj, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(data)

    def _file(self, path, download_name=None):
        if not os.path.isfile(path):
            self._json({"error": "not found"}, 404)
            return
        ctype = mimetypes.guess_type(path)[0] or "application/octet-stream"
        with open(path, "rb") as fh:
            data = fh.read()
        self.send_response(200)
        self.send_header("Content-Type", ctype + ("; charset=utf-8" if ctype.startswith("text/") else ""))
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Cache-Control", "no-store")
        if download_name:
            q = urllib.parse.quote(download_name)
            self.send_header("Content-Disposition", f"attachment; filename*=UTF-8''{q}")
        self.end_headers()
        self.wfile.write(data)

    def _read_json(self):
        n = int(self.headers.get("Content-Length", "0") or 0)
        raw = self.rfile.read(n) if n else b"{}"
        return json.loads(raw.decode("utf-8") or "{}")

    # -------------------------------------------------------------- GET
    def do_GET(self):
        u = urllib.parse.urlparse(self.path)
        q = urllib.parse.parse_qs(u.query)
        if u.path == "/":
            return self._file(os.path.join(UI_DIR, "index.html"))
        if u.path.startswith("/ui/"):
            p = os.path.normpath(os.path.join(UI_DIR, u.path[4:]))
            if p.startswith(UI_DIR):
                return self._file(p)
        if u.path == "/api/rules":
            return self._json({"rules": load_rules(), "defaults": DEFAULT_RULES, "path": RULES_PATH})
        if u.path == "/api/fonts":
            return self._json({"fonts": list_fonts()})
        if u.path == "/api/info":
            return self._json({"output_dir": OUTPUT_DIR, "rules_path": RULES_PATH, "platform": platform.platform(),
                               "python": sys.version.split()[0]})
        if u.path == "/api/download":
            p = q.get("path", [""])[0]
            if not p or not os.path.isfile(p):
                return self._json({"error": "not found"}, 404)
            return self._file(p, os.path.basename(p))
        self._json({"error": "unknown route"}, 404)

    # -------------------------------------------------------------- POST
    def do_POST(self):
        u = urllib.parse.urlparse(self.path)
        try:
            if u.path == "/api/rules":
                body = self._read_json()
                rules = merge_rules(DEFAULT_RULES, body.get("rules", {}))
                save_rules(rules)
                return self._json({"ok": True, "rules": rules})
            if u.path == "/api/rules/reset":
                if os.path.exists(RULES_PATH):
                    os.remove(RULES_PATH)
                return self._json({"ok": True, "rules": load_rules()})
            if u.path == "/api/final-pdf":
                body = self._read_json()
                src = body.get("path", "").strip().strip('"')
                if not src or not os.path.isfile(src):
                    return self._json({"error": "الملف غير موجود: " + src}, 400)
                rules = merge_rules(load_rules(), body.get("rules") or {})
                out_pdf = os.path.splitext(src)[0] + "_طباعة.pdf"
                rep = final_pdf(src, out_pdf, rules)
                return self._json({"ok": True, "pdf": rep})
            if u.path == "/api/format-path":
                body = self._read_json()
                src = body.get("path", "").strip().strip('"')
                if not src or not os.path.isfile(src):
                    return self._json({"error": "الملف غير موجود: " + src}, 400)
                if not src.lower().endswith(".docx"):
                    return self._json({"error": "الأداة تعمل على ملفات .docx فقط (احفظ ملف .doc بصيغة .docx أولًا)"}, 400)
                rules = merge_rules(load_rules(), body.get("rules") or {})
                rep = run_format(src, rules)
                return self._json({"ok": True, "report": rep})
            if u.path == "/api/format-upload":
                ctype = self.headers.get("Content-Type", "")
                if not ctype.startswith("multipart/form-data"):
                    return self._json({"error": "expected multipart"}, 400)
                n = int(self.headers.get("Content-Length", "0") or 0)
                form = parse_multipart(ctype, self.rfile.read(n))
                item = form.get("file")
                if item is None or not item["filename"]:
                    return self._json({"error": "لم يصل ملف"}, 400)
                rules_raw = form.get("rules", {}).get("data", b"{}").decode("utf-8") or "{}"
                rules = merge_rules(load_rules(), json.loads(rules_raw))
                os.makedirs(os.path.join(OUTPUT_DIR, "uploads"), exist_ok=True)
                safe = os.path.basename(item["filename"].replace("\\", "/"))
                src = os.path.join(OUTPUT_DIR, "uploads", safe)
                with open(src, "wb") as fh:
                    fh.write(item["data"])
                rep = run_format(src, rules, OUTPUT_DIR)
                return self._json({"ok": True, "report": rep})
            self._json({"error": "unknown route"}, 404)
        except Exception as e:  # report real errors, never pretend success
            traceback.print_exc()
            self._json({"error": f"{type(e).__name__}: {e}", "trace": traceback.format_exc()}, 500)


def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    port = PORT
    for _ in range(20):
        try:
            httpd = ThreadingHTTPServer((HOST, port), Handler)
            break
        except OSError:
            port += 1
    else:
        print("لا يوجد منفذ متاح")
        sys.exit(1)
    url = f"http://{HOST}:{port}/"
    print("ServiceWord-AR يعمل على:", url)
    print("مجلد المخرجات:", OUTPUT_DIR)
    if "--no-browser" not in sys.argv:
        threading.Timer(0.6, lambda: webbrowser.open(url)).start()
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
