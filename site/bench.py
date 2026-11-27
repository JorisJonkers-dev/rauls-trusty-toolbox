# SPDX-License-Identifier: GPL-3.0-only
"""The benchmark pages of the public site: one overview and one page per project, drawn from toolbox-bench/1 records.

Records are small JSON files (the wiki page "Benchmarks" has the format). They come from docs/bench/<project>/ in this
repository, from a folder the publishing job checks out of the gh-pages branch (bench/results/<project>/), and, for the
Toolbox's own `raul bench`, from the github-action-benchmark history (data.js). Only what a record holds is drawn: a
comparison nobody has measured is drawn as "not measured yet", never as a number.
"""
import datetime
import json
import re
import sys
from pathlib import Path

import bench_svg as svg
from bench_svg import esc, num
from shell import ic

SCHEMA = "toolbox-bench/1"

# key, name, what it replaces, repository, blurb
PROJECTS = [
    ("nv-rs", "nv-rs", "the original game executable", "nv-rs",
     "The game engine. Frame rate and frame time on the places of the benchmark suite."),
    ("loot-rs", "loot-rs", "LOOT and libloot", "loot-rs",
     "Sorting the load order and evaluating the masterlist."),
    ("ttwright-rs", "ttwright-rs", "the official TTW installer", "ttwright-rs",
     "Installing Tale of Two Wastelands: total time and the patching steps."),
    ("lodwright-rs", "lodwright-rs", "xLODGen and DynDOLOD", "lodwright-rs",
     "Generating distant terrain, objects and trees: time per step and memory."),
    ("nvmod-rs", "nvmod-rs", "Mod Organizer 2", "nvmod-rs",
     "Instances, profiles, installing mods and conflicts."),
    ("toolbox", "Raul's Toolbox", "no other tool: it watches itself for slowdowns", "rauls-trusty-toolbox",
     "raul bench: a synthetic suite that runs on every change to main, to catch slowdowns."),
]
OWNER = "https://github.com/JorisJonkers-dev/"
ISSUES = {  # filled in by the maintainers when an issue exists: tool -> url (also readable from a record's pending list)
}
TIME_UNITS = {"ms", "s", "min", "h"}


class BenchError(Exception):
    pass


def _need(cond: bool, where: str, what: str) -> None:
    if not cond:
        raise BenchError(f"{where}: {what}")


def validate(rec: dict, where: str) -> None:
    _need(rec.get("schema") == SCHEMA, where, f'schema must be "{SCHEMA}"')
    _need(rec.get("project") in {p[0] for p in PROJECTS}, where, "unknown project")
    run = rec.get("run")
    if run is not None:
        for k in ("id", "date", "machine", "method"):
            _need(isinstance(run.get(k), str) and run[k], where, f"run.{k} is required")
        _need(re.fullmatch(r"\d{4}-\d{2}-\d{2}", run["date"]) is not None, where, "run.date must be YYYY-MM-DD")
    for g in rec.get("groups", []):
        gw = f"{where} group {g.get('id')}"
        for k in ("id", "title", "metric", "unit"):
            _need(isinstance(g.get(k), str) and g[k], gw, f"{k} is required")
        _need(g.get("better") in ("lower", "higher"), gw, 'better must be "lower" or "higher"')
        _need(run is not None, gw, "groups need a run")
        ids = [s.get("id") for s in g.get("series", [])]
        _need(ids and len(ids) == len(set(ids)), gw, "series ids are required and must differ")
        _need(any(s.get("role") == "ours" for s in g["series"]), gw, "at least one series with role ours")
        for s in g["series"]:
            _need(s.get("role") in ("ours", "against"), gw, "series.role must be ours or against")
            _need(isinstance(s.get("label"), str) and s["label"], gw, "series.label is required")
        for c in g.get("cases", []):
            _need(isinstance(c.get("name"), str), gw, "case.name is required")
            for sid, v in c.get("values", {}).items():
                _need(sid in ids, gw, f"case {c['name']}: value for unknown series {sid}")
                _need(isinstance(v, (int, float)) and not isinstance(v, bool) and v >= 0, gw, f"case {c['name']}: {sid} must be a number")


def load_records(dirs: list[Path]) -> list[dict]:
    recs = []
    for d in dirs:
        if not d or not d.is_dir():
            continue
        for f in sorted(d.rglob("*.json")):
            rec = json.loads(f.read_text())
            validate(rec, str(f))
            rec["_file"] = f
            recs.append(rec)
    return recs


def import_action_history(data_js: Path) -> dict | None:
    """The history github-action-benchmark wrote (medians in ms) as a toolbox record for the Toolbox's own page."""
    if not data_js or not data_js.is_file():
        return None
    text = data_js.read_text()
    data = json.loads(text[text.index("{"):])
    entries = next(iter(data["entries"].values()), [])
    if not entries:
        return None
    names: list[str] = []
    for e in entries:
        for b in e["benches"]:
            if b["name"] not in names:
                names.append(b["name"])
    hist = []
    for e in entries:
        sha = e["commit"]["id"]
        date = _day(e["date"] / 1000)
        hist.append({"date": date, "commit": sha[:7], "values": {b["name"]: {"ours": b["value"]} for b in e["benches"]}})
    last = entries[-1]
    return {
        "schema": SCHEMA, "project": "toolbox", "pending": [], "imported": True,
        "run": {"id": "raul-bench", "date": _day(last["date"] / 1000), "commit": last["commit"]["id"][:7], "version": None,
                "machine": "GitHub-hosted ubuntu-latest runner (shared, so noisy)",
                "method": "raul bench on synthetic fixtures (no game data), median wall time of the timed runs. Read from the history that "
                          "github-action-benchmark keeps on the gh-pages branch; the raw chart page is kept at bench/raw/.",
                "source": "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/blob/main/docs/BENCHMARKS.md"},
        "groups": [{"id": "cases", "title": "The raul bench suite", "metric": "Median wall time of one run", "unit": "ms", "better": "lower",
                    "series": [{"id": "ours", "label": "raul", "role": "ours"}],
                    "cases": [{"name": n, "values": {"ours": last_val(last, n)}} for n in names if last_val(last, n) is not None],
                    "history": hist}],
    }


def last_val(entry: dict, name: str):
    for b in entry["benches"]:
        if b["name"] == name:
            return b["value"]
    return None


def _day(ts: float) -> str:
    import datetime
    return datetime.datetime.fromtimestamp(ts, datetime.timezone.utc).strftime("%Y-%m-%d")




# ---------- reading the numbers ----------

ICON = {"nv-rs": "pad", "loot-rs": "load", "ttwright-rs": "ttw", "lodwright-rs": "lod", "nvmod-rs": "mods", "toolbox": "app"}
RANGES = [("30", "30 days", 30), ("90", "90 days", 90), ("all", "All", None)]


def ratio(g: dict, case: dict):
    """(ratio, other tool, words) for the first own series against the first measured other one; ratio > 1 is ours better."""
    ours = next((s for s in g["series"] if s["role"] == "ours" and case["values"].get(s["id"]) is not None), None)
    other = next((s for s in g["series"] if s["role"] == "against" and case["values"].get(s["id"]) is not None), None)
    if not ours or not other:
        return None
    a, b = case["values"][ours["id"]], case["values"][other["id"]]
    if a <= 0 or b <= 0:
        return None
    r = b / a if g["better"] == "lower" else a / b
    if g["unit"] in TIME_UNITS:
        words = ("faster", "slower")
    elif g["better"] == "higher":
        words = ("higher", "lower")
    else:
        words = ("less", "more")
    return r, other.get("tool") or other["label"], words


def delta(g: dict, case: dict) -> tuple[str, str] | None:
    """('good'|'bad'|'same', short text) for the pill over a group of bars."""
    got = ratio(g, case)
    if not got:
        return None
    r, _, (good, bad) = got
    if r >= 1.05:
        return "good", f"{r:.1f}× {good}"
    if r <= 0.95:
        return "bad", f"{1 / r:.1f}× {bad}"
    return "same", "about the same"


def merged(records: list[dict], project: str) -> dict:
    """Latest run per group id, plus a history of every run of the same group, for one project."""
    mine = sorted((r for r in records if r["project"] == project and r.get("run")), key=lambda r: (r["run"]["date"], bool(r.get("imported"))))
    groups: dict[str, dict] = {}
    for rec in mine:
        for g in rec["groups"]:
            entry = groups.setdefault(g["id"], {"history": {}})
            entry.update({k: v for k, v in g.items() if k != "history"})
            entry["run"] = rec["run"]
            entry["record"] = rec
            for h in g.get("history", []):
                entry["history"][h["commit"] or h["date"]] = h
            sha = rec["run"].get("commit")
            entry["history"][sha or rec["run"]["date"]] = {
                "date": rec["run"]["date"], "commit": sha,
                "values": {c["name"]: c["values"] for c in g["cases"]}}
    for entry in groups.values():
        entry["history"] = sorted(entry["history"].values(), key=lambda h: h["date"])
    pending = []
    for rec in records:
        if rec["project"] == project:
            pending += rec.get("pending", [])
    return {"groups": list(groups.values()), "pending": pending}


def daily(g: dict) -> list[dict]:
    """The history as one point per day: the day's median per case and series (the CI history has many runs a day)."""
    days: dict[str, list] = {}
    for h in g.get("history", []):
        days.setdefault(h["date"], []).append(h)
    out = []
    for d, hs in sorted(days.items()):
        vals: dict = {}
        for h in hs:
            for case, sv in h["values"].items():
                for sid, v in sv.items():
                    if v is not None:
                        vals.setdefault(case, {}).setdefault(sid, []).append(v)
        commits = [h["commit"] for h in hs if h.get("commit")]
        out.append({"date": d, "runs": len(hs), "commits": (commits[0] if len(commits) == 1 else f"{commits[0]} to {commits[-1]}") if commits else "",
                    "values": {c: {sid: _median(v) for sid, v in sv.items()} for c, sv in vals.items()},
                    "counts": {c: {sid: len(v) for sid, v in sv.items()} for c, sv in vals.items()}})
    return out


def _median(v: list[float]) -> float:
    v = sorted(v)
    n = len(v)
    return v[n // 2] if n % 2 else (v[n // 2 - 1] + v[n // 2]) / 2


def short_machine(run: dict) -> str:
    m = run.get("machine", "")
    for sep in (";", " ("):
        m = m.split(sep)[0]
    return m if len(m) <= 46 else m[:44].rsplit(" ", 1)[0].rstrip(",") + "…"


def issue_for(pending: list[dict], tool: str | None) -> str | None:
    for p in pending:
        if p.get("issue") and tool and (tool.lower() in p["tool"].lower() or p["tool"].lower() in tool.lower()):
            return p["issue"]
    return next((p["issue"] for p in pending if p.get("issue")), None)


def kpi(g: dict, hist: list[dict] | None = None) -> dict:
    """The headline of one group: the best ratio when there is a comparison, else the best value."""
    rs = [(r, c) for c in g["cases"] if (r := ratio(g, c))]
    if rs:
        (r, other, words), case = max(rs, key=lambda x: x[0][0])
        good = r >= 1.05
        lo = min(x[0][0] for x in rs)
        rng = f"{lo:.1f}× to {r:.1f}×" if len(rs) > 1 and abs(lo - r) >= .05 else f"{r:.1f}×"
        if r < 1:
            worst = min(rs, key=lambda x: x[0][0])
            r, case = worst[0][0], worst[1]
            return {"big": f"{1 / r:.1f}×", "label": f"{words[1]} than {other}", "sub": f'{g["title"]} · {case["name"]}',
                    "chip": ("bad", f"vs {other}")}
        return {"big": f"{r:.1f}×", "label": f"{words[0] if good else 'about the same as'}{' than' if good else ''} {other}",
                "sub": f'{g["title"]} · best at {case["name"]}', "chip": ("good" if good else "same", rng + f" over {len(rs)} case{'s' if len(rs) > 1 else ''}")}
    ours = next(s for s in g["series"] if s["role"] == "ours")
    vals = [(c["values"][ours["id"]], c) for c in g["cases"] if c["values"].get(ours["id"]) is not None]
    if not vals:
        return {"big": "—", "label": g["title"], "sub": "not measured yet", "chip": ("muted", "not measured")}
    pick = min if g["better"] == "lower" else max
    v, case = pick(vals, key=lambda x: x[0])
    chip = ("muted", "no comparison yet")
    miss = [s for s in g["series"] if s.get("status") == "not-measured"]
    if miss:
        chip = ("warn", f'{miss[0].get("tool") or miss[0]["label"]}: not measured')
    if hist and len(hist) >= 2:
        first = (hist[0]["values"].get(case["name"]) or {}).get(ours["id"])
        last = (hist[-1]["values"].get(case["name"]) or {}).get(ours["id"])
        if first and last:
            ch = (last - first) / first * 100
            better = ch < 0 if g["better"] == "lower" else ch > 0
            kind = "same" if abs(ch) < 3 else ("good" if better else "bad")
            chip = (kind, f"{'+' if ch >= 0 else '−'}{abs(ch):.0f}% since {svg.day_label(svg._d(hist[0]['date']))}")
    best = "fastest" if g["unit"] in TIME_UNITS else ("best" if g["better"] == "higher" else "lowest")
    return {"big": num(v), "unit": g["unit"], "label": f'{ours["label"]} · {g["title"]}', "sub": f"{best}: {case['name']}", "chip": chip}


def kpi_tile(k: dict, extra_cls: str = "") -> str:
    unit = f'<small>{esc(k["unit"])}</small>' if k.get("unit") else ""
    kind, text = k["chip"]
    return (f'<div class="bx-kpi {extra_cls}"><div class="bx-kv">{esc(k["big"])}{unit}</div>'
            f'<div class="bx-kl">{esc(k["label"])}</div><div class="bx-kf"><span class="bx-chip bx-{kind}">{esc(text)}</span>'
            f'<span class="bx-ks">{esc(k["sub"])}</span></div></div>')


# ---------- the pieces of a page ----------

def details(g: dict, project: str, data_href: str, extra: str = "") -> str:
    run = g["run"]
    rows = [("Method", esc(run["method"])), ("Machine", esc(run["machine"]))]
    if run.get("version"):
        rows.append(("Version", esc(run["version"])))
    rows.append(("Commit", esc(run.get("commit") or "not recorded")))
    rows.append(("Run", f'{esc(run["id"])}, {esc(run["date"])}'))
    if run.get("source"):
        src = run["source"]
        rows.append(("Source", f'<a href="{esc(src)}">{esc(src.split("github.com/")[-1])}</a>' if src.startswith("http") else esc(src)))
    dl = "".join(f"<dt>{k}</dt><dd>{v}</dd>" for k, v in rows)
    head = "".join(f'<th scope="col">{esc(s["label"])}</th>' for s in g["series"])
    body = []
    for c in g["cases"]:
        cells = []
        for s in g["series"]:
            v = c["values"].get(s["id"])
            flag = (c.get("flags") or {}).get(s["id"])
            cells.append(f'<td>{num(v)} <small>{esc(g["unit"])}</small>{" <em>" + esc(flag) + "</em>" if flag else ""}</td>' if v is not None
                         else '<td class="bx-na">not measured</td>' if s.get("status") == "not-measured" else '<td class="bx-na">—</td>')
        body.append(f'<tr><th scope="row">{esc(c["name"])}</th>{"".join(cells)}</tr>')
    notes = "".join(f'<p class="bx-note"><b>{esc(s["label"])}:</b> {esc(s["note"])}</p>' for s in g["series"] if s.get("note"))
    return (f'<details class="bx-det"><summary><span class="bx-meta">{ic("clock", 15)}{esc(run["date"])} · {esc(short_machine(run))}</span>'
            f'<span class="bx-btn">{ic("info", 15)}Details</span></summary>'
            f'<div class="bx-panel"><div class="bx-pcols"><dl class="bx-dl">{dl}</dl>'
            f'<div class="bx-tablebox"><table class="bx-table"><caption>{esc(g["metric"])}, in {esc(g["unit"])}</caption>'
            f'<thead><tr><th scope="col">Case</th>{head}</tr></thead><tbody>{"".join(body)}</tbody></table>{notes}'
            f'<div class="bx-actions"><a class="bx-btn bx-primary" href="{esc(data_href)}" download>{ic("dl", 15)}Download the JSON record</a>{extra}</div>'
            f'</div></div></div></details>')


def better_chip(g: dict) -> str:
    lower = g["better"] == "lower"
    return f'<span class="bx-better">{ic("down" if lower else "up", 14)}{"Lower" if lower else "Higher"} is better</span>'


def figure(g: dict, project: str, pending: list[dict], data_href: str, span: str, extra: str = "") -> str:
    against = next((s for s in g["series"] if s["role"] == "against"), None)
    issue = issue_for(pending, (against or {}).get("tool") or (against or {}).get("label"))
    meta = f'{g["run"]["date"]} · {short_machine(g["run"])}'
    ann = lambda c: delta(g, c)
    # the same figure twice: sized for a wide card, and for a phone (drawn at its own size, scrolled sideways when wider)
    chart = (f'<div class="bx-wide">{svg.grouped_bars(g, meta, issue, 1080 if span == "full" else 520, ann)}</div>'
             f'<div class="bx-narrow">{svg.grouped_bars(g, meta, issue, 330, ann)}</div>')
    k = kpi(g)
    kind, text = k["chip"]
    head_kpi = f'<span class="bx-chip bx-{kind}">{esc(text)}</span>' if kind in ("good", "bad") else ""
    return (f'<figure class="bx-fig bx-{span}" id="{esc(g["id"])}"><header class="bx-fh"><div class="bx-fhl"><h2 class="bx-ft">{esc(g["title"])}</h2>'
            f'<p class="bx-fs">{esc(g["metric"])} · {esc(g["unit"])}</p></div><div class="bx-fhr">{head_kpi}{better_chip(g)}</div></header>'
            f'{svg.legend(g, issue)}<div class="bx-scroll" tabindex="-1">{chart}</div>{details(g, project, data_href, extra)}</figure>')


def history_figure(g: dict, data_href: str) -> str:
    hist = daily(g)
    if len(hist) < 2:
        return ""
    end = svg._d(hist[-1]["date"])
    first = svg._d(hist[0]["date"])
    col = svg.colours(g["series"])
    shown = [s for s in g["series"] if any(s["id"] in (h["values"].get(c["name"]) or {}) for h in hist for c in g["cases"])]
    gid = f'h-{g["id"]}'
    radios, labels, panes = [], [], []
    for key, text, days in RANGES:
        start = max(first, end - datetime.timedelta(days=days - 1)) if days else first
        radios.append(f'<input type="radio" class="bx-r bx-r{key}" name="{gid}" id="{gid}-{key}"{" checked" if key == "30" else ""}>')
        labels.append(f'<label class="bx-l{key}" for="{gid}-{key}">{text}</label>')
        cells = []
        for c in g["cases"]:
            for s in shown:
                pts = [{"date": h["date"], "v": h["values"][c["name"]][s["id"]], "runs": h["counts"][c["name"]][s["id"]], "commits": h["commits"]}
                       for h in hist if s["id"] in (h["values"].get(c["name"]) or {})]
                inr = [p for p in pts if p["date"] >= start.isoformat()]
                if not inr:
                    continue
                ch = ""
                if len(inr) >= 2:
                    pct = (inr[-1]["v"] - inr[0]["v"]) / inr[0]["v"] * 100
                    good = pct < 0 if g["better"] == "lower" else pct > 0
                    kind = "same" if abs(pct) < 3 else "good" if good else "bad"
                    ch = f'<span class="bx-chip bx-{kind}">{"+" if pct >= 0 else "−"}{abs(pct):.0f}%</span>'
                name = c["name"] if len(shown) == 1 else f'{c["name"]} · {s["label"]}'
                cells.append(f'<div class="bx-hcell"><div class="bx-hh"><span class="bx-hn">{esc(name)}</span><span class="bx-hv">{num(inr[-1]["v"])}'
                             f'<small>{esc(g["unit"])}</small></span>{ch}</div>'
                             + svg.line_chart(name, pts, g["unit"], g["better"], start, end, col[s["id"]][0], meta=short_machine(g["run"])) + "</div>")
        panes.append(f'<div class="bx-pane bx-p{key}">{"".join(cells)}</div>')
    runs = sum(h["runs"] for h in hist)
    # the raw table of daily medians
    first_s = shown[0]["id"] if shown else None
    head = "".join(f'<th scope="col">{esc(c["name"])}</th>' for c in g["cases"])
    body = "".join(f'<tr><th scope="row">{h["date"]}</th><td>{h["runs"]}</td>'
                   + "".join(f'<td>{num(v)}</td>' if (v := (h["values"].get(c["name"]) or {}).get(first_s)) is not None else "<td>—</td>" for c in g["cases"]) + "</tr>"
                   for h in reversed(hist))
    det = (f'<details class="bx-det"><summary><span class="bx-meta">{ic("clock", 15)}{runs} runs over {len(hist)} days · {esc(short_machine(g["run"]))}</span>'
           f'<span class="bx-btn">{ic("info", 15)}Details</span></summary><div class="bx-panel"><div class="bx-pcols"><dl class="bx-dl">'
           f'<dt>Method</dt><dd>Each point is the median of that day\'s runs; the latest run is the ringed point. {esc(g["run"]["method"])}</dd>'
           f'<dt>Machine</dt><dd>{esc(g["run"]["machine"])}</dd><dt>Commit</dt><dd>{esc(g["run"].get("commit") or "not recorded")} (latest)</dd></dl>'
           f'<div class="bx-tablebox"><table class="bx-table"><caption>Daily median, in {esc(g["unit"])}</caption><thead><tr><th scope="col">Day</th>'
           f'<th scope="col">Runs</th>{head}</tr></thead><tbody>{body}</tbody></table>'
           f'<div class="bx-actions"><a class="bx-btn bx-primary" href="{esc(data_href)}" download>{ic("dl", 15)}Download the JSON record</a></div></div></div></div></details>')
    return (f'<figure class="bx-fig bx-full bx-hist" id="history-{esc(g["id"])}">{"".join(radios)}'
            f'<header class="bx-fh"><div class="bx-fhl"><h2 class="bx-ft">History · {esc(g["title"])}</h2>'
            f'<p class="bx-fs">Daily median of {esc(g["metric"].lower())} · {esc(g["unit"])} · one chart per case, each on its own scale</p></div>'
            f'<div class="bx-fhr"><div class="bx-range" role="group" aria-label="Range">{"".join(labels)}</div>{better_chip(g)}</div></header>'
            f'{svg.legend(g, None, "the ringed point is the latest day")}<div class="bx-panes">{"".join(panes)}</div>{det}</figure>')


def todo_cards(pending: list[dict], replaces: str, note: str = "") -> str:
    if not pending:
        return ""
    cards = []
    for p in pending:
        link = (f'<a class="bx-btn" href="{esc(p["issue"])}">{ic("ext", 15)}Issue {esc(p["issue"].rsplit("/", 1)[-1])}</a>' if p.get("issue") else "")
        cards.append(f'<div class="bx-todo">{svg.mini_empty(320, 110, "not measured yet")}<div class="bx-tdb"><h3>{esc(p["tool"])}</h3>'
                     f'<p>{esc(p["what"])}.</p>{link}</div></div>')
    return (f'<section class="bx-section" id="to-measure"><h2 class="bx-sh">{ic("flask", 18)}Still to measure <span class="bx-count">{len(pending)}</span></h2>'
            + (f'<p class="bx-shs">{esc(note)}</p>' if note else "") + f'<div class="bx-todos">{"".join(cards)}</div></section>')


def spans(groups: list[dict]) -> list[str]:
    out = []
    for g in groups:
        slots = len(g["series"]) * len(g["cases"])
        out.append("full" if len(g["cases"]) > 4 or slots > 8 else "half")
    if out.count("half") % 2:
        out[len(out) - 1 - out[::-1].index("half")] = "full"
    return out


def data_file(dest: Path, key: str, g: dict) -> str:
    """Copies the record behind a group next to the page; returns its link from the page."""
    rec = g["record"]
    name = rec["_file"].name if rec.get("_file") else f'{rec["run"]["id"]}-history.json'
    (dest / key / "data").mkdir(parents=True, exist_ok=True)
    (dest / key / "data" / name).write_text(json.dumps({k: v for k, v in rec.items() if not k.startswith("_")}, indent=1) + "\n")
    return f"data/{name}"


# ---------- pages ----------

def project_page(key, name, replaces, repo, blurb, m: dict, dest: Path) -> tuple[str, dict]:
    groups, pending = m["groups"], m["pending"]
    body = [f'<nav class="bx-crumbs" aria-label="Breadcrumb"><a href="../">{ic("bolt", 14)}Benchmarks</a>{ic("chev", 14)}<span>{esc(name)}</span></nav>']
    last = max((g["run"]["date"] for g in groups), default=None)
    sub = f"Set beside {esc(replaces)}" + (f" · last run {esc(last)}" if last else " · nothing measured yet")
    extra_btn = ('<a class="bx-btn" href="../raw/">' + ic("versions", 15) + 'Raw chart page</a>') if key == "toolbox" else ""
    body.append(f'<header class="bx-phead"><span class="bx-pico">{ic(ICON[key], 26)}</span><div class="bx-ptx"><h1>{esc(name)}</h1>'
                f'<p class="bx-psub">{sub}</p></div><div class="bx-pact">{extra_btn}<a class="bx-btn" href="{OWNER}{repo}">{ic("ext", 15)}{esc(repo)}</a></div></header>')
    hists = {g["id"]: daily(g) for g in groups}
    if groups:
        # a tile per group that is set beside another tool (or the first group): a split of our own run is no headline
        shown = [g for i, g in enumerate(groups) if i == 0 or any(s["role"] == "against" for s in g["series"])][:3]
        tiles = [kpi_tile(kpi(g, hists[g["id"]] if len(hists[g["id"]]) >= 2 else None)) for g in shown]
        tiles.append(f'<div class="bx-kpi bx-kpim"><div class="bx-kv">{len(pending)}</div><div class="bx-kl">comparison{"s" if len(pending) != 1 else ""} still to measure</div>'
                     f'<div class="bx-kf"><a class="bx-chip bx-{"warn" if pending else "good"}" href="#to-measure">{"see below" if pending else "none open"}</a></div></div>'
                     if pending else
                     f'<div class="bx-kpi bx-kpim"><div class="bx-kv">{sum(len(hists[g["id"]]) for g in groups)}</div><div class="bx-kl">days with runs</div>'
                     f'<div class="bx-kf"><span class="bx-chip bx-muted">{esc(short_machine(groups[0]["run"]))}</span></div></div>')
        body.append(f'<div class="bx-kpis">{"".join(tiles)}</div>')
        figs = []
        for g, sp in zip(groups, spans(groups)):
            href = data_file(dest, key, g)
            raw = '<a class="bx-btn" href="../raw/">' + ic("versions", 15) + 'Raw chart page</a>' if key == "toolbox" else ""
            figs.append(figure(g, key, pending, href, sp, raw))
            figs.append(history_figure(g, href))
        body.append(f'<div class="bx-grid">{"".join(figs)}</div>')
    note = ("The Linux installer this project rewrites states about two minutes on a Ryzen 7 9800X3D in its own changelog; "
            "that is another program measured by someone else, so it is not drawn.") if key == "ttwright-rs" else ""
    if groups:
        body.append(todo_cards(pending, replaces, note))
    else:
        # nothing measured: the figure the numbers will go in, empty, with what to measure and where it is tracked
        for p in pending or [{"tool": replaces, "what": "Not measured yet"}]:
            link = f'<a class="bx-btn bx-primary" href="{esc(p["issue"])}">{ic("ext", 15)}Tracked in issue {esc(p["issue"].rsplit("/", 1)[-1])}</a>' if p.get("issue") else ""
            body.append(f'<section class="bx-fig bx-emptyfig"><header class="bx-fh"><div class="bx-fhl"><h2 class="bx-ft">{esc(name)} beside the {esc(p["tool"])}</h2>'
                        f'<p class="bx-fs">{esc(blurb)}</p></div><div class="bx-fhr"><span class="bx-chip bx-muted">not measured yet</span></div></header>'
                        f'<ul class="bx-legend"><li class="bx-li"><i class="bx-sw bx-swnm"></i>{esc(name)}<span class="bx-role">ours</span></li>'
                        f'<li class="bx-li"><i class="bx-sw bx-swnm"></i>{esc(p["tool"])}<span class="bx-role">replaced tool</span></li></ul>'
                        f'{svg.mini_empty(1000, 230, "no numbers yet: nothing is drawn until a run is recorded")}'
                        f'<div class="bx-cta"><p>{esc(p["what"])}.' + (f" {esc(note)}" if note else "") + f'</p>{link}</div></section>')
    body.append(f'<p class="bx-foot"><a href="{OWNER}rauls-trusty-toolbox/blob/main/docs/wiki/benchmarks.md">How results are added</a> · '
                f'every bar is a recorded run; none are estimated.</p>')
    return "".join(body), {"groups": groups, "pending": pending, "hists": hists}


def overview_card(key, name, replaces, info: dict) -> str:
    groups, pending, hists = info["groups"], info["pending"], info["hists"]
    issue = next((p["issue"] for p in pending if p.get("issue")), None)
    if not groups:
        status, k = ("muted", "Not measured"), None
        chart = svg.mini_empty()
    else:
        g0 = groups[0]
        h0 = hists[g0["id"]]
        compared = any(ratio(g, c) for g in groups for c in g["cases"])
        status = ("good", "Compared") if compared else ("info", "CI history") if len(h0) >= 2 else ("warn", "No comparison yet")
        k = kpi(g0, h0 if len(h0) >= 2 else None)
        if len(h0) >= 2:
            ours = next(s for s in g0["series"] if s["role"] == "ours")["id"]
            big = max(g0["cases"], key=lambda c: c["values"].get(ours) or 0)["name"]
            pts = [{"date": h["date"], "v": h["values"][big][ours]} for h in h0 if ours in (h["values"].get(big) or {})]
            chart = svg.sparkline(pts, label=big, unit=g0["unit"])
            k["sub"] = f"{big} · daily median"
            k["big"], k["unit"] = num(pts[-1]["v"]), g0["unit"]
            k["label"] = "latest day, the largest case"
        else:
            chart = svg.mini_bars(g0, issue)
    tile = kpi_tile(k, "bx-kpic") if k else (f'<div class="bx-kpi bx-kpic"><div class="bx-kv bx-dim">—</div><div class="bx-kl">vs {esc(replaces)}</div>'
                                             f'<div class="bx-kf"><span class="bx-chip bx-muted">not measured yet</span></div></div>')
    cap = f'{esc(groups[0]["title"])} · {esc(groups[0]["unit"])}' if groups else f"vs {esc(replaces)}"
    when = max((g["run"]["date"] for g in groups), default=None)
    foot = (f'<span class="bx-meta">{ic("clock", 14)}{esc(when)}</span>' if when else
            (f'<a class="bx-meta" href="{esc(issue)}">{ic("ext", 14)}issue {esc(issue.rsplit("/", 1)[-1])}</a>' if issue else '<span></span>'))
    return (f'<article class="bx-card"><header class="bx-ch"><span class="bx-cico">{ic(ICON[key], 20)}</span><h2><a href="{key}/">{esc(name)}</a></h2>'
            f'<span class="bx-chip bx-{status[0]}">{status[1]}</span></header>{tile}<div class="bx-cfig"><p class="bx-cap">{cap}</p>{chart}</div>'
            f'<footer class="bx-cf">{foot}<a class="bx-btn bx-primary" href="{key}/" aria-label="Open {esc(name)}">Open{ic("chev", 15)}</a></footer></article>')


def summary(infos: list[tuple]) -> str:
    best = None
    compared = 0
    measured = 0
    days: set[str] = set()
    runs = 0
    for (key, name, *_), info in infos:
        if info["groups"]:
            measured += 1
        if any(ratio(g, c) for g in info["groups"] for c in g["cases"]):
            compared += 1
        for g in info["groups"]:
            runs += sum(h["runs"] for h in info["hists"][g["id"]]) or 1
            days |= {h["date"] for h in info["hists"][g["id"]]}
            for c in g["cases"]:
                r = ratio(g, c)
                if r and (best is None or r[0] > best[0]):
                    best = (r[0], r[1], r[2][0], name, c["name"], g["title"])
    n = len(infos)
    others = n - 1  # the Toolbox watches itself
    pct = round(compared / others * 100) if others else 0
    tiles = [
        f'<div class="bx-kpi bx-hero"><div class="bx-kv">{f"{best[0]:.1f}×" if best else "—"}</div><div class="bx-kl">best speed-up · {esc(best[3]) if best else "none yet"}</div>'
        f'<div class="bx-kf"><span class="bx-chip bx-good">{esc(best[2])} than {esc(best[1])}</span><span class="bx-ks">{esc(best[5])} · {esc(best[4])}</span></div></div>' if best else
        '<div class="bx-kpi"><div class="bx-kv">—</div><div class="bx-kl">best speed-up</div></div>',
        f'<div class="bx-kpi"><div class="bx-kv">{compared}<small>of {others}</small></div><div class="bx-kl">projects compared with the tool they replace</div>'
        f'<div class="bx-kf"><span class="bx-meter" role="img" aria-label="{pct}%"><i style="width:{pct}%"></i></span></div></div>',
        f'<div class="bx-kpi"><div class="bx-kv">{measured}<small>of {n}</small></div><div class="bx-kl">projects with measurements</div>'
        f'<div class="bx-kf"><span class="bx-meter" role="img" aria-label="{round(measured / n * 100)}%"><i style="width:{round(measured / n * 100)}%"></i></span></div></div>',
        f'<div class="bx-kpi"><div class="bx-kv">{runs:,}</div><div class="bx-kl">recorded runs</div>'
        f'<div class="bx-kf"><span class="bx-chip bx-muted">{len(days)} days with runs</span></div></div>',
    ]
    return f'<div class="bx-kpis bx-sum">{"".join(tiles)}</div>'


def build(out: Path, template: str, fill, records: list[dict]) -> None:
    dest = out / "bench"
    dest.mkdir(parents=True, exist_ok=True)
    defs = svg.defs()
    infos = []
    for key, name, replaces, repo, blurb in PROJECTS:
        (dest / key).mkdir(exist_ok=True)
        body, info = project_page(key, name, replaces, repo, blurb, merged(records, key), dest)
        infos.append(((key, name, replaces, repo, blurb), info))
        (dest / key / "index.html").write_text(fill(template, "bench", "../../", title=esc(name) + " benchmarks", body=defs + body))
    cards = "".join(overview_card(p[0], p[1], p[2], info) for p, info in infos)
    intro = ('<header class="bx-phead"><span class="bx-pico">' + ic("bolt", 26) + '</span><div class="bx-ptx"><h1>Benchmarks</h1>'
             '<p class="bx-psub">Each project set beside the tool it replaces · every bar a recorded run</p></div></header>')
    (dest / "index.html").write_text(fill(template, "bench", "../", title="Benchmarks", body=defs + intro + summary(infos)
        + f'<h2 class="bx-sh">{ic("box", 18)}Projects</h2><div class="bx-cards">{cards}</div>'
        '<p class="bx-foot">Runs come from the maintainer\'s machines and, for raul bench, shared CI runners: compare bars within one figure, not across. '
        f'<a href="{OWNER}rauls-trusty-toolbox/blob/main/docs/wiki/benchmarks.md">How results are added</a>.</p>'))
