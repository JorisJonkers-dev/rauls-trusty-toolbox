// SPDX-License-Identifier: GPL-3.0-only
// Checks of the download page and the wiki shell, called by check.js with a running site.
"use strict";
const path = require("path");

const REL = (tag, extra) => Object.assign({
  tag_name: tag, name: tag, published_at: "2030-01-02T03:04:05Z", html_url: "https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/releases/tag/" + tag,
  draft: false, prerelease: false,
  body: "## Notes\n\n* a **fix** ([#1](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues/1))\n* <img src=x onerror=\"window.__xss=1\"> [bad](javascript:window.__xss=2)\n",
  assets: [
    ["raul-9.9.9-linux-x64.AppImage", 1000000, "a"], ["raul-9.9.9-macos-universal.dmg", 2000000, "b"], ["raul-9.9.9-windows-x64.setup.exe", 3000000, "c"], ["SHA256SUMS", 100, "d"],
  ].map(([name, size, c]) => ({ name, size, digest: "sha256:" + c.repeat(64), browser_download_url: `https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/releases/download/${tag}/${name}` })),
}, extra);

module.exports = async function run({ browser, origin, ok, bad, shots, wikiPages }) {
  const open = async (opts, route) => {
    const ctx = await browser.newContext(Object.assign({ viewport: { width: 1440, height: 900 } }, opts));
    const page = await ctx.newPage();
    const errs = [];
    page.on("pageerror", (e) => errs.push("pageerror: " + e.message));
    page.on("console", (m) => { if (m.type() === "error" && !/favicon|Failed to load resource/.test(m.text() + m.location().url)) errs.push(m.text()); });
    await ctx.route("**://api.github.com/**", route);
    await page.goto(origin + "download/", { waitUntil: "load" });
    return { ctx, page, errs };
  };

  /* 1. live data: a newer release than the build's replaces it; its notes are text, never markup */
  {
    const latest = REL("v9.9.9");
    const { ctx, page, errs } = await open({}, (r) => {
      const u = r.request().url();
      r.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(u.includes("/latest") ? latest : [latest, REL("v9.9.8", { assets: [], body: "" })]) });
    });
    await page.waitForSelector('#dl[data-source="api"]', { timeout: 8000 }).catch(() => bad("download: live data was not used"));
    ok((await page.locator("#ver").innerText()) === "v9.9.9", "download: version is not the live one");
    ok((await page.locator("#bigdl").getAttribute("href")).endsWith("/v9.9.9/raul-9.9.9-linux-x64.AppImage"), "download: big button is not the live Linux file");
    ok((await page.locator("#older-rows a").count()) === 1, "download: older releases not listed from the API");
    ok((await page.locator("#os-linux .sha code").first().innerText()) === "a".repeat(64), "download: checksum missing");
    ok((await page.locator("#notes img").count()) === 0 && (await page.evaluate(() => !window.__xss)), "download: release notes ran markup");
    ok((await page.locator('#notes a[href^="javascript"]').count()) === 0, "download: a javascript link got through");
    ok((await page.locator("#notes").innerText()).includes("<img"), "download: notes should show the raw text");
    ok((await page.locator("#oses section").count()) === 3, "download: three systems expected");
    errs.forEach((e) => bad("download (live): " + e));
    await ctx.close();
  }
  /* 2. API blocked: the build's data is shown, with its checksums, a note, and every file linked */
  {
    const { ctx, page, errs } = await open({}, (r) => r.abort());
    await page.waitForSelector('#dl[data-ready="true"]');
    await page.waitForTimeout(500);
    ok((await page.locator("#dl").getAttribute("data-source")) === "build", "download: fallback not in use");
    ok(/^v\d+\.\d+\.\d+/.test(await page.locator("#ver").innerText()), "download: fallback has no version");
    ok((await page.locator("#status").innerText()).includes("could not be reached"), "download: fallback gives no note");
    ok((await page.locator("#oses .file").count()) >= 6, "download: fallback lists too few files");
    const sha = await page.locator("#os-linux .sha code").allInnerTexts();
    ok(sha.length > 0 && sha.every((s) => /^[0-9a-f]{64}$/.test(s)), "download: fallback checksums are not SHA-256");
    ok((await page.locator('a[href*="/releases/download/"]').count()) >= 6, "download: fallback has no download links");
    ok((await page.locator("#older-rows a").count()) >= 3, "download: fallback has no older releases");
    ok((await page.locator("#notes").innerText()).length > 50, "download: fallback has no release notes");
    ok((await page.locator("#oses section").first().getAttribute("data-os")) === "linux", "download: Linux should come first on Linux");
    if (shots) await page.screenshot({ path: path.join(shots, "download.png") });
    if (shots) await page.screenshot({ path: path.join(shots, "download-full.png"), fullPage: true });
    errs.forEach((e) => bad("download (blocked): " + e));
    await ctx.close();
  }
  /* 3. the visitor's system comes first */
  for (const [name, ua, expect] of [["Windows", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120 Safari/537.36", "windows"], ["macOS", "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_0) AppleWebKit/537.36 Chrome/120 Safari/537.36", "macos"], ["Android", "Mozilla/5.0 (Linux; Android 13) AppleWebKit/537.36 Chrome/120 Mobile Safari/537.36", null]]) {
    const { ctx, page } = await open({ userAgent: ua }, (r) => r.abort());
    await page.waitForSelector('#dl[data-ready="true"]');
    const first = await page.locator("#oses section").first().getAttribute("data-os");
    ok(expect ? first === expect : first === "linux", `download: ${name} visitor sees ${first} first`);
    ok(expect ? (await page.locator("#bigdl").count()) === 1 : (await page.locator("#bigdl").count()) === 0, `download: big button for ${name}`);
    await ctx.close();
  }
  /* 4. the wiki looks like the app: sidebar with groups and icons, rounded cards */
  {
    const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
    const page = await ctx.newPage();
    const errs = [];
    page.on("pageerror", (e) => errs.push(e.message));
    page.on("console", (m) => { if (m.type() === "error" || m.type() === "warning") errs.push(m.text()); });
    for (const p of wikiPages) {
      await page.goto(origin + p, { waitUntil: "load" });
      const hrefs = await page.locator(".side .nav").evaluateAll((a) => a.map((x) => x.getAttribute("href")));
      ok(["getting-started.html", "faq.html", "using-the-toolbox.html"].every((h) => hrefs.includes(h)), `${p}: sidebar does not list the wiki pages`);
      ok((await page.locator(".side .nav").count()) >= 8, `${p}: wiki sidebar has too few pages`);
      ok((await page.locator(".side .nav svg").count()) === (await page.locator(".side .nav").count()), `${p}: a sidebar item has no icon`);
      ok((await page.locator(".side .navh").count()) === 2, `${p}: sidebar groups missing`);
      ok((await page.locator(".side .nav.on").count()) === 1, `${p}: exactly one current page expected`);
      ok((await page.locator("main .card").count()) >= 1, `${p}: no cards`);
      ok((await page.locator(".top .brand").count()) === 1, `${p}: no brand`);
      ok((await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)), `${p}: sideways scroll`);
    }
    errs.forEach((e) => bad("wiki: " + e));
    await ctx.close();
  }
  console.log("download and wiki checks done");
};
