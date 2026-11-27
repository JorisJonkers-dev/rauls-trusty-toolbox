// SPDX-License-Identifier: GPL-3.0-only
// The guided tour: watches the demo (same origin, in the iframe) and ticks a step when its state shows the step was done.
// The demo keeps its state in the globals S (stored) and U (screen) and in localStorage; it is never changed.
"use strict";
(function () {
  const TOUR_KEY = "rauls-toolbox-tour-1";
  const DEMO_KEY = "rauls-toolbox-state-1";
  const frame = document.getElementById("demo");
  const ring = document.getElementById("ring");

  /* One read of the demo's state, as plain data. Runs inside the iframe, where S, U, J and P() are in scope. */
  const SNAP = `(() => {
    if (typeof S === "undefined" || !S || typeof U === "undefined") return null;
    const p = P();
    const sum = f => S.profiles.reduce((n, x) => n + f(x), 0);
    return {
      wiz: !!S.wiz.done, page: U.page, tab: U.tab, active: S.active, pid: p.id, menu: !!U.menu, dlg: U.dlg ? U.dlg.t : null,
      groups: p.groups.map(g => g.id), on: p.mods.filter(m => m.on).map(m => m.id).join(","),
      sorted: sum(x => x.events.filter(e => e.label === "Load order sorted").length),
      builtAt: Math.max(0, ...S.profiles.map(x => (x.far.built && x.far.built.t) || 0)),
      stab: sum(x => x.stab.length), zoom: U.zoom || 1, preview: !!U.sortPrev,
      lastCheck: S.g.lastCheck || 0, docRun: S.g.docRun || 0
    };
  })()`;

  /* Steps. done(s, b, m): s is the snapshot now, b the one taken when setup was finished, m scratch memory for steps
     that need a sequence. target(s): selectors to try in the demo, first visible one wins, with a label. */
  const nav = (page, label) => ({ sel: `.side [data-act="nav"][data-arg="${page}"]`, label });
  const onPage = (s, page, label) => (s.page === page ? null : nav(page, label || "Open this page"));
  const modsTab = (s, tab) => onPage(s, "mods", "Open Mods") || (s.tab === tab ? null : { sel: `[data-act="tab"][data-arg="${tab}"]`, label: "Switch to this view" });

  const STEPS = [
    {
      id: "setup", title: "Finish setup",
      text: "The demo starts with the setup wizard. Go through its eight steps with the main button at the bottom right. Each step takes a few seconds. You can skip Tale of Two Wastelands.",
      done: (s) => s.wiz,
      target: (s) => (s.wiz ? null : { sel: ".abar .btn.primary:not(:disabled), .abar .btn:not(:disabled):not([data-act='wizBack'])", label: "" }),
    },
    {
      id: "profile", title: "Switch profile",
      text: "A profile is a whole setup: mods, load order, settings and LOD. Open the profile menu at the top and pick another one.",
      done: (s, b) => s.wiz && s.active !== b.active,
      target: (s) => (s.menu ? { sel: `.menu [data-act="profSwitch"]:not([data-arg="${s.active}"])`, label: "Pick one" } : { sel: '[data-k="profMenu:top"]', label: "Profiles" }),
    },
    {
      id: "group", title: "Add a mod group",
      text: "Groups keep a long mod list tidy. On Mods, press New group, type a name and press Create group.",
      done: (s, b) => s.wiz && s.groups.some((g) => !b.groups.includes(g)),
      target: (s) => {
        if (s.dlg === "name") return { sel: '[data-act="nameOk"]', label: "Create group" };
        return modsTab(s, "list") || { sel: '[data-act="newGroup"]', label: "New group" };
      },
    },
    {
      id: "undo", title: "Turn a mod off, then undo",
      text: "On Mods, switch a mod off with its tick box. Then press Undo (bottom right) to bring it back.",
      done: (s, b, m) => m.undoDone === true,
      track: (s, b, m) => {
        if (!s.wiz) return;
        if (m.pid !== s.pid) { m.pid = s.pid; m.phase = 0; m.last = s.on; return; }
        const count = (x) => (x ? x.split(",").length : 0);
        if (m.phase !== 1 && s.on !== m.last && count(s.on) < count(m.last)) { m.phase = 1; m.orig = m.last; }
        else if (m.phase === 1 && s.on === m.orig) m.undoDone = true;
        m.last = s.on;
      },
      target: (s, b, m) => {
        if (m.phase === 1) return { sel: '[data-act="undo"]', label: "Undo" };
        return modsTab(s, "list") || { sel: 'button.cb[data-act="modToggle"][aria-checked="true"], [data-act="grpFold"]', label: "Turn off" };
      },
    },
    {
      id: "sort", title: "Sort the load order",
      text: "On Load order, press Sort… to see every move first, then Apply. The Hardcore profile starts out of order; the others may already be sorted.",
      done: (s, b) => s.wiz && s.sorted > b.sorted,
      target: (s) => onPage(s, "load", "Open Load order") || (s.preview ? { sel: '[data-act="sortApply"]:not(:disabled)', label: "Apply" } : { sel: '[data-act="sortGo"]', label: "Sort…" }),
    },
    {
      id: "lod", title: "Build LOD",
      text: "LOD is the far view. Open the LOD page and press Build LOD (or Rebuild). The demo takes about nine seconds.",
      done: (s, b) => s.wiz && s.builtAt > b.builtAt,
      target: (s) => onPage(s, "far", "Open LOD") || { sel: '[data-act="farBuild"]', label: "Build LOD" },
    },
    {
      id: "stability", title: "Run a stability test",
      text: "On Mods, open Test your setup and press Run stability test. It loads 40 places in the real app; the demo fakes it in nine seconds.",
      done: (s, b) => s.wiz && s.stab > b.stab,
      target: (s) => modsTab(s, "test") || { sel: '[data-act="stabGo"]', label: "Run stability test" },
    },
    {
      id: "pictures", title: "Compare location pictures, with zoom",
      text: "On Test your setup, scroll to the location pictures. They show the same spot before and after a change. Press a zoom button, or scroll the wheel over a picture.",
      done: (s, b, m) => m.zoomed === true,
      track: (s, b, m) => { if (s.wiz && s.page === "mods" && s.tab === "test" && s.zoom > 1) m.zoomed = true; },
      target: (s) => modsTab(s, "test") || { sel: '[data-act="zoom"][data-arg="4"]', label: "Zoom" },
    },
    {
      id: "updates", title: "Check for updates",
      text: "Open System. It checks for updates by itself; you can also press Check again.",
      done: (s, b) => s.wiz && s.lastCheck > b.lastCheck,
      target: (s) => onPage(s, "system", "Open System") || { sel: '[data-act="checkUpdates"]', label: "Check again" },
    },
    {
      id: "doctor", title: "Open Doctor",
      text: "Doctor runs every check on your setup in one go. Open it and press Run all checks.",
      done: (s, b) => s.wiz && s.docRun > b.docRun,
      target: (s) => onPage(s, "doctor", "Open Doctor") || { sel: '[data-act="docRun"]', label: "Run all checks" },
    },
  ];

  /* ---- stored progress ---- */
  let saved = { done: {}, base: null };
  try { const raw = localStorage.getItem(TOUR_KEY); if (raw) saved = Object.assign(saved, JSON.parse(raw)); } catch (e) { /* storage blocked: progress lasts until reload */ }
  const memory = {}; STEPS.forEach((st) => { memory[st.id] = {}; });
  const save = () => { try { localStorage.setItem(TOUR_KEY, JSON.stringify(saved)); } catch (e) { /* ignore */ } };

  /* ---- panel ---- */
  const list = document.getElementById("steps");
  let open = null, shown = null, resetTimer = 0;
  const items = STEPS.map((st, i) => {
    const li = document.createElement("li");
    li.className = "step"; li.dataset.step = st.id; li.dataset.done = "false";
    li.innerHTML = `<button type="button" class="head" aria-expanded="false"><span class="tick" aria-hidden="true"></span><span class="t"></span></button>
      <div class="body" hidden><p></p><button type="button" class="btn sm primary show">Show me</button></div>`;
    li.querySelector(".t").textContent = `${i + 1}. ${st.title}`;
    li.querySelector("p").textContent = st.text;
    li.querySelector(".head").addEventListener("click", () => { open = open === st.id ? null : st.id; render(); });
    li.querySelector(".show").addEventListener("click", () => { shown = st.id; open = st.id; render(); place(); });
    list.appendChild(li);
    return li;
  });

  function currentId() { const st = STEPS.find((x) => !saved.done[x.id]); return st ? st.id : null; }

  function render() {
    const cur = currentId();
    if (open === null && cur) open = cur;
    let n = 0;
    STEPS.forEach((st, i) => {
      const li = items[i], done = !!saved.done[st.id];
      if (done) n++;
      li.dataset.done = String(done);
      li.classList.toggle("cur", st.id === cur);
      li.querySelector(".tick").textContent = done ? "✓" : "";
      const head = li.querySelector(".head");
      head.setAttribute("aria-expanded", String(open === st.id));
      head.setAttribute("aria-label", `${st.title}${done ? ", done" : ""}`);
      li.querySelector(".body").hidden = open !== st.id;
    });
    document.getElementById("count").textContent = `${n} of ${STEPS.length}`;
    document.getElementById("barfill").style.width = `${(100 * n) / STEPS.length}%`;
    document.getElementById("finished").hidden = n < STEPS.length;
  }

  /* ---- watching the demo ---- */
  function read() {
    try {
      const w = frame.contentWindow;
      if (!w || !w.document || w.document.readyState === "loading") return null;
      return { w, snap: w.eval(SNAP) };
    } catch (e) { return null; }
  }

  function poll() {
    const r = read();
    if (!r || !r.snap) return;
    const s = r.snap;
    if (!s.wiz) saved.base = null; /* counters start from the end of setup, so the wizard's own work does not count */
    else if (!saved.base) { saved.base = { active: s.active, groups: s.groups, sorted: s.sorted, builtAt: s.builtAt, stab: s.stab, lastCheck: s.lastCheck, docRun: s.docRun }; save(); }
    const b = saved.base || { active: s.active, groups: s.groups, sorted: s.sorted, builtAt: s.builtAt, stab: s.stab, lastCheck: s.lastCheck, docRun: s.docRun };
    let changed = false;
    STEPS.forEach((st) => {
      if (st.track) st.track(s, b, memory[st.id]);
      if (!saved.done[st.id] && st.done(s, b, memory[st.id])) { saved.done[st.id] = true; changed = true; if (shown === st.id) shown = null; if (open === st.id) open = null; }
    });
    if (changed) { save(); render(); }
    place(r, s, b);
  }

  /* ---- highlight: a ring in this page over the demo's element ---- */
  let lastSel = null;
  function place(r, s, b) {
    if (!shown) { ring.style.display = "none"; lastSel = null; return; }
    r = r || read(); if (!r || !r.snap) return;
    s = s || r.snap; b = b || saved.base || s;
    const st = STEPS.find((x) => x.id === shown);
    const t = st && st.target(s, b, memory[st.id]);
    const doc = r.w.document;
    let el = null;
    if (t) for (const e of doc.querySelectorAll(t.sel)) { const bb = e.getBoundingClientRect(); if (bb.width > 0 && bb.height > 0) { el = e; break; } }
    if (!el) { ring.style.display = "none"; return; }
    if (t.sel !== lastSel) { lastSel = t.sel; try { el.scrollIntoView({ block: "center", inline: "nearest" }); } catch (e) { /* ignore */ } }
    const bb = el.getBoundingClientRect(), fb = frame.getBoundingClientRect(), sb = frame.parentElement.getBoundingClientRect();
    const x = fb.left - sb.left + bb.left - 4, y = fb.top - sb.top + bb.top - 4;
    ring.style.display = "block"; ring.style.left = `${x}px`; ring.style.top = `${y}px`; ring.style.width = `${bb.width + 8}px`; ring.style.height = `${bb.height + 8}px`;
    ring.dataset.target = t.label || (el.textContent || "").trim().slice(0, 30);
  }

  document.getElementById("hide").addEventListener("click", () => { shown = null; place(); });
  const resetBtn = document.getElementById("reset");
  resetBtn.addEventListener("click", () => {
    if (!resetTimer) { resetBtn.textContent = "Click again to reset"; resetTimer = setTimeout(() => { resetTimer = 0; resetBtn.textContent = "Reset"; }, 3000); return; }
    clearTimeout(resetTimer); resetTimer = 0; resetBtn.textContent = "Reset";
    saved = { done: {}, base: null }; STEPS.forEach((st) => { memory[st.id] = {}; });
    try { localStorage.removeItem(TOUR_KEY); localStorage.removeItem(DEMO_KEY); } catch (e) { /* ignore */ }
    shown = null; open = null; render(); place();
    try { frame.contentWindow.location.reload(); } catch (e) { frame.src = frame.src; }
  });

  render();
  setInterval(poll, 150);
  window.addEventListener("resize", () => place());
  window.__tour = { steps: STEPS.map((x) => x.id), snapshot: () => { const r = read(); return r && r.snap; } };
})();
