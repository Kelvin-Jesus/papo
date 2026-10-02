#!/usr/bin/env python3
"""Quality gate: the landing page stays inside its weight budget.

Budgets (gzip, level 9) come from design/brand-book.md and section 5.7 of
design/pesquisa-ui-ux.md: HTML <= 30 KB, CSS <= 15 KB, JS <= 20 KB. The self-hosted font
is reported but has its own ceiling (160 KB raw), since woff2 is already compressed.

Usage: scripts/check-site-budget.py [site-dir]   (default: site/)
"""

import gzip
import sys
from pathlib import Path

KB = 1024
BUDGETS = {  # suffix -> max gzip bytes for all files of that kind together
    ".html": 30 * KB,
    ".css": 15 * KB,
    ".js": 20 * KB,
}
FONT_MAX = 160 * KB


def gz(data: bytes) -> int:
    return len(gzip.compress(data, compresslevel=9, mtime=0))


def main(argv: list[str]) -> int:
    root = Path(argv[0] if argv else "site")
    if not root.is_dir():
        print(f"budget: {root} não existe", file=sys.stderr)
        return 2
    failed = False
    for suffix, limit in BUDGETS.items():
        files = sorted(p for p in root.rglob(f"*{suffix}") if p.is_file())
        raw = sum(p.stat().st_size for p in files)
        packed = sum(gz(p.read_bytes()) for p in files)
        ok = packed <= limit
        failed |= not ok
        print(
            f"{'ok  ' if ok else 'FAIL'} {suffix:6} {packed / KB:6.1f} KB gzip "
            f"(limite {limit // KB} KB; {raw / KB:.1f} KB sem compressão, {len(files)} arquivo(s))"
        )
    for font in sorted(root.rglob("*.woff2")):
        size = font.stat().st_size
        ok = size <= FONT_MAX
        failed |= not ok
        print(f"{'ok  ' if ok else 'FAIL'} fonte  {size / KB:6.1f} KB ({font.name}; limite {FONT_MAX // KB} KB)")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
