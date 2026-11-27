#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Builds the public site (landing page, wiki, demo, guided tour) into a folder.

    python3 site/build.py --out site/dist

Needs the Markdown package pinned in site/requirements.txt, nothing else. The wiki is docs/wiki/*.md in the order of
docs/wiki/_order.txt; the demo is docs/design/demo/index.html, copied unchanged.
"""
import argparse
import html
import posixpath
import re
import shutil
import sys
from pathlib import Path

import markdown

ROOT = Path(__file__).resolve().parent.parent
SITE = ROOT / "site"
WIKI = ROOT / "docs" / "wiki"
DEMO = ROOT / "docs" / "design" / "demo" / "index.html"
SHOTS = ROOT / "docs" / "design" / "screenshots"
REPO = "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox"
PAGES = "https://jorisjonkers-dev.github.io/rauls-trusty-toolbox/"

LOGO = (
    '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" '
    'stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="3.2"/>'
    '<path d="M12 2.5v3M12 18.5v3M2.5 12h3M18.5 12h3M5.3 5.3l2.1 2.1M16.6 16.6l2.1 2.1M18.7 5.3l-2.1 2.1M7.4 16.6l-2.1 2.1"/></svg>'
)

CUR = ' aria-current="page"'
SECTIONS = [("demo", "Demo", "demo/"), ("tour", "Guided tour", "tour/"), ("wiki", "Wiki", "wiki/"), ("bench", "Benchmarks", "bench/")]


def header(current: str, up: str) -> str:
    """The top bar. `up` is the relative path from the page to the site root ('' or '../')."""
    links = "".join(
        f'<a href="{up}{href}"{CUR if key == current else ""}>{label}</a>' for key, label, href in SECTIONS
    )
    links += f'<a href="{REPO}">GitHub</a>'
    return f'<header class="top"><a class="brand" href="{up}index.html">{LOGO}Raul\'s Toolbox</a><nav aria-label="Site">{links}</nav></header>'


def fill(template: str, current: str, up: str, **extra: str) -> str:
    out = template.replace("<!--HEADER-->", header(current, up)).replace("{{up}}", up)
    for key, value in extra.items():
        out = out.replace("{{" + key + "}}", value)
    return out


def slugs() -> list[str]:
    order = [line.strip() for line in (WIKI / "_order.txt").read_text().splitlines() if line.strip()]
    found = sorted(p.stem for p in WIKI.glob("*.md"))
    if sorted(order) != found:
        sys.exit(f"docs/wiki/_order.txt and the pages differ: order={sorted(order)} files={found}")
    return order


def rewrite_links(body: str, names: set[str], images: set[str]) -> str:
    def fix(match: re.Match) -> str:
        attr, url = match.group(1), match.group(2)
        if url.startswith(PAGES):  # a site page, written absolute so it also works on GitHub
            return f'{attr}="../{url[len(PAGES):]}"'
        if re.match(r"^[a-z][a-z0-9+.-]*:|^#|^/", url):
            return match.group(0)
        path, _, frag = url.partition("#")
        suffix = f"#{frag}" if frag else ""
        if "/" not in path and path.endswith(".md") and path[:-3] in names:
            return f'{attr}="{path[:-3]}.html{suffix}"'
        resolved = posixpath.normpath(posixpath.join("docs/wiki", path))
        if attr == "src" and resolved.startswith("docs/design/screenshots/"):
            images.add(posixpath.basename(resolved))
            return f'src="img/{posixpath.basename(resolved)}"'
        if resolved.startswith(".."):
            sys.exit(f"link leaves the repository: {url}")
        return f'{attr}="{REPO}/blob/main/{resolved}{suffix}"'

    return re.sub(r'(href|src)="([^"]*)"', fix, body)


def build_wiki(out: Path) -> None:
    order = slugs()
    template = (SITE / "wiki.html").read_text()
    pages = []
    md = markdown.Markdown(extensions=["tables", "fenced_code", "toc", "sane_lists"], output_format="html")
    images: set[str] = set()
    for slug in order:
        source = (WIKI / f"{slug}.md").read_text()
        md.reset()
        body = md.convert(source)
        title = re.search(r"^# (.+)$", source, re.M)
        if not title:
            sys.exit(f"{slug}.md has no # title")
        pages.append((slug, title.group(1).strip(), rewrite_links(body, set(order), images)))
    dest = out / "wiki"
    dest.mkdir(parents=True)
    for i, (slug, title, body) in enumerate(pages):
        side = "".join(
            f'<a href="{s}.html"{CUR if s == slug else ""}>{html.escape("Home" if s == "index" else t)}</a>' for s, t, _ in pages
        )
        prev = f'<a class="btn sm" href="{pages[i-1][0]}.html">&larr; {html.escape(pages[i-1][1])}</a>' if i else "<span></span>"
        nxt = f'<a class="btn sm" href="{pages[i+1][0]}.html">{html.escape(pages[i+1][1])} &rarr;</a>' if i + 1 < len(pages) else "<span></span>"
        edit = f"{REPO}/blob/main/docs/wiki/{slug}.md"
        page = fill(template, "wiki", "../", title=html.escape(title if slug != "index" else "Wiki"), side=side, body=body,
                    pager=f'<div class="pager">{prev}{nxt}</div>', edit=edit)
        (dest / ("index.html" if slug == "index" else f"{slug}.html")).write_text(page)
    if images:
        (dest / "img").mkdir()
        for name in sorted(images):
            shutil.copy(SHOTS / name, dest / "img" / name)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True, type=Path)
    out = parser.parse_args().out.resolve()
    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)
    (out / ".nojekyll").write_text("")
    (out / "assets").mkdir()
    shutil.copy(SITE / "site.css", out / "assets" / "site.css")
    (out / "index.html").write_text(fill((SITE / "landing.html").read_text(), "", "", repo=REPO))
    (out / "demo").mkdir()
    shutil.copy(DEMO, out / "demo" / "index.html")  # unchanged
    (out / "tour").mkdir()
    (out / "tour" / "index.html").write_text(fill((SITE / "tour.html").read_text(), "tour", "../"))
    shutil.copy(SITE / "tour.js", out / "tour" / "tour.js")
    build_wiki(out)
    print(f"built {sum(1 for p in out.rglob('*') if p.is_file())} files into {out}")


if __name__ == "__main__":
    main()
