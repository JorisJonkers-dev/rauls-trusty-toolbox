# SPDX-License-Identifier: GPL-3.0-only
"""The figures of the benchmark pages, drawn as static SVG: grouped vertical bars, line charts and their minis.

Every figure is complete without script: axes, labels and values are in the SVG, each mark carries a <title>, and
bench.js only adds a tooltip on hover and focus. Colours follow the app (docs/design/demo/src/style.css): teal for our
own numbers, blue and violet for our variants, orange for the tool we replace, rose for a second one.
"""
import datetime
import html
import math

# (class, colour): fixed order per role, never cycled past the end
OURS = [("teal", "#3DD6C6"), ("blue", "#6EA8FE"), ("violet", "#B79CFF")]
AGAINST = [("orange", "#F59E3B"), ("rose", "#FF7A7A")]
ALL = OURS + AGAINST


def esc(s) -> str:
    return html.escape(str(s), quote=True)


def num(v: float) -> str:
    if v >= 1000:
        return f"{v:,.0f}"
    if v >= 100:
        return f"{v:.0f}"
    if v >= 10:
        return f"{v:.1f}".rstrip("0").rstrip(".")
    return f"{v:.2f}".rstrip("0").rstrip(".")


def tick(v: float) -> str:
    if v == 0:
        return "0"
    if abs(v) >= 1000:
        return f"{v:,.0f}"
    return f"{v:g}"


def defs() -> str:
    """Gradients and the hatch the figures share; once per page."""
    grads = "".join(
        f'<linearGradient id="bxg-{k}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{c}"/>'
        f'<stop offset="1" stop-color="{c}" stop-opacity=".55"/></linearGradient>'
        f'<linearGradient id="bxa-{k}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{c}" stop-opacity=".22"/>'
        f'<stop offset="1" stop-color="{c}" stop-opacity="0"/></linearGradient>'
        for k, c in ALL)
    return ('<svg width="0" height="0" style="position:absolute" aria-hidden="true" focusable="false"><defs>' + grads
            + '<pattern id="bx-hatch" width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">'
              '<rect width="2.2" height="6" fill="#0B0D10" fill-opacity=".55"/></pattern></defs></svg>')


def colours(series: list[dict]) -> dict[str, tuple[str, str]]:
    out, ni, na = {}, 0, 0
    for s in series:
        if s["role"] == "ours":
            out[s["id"]] = OURS[min(ni, len(OURS) - 1)]; ni += 1
        else:
            out[s["id"]] = AGAINST[min(na, len(AGAINST) - 1)]; na += 1
    return out


def nice_step(span: float, n: int) -> float:
    raw = span / max(n, 1)
    mag = 10 ** math.floor(math.log10(raw)) if raw > 0 else 1
    for m in (1, 2, 2.5, 5, 10):
        if m * mag >= raw:
            return m * mag
    return 10 * mag


def ticks_from_zero(hi: float, n: int = 5) -> list[float]:
    hi = hi if hi > 0 else 1
    step = nice_step(hi, n)
    top = math.ceil(hi / step - 1e-9) * step
    return [round(i * step, 10) for i in range(int(round(top / step)) + 1)]


def ticks_range(lo: float, hi: float, n: int = 4) -> list[float]:
    if hi <= lo:
        lo, hi = lo * 0.9, (hi * 1.1 if hi else 1)
    step = nice_step(hi - lo, n)
    a, b = math.floor(lo / step) * step, math.ceil(hi / step) * step
    return [round(a + i * step, 10) for i in range(int(round((b - a) / step)) + 1)]


def wrap(text: str, width: int, lines: int = 3) -> list[str]:
    words, out, cur = text.split(), [], ""
    for w in words:
        if cur and len(cur) + 1 + len(w) > width:
            out.append(cur); cur = w
        else:
            cur = f"{cur} {w}".strip()
    if cur:
        out.append(cur)
    if len(out) > lines:
        out = out[:lines]
        out[-1] = out[-1][: max(width - 1, 1)].rstrip() + "…"
    return [o if len(o) <= width + 2 else o[: width] + "…" for o in out]


def bar_path(x: float, y: float, w: float, h: float, r: float = 4) -> str:
    """A column with a rounded top and a square foot on the baseline."""
    r = min(r, h, w / 2)
    if h <= 0:
        return f"M{x:.1f} {y:.1f}h{w:.1f}"
    return (f"M{x:.1f} {y + h:.1f}V{y + r:.1f}Q{x:.1f} {y:.1f} {x + r:.1f} {y:.1f}H{x + w - r:.1f}"
            f"Q{x + w:.1f} {y:.1f} {x + w:.1f} {y + r:.1f}V{y + h:.1f}Z")


def tip_attrs(title: str, value: str, meta: str) -> str:
    return f' data-tip="{esc(title)}" data-tip-v="{esc(value)}" data-tip-m="{esc(meta)}"'


FLAG_TEXT = {"capped": "at the frame cap, not a headline number", "noisy": "noisy run, not a headline number"}


def grouped_bars(g: dict, meta: str, issue: str | None, target_w: int, annotate) -> str:
    """The main figure: one group of vertical bars per case, every series side by side, the ratio over each group.

    annotate(case) returns (kind, text) or None for the pill over a group."""
    series = g["series"]
    col = colours(series)
    unit = g["unit"]
    cases = g["cases"]
    nm = {s["id"] for s in series if s.get("status") == "not-measured" and all(c["values"].get(s["id"]) is None for c in cases)}
    slots = [[s for s in series if c["values"].get(s["id"]) is not None or s["id"] in nm] for c in cases]
    nslot = max((len(s) for s in slots), default=1) or 1
    L, R, T, B = 58, 14, 46, 30
    PH = 250
    bw = 26 if nslot <= 4 else 22
    gap = 6
    inner = nslot * bw + (nslot - 1) * gap
    gw = max(inner + 40, 118, (target_w - L - R) / max(len(cases), 1))
    W = int(L + R + gw * len(cases))
    chars = max(int((gw - 10) / 6.6), 8)
    B += 15 * max((len(wrap(c["name"], chars)) for c in cases), default=1)
    H = T + PH + B
    top_v = max([v for c in cases for v in c["values"].values() if v is not None] or [1])
    ticks = ticks_from_zero(top_v * 1.08)
    ymax = ticks[-1]
    y = lambda v: T + PH * (1 - v / ymax)
    out = [f'<svg class="bx-svg bx-bars" viewBox="0 0 {W} {H}" width="{W}" height="{H}" style="--w:{W}px" role="group" '
           f'aria-label="{esc(g["title"])}: {esc(g["metric"])} in {esc(unit)}, {"lower" if g["better"] == "lower" else "higher"} is better">']
    # y axis: gridlines, ticks, unit
    out.append('<g class="bx-yaxis" aria-hidden="true">')
    for t in ticks:
        yy = y(t)
        out.append(f'<line class="bx-gl{" bx-base" if t == 0 else ""}" x1="{L}" x2="{W - R}" y1="{yy:.1f}" y2="{yy:.1f}"/>'
                   f'<text class="bx-tick" x="{L - 10}" y="{yy + 4:.1f}" text-anchor="end">{tick(t)}</text>')
    out.append(f'<text class="bx-unit" x="{L - 10}" y="{T - 12}" text-anchor="end">{esc(unit)}</text></g>')
    # bars, a group per case
    k = 0
    xaxis = ['<g class="bx-xaxis" aria-hidden="true">']
    for ci, c in enumerate(cases):
        gx = L + ci * gw
        cx = gx + gw / 2
        row = slots[ci]
        width = len(row) * bw + (len(row) - 1) * gap
        x0 = cx - width / 2
        if ci:
            out.append(f'<line class="bx-sep" x1="{gx:.1f}" x2="{gx:.1f}" y1="{T + 8}" y2="{T + PH}"/>')
        out.append(f'<g class="bx-group" role="list" aria-label="{esc(c["name"])}">')
        for si, s in enumerate(row):
            bx = x0 + si * (bw + gap)
            v = c["values"].get(s["id"])
            cls, colour = col[s["id"]]
            if v is None:
                h = PH * 0.62
                body = (f'<rect class="bx-slot" x="{bx + .75:.1f}" y="{T + PH - h:.1f}" width="{bw - 1.5:.1f}" height="{h:.1f}" rx="4"/>'
                        f'<text class="bx-slott" transform="translate({bx + bw / 2 + 4:.1f} {T + PH - 10:.1f}) rotate(-90)">not measured</text>')
                label = f'{c["name"]}, {s["label"]}: not measured yet'
                tip = tip_attrs(f'{c["name"]} · {s["label"]}', "not measured yet", "Open the issue that tracks it" if issue else s.get("note", ""))
                inner_g = (f'<g class="bx-bar bx-nmbar" role="listitem" tabindex="0" aria-label="{esc(label)}"{tip}>'
                           f'<title>{esc(label)}</title><rect class="bx-hit" x="{bx - gap / 2:.1f}" y="{T}" width="{bw + gap:.1f}" height="{PH}"/>{body}</g>')
                out.append(f'<a href="{esc(issue)}" class="bx-nmlink">{inner_g}</a>' if issue else inner_g)
                continue
            flag = (c.get("flags") or {}).get(s["id"])
            yy = y(v)
            h = T + PH - yy
            vtxt = f"{num(v)} {unit}"
            label = f'{c["name"]}, {s["label"]}: {vtxt}' + (f" ({FLAG_TEXT.get(flag, flag)})" if flag else "")
            tip = tip_attrs(f'{c["name"]} · {s["label"]}', vtxt + (" · " + FLAG_TEXT.get(flag, flag) if flag else ""), meta)
            hatch = f'<path class="bx-hatch" d="{bar_path(bx, yy, bw, h)}" fill="url(#bx-hatch)"/>' if flag else ""
            out.append(f'<g class="bx-bar bx-{cls}" role="listitem" tabindex="0" style="--i:{k}" aria-label="{esc(label)}"{tip}>'
                       f'<title>{esc(label)}</title><rect class="bx-hit" x="{bx - gap / 2:.1f}" y="{T}" width="{bw + gap:.1f}" height="{PH}"/>'
                       f'<path class="bx-m" d="{bar_path(bx, yy, bw, h)}" fill="url(#bxg-{cls})"/>{hatch}'
                       f'<text class="bx-val" x="{bx + bw / 2:.1f}" y="{yy - 7:.1f}" text-anchor="middle">{num(v)}</text></g>')
            k += 1
        out.append("</g>")
        ann = annotate(c)
        if ann:
            kind, text = ann
            pw = len(text) * 6.7 + 22
            pw = min(pw, gw - 6)
            out.append(f'<g class="bx-ann bx-{kind}"><rect x="{cx - pw / 2:.1f}" y="6" width="{pw:.1f}" height="24" rx="12"/>'
                       f'<text x="{cx:.1f}" y="22" text-anchor="middle">{esc(text)}</text></g>')
        for li, line in enumerate(wrap(c["name"], chars)):
            xaxis.append(f'<text class="bx-case" x="{cx:.1f}" y="{T + PH + 22 + li * 15}" text-anchor="middle">{esc(line)}</text>')
    xaxis.append("</g>")
    out += xaxis
    out.append("</svg>")
    return "".join(out)


def legend(g: dict, issue: str | None, extra: str = "") -> str:
    col = colours(g["series"])
    items = []
    for s in g["series"]:
        cls, _ = col[s["id"]]
        nm = s.get("status") == "not-measured" and all(c["values"].get(s["id"]) is None for c in g["cases"])
        role = "ours" if s["role"] == "ours" else "replaced tool"
        if nm:
            items.append(f'<li class="bx-li bx-linm"><i class="bx-sw bx-swnm"></i>{esc(s["label"])} <em>not measured</em></li>')
        else:
            items.append(f'<li class="bx-li"><i class="bx-sw bx-sw-{cls}"></i>{esc(s["label"])}<span class="bx-role">{role}</span></li>')
    if any(c.get("flags") for c in g["cases"]):
        items.append('<li class="bx-li"><i class="bx-sw bx-swhatch"></i>striped: at the frame cap, not a headline number</li>')
    if extra:
        items.append(f'<li class="bx-li"><i class="bx-sw bx-swring"></i>{esc(extra)}</li>')
    return f'<ul class="bx-legend" aria-label="Legend">{"".join(items)}</ul>'


def mini_bars(g: dict, issue: str | None, W: int = 360, H: int = 150) -> str:
    """The overview card's figure: the first own series against the first other tool, per case, with values and labels."""
    series = g["series"]
    col = colours(series)
    ours = next(s for s in series if s["role"] == "ours")
    other = next((s for s in series if s["role"] == "against"), None)
    cases = g["cases"][:6]
    use = [ours] + ([other] if other else [])
    if other and len(cases) > 4:
        use = [ours, other]
    L, R, T, B = 6, 6, 18, 30
    PH = H - T - B
    gw = (W - L - R) / max(len(cases), 1)
    bw = min(22, (gw - 18) / len(use) - 4)
    top_v = max([c["values"].get(s["id"]) or 0 for c in cases for s in use] or [1]) or 1
    out = [f'<svg class="bx-svg bx-mini" viewBox="0 0 {W} {H}" role="img" aria-label="{esc(g["title"])} in {esc(g["unit"])}">']
    for f in (0.5, 1.0):
        out.append(f'<line class="bx-gl" x1="{L}" x2="{W - R}" y1="{T + PH * (1 - f):.1f}" y2="{T + PH * (1 - f):.1f}"/>')
    out.append(f'<line class="bx-gl bx-base" x1="{L}" x2="{W - R}" y1="{T + PH}" y2="{T + PH}"/>')
    k = 0
    for ci, c in enumerate(cases):
        cx = L + gw * ci + gw / 2
        width = len(use) * bw + (len(use) - 1) * 4
        for si, s in enumerate(use):
            bx = cx - width / 2 + si * (bw + 4)
            v = c["values"].get(s["id"])
            cls, _ = col[s["id"]]
            if v is None:
                if s.get("status") == "not-measured":
                    h = PH * 0.6
                    out.append(f'<rect class="bx-slot" x="{bx:.1f}" y="{T + PH - h:.1f}" width="{bw:.1f}" height="{h:.1f}" rx="3"><title>{esc(c["name"])}, {esc(s["label"])}: not measured yet</title></rect>')
                continue
            h = max(PH * v / top_v, 1.5)
            yy = T + PH - h
            out.append(f'<g class="bx-bar bx-{cls}" style="--i:{k}"><title>{esc(c["name"])}, {esc(s["label"])}: {num(v)} {esc(g["unit"])}</title>'
                       f'<path class="bx-m" d="{bar_path(bx, yy, bw, h, 3)}" fill="url(#bxg-{cls})"/>'
                       f'<text class="bx-val bx-sm" x="{bx + bw / 2:.1f}" y="{yy - 5:.1f}" text-anchor="middle">{num(v)}</text></g>')
            k += 1
        name = c["name"]
        maxc = max(int(gw / 6.2), 5)
        out.append(f'<text class="bx-case bx-sm" x="{cx:.1f}" y="{H - 10}" text-anchor="middle">{esc(name if len(name) <= maxc else name[:maxc - 1] + "…")}</text>')
    out.append("</svg>")
    return "".join(out)


def mini_empty(W: int = 360, H: int = 150, label: str = "not measured yet") -> str:
    """A card's figure before anything is measured: dashed slots where the bars will be."""
    out = [f'<svg class="bx-svg bx-mini" viewBox="0 0 {W} {H}" role="img" aria-label="Nothing measured yet">']
    T, PH = 18, H - 48
    out.append(f'<line class="bx-gl bx-base" x1="6" x2="{W - 6}" y1="{T + PH}" y2="{T + PH}"/>')
    for i in range(3):
        cx = 6 + (W - 12) / 3 * (i + .5)
        for j, f in enumerate((.55, .8)):
            out.append(f'<rect class="bx-slot" x="{cx - 26 + j * 30:.1f}" y="{T + PH * (1 - f):.1f}" width="22" height="{PH * f:.1f}" rx="3"/>')
    out.append(f'<text class="bx-emptyt" x="{W / 2}" y="{H - 10}" text-anchor="middle">{esc(label)}</text></svg>')
    return "".join(out)


# ---------- lines ----------

def _d(s: str) -> datetime.date:
    return datetime.date.fromisoformat(s)


def day_label(d: datetime.date) -> str:
    return d.strftime("%b ") + str(d.day)


def line_chart(name: str, pts: list[dict], unit: str, better: str, start: datetime.date, end: datetime.date,
               cls: str = "teal", W: int = 380, H: int = 176, meta: str = "") -> str:
    """One case over time: the day's median per point, a dated x axis, its own y axis in the case's unit."""
    L, R, T, B = 50, 14, 14, 28
    PW, PH = W - L - R, H - T - B
    pts = [p for p in pts if start <= _d(p["date"]) <= end]
    span = max((end - start).days, 1)
    x = lambda d: L + PW * ((_d(d) - start).days / span) if isinstance(d, str) else L + PW * ((d - start).days / span)
    vals = [p["v"] for p in pts] or [0, 1]
    ticks = ticks_range(min(vals), max(vals), 3)
    lo, hi = ticks[0], ticks[-1]
    if hi == lo:
        hi = lo + 1
    y = lambda v: T + PH * (1 - (v - lo) / (hi - lo))
    out = [f'<svg class="bx-svg bx-line" viewBox="0 0 {W} {H}" role="group" aria-label="{esc(name)} over time, daily median in {esc(unit)}">']
    out.append('<g class="bx-yaxis" aria-hidden="true">')
    for t in ticks:
        out.append(f'<line class="bx-gl" x1="{L}" x2="{W - R}" y1="{y(t):.1f}" y2="{y(t):.1f}"/>'
                   f'<text class="bx-tick" x="{L - 8}" y="{y(t) + 4:.1f}" text-anchor="end">{tick(t)}</text>')
    out.append("</g>")
    # dated x axis: up to five evenly spaced days
    n = min(5, span + 1)
    days = sorted({start + datetime.timedelta(days=round(span * i / max(n - 1, 1))) for i in range(n)})
    out.append(f'<g class="bx-xaxis" aria-hidden="true"><line class="bx-gl bx-base" x1="{L}" x2="{W - R}" y1="{T + PH}" y2="{T + PH}"/>')
    for i, d in enumerate(days):
        anchor = "start" if i == 0 else "end" if i == len(days) - 1 else "middle"
        out.append(f'<line class="bx-xt" x1="{x(d):.1f}" x2="{x(d):.1f}" y1="{T + PH}" y2="{T + PH + 4}"/>'
                   f'<text class="bx-tick" x="{x(d):.1f}" y="{H - 8}" text-anchor="{anchor}">{day_label(d)}</text>')
    out.append(f'<text class="bx-unit" x="{L - 8}" y="{H - 8}" text-anchor="end">{esc(unit)}</text></g>')
    if pts:
        line = " ".join(f'{"M" if i == 0 else "L"}{x(p["date"]):.1f} {y(p["v"]):.1f}' for i, p in enumerate(pts))
        area = line + f" L{x(pts[-1]['date']):.1f} {T + PH} L{x(pts[0]['date']):.1f} {T + PH} Z"
        out.append(f'<path class="bx-area" d="{area}" fill="url(#bxa-{cls})"/>'
                   f'<path class="bx-ln bx-{cls}" d="{line}" pathLength="1"/>')
        for i, p in enumerate(pts):
            last = i == len(pts) - 1
            label = f'{name}, {p["date"]}: median {num(p["v"])} {unit} of {p["runs"]} run{"s" if p["runs"] != 1 else ""}'
            tip = tip_attrs(f'{name} · {p["date"]}', f'{num(p["v"])} {unit}',
                            f'median of {p["runs"]} run{"s" if p["runs"] != 1 else ""}' + (f' · {p["commits"]}' if p.get("commits") else "") + (f" · {meta}" if meta else ""))
            out.append(f'<g class="bx-pt bx-{cls}{" bx-last" if last else ""}" tabindex="0" role="img" aria-label="{esc(label)}"{tip}><title>{esc(label)}</title>'
                       f'<circle class="bx-hit" cx="{x(p["date"]):.1f}" cy="{y(p["v"]):.1f}" r="12"/>'
                       f'<circle class="bx-dot" cx="{x(p["date"]):.1f}" cy="{y(p["v"]):.1f}" r="{4.5 if last else 3.4}"/></g>')
    else:
        out.append(f'<text class="bx-emptyt" x="{L + PW / 2}" y="{T + PH / 2}" text-anchor="middle">no runs in this range</text>')
    out.append("</svg>")
    return "".join(out)


def sparkline(pts: list[dict], cls: str = "teal", W: int = 360, H: int = 150, label: str = "", unit: str = "") -> str:
    L, R, T, B = 8, 8, 16, 30
    PW, PH = W - L - R, H - T - B
    if len(pts) < 2:
        return mini_empty(W, H, "one run so far")
    vals = [p["v"] for p in pts]
    lo, hi = min(vals), max(vals)
    pad = (hi - lo) * .15 or hi * .1 or 1
    lo, hi = lo - pad, hi + pad
    n = len(pts)
    x = lambda i: L + PW * i / (n - 1)
    y = lambda v: T + PH * (1 - (v - lo) / (hi - lo))
    line = " ".join(f'{"M" if i == 0 else "L"}{x(i):.1f} {y(p["v"]):.1f}' for i, p in enumerate(pts))
    area = line + f" L{x(n - 1):.1f} {T + PH} L{x(0):.1f} {T + PH} Z"
    last = pts[-1]
    out = [f'<svg class="bx-svg bx-mini" viewBox="0 0 {W} {H}" role="img" aria-label="{esc(label)}: daily median over {n} days">',
           f'<line class="bx-gl" x1="{L}" x2="{W - R}" y1="{T + PH / 2:.1f}" y2="{T + PH / 2:.1f}"/>',
           f'<line class="bx-gl bx-base" x1="{L}" x2="{W - R}" y1="{T + PH}" y2="{T + PH}"/>',
           f'<path class="bx-area" d="{area}" fill="url(#bxa-{cls})"/><path class="bx-ln bx-{cls}" d="{line}" pathLength="1"/>',
           f'<circle class="bx-dot bx-{cls}" cx="{x(n - 1):.1f}" cy="{y(last["v"]):.1f}" r="4.5"><title>{esc(label)}, {last["date"]}: {num(last["v"])} {esc(unit)}</title></circle>',
           f'<text class="bx-val bx-sm" x="{x(n - 1) - 8:.1f}" y="{y(last["v"]) - 9:.1f}" text-anchor="end">{num(last["v"])} {esc(unit)}</text>',
           f'<text class="bx-case bx-sm" x="{L}" y="{H - 10}">{day_label(_d(pts[0]["date"]))}</text>',
           f'<text class="bx-case bx-sm" x="{W - R}" y="{H - 10}" text-anchor="end">{day_label(_d(last["date"]))}</text></svg>']
    return "".join(out)
