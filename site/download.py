# SPDX-License-Identifier: GPL-3.0-only
"""The download page (/download/).

The page is current without a site rebuild: download.js asks the GitHub REST API for the latest release when it is
opened. This module only builds the fallback it shows when the API is out of reach (rate limit, offline): the
releases as they were at build time. They come from the API when the build can reach it, else from the committed
site/releases.snapshot.json (refresh it with `python3 site/download.py --refresh`).
"""
import html
import json
import os
import shutil
import sys
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

from shell import REPO

SITE = Path(__file__).resolve().parent
SNAPSHOT = SITE / "releases.snapshot.json"
API = "https://api.github.com/repos/JorisJonkers-dev/rauls-trusty-toolbox/releases?per_page=30"


def fetch() -> list[dict]:
    headers = {"Accept": "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28", "User-Agent": "raul-site-build"}
    token = os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = f"Bearer {token}"
    with urllib.request.urlopen(urllib.request.Request(API, headers=headers), timeout=20) as res:
        return json.load(res)


def normalise(releases: list[dict]) -> dict:
    """What the page needs: every release's tag, date and link; the newest published one in full."""
    shown = [r for r in releases if not r.get("draft")]
    out = []
    for i, r in enumerate(shown):
        item = {"tag": r["tag_name"], "name": r.get("name") or r["tag_name"], "date": (r.get("published_at") or "")[:10],
                "url": r["html_url"], "prerelease": bool(r.get("prerelease"))}
        if i == 0:
            item["body"] = r.get("body") or ""
            item["assets"] = [{"name": a["name"], "size": a["size"], "digest": a.get("digest") or "", "url": a["browser_download_url"]}
                              for a in r.get("assets", [])]
        out.append(item)
    return {"fetched": datetime.now(timezone.utc).strftime("%Y-%m-%d"), "releases": out}


def load() -> dict:
    try:
        return normalise(fetch())
    except Exception as e:  # offline or rate limited: the committed snapshot is the fallback
        print(f"download page: GitHub not reachable ({e}); using site/releases.snapshot.json", file=sys.stderr)
        return json.loads(SNAPSHOT.read_text())


def build(out: Path, fill) -> None:
    data = load()
    latest = data["releases"][0]
    blob = json.dumps(data).replace("<", "\\u003c")
    dest = out / "download"
    dest.mkdir(parents=True)
    page = fill((SITE / "download.html").read_text(), "download", "../", data=blob, version=html.escape(latest["tag"]), repo=REPO)
    (dest / "index.html").write_text(page)
    shutil.copy(SITE / "download.js", dest / "download.js")


if __name__ == "__main__":
    if sys.argv[1:] == ["--refresh"]:
        SNAPSHOT.write_text(json.dumps(normalise(fetch()), indent=1) + "\n")
        print(f"wrote {SNAPSHOT}")
    else:
        sys.exit("usage: python3 site/download.py --refresh")
