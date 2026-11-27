#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Builds the public site (landing page, download page, wiki, demo, guided tour) into a folder.

    python3 site/build.py --out site/dist

Needs the Markdown package pinned in site/requirements.txt, nothing else. The wiki is docs/wiki/*.md in the order of
docs/wiki/_order.txt; the demo is docs/design/demo/index.html, copied unchanged.
"""
import argparse
import html
import json
import posixpath
import re
import shutil
import sys
from pathlib import Path

import markdown

import bench
import bench_raw
import download
from shell import CUR, REPO, fill, ic, sidebar

ROOT = Path(__file__).resolve().parent.parent
SITE = ROOT / "site"
WIKI = ROOT / "docs" / "wiki"
DEMO = ROOT / "docs" / "design" / "demo" / "index.html"
BEAUTIFUL = ROOT / "docs" / "images" / "beautiful"
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


def build_compare(out: Path) -> None:
    """The stock-against-Beautiful page: one wipe slider per view in docs/images/beautiful/views.json."""
    data = json.loads((BEAUTIFUL / "views.json").read_text())
    dest = out / "compare"
    (dest / "img").mkdir(parents=True)
    jump, views = [], []
    for group in data["groups"]:
        jump.append(f'<a href="#{group["id"]}">{html.escape(group["title"])}</a>')
        views.append(f'<h2 class="group" id="{group["id"]}">{html.escape(group["title"])}</h2>')
        for v in group["views"]:
            name, title = v["name"], html.escape(v["title"])
            for side in ("stock", "beautiful"):
                shutil.copy(BEAUTIFUL / "web" / f"{name}-{side}.jpg", dest / "img" / f"{name}-{side}.jpg")
            alt = html.escape(v["caption"])
            views.append(
                f'<section class="card view" id="{name}"><h3>{title}</h3>'
                f'<figure class="cmp"><img class="beautiful" src="img/{name}-beautiful.jpg" alt="{title}, Beautiful" loading="lazy" width="1600" height="900">'
                f'<img class="stock" src="img/{name}-stock.jpg" alt="{title}, the stock game" loading="lazy" width="1600" height="900">'
                f'<span class="tagl">Stock</span><span class="tagr">Beautiful</span><span class="bar"></span><span class="knob" aria-hidden="true">&harr;</span>'
                f'<input type="range" min="0" max="100" value="50" aria-label="{title}: slide between stock (left) and Beautiful (right)"></figure>'
                f'<p class="cap">{alt}</p></section>')
    page = fill((SITE / "compare.html").read_text(), "compare", "../", icon=ic("eye", 24), jump="".join(jump), views="\n".join(views))
    (dest / "index.html").write_text(page)
    shutil.copy(SITE / "compare.css", dest / "compare.css")
    shutil.copy(SITE / "compare.js", dest / "compare.js")


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
    shutil.copy(SITE / "site.css", out / "assets" / "site.css")
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
    build_compare(out)
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
    print(f"built {sum(1 for p in out.rglob('*') if p.is_file())} files into {out}")


if __name__ == "__main__":
    main()
