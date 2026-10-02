#!/usr/bin/env python3
"""Check every local href/src (and #anchor) in a built mdBook or site directory.

mdBook rewrites `x.md` links to `x.html` but not `README.md` to `index.html`, and it does not
check anchors, so broken links only show up for readers. Build first, then point this at the
output directory.

Usage:
  mdbook build docs -d /tmp/papo-book
  python3 scripts/check-book-links.py /tmp/papo-book [--base /papo/docs/]

Absolute links that start with --base (the book's `site-url`, default /papo/docs/) are resolved
against the output directory; other absolute links are reported, since they would leave the book.
"""

import html.parser
import os
import re
import sys
import urllib.parse

SKIP = re.compile(r"^(https?:|mailto:|javascript:|data:|#$)")


class Page(html.parser.HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.links: list[str] = []
        self.ids: set[str] = set()

    def handle_starttag(self, tag, attrs):
        attributes = dict(attrs)
        if attributes.get("id"):
            self.ids.add(attributes["id"])
        for key in ("href", "src"):
            if attributes.get(key):
                self.links.append(attributes[key])


def main() -> int:
    args = sys.argv[1:]
    base = "/papo/docs/"
    if "--base" in args:
        i = args.index("--base")
        base = args[i + 1] if i + 1 < len(args) else base
        del args[i : i + 2]
    if len(args) != 1 or not os.path.isdir(args[0]):
        print(__doc__.strip())
        return 2
    root = os.path.abspath(args[0])
    pages: dict[str, Page] = {}
    for directory, _, files in os.walk(root):
        for name in files:
            if name.endswith(".html"):
                path = os.path.join(directory, name)
                page = Page()
                with open(path, encoding="utf-8") as handle:
                    page.feed(handle.read())
                pages[path] = page

    problems = checked = 0
    for path, page in pages.items():
        for link in page.links:
            if SKIP.match(link):
                continue
            parts = urllib.parse.urlparse(link)
            link_path = urllib.parse.unquote(parts.path)
            if link_path.startswith(base):
                target = os.path.normpath(os.path.join(root, link_path[len(base) :]))
            elif link_path.startswith("/"):
                target = os.path.join(root, "<outside the book>" + link_path)
            elif link_path:
                target = os.path.normpath(os.path.join(os.path.dirname(path), link_path))
            else:
                target = path
            if os.path.isdir(target):
                target = os.path.join(target, "index.html")
            checked += 1
            if not os.path.exists(target):
                problems += 1
                print(f"MISSING {os.path.relpath(path, root)} -> {link}")
            elif parts.fragment and target in pages and urllib.parse.unquote(parts.fragment) not in pages[target].ids:
                problems += 1
                print(f"ANCHOR  {os.path.relpath(path, root)} -> {link}")
    print(f"checked {checked} local links in {len(pages)} pages, {problems} problems")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
