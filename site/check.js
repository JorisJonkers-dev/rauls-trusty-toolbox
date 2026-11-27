// SPDX-License-Identifier: GPL-3.0-only
// Checks a built site: every internal link and anchor resolves, no page logs a console error, and every step of the
// guided tour ticks when it is done in the demo.
//   node site/check.js <built site folder> [screenshot folder]
// Needs the `playwright` (or `playwright-core`) package. PW_CHANNEL=chrome uses an installed Chrome instead of
// Playwright's own browser. The site is served under /rauls-trusty-toolbox/, like GitHub Pages serves it.
"use strict";
const fs = require("fs");
const http = require("http");
const path = require("path");

const pw = (() => { try { return require("playwright"); } catch (e) { return require("playwright-core"); } })();
const root = path.resolve(process.argv[2] || "");
const shots = process.argv[3] ? path.resolve(process.argv[3]) : null;
if (!process.argv[2] || !fs.existsSync(root)) { console.error("usage: node site/check.js <built site folder> [screenshot folder]"); process.exit(2); }
if (shots) fs.mkdirSync(shots, { recursive: true });
const BASE = "/rauls-trusty-toolbox/";
/* The raw chart page lives on the gh-pages branch, written by the bench workflow, so it is not in the built folder. */
const OUTSIDE = ["bench/raw/"];
const problems = [];
const bad = (m) => { problems.push(m); console.log("FAIL", m); };
const ok = (c, m) => { if (!c) bad(m); };

/* ---- static server ---- */
const TYPES = { ".html": "text/html; charset=utf-8", ".css": "text/css", ".js": "text/javascript", ".png": "image/png", ".jpg": "image/jpeg", ".svg": "image/svg+xml" };
function resolveFile(urlPath) {
  if (!urlPath.startsWith(BASE)) return null;
  let f = path.join(root, decodeURIComponent(urlPath.slice(BASE.length)));
  if (!f.startsWith(root)) return null;
  if (fs.existsSync(f) && fs.statSync(f).isDirectory()) f = path.join(f, "index.html");
  return fs.existsSync(f) && fs.statSync(f).isFile() ? f : null;
}
const server = http.createServer((req, res) => {
  const f = resolveFile(new URL(req.url, "http://x").pathname);
  if (!f) { res.writeHead(404); res.end("not found"); return; }
  res.writeHead(200, { "content-type": TYPES[path.extname(f)] || "application/octet-stream" });
  fs.createReadStream(f).pipe(res);
});

/* ---- links ---- */
function allHtml(dir, out = []) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) allHtml(p, out); else if (e.name.endsWith(".html")) out.push(p);
  }
  return out;
}
function checkLinks() {
  let n = 0;
  const idsOf = (file) => new Set([...fs.readFileSync(file, "utf8").matchAll(/\sid="([^"]+)"/g)].map((m) => m[1]));
  for (const file of allHtml(root)) {
    if (file === path.join(root, "demo", "index.html")) continue; /* the demo is not ours to rewrite; it has no page links */
    const html = fs.readFileSync(file, "utf8");
    const pageUrl = new URL(path.relative(root, file).replace(/index\.html$/, ""), "http://x" + BASE);
    for (const m of html.matchAll(/\s(?:href|src)="([^"]*)"/g)) {
      const raw = m[1];
      if (/^[a-z][a-z0-9+.-]*:/i.test(raw) || raw === "") continue;
      const u = new URL(raw, pageUrl);
      n++;
      const rel = u.pathname.slice(BASE.length);
      if (OUTSIDE.some((o) => rel === o || rel.startsWith(o))) continue;
      const target = resolveFile(u.pathname);
      if (!target) { bad(`${path.relative(root, file)}: link ${raw} does not resolve`); continue; }
      if (u.hash && target.endsWith(".html") && !idsOf(target).has(decodeURIComponent(u.hash.slice(1)))) bad(`${path.relative(root, file)}: anchor ${raw} not found`);
    }
  }
  console.log(`links checked: ${n}`);
}

/* ---- browser ---- */
async function main() {
  checkLinks();
  await new Promise((r) => server.listen(0, "127.0.0.1", r));
  const origin = `http://127.0.0.1:${server.address().port}${BASE}`;
  const browser = await pw.chromium.launch(process.env.PW_CHANNEL ? { channel: process.env.PW_CHANNEL } : {});
  const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  const page = await ctx.newPage();
  const errors = [];
  /* The demo page is published unchanged and has no icon, so Chrome's request for /favicon.ico when it is opened on its own is not a finding. */
  const icon = (u) => u.endsWith("/favicon.ico");
  page.on("console", (m) => { if ((m.type() === "error" || m.type() === "warning") && !icon(m.location().url || "")) errors.push(`${m.type()}: ${m.text()} (${m.location().url})`); });
  page.on("pageerror", (e) => errors.push("pageerror: " + e.message));
  page.on("response", (r) => { if (r.status() >= 400 && !icon(r.url())) errors.push(`${r.status()} ${r.url()}`); });

  /* every page loads without errors */
  const pages = allHtml(root).map((f) => path.relative(root, f).replace(/index\.html$/, ""));
  for (const p of pages) { await page.goto(origin + p, { waitUntil: "load" }); await page.waitForTimeout(200); ok((await page.title()).length > 0, `${p}: no title`); }
  console.log(`pages opened: ${pages.length}`);
  ok(pages.includes("wiki/") && pages.includes("tour/") && pages.includes("demo/") && pages.includes(""), "landing, wiki, demo and tour pages exist");
  if (shots) { await page.goto(origin); await page.screenshot({ path: path.join(shots, "landing.png") }); await page.goto(origin + "wiki/using-the-toolbox.html"); await page.screenshot({ path: path.join(shots, "wiki.png") }); }

  /* the benchmark pages: an overview and one page per project, each with bars, no script */
  for (const p of ["bench/", "bench/nv-rs/", "bench/loot-rs/", "bench/ttwright-rs/", "bench/lodwright-rs/", "bench/nvmod-rs/", "bench/toolbox/"]) {
    ok(pages.includes(p), `${p} exists`);
    await page.goto(origin + p, { waitUntil: "load" }); await page.waitForTimeout(300);
    ok((await page.locator("h1").count()) === 1, `${p}: one h1`);
    ok((await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1)), `${p}: scrolls sideways`);
  }
  await page.goto(origin + "bench/");
  ok((await page.locator(".bcard").count()) === 6, "overview has one card per project");
  await page.goto(origin + "bench/loot-rs/");
  ok((await page.locator(".track .fill").count()) >= 6, "loot-rs page draws bars");
  await page.goto(origin + "bench/nv-rs/");
  ok((await page.locator(".b.missing").count()) >= 5, "nv-rs shows the original game as not measured yet");
  if (shots) for (const [u, n] of [["bench/", "bench-overview"], ["bench/loot-rs/", "bench-loot-rs"]]) { await page.goto(origin + u); await page.waitForTimeout(1200); await page.screenshot({ path: path.join(shots, n + ".png"), fullPage: true }); }

  /* the guided tour */
  await page.goto(origin + "tour/");
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  const demo = page.frameLocator("#demo");
  const dwin = async () => (await (await page.locator("#demo").elementHandle()).contentFrame());
  const dev = async (fn, arg) => (await dwin()).evaluate(fn, arg);
  const click = async (sel) => { await demo.locator(sel).first().click(); await page.waitForTimeout(120); };
  const act = (a, arg) => `[data-act="${a}"]${arg !== undefined ? `[data-arg="${arg}"]` : ""}`;
  const waitText = async (t, ms = 15000) => (await dwin()).waitForFunction((x) => document.querySelector("#app").innerText.includes(x), t, { timeout: ms });
  const isDone = (id) => page.locator(`[data-step="${id}"]`).getAttribute("data-done").then((v) => v === "true");
  const expectTick = async (id) => {
    try { await page.waitForFunction((i) => document.querySelector(`[data-step="${i}"]`).dataset.done === "true", id, { timeout: 15000 }); }
    catch (e) { bad(`tour step "${id}" did not tick`); }
  };
  const ids = await page.evaluate(() => window.__tour.steps);
  ok(ids.length >= 8 && ids.length <= 12, `tour has ${ids.length} steps`);
  await page.waitForSelector("#demo");
  await dwin().then((f) => f.waitForFunction(() => typeof S !== "undefined" && S !== null));
  for (const id of ids) ok(!(await isDone(id)), `step ${id} is ticked before anything was done`);

  /* the demo fills the stage: its window is drawn at the largest size that fits */
  const scale = await dev(() => parseFloat(getComputedStyle(document.getElementById("win")).getPropertyValue("--ws")));
  const stage = await page.locator(".stage").boundingBox();
  ok(scale >= Math.min(1, (stage.width - 32) / 1280) - 0.02, `demo drawn at ${scale}, smaller than the stage allows`);
  ok(stage.height >= 720 * scale, "stage is smaller than the demo window");
  /* Show me highlights an element of the demo */
  const showMe = async (id, label) => {
    await page.locator(`[data-step="${id}"] .head`).click();
    await page.locator(`[data-step="${id}"] .show`).click();
    try { await page.waitForFunction(() => document.getElementById("ring").style.display === "block", null, { timeout: 4000 }); }
    catch (e) { bad(`"Show me" for ${id} shows no highlight`); return; }
    const box = await page.locator("#ring").boundingBox();
    const fb = await page.locator("#demo").boundingBox();
    ok(box && box.x >= fb.x - 10 && box.y >= fb.y - 10 && box.x + box.width <= fb.x + fb.width + 10, `highlight for ${id} is outside the demo`);
    if (label) ok((await page.locator("#ring").getAttribute("data-target")) === label, `highlight label for ${id}`);
  };
  await showMe("setup", "Start");
  if (shots) await page.screenshot({ path: path.join(shots, "tour-start.png") });

  /* 1 setup */
  await click(act("wizNext")); await waitText("Found"); await click(act("wizNext")); await click(act("wizNext"));
  await click(act("wizEngine")); await waitText("Installed", 20000); await click(act("wizNext"));
  await click(act("wizTtw", "skip")); await click(act("wizNext")); await click(act("wizProf", "hard")); await click(act("wizNext"));
  await click(act("wizFar", "later")); await click(act("wizNext")); await click(act("wizFinish"));
  await expectTick("setup");
  await page.waitForTimeout(300);
  for (const id of ids.slice(1)) ok(!(await isDone(id)), `step ${id} ticked by setup alone`);

  /* 2 profile */
  await showMe("profile", "Profiles");
  await click(act("profMenu", "top")); await click('.menu [data-act="profSwitch"][data-arg="beaut"]');
  await expectTick("profile");
  await click(act("profMenu", "top")); await click('.menu [data-act="profSwitch"][data-arg="hard"]');

  /* 3 group */
  await showMe("group");
  await click(act("nav", "mods")); await click(act("newGroup")); await demo.locator("#nameIn").fill("Bug fixes"); await click(act("nameOk"));
  await expectTick("group");

  /* 4 off and undo: the tick comes only after both */
  await showMe("undo");
  await click('button.cb[data-act="modToggle"][aria-checked="true"]'); await page.waitForTimeout(500);
  ok(!(await isDone("undo")), "undo step ticked after only turning a mod off");
  await showMe("undo", "Undo");
  await click(act("undo"));
  await expectTick("undo");

  /* 5 sort */
  await showMe("sort");
  await click(act("nav", "load")); await click(act("sortGo")); await page.waitForTimeout(300);
  ok(!(await isDone("sort")), "sort step ticked by the preview alone");
  await click(act("sortApply"));
  await expectTick("sort");

  /* 6 LOD */
  await showMe("lod", "Open LOD");
  await click(act("nav", "far")); await click(act("farBuild", ""));
  await expectTick("lod");

  /* 7 stability test */
  await showMe("stability");
  await click(act("nav", "mods")); await click(act("tab", "test")); await click(act("stabGo", "tab"));
  await expectTick("stability");

  /* 8 pictures with zoom */
  await showMe("pictures");
  await click(act("zoom", "4"));
  await expectTick("pictures");

  /* 9 updates */
  await showMe("updates", "Open System");
  await click(act("nav", "system"));
  await expectTick("updates");

  /* 10 doctor */
  await showMe("doctor", "Open Doctor");
  await click(act("nav", "doctor")); await page.waitForTimeout(300);
  ok(!(await isDone("doctor")), "doctor step ticked by opening the page alone");
  await click(act("docRun"));
  await expectTick("doctor");
  ok((await page.locator("#count").innerText()) === `${ids.length} of ${ids.length}`, "count shows all steps");
  ok(await page.locator("#finished").isVisible(), "finished note is shown");
  if (shots) await page.screenshot({ path: path.join(shots, "tour-done.png") });

  /* progress survives a reload; Reset clears it and the demo */
  await page.reload(); await page.waitForTimeout(500);
  ok((await page.locator("#count").innerText()) === `${ids.length} of ${ids.length}`, "progress kept after reload");
  await page.locator("#reset").click(); await page.locator("#reset").click();
  await page.waitForTimeout(800);
  ok((await page.locator("#count").innerText()) === `0 of ${ids.length}`, "reset clears progress");
  await dwin().then((f) => f.waitForFunction(() => S && !S.wiz.done));
  ok(true, "reset restarts the demo setup");
  if (shots) { await page.waitForTimeout(500); await page.screenshot({ path: path.join(shots, "tour-reset.png") }); }

  console.log(`console errors: ${errors.length}`);
  errors.forEach((e) => bad(e));
  await browser.close(); server.close();
  console.log(problems.length ? `FAILED: ${problems.length} problem(s)` : "site check passed");
  process.exit(problems.length ? 1 : 0);
}
main().catch((e) => { console.error(e); process.exit(2); });
