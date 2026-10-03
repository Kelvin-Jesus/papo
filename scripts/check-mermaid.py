#!/usr/bin/env python3
"""Render every ```mermaid block of the given Markdown files in headless Chromium.

mdBook and GitHub only show a broken diagram in the browser, so a syntax error goes
unnoticed until someone opens the page. This renders each block with the same Mermaid
version the book loads (docs/theme/mermaid-init.js) and fails if any block does not render.

Usage: python3 scripts/check-mermaid.py README.md docs/arquitetura.md docs/engenharia/diagramas.md
Needs Chromium (or Chrome) and internet access (Mermaid comes from jsdelivr).
"""

import functools
import html
import http.server
import os
import json
import re
import shutil
import subprocess
import sys
import tempfile
import threading
from pathlib import Path

MERMAID = "https://cdn.jsdelivr.net/npm/mermaid@11.4.1/dist/mermaid.esm.min.mjs"
BROWSERS = ["chromium", "chromium-browser", "google-chrome", "google-chrome-stable", "chrome"]

HARNESS = """<!doctype html><meta charset="utf-8"><body><textarea id="out">pending</textarea>
<script type="module">
import mermaid from "%s";
mermaid.initialize({startOnLoad: false});
const blocks = %s;
const results = [];
for (const [name, src] of blocks) {
  try { await mermaid.render("d" + results.length, src); results.push([name, "OK"]); }
  catch (e) { results.push([name, String(e.message || e).slice(0, 400)]); }
}
document.getElementById("out").textContent = JSON.stringify(results);
</script>"""


class QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *args) -> None:
        pass


def blocks_of(path: Path) -> list[tuple[str, str]]:
    text = path.read_text(encoding="utf-8")
    found = re.finditer(r"```mermaid\n(.*?)```", text, re.S)
    return [(f"{path}#{i + 1}", m.group(1)) for i, m in enumerate(found)]


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__.strip())
        return 2
    browser = next((b for b in BROWSERS if shutil.which(b)), None)
    if browser is None:
        print("Chromium/Chrome not found in PATH", file=sys.stderr)
        return 2
    blocks = [b for arg in sys.argv[1:] for b in blocks_of(Path(arg))]
    if not blocks:
        print("no mermaid blocks found")
        return 0

    with tempfile.TemporaryDirectory() as tmp:
        Path(tmp, "harness.html").write_text(HARNESS % (MERMAID, json.dumps(blocks)), encoding="utf-8")
        handler = functools.partial(QuietHandler, directory=tmp)
        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        url = f"http://127.0.0.1:{server.server_address[1]}/harness.html"
        args = [browser, "--headless=new", "--disable-gpu", "--virtual-time-budget=30000", "--dump-dom", url]
        # Ubuntu 24.04 runners restrict unprivileged user namespaces through AppArmor, so Chrome's
        # sandbox cannot start there. The page is our own local harness, so CI runs it unsandboxed.
        if os.environ.get("CI"):
            args.insert(1, "--no-sandbox")
        try:
            run = subprocess.run(args, capture_output=True, text=True, timeout=120)
        finally:
            server.shutdown()
        dom = run.stdout

    match = re.search(r'<textarea id="out">(.*?)</textarea>', dom, re.S)
    raw = html.unescape(match.group(1)) if match else "pending"
    if raw == "pending":
        print("the harness did not finish (no internet to load Mermaid, or the browser failed)", file=sys.stderr)
        print(f"browser exit code {run.returncode}; last stderr lines:", file=sys.stderr)
        for line in run.stderr.strip().splitlines()[-15:]:
            print(f"  {line}", file=sys.stderr)
        return 1
    failures = 0
    for name, result in json.loads(raw):
        print(f"{name}: {result}")
        failures += result != "OK"
    print(f"{len(blocks) - failures}/{len(blocks)} diagrams render")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
