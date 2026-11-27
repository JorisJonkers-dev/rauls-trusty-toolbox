#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Builds the public site (landing page, download page, wiki, demo, guided tour) into a folder.

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

import bench
import bench_raw
import download
from shell import CUR, REPO, fill, ic, sidebar, version_links

ROOT = Path(__file__).resolve().parent.parent
SITE = ROOT / "site"
WIKI = ROOT / "docs" / "wiki"
DEMO = ROOT / "docs" / "design" / "demo" / "index.html"
SHOTS = ROOT / "docs" / "design" / "screenshots"
PAGES = "https://jorisjonkers-dev.github.io/rauls-trusty-toolbox/"

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


WIKI_ICON = {"index": "home", "getting-started": "dl", "using-the-toolbox": "mods", "features": "bolt",
             "roadmap": "lod", "components": "box", "faq": "help", "contributing": "code"}
WIKI_PLAY = ["index", "getting-started", "using-the-toolbox", "features"]


def wiki_groups(pages: list, current: str) -> list:
    """The wiki's pages as the app's sidebar: Play (using it) and System (about it)."""
    def item(p):
        slug, title, _ = p
        return ("Home" if slug == "index" else title, f"{slug}.html", WIKI_ICON.get(slug, "info"), slug == current)

    return [("PLAY", [item(p) for p in pages if p[0] in WIKI_PLAY]), ("SYSTEM", [item(p) for p in pages if p[0] not in WIKI_PLAY])]


def cards(body: str, icon: str, title: str) -> str:
    """The page header, then every section (from one h2 to the next) as a rounded card with the app's section header."""
    body = re.sub(r"\s*<h1[^>]*>.*?</h1>", "", body, count=1, flags=re.S)
    parts = re.split(r"(?=<h2[ >])", body)
    out = [f'<div class="phdr"><span class="ico">{ic(icon, 24)}</span><h1>{html.escape(title)}</h1></div>']
    if re.sub(r"<!--.*?-->", "", parts[0], flags=re.S).strip():
        out.append(f'<section class="card"><div class="prose">{parts[0]}</div></section>')
    for part in parts[1:]:
        m = re.match(r"<h2([^>]*)>(.*?)</h2>(.*)", part, re.S)
        out.append(f'<section class="card"><div class="sech"><h2 class="t"{m.group(1)}>{m.group(2)}</h2></div><div class="prose">{m.group(3)}</div></section>')
    return "\n".join(out)


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
        side = sidebar(wiki_groups(pages, slug), "Wiki pages")
        prev = f'<a class="btn sm" href="{pages[i-1][0]}.html">{ic("chev", 14, "flip")}{html.escape(pages[i-1][1])}</a>' if i else "<span></span>"
        nxt = f'<a class="btn sm" href="{pages[i+1][0]}.html">{html.escape(pages[i+1][1])}{ic("chev", 14)}</a>' if i + 1 < len(pages) else "<span></span>"
        edit = f"{REPO}/blob/main/docs/wiki/{slug}.md"
        shown = "Wiki" if slug == "index" else title
        page = fill(template.replace("<!--SIDE-->", side), "wiki", "../", title=html.escape(shown), body=cards(body, WIKI_ICON.get(slug, "info"), shown),
                    pager=f'<div class="pager">{prev}{nxt}</div>', edit=edit)
        (dest / ("index.html" if slug == "index" else f"{slug}.html")).write_text(page)
    if images:
        (dest / "img").mkdir()
        for name in sorted(images):
            shutil.copy(SHOTS / name, dest / "img" / name)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--gh-pages", type=Path, help="a checkout of the gh-pages branch: its bench/results/ records and the "
                        "github-action-benchmark history (bench/raw/data.js, or bench/data.js before the move) join the repository's own")
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)
    (out / ".nojekyll").write_text("")
    (out / "assets").mkdir()
    for name in ("site.css", "bench.css", "bench.js"):
        shutil.copy(SITE / name, out / "assets" / name)
    (out / "index.html").write_text(fill((SITE / "landing.html").read_text(), "home", "", repo=REPO))
    (out / "demo").mkdir()
    (out / "demo" / "app").mkdir()
    shutil.copy(DEMO, out / "demo" / "app" / "index.html")  # unchanged; /demo/ shows it under the site's navbar
    (out / "demo" / "index.html").write_text(fill((SITE / "demo.html").read_text(), "demo", "../"))
    # GitHub Pages serves this for any unknown path, so its links are absolute
    (out / "404.html").write_text(fill((SITE / "404.html").read_text(), "", "/rauls-trusty-toolbox/"))
    (out / "tour").mkdir()
    (out / "tour" / "index.html").write_text(fill((SITE / "tour.html").read_text(), "tour", "../"))
    shutil.copy(SITE / "tour.js", out / "tour" / "tour.js")
    build_wiki(out)
    download.build(out, fill)
    records = bench.load_records([ROOT / "docs" / "bench"] + ([args.gh_pages / "bench" / "results"] if args.gh_pages else []))
    if args.gh_pages:
        raw = next((p for p in (args.gh_pages / "bench" / "raw" / "data.js", args.gh_pages / "bench" / "data.js") if p.is_file()), None)
        history = bench.import_action_history(raw) if raw else None
        if history:
            records.append(history)
    bench.build(out, (SITE / "bench.html").read_text(), fill, records)
    if args.gh_pages and (args.gh_pages / "bench" / "raw").is_dir():
        # the action's own chart page, copied with the navbar so that check.js sees it; pages.yml never publishes it from here
        shutil.copytree(args.gh_pages / "bench" / "raw", out / "bench" / "raw")
        bench_raw.apply(out / "bench")
    # last, once every page and asset is in place; the demo itself (demo/app/) is copied unchanged and links nothing
    version_links(out)
    print(f"built {sum(1 for p in out.rglob('*') if p.is_file())} files into {out}")


if __name__ == "__main__":
    main()
