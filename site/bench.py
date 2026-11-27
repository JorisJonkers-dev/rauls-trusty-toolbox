# SPDX-License-Identifier: GPL-3.0-only
"""The benchmark pages of the public site: one overview and one page per project, drawn from toolbox-bench/1 records.

Records are small JSON files (the wiki page "Benchmarks" has the format). They come from docs/bench/<project>/ in this
repository, from a folder the publishing job checks out of the gh-pages branch (bench/results/<project>/), and, for the
Toolbox's own `raul bench`, from the github-action-benchmark history (data.js). Only what a record holds is drawn: a
comparison nobody has measured is drawn as "not measured yet", never as a number.
"""
import html
import json
import re
import sys
from pathlib import Path

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
OURS_COLOURS = ["a", "b", "c"]
AGAINST_COLOURS = ["x", "y"]


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


# ---------- formatting ----------

def num(v: float) -> str:
    if v >= 1000:
        return f"{v:,.0f}"
    if v >= 100:
        return f"{v:.0f}"
    if v >= 10:
        return f"{v:.1f}".rstrip("0").rstrip(".")
    return f"{v:.2f}".rstrip("0").rstrip(".")


def esc(s) -> str:
    return html.escape(str(s), quote=True)


def delta(g: dict, case: dict) -> tuple[str, str] | None:
    """('good'|'bad'|'same', text) comparing the first ours series with the first measured against series, if both exist."""
    ours = next((s for s in g["series"] if s["role"] == "ours" and case["values"].get(s["id"]) is not None), None)
    other = next((s for s in g["series"] if s["role"] == "against" and case["values"].get(s["id"]) is not None), None)
    if not ours or not other:
        return None
    a, b = case["values"][ours["id"]], case["values"][other["id"]]
    if a <= 0 or b <= 0:
        return None
    ratio = b / a if g["better"] == "lower" else a / b
    name = other.get("tool") or other["label"]
    if g["unit"] in TIME_UNITS:
        good, bad = "faster", "slower"
    elif g["better"] == "higher":
        good, bad = "higher", "lower"
    else:
        good, bad = "less", "more"
    if ratio >= 1.05:
        return "good", f"{ratio:.1f}x {good} than {name}"
    if ratio <= 0.95:
        return "bad", f"{1 / ratio:.1f}x {bad} than {name}"
    return "same", f"about the same as {name}"


def compare_line(g: dict) -> str | None:
    ds = [d for c in g["cases"] if (d := delta(g, c))]
    if not ds:
        return None
    ratios = sorted(float(d[1].split("x")[0]) for d in ds if d[0] != "same")
    if not ratios or len({d[0] for d in ds}) > 1:
        return ds[0][1]
    word = ds[0][1].split("x ", 1)[1]
    lo, hi = ratios[0], ratios[-1]
    return f"{lo:.1f}x {word}" if lo == hi else f"{lo:.1f}x to {hi:.1f}x {word}"


def merged(records: list[dict], project: str) -> dict:
    """Latest run per group id, plus a history of every run of the same group, for one project."""
    mine = sorted((r for r in records if r["project"] == project and r.get("run")), key=lambda r: r["run"]["date"])
    groups: dict[str, dict] = {}
    for rec in mine:
        for g in rec["groups"]:
            entry = groups.setdefault(g["id"], {"history": {}})
            entry.update({k: v for k, v in g.items() if k != "history"})
            entry["run"] = rec["run"]
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


# ---------- drawing ----------

DEFS = (
    '<svg width="0" height="0" style="position:absolute" aria-hidden="true" focusable="false"><defs>'
    + "".join(
        f'<linearGradient id="gr-{k}" x1="0" x2="1"><stop offset="0" stop-color="{a}"/><stop offset="1" stop-color="{b}"/></linearGradient>'
        for k, a, b in [("a", "#1FA99C", "#5CF0DF"), ("b", "#3F7FE0", "#8DBBFF"), ("c", "#8466E0", "#C0A8FF"),
                        ("x", "#D9760F", "#FFB45E"), ("y", "#D9544F", "#FF9A94")])
    + '<pattern id="hatch" width="8" height="8" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">'
      '<rect width="8" height="8" fill="none"/><rect width="3" height="8" fill="#FFFFFF" fill-opacity=".28"/></pattern>'
    '</defs></svg>'
)


def bar(width_pct: float, grad: str, hatched: bool) -> str:
    w = max(width_pct, 1.2)
    extra = f'<rect width="{w:.2f}%" height="22" rx="7" fill="url(#hatch)"/>' if hatched else ""
    return (f'<svg class="track" width="100%" height="22" role="presentation">'
            f'<rect width="100%" height="22" rx="7" class="rail"/>'
            f'<rect class="fill" style="--w:{w:.2f}%" width="{w:.2f}%" height="22" rx="7" fill="url(#gr-{grad})"/>{extra}</svg>')


def ghost() -> str:
    return ('<svg class="track" width="100%" height="22" role="presentation"><rect x="1" y="1" width="calc(100% - 2px)" height="20" rx="7" '
            'class="ghost"/></svg>')


def group_html(g: dict, project: str) -> str:
    series = g["series"]
    colour = {}
    ni = na = 0
    for s in series:
        if s["role"] == "ours":
            colour[s["id"]] = OURS_COLOURS[ni % 3]; ni += 1
        else:
            colour[s["id"]] = AGAINST_COLOURS[na % 2]; na += 1
    top = max([v for c in g["cases"] for v in c["values"].values() if v is not None] or [1])
    # a tool with no number on any case gets one note under the bars (below), not a "not measured yet" bar per case
    unmeasured = {s["id"] for s in series if all(c["values"].get(s["id"]) is None for c in g["cases"])}
    rows = []
    for c in g["cases"]:
        d = delta(g, c)
        badge = f'<span class="delta {d[0]}">{esc(d[1])}</span>' if d else ""
        bars = []
        for s in series:
            v = c["values"].get(s["id"])
            flag = c.get("flags", {}).get(s["id"])
            if v is None:
                if s.get("status") == "not-measured" and s["id"] not in unmeasured:
                    bars.append(f'<div class="b against missing"><span class="who">{esc(s["label"])}</span>{ghost()}<span class="val muted">not measured yet</span></div>')
                continue
            flagtxt = {"capped": " (at the frame cap)", "noisy": " (noisy run)"}.get(flag, "")
            tip = f"{c['name']}, {s['label']}: {num(v)} {g['unit']}{flagtxt}"
            bars.append(
                f'<div class="b {s["role"]}" title="{esc(tip)}"><span class="who">{esc(s["label"])}</span>{bar(v / top * 100, colour[s["id"]], bool(flag))}'
                f'<span class="val">{num(v)} <small>{esc(g["unit"])}</small>{"<em>" + esc(flag) + "</em>" if flag else ""}</span></div>')
        rows.append(f'<div class="case"><div class="cname"><b>{esc(c["name"])}</b>{badge}</div>{"".join(bars)}</div>')
    missing = [s for s in series if s["role"] == "against" and s.get("status") == "not-measured"]
    notes = "".join(f'<li><b>{esc(s["label"])}:</b> {esc(s.get("note", "Not measured yet."))}</li>' for s in missing)
    flagged = any(c.get("flags") for c in g["cases"])
    legend = '<p class="key">Striped bars: the measurement sat at the frame-rate cap or came from a noisy run, so it is not a headline number.</p>' if flagged else ""
    hist = history_svg(g, colour)
    run = g["run"]
    meta = (f'<p class="meta">{esc(run["date"])} · commit {esc(run.get("commit") or "not recorded")} · '
            f'<a href="#method-{esc(g["id"])}">how it was measured</a></p>')
    method = (f'<details id="method-{esc(g["id"])}" class="how"><summary>How it was measured</summary>'
              f'<p>{esc(run["method"])}</p><p><b>Machine:</b> {esc(run["machine"])}</p>'
              + (f'<p><b>Source:</b> <a href="{esc(run["source"])}">{esc(run["source"])}</a></p>' if run.get("source", "").startswith("http")
                 else (f'<p><b>Source:</b> {esc(run["source"])}</p>' if run.get("source") else ""))
              + '</details>')
    return (f'<section class="bg" id="{esc(g["id"])}"><div class="bghead"><div><h2>{esc(g["title"])}</h2>'
            f'<p class="metric">{esc(g["metric"])}, in {esc(g["unit"])}</p></div>'
            f'<span class="better">{"lower" if g["better"] == "lower" else "higher"} is better</span></div>'
            f'{meta}<div class="rows">{"".join(rows)}</div>{legend}'
            + (f'<ul class="nm">{notes}</ul>' if notes else "") + hist + method + '</section>')


def history_svg(g: dict, colour: dict) -> str:
    raw = g.get("history", [])
    if len(raw) < 2:
        return ""
    # more than one run a day (the CI history has many): the day's median per case, so the line shows the trend, not the noise
    days: dict[str, list] = {}
    for h in raw:
        days.setdefault(h["date"], []).append(h)
    hist = []
    for d, hs in sorted(days.items()):
        vals = {}
        for h in hs:
            for case, sv in h["values"].items():
                for sid, v in sv.items():
                    vals.setdefault(case, {}).setdefault(sid, []).append(v)
        hist.append({"date": d, "runs": len(hs), "values": {c: {sid: sorted(v)[len(v) // 2] for sid, v in sv.items()} for c, sv in vals.items()}})
    if len(hist) < 2:
        return ""
    W, H, L, R, T, B = 720, 230, 46, 16, 14, 30
    # one line per case on the first ours series, drawn relative to the case's own median so cases of different size share the axis
    first = next(s for s in g["series"] if s["role"] == "ours")["id"]
    names = [c["name"] for c in g["cases"]]
    series = {}
    for n in names:
        pts = []
        for i, h in enumerate(hist):
            v = (h["values"].get(n) or {}).get(first)
            if v is not None:
                pts.append((i, v))
        if len(pts) >= 2:
            med = sorted(p[1] for p in pts)[len(pts) // 2]
            series[n] = [(i, v / med) for i, v in pts]
    if not series:
        return ""
    allv = [v for pts in series.values() for _, v in pts]
    lo, hi = max(min(allv), 0.5), min(max(allv), 2.0)
    lo, hi = min(lo, 0.8), max(hi, 1.2)
    n = len(hist)
    x = lambda i: L + (W - L - R) * (i / (n - 1))
    y = lambda v: T + (H - T - B) * (1 - (min(max(v, lo), hi) - lo) / (hi - lo))
    cols = ["#3DD6C6", "#6EA8FE", "#B79CFF", "#F59E3B", "#FF7A7A", "#4CC38A"]
    out = [f'<svg class="hist" viewBox="0 0 {W} {H}" role="img" aria-label="History of {esc(g["title"])}, each case relative to its own median">']
    for t in sorted({round(lo, 1), 1.0, round(hi, 1)}):
        out.append(f'<line x1="{L}" x2="{W - R}" y1="{y(t):.1f}" y2="{y(t):.1f}" class="gl{" mid" if t == 1.0 else ""}"/>'
                   f'<text x="{L - 8}" y="{y(t) + 4:.1f}" text-anchor="end" class="ax">{t:.1f}x</text>')
    for i in sorted({0, n // 2, n - 1}):
        out.append(f'<text x="{x(i):.1f}" y="{H - 8}" text-anchor="{"start" if i == 0 else "end" if i == n - 1 else "middle"}" class="ax">{esc(hist[i]["date"])}</text>')
    legend = []
    for k, (name, pts) in enumerate(series.items()):
        col = cols[k % len(cols)]
        path = " ".join(f'{"M" if j == 0 else "L"}{x(i):.1f} {y(v):.1f}' for j, (i, v) in enumerate(pts))
        out.append(f'<path d="{path}" fill="none" stroke="{col}" stroke-width="1.8" stroke-linejoin="round" opacity=".92"><title>{esc(name)}</title></path>')
        i, v = pts[-1]
        out.append(f'<circle cx="{x(i):.1f}" cy="{y(v):.1f}" r="3.2" fill="{col}"/>')
        legend.append(f'<span><i style="background:{col}"></i>{esc(name)}</span>')
    out.append("</svg>")
    return (f'<div class="histbox"><h3>History <small>{len(raw)} runs over {n} days, the day\'s median, each case relative to its own median</small></h3>{"".join(out)}'
            f'<div class="hkey">{"".join(legend)}</div></div>')


def pending_html(items: list[dict]) -> str:
    if not items:
        return ""
    lis = []
    for p in items:
        link = f' <a href="{esc(p["issue"])}">Tracked in {esc(p["issue"].split("github.com/")[-1])}</a>' if p.get("issue") else ""
        lis.append(f'<li><b>{esc(p["tool"])}</b>: {esc(p["what"])}.{link}</li>')
    return f'<section class="bg todo"><h2>Not measured yet</h2><p class="metric">These comparisons have no numbers. None are guessed.</p><ul>{"".join(lis)}</ul></section>'


def mini(g: dict) -> str:
    """Two or three bars of a project's first group, for the overview card."""
    if not g["cases"]:
        return ""
    c = g["cases"][-1] if len(g["cases"]) <= 3 else g["cases"][0]
    ours = [s for s in g["series"] if s["role"] == "ours"][0]
    others = [s for s in g["series"] if s["role"] == "against" and c["values"].get(s["id"]) is not None]
    use = [(ours, "a")] + [(s, "x") for s in others[:1]]
    top = max(c["values"][s["id"]] for s, _ in use if c["values"].get(s["id"]) is not None)
    rows = []
    for s, col in use:
        v = c["values"].get(s["id"])
        if v is None:
            continue
        rows.append(f'<div class="b"><span class="who">{esc(s["label"])}</span>{bar(v / top * 100, col, False)}<span class="val">{num(v)} <small>{esc(g["unit"])}</small></span></div>')
    return f'<div class="mini"><p class="metric">{esc(g["title"])}: {esc(c["name"])}</p>{"".join(rows)}</div>'


def build(out: Path, template: str, fill, records: list[dict]) -> None:
    dest = out / "bench"
    dest.mkdir(parents=True, exist_ok=True)
    cards = []
    for key, name, replaces, repo, blurb in PROJECTS:
        m = merged(records, key)
        groups, pending = m["groups"], m["pending"]
        measured = bool(groups)
        status, headline = '<span class="chip">no data yet</span>', f'<p class="big dim">Nothing measured</p><p class="sub">Would be compared with {esc(replaces)}.</p>'
        if measured:
            g0 = groups[0]
            line = compare_line(g0)
            if line:
                status = '<span class="chip ok">compared</span>'
                extra = "".join(f"<br>{esc(g['title'])}: {esc(l)}" for g in groups[1:] if (l := compare_line(g)))
                headline = f'<p class="big">{esc(line)}</p><p class="sub">{esc(g0["title"])}{extra}</p>'
            else:
                ours = next(s for s in g0["series"] if s["role"] == "ours")
                vals = [c["values"][ours["id"]] for c in g0["cases"] if c["values"].get(ours["id"]) is not None]
                rng = f"{num(min(vals))} to {num(max(vals))} {g0['unit']}" if vals and min(vals) != max(vals) else f"{num(vals[0])} {g0['unit']}"
                miss = [s["label"] for s in g0["series"] if s.get("status") == "not-measured"]
                tail = f" {' and '.join(miss)}: not measured yet." if miss else ""
                status = '<span class="chip">history</span>' if key == "toolbox" else '<span class="chip warn">no comparison yet</span>'
                headline = f'<p class="big">{esc(rng)}</p><p class="sub">{esc(ours["label"])}, {esc(g0["title"].lower())}, {len(vals)} cases.{esc(tail)}</p>'
        card = (f'<a class="bcard" href="{key}/"><div class="bchead"><h2>{esc(name)}</h2>{status}</div>'
                f'<p class="blurb">{esc(blurb)}</p>{headline}{mini(groups[0]) if groups else ""}</a>')
        cards.append(card)
        # project page
        body = [f'<p class="crumbs"><a href="../">Benchmarks</a> / {esc(name)}</p>',
                f'<h1>{esc(name)}</h1><p class="lead">{esc(blurb)} Compared with: <b>{esc(replaces)}</b>.</p>']
        if key == "ttwright-rs":
            body.append('<p class="lead sm">The Linux installer this project is a rewrite of states about two minutes on a Ryzen 7 9800X3D in its own changelog. '
                        'That is another program measured by someone else, so it is not drawn here.</p>')
        if key == "toolbox":
            body.append('<p class="lead sm">The original chart page that github-action-benchmark writes is kept at <a href="../raw/">/bench/raw/</a>; nothing was removed from it.</p>')
        if groups:
            body.append(f'<p class="jump">{" ".join(f"<a href=#{esc(g['id'])}>{esc(g['title'])}</a>" for g in groups)}</p>')
        body += [group_html(g, key) for g in groups]
        body.append(pending_html(pending))
        body.append(f'<p class="foot">Data: <a href="{OWNER}rauls-trusty-toolbox/blob/main/docs/wiki/benchmarks.md">how results are added</a> · '
                    f'<a href="{OWNER}{repo}">{esc(repo)} on GitHub</a></p>')
        (dest / key).mkdir(exist_ok=True)
        (dest / key / "index.html").write_text(fill(template, "bench", "../../", title=esc(name) + " benchmarks", body=DEFS + "".join(body)))
    intro = ('<h1>Benchmarks</h1><p class="lead">Each project is measured on its own and set beside the tool it replaces. '
             'Every bar is a number someone measured, with the machine and the method one click away. '
             'Where nobody has measured the other tool yet, the page says so.</p>')
    (dest / "index.html").write_text(fill(template, "bench", "../", title="Benchmarks", body=DEFS + intro + f'<div class="bcards">{"".join(cards)}</div>'
        '<p class="foot">Results are recorded runs on the maintainer\'s machines and, for raul bench, on shared CI runners: compare bars within one card, not across cards. '
        f'<a href="{OWNER}rauls-trusty-toolbox/blob/main/docs/wiki/benchmarks.md">How results are added</a>.</p>'))
