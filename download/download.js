// SPDX-License-Identifier: GPL-3.0-only
// The download page. It shows the latest release from the public GitHub API, and the data of the last site build
// (the JSON in the page) when the API cannot be reached. Text from the API is only ever set as text or as a checked
// https link; nothing from it goes through innerHTML.
"use strict";
(function () {
  var root = document.getElementById("dl");
  var API = "https://api.github.com/repos/JorisJonkers-dev/rauls-trusty-toolbox/releases";
  var built = JSON.parse(document.getElementById("rel-data").textContent);

  var OS = {
    linux: {
      name: "Linux",
      needs: ["64-bit Linux with glibc 2.35 or newer (Ubuntu 22.04, Debian 12, Fedora 36 or newer, Steam Deck)", "A GPU with Vulkan, and a few GB of free space"],
      steps: ["AppImage: make it runnable (chmod +x raul-*.AppImage) and start it.", "Debian or Ubuntu: sudo apt install ./raul-*.deb. It brings the libraries the window needs.", "The AppImage and the .tar.gz do not bring those libraries; the install guide lists them."]
    },
    windows: {
      name: "Windows",
      needs: ["Windows 10 22H2 or Windows 11, 64-bit", "A GPU with Vulkan or DirectX 12, and a few GB of free space"],
      steps: ["Run the setup program. It installs for your user only.", "It is not signed yet. If SmartScreen warns, choose More info, then Run anyway.", "Or unzip the portable archive anywhere and start raul.exe."]
    },
    macos: {
      name: "macOS",
      needs: ["macOS 13 or newer, on Apple silicon or Intel", "A few GB of free space"],
      steps: ["Open the disk image and drag raul to Applications.", "It is not signed yet. The first time, macOS asks you to allow it under System Settings, Privacy and Security."]
    }
  };
  var ORDER = ["linux", "windows", "macos"];

  function kind(name) {
    var n = name.toLowerCase(), os = null;
    if (/linux/.test(n)) os = "linux"; else if (/windows/.test(n)) os = "windows"; else if (/macos|darwin/.test(n)) os = "macos";
    if (n === "sha256sums") return { os: null, what: "Checksums of every file", extra: true };
    if (/\.cdx\.json$/.test(n)) return { os: null, what: "Software bill of materials (CycloneDX)", extra: true };
    if (/third-party/.test(n)) return { os: null, what: "Licences of the parts raul is made of", extra: true };
    if (!os) return { os: null, what: "Other file", extra: true };
    if (/\.appimage$/.test(n)) return { os: os, what: "Runs anywhere, no install (AppImage)", rank: 0 };
    if (/\.deb$/.test(n)) return { os: os, what: "Package for Debian and Ubuntu", rank: 1 };
    if (/setup\.exe$/.test(n)) return { os: os, what: "Installer", rank: 0 };
    if (/portable\.zip$/.test(n)) return { os: os, what: "Portable archive, no install", rank: 1 };
    if (/\.dmg$/.test(n)) return { os: os, what: "Disk image (installer)", rank: 0 };
    if (/\.tar\.gz$/.test(n)) return { os: os, what: "Archive", rank: 3 };
    return { os: os, what: "File", rank: 5 };
  }

  function detectOS() {
    var plat = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || "";
    var ua = navigator.userAgent || "";
    if (/android|iphone|ipad|ipod/i.test(ua)) return null;
    if (/win/i.test(plat) || /Windows/.test(ua)) return "windows";
    if (/mac/i.test(plat) || /Macintosh|Mac OS X/.test(ua)) return "macos";
    if (/linux|cros/i.test(plat) || /Linux|X11/.test(ua)) return "linux";
    return null;
  }

  function el(tag, attrs) {
    var e = document.createElement(tag);
    if (attrs) for (var k in attrs) { if (k === "class") e.className = attrs[k]; else e.setAttribute(k, attrs[k]); }
    for (var i = 2; i < arguments.length; i++) { var c = arguments[i]; if (c == null) continue; e.appendChild(typeof c === "string" ? document.createTextNode(c) : c); }
    return e;
  }
  function clear(n) { while (n.firstChild) n.removeChild(n.firstChild); return n; }
  function safe(url) { return typeof url === "string" && /^https:\/\/(github\.com|api\.github\.com)\//.test(url) ? url : "#"; }
  function size(b) { if (b < 1e6) return Math.max(1, Math.round(b / 1e3)) + " KB"; return b >= 1e9 ? (b / 1e9).toFixed(1) + " GB" : (b / 1e6).toFixed(b >= 1e8 ? 0 : 1) + " MB"; }
  function date(d) { return d ? new Date(d + (d.length === 10 ? "T00:00:00Z" : "")).toLocaleDateString(undefined, { year: "numeric", month: "long", day: "numeric", timeZone: "UTC" }) : ""; }
  function icon(path) {
    var ns = "http://www.w3.org/2000/svg", s = document.createElementNS(ns, "svg");
    var set = { "class": "i", width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", "stroke-width": "1.6", "stroke-linecap": "round", "stroke-linejoin": "round", "aria-hidden": "true" };
    for (var k in set) s.setAttribute(k, set[k]);
    var p = document.createElementNS(ns, "path"); p.setAttribute("d", path); s.appendChild(p); return s;
  }
  var DL = "M12 4v11M7 11l5 5 5-5M4.5 20h15", CHEV = "M9 5l7 7-7 7";

  /* The API's release and the build's release, as the same shape. */
  function fromApi(r) {
    var o = { tag: r.tag_name, name: r.name || r.tag_name, date: (r.published_at || "").slice(0, 10), url: r.html_url, prerelease: !!r.prerelease };
    if (r.body !== undefined) o.body = r.body || "";
    if (r.assets) o.assets = r.assets.map(function (a) { return { name: a.name, size: a.size, digest: a.digest || "", url: a.browser_download_url }; });
    return o;
  }

  /* SHA-256 of a file: the API's digest, else the release's own table of checksums. */
  function sums(rel) {
    var map = {}, m, re = /\|\s*`([^`|]+)`\s*\|\s*`([0-9a-f]{64})`\s*\|/g;
    while ((m = re.exec(rel.body || ""))) map[m[1]] = m[2];
    return function (a) { var d = /^sha256:([0-9a-f]{64})$/.exec(a.digest || ""); return d ? d[1] : map[a.name] || ""; };
  }

  function copyBtn(text) {
    var b = el("button", { type: "button", class: "btn sm ghost", title: "Copy the checksum" }, "Copy");
    b.addEventListener("click", function () {
      var done = function () { b.textContent = "Copied"; setTimeout(function () { b.textContent = "Copy"; }, 1500); };
      try { navigator.clipboard.writeText(text).then(done, function () {}); } catch (e) { /* no clipboard here */ }
    });
    return b;
  }

  function filesOf(rel, os) {
    return rel.assets.map(function (a) { return { a: a, k: kind(a.name) }; }).filter(function (x) { return x.k.os === os; })
      .sort(function (x, y) { return x.k.rank - y.k.rank; });
  }

  function fileRow(a, k, sha, primary) {
    var info = el("div", { class: "fi" },
      el("div", { class: "nm" }, a.name),
      el("div", { class: "sub" }, k.what + " · " + size(a.size)),
      sha ? el("div", { class: "sha" }, el("span", { class: "muted" }, "SHA-256"), el("code", { class: "mono" }, sha), copyBtn(sha)) : el("div", { class: "sub" }, "No checksum listed for this file.")
    );
    var btn = el("a", { class: "btn sm" + (primary ? " primary" : ""), href: safe(a.url), download: "" }, icon(DL), "Download");
    return el("div", { class: "r file" }, info, el("span", { class: "end" }, btn));
  }

  function osCard(id, rel, sha, mine) {
    var o = OS[id], files = filesOf(rel, id);
    var rows = el("div", { class: "rows" });
    if (!files.length) rows.appendChild(el("div", { class: "empty" }, "This release has no " + o.name + " file."));
    files.forEach(function (x, i) { rows.appendChild(fileRow(x.a, x.k, sha(x.a), i === 0 && mine)); });
    var steps = el("ol", { class: "steps" }); o.steps.forEach(function (s) { steps.appendChild(el("li", null, s)); });
    var needs = el("ul"); o.needs.forEach(function (s) { needs.appendChild(el("li", null, s)); });
    return el("section", { class: "card os", id: "os-" + id, "data-os": id },
      el("div", { class: "sech" }, el("h2", { class: "t" }, o.name), mine ? el("span", { class: "tag acc" }, "Your system") : null),
      rows,
      el("div", { class: "cols" }, el("div", null, el("h3", null, "Install"), steps), el("div", null, el("h3", null, "You need"), needs)));
  }

  function renderPrimary(rel, os) {
    var box = clear(document.getElementById("primary"));
    if (!os) {
      box.appendChild(el("div", { class: "pt" }, el("h2", null, "Raul's Toolbox " + rel.tag),
        el("p", { class: "muted" }, "Choose your system below. raul runs on Linux, Windows and macOS computers; there is no phone or tablet version.")));
      return;
    }
    var o = OS[os], best = filesOf(rel, os)[0];
    if (!best) { box.appendChild(el("p", null, "This release has no file for " + o.name + ". See the other systems below or the older releases.")); return; }
    box.appendChild(el("div", { class: "pt" },
      el("h2", null, "Download Raul's Toolbox for " + o.name),
      el("p", { class: "muted" }, rel.tag + " · released " + date(rel.date) + " · " + best.a.name + " · " + size(best.a.size)),
      el("p", { class: "muted small" }, best.k.what + ". The other files for " + o.name + " and the other systems are below.")));
    box.appendChild(el("a", { class: "btn primary big", id: "bigdl", href: safe(best.a.url), download: "" }, icon(DL), "Download for " + o.name));
  }

  /* A small Markdown reader that builds nodes: headings, lists, tables as text, links to https only. */
  function inline(text, parent) {
    var re = /\[([^\]]+)\]\((https:\/\/[^)\s]+)\)|`([^`]+)`|\*\*([^*]+)\*\*/g, last = 0, m;
    while ((m = re.exec(text))) {
      if (m.index > last) parent.appendChild(document.createTextNode(text.slice(last, m.index)));
      if (m[2]) parent.appendChild(el("a", { href: m[2], rel: "noopener" }, m[1]));
      else if (m[3]) parent.appendChild(el("code", null, m[3]));
      else parent.appendChild(el("strong", null, m[4]));
      last = re.lastIndex;
    }
    if (last < text.length) parent.appendChild(document.createTextNode(text.slice(last)));
  }
  function mdToDom(src, out) {
    var list = null, para = null, table = null;
    var flush = function () { list = null; para = null; if (table) { out.appendChild(el("pre", null, table.join("\n"))); table = null; } };
    src.split(/\r?\n/).forEach(function (line) {
      var h = /^#{1,6}\s+(.*)$/.exec(line), li = /^\s*[*-]\s+(.*)$/.exec(line);
      if (/^\s*\|/.test(line)) { if (!table) { list = null; para = null; table = []; } table.push(line); return; }
      if (table) flush();
      if (h) { flush(); var hh = el("h4"); inline(h[1], hh); out.appendChild(hh); }
      else if (li) { if (!list) { list = el("ul"); out.appendChild(list); } var item = el("li"); inline(li[1], item); list.appendChild(item); para = null; }
      else if (!line.trim()) { list = null; para = null; }
      else { if (!para) { para = el("p"); out.appendChild(para); list = null; } else para.appendChild(document.createTextNode(" ")); inline(line, para); }
    });
    flush();
  }

  function renderNotes(rel) {
    var box = clear(document.getElementById("notes"));
    document.getElementById("notes-ver").textContent = rel.tag;
    mdToDom(rel.body || "", box);
    if (!box.firstChild) box.appendChild(el("p", { class: "muted" }, "This release has no notes."));
  }

  function renderOlder(releases, latestTag) {
    var rows = clear(document.getElementById("older-rows"));
    var old = releases.filter(function (r) { return r.tag !== latestTag; });
    if (!old.length) { rows.appendChild(el("div", { class: "empty" }, "There are no older releases.")); return; }
    old.forEach(function (r) {
      rows.appendChild(el("a", { class: "r click", href: safe(r.url) }, el("span", { class: "nm" }, r.name || r.tag),
        r.prerelease ? el("span", { class: "tag warn" }, "pre-release") : null, el("span", { class: "sub" }, date(r.date)), el("span", { class: "end" }, icon(CHEV))));
    });
  }

  function render(releases, rel, live) {
    var os = detectOS(), sha = sums(rel);
    document.getElementById("ver").textContent = rel.tag;
    document.getElementById("status").textContent = live
      ? "Latest release, read from GitHub just now."
      : "GitHub could not be reached (offline or rate limited), so this is the latest release as of " + date(built.fetched) + ". The links still work.";
    renderPrimary(rel, os);
    var box = clear(document.getElementById("oses"));
    var order = ORDER.slice();
    if (os) { order.splice(order.indexOf(os), 1); order.unshift(os); }
    order.forEach(function (id) { box.appendChild(osCard(id, rel, sha, id === os)); });
    var extras = clear(document.getElementById("extras"));
    rel.assets.map(function (a) { return { a: a, k: kind(a.name) }; }).filter(function (x) { return x.k.extra; }).forEach(function (x) {
      extras.appendChild(el("a", { class: "r click", href: safe(x.a.url) }, el("span", { class: "nm" }, x.a.name), el("span", { class: "sub" }, x.k.what + " · " + size(x.a.size)), el("span", { class: "end" }, icon(DL))));
    });
    renderNotes(rel);
    renderOlder(releases, rel.tag);
    root.setAttribute("data-source", live ? "api" : "build");
    root.setAttribute("data-ready", "true");
  }

  /* first the build's data, so the page is whole at once; then the live data replaces it */
  var b0 = built.releases;
  render(b0, b0[0], false);

  function get(url, ms) {
    var ctl = typeof AbortController === "function" ? new AbortController() : null;
    var t = setTimeout(function () { if (ctl) ctl.abort(); }, ms);
    return fetch(url, { headers: { Accept: "application/vnd.github+json" }, signal: ctl ? ctl.signal : undefined }).then(function (r) {
      clearTimeout(t);
      if (!r.ok) throw new Error("HTTP " + r.status);
      return r.json();
    });
  }
  Promise.all([get(API + "/latest", 8000), get(API + "?per_page=30", 8000)]).then(function (res) {
    var latest = fromApi(res[0]);
    var list = res[1].filter(function (r) { return !r.draft; }).map(fromApi);
    if (!latest.assets || !latest.assets.length) throw new Error("no assets");
    render(list.length ? list : [latest], latest, true);
  }).catch(function () { root.setAttribute("data-source", "build"); });
})();
