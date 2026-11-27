# SPDX-License-Identifier: GPL-3.0-only
"""The shared page shell of the public site: the app's top bar and sidebar (docs/design is the standard).

Icons are read from the demo's own sources (docs/design/demo/src), so the site and the app draw the same ones.
"""
import html
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REPO = "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox"
CUR = ' aria-current="page"'

_ICONS: dict[str, str] = {}


def icons() -> dict[str, str]:
    if not _ICONS:
        for name in ("icons.js", "core.js"):
            text = (ROOT / "docs" / "design" / "demo" / "src" / name).read_text()
            for m in re.finditer(r"(?:^\s+|\bI\.)(\w+)\s*[:=]\s*'([^']*)'", text, re.M):
                _ICONS.setdefault(m.group(1), m.group(2))
    return _ICONS


def ic(name: str, size: int = 18, cls: str = "") -> str:
    body = icons()[name]
    return (
        f'<svg class="i {cls}" width="{size}" height="{size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" '
        f'stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">{body}</svg>'
    )


# (key, label, href, icon); the key marks the current section of the top bar.
SECTIONS = [
    ("home", "Home", "index.html", "home"),
    ("download", "Download", "download/", "dl"),
    ("demo", "Demo", "demo/", "play"),
    ("tour", "Guided tour", "tour/", "flask"),
    ("wiki", "Wiki", "wiki/", "help"),
    ("compare", "Compare", "compare/", "eye"),
    ("bench", "Benchmarks", "bench/", "bolt"),
]


def header(current: str, up: str) -> str:
    """The top bar. `up` is the relative path from the page to the site root ('' or '../')."""
    chips = "".join(
        f'<a class="tchip{" on" if key == current else ""}" href="{up}{href}"{CUR if key == current else ""}>{ic(icon, 16)}{label}</a>'
        for key, label, href, icon in SECTIONS
    )
    chips += f'<a class="tchip" href="{REPO}">{ic("ext", 16)}GitHub</a>'
    return (
        f'<header class="top"><a class="brand" href="{up}index.html">{ic("app", 22)}Raul\'s Toolbox</a>'
        f'<nav class="tnav" aria-label="Site">{chips}</nav></header>'
    )


def nav_item(label: str, href: str, icon: str, on: bool) -> str:
    return f'<a class="nav{" on" if on else ""}" href="{href}"{CUR if on else ""}>{ic(icon, 18)}<span>{html.escape(label)}</span></a>'


def sidebar(groups: list[tuple[str, list[tuple[str, str, str, bool]]]], label: str) -> str:
    """groups: [(heading, [(label, href, icon, current)])]"""
    body = "".join(
        f'<div class="navh">{html.escape(h)}</div>' + "".join(nav_item(*item) for item in items) for h, items in groups
    )
    return f'<nav class="side" aria-label="{html.escape(label)}">{body}</nav>'


def site_sidebar(current: str, up: str) -> str:
    """The sidebar of the pages that are not wiki pages."""
    return sidebar(
        [
            ("PLAY", [("Home", f"{up}index.html", "home", current in ("", "home")),
                      ("Download", f"{up}download/", "dl", current == "download"),
                      ("Demo", f"{up}demo/", "play", current == "demo"),
                      ("Guided tour", f"{up}tour/", "flask", current == "tour")]),
            ("SYSTEM", [("Wiki", f"{up}wiki/", "help", current == "wiki"),
                        ("Compare", f"{up}compare/", "eye", current == "compare"),
                        ("Benchmarks", f"{up}bench/", "bolt", current == "bench"),
                        ("Releases", f"{REPO}/releases", "versions", False),
                        ("Report a bug", f"{REPO}/issues/new/choose", "doctor", False)]),
        ],
        "Site pages",
    )


def fill(template: str, current: str, up: str, **extra: str) -> str:
    out = template.replace("<!--HEADER-->", header(current, up)).replace("<!--SIDE-->", site_sidebar("" if current == "home" else current, up))
    out = out.replace("{{up}}", up)
    for key, value in extra.items():
        out = out.replace("{{" + key + "}}", value)
    return out
