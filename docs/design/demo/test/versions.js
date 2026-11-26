// SPDX-License-Identifier: GPL-3.0-only
// usage: node test/versions.js — versions, revert, pick a later version, benchmark points, System wording; writes shots/versions/
const { chromium } = require('playwright');
const path = require('path'); const fs = require('fs');
const OUT = path.resolve(__dirname, '../shots/versions'); fs.mkdirSync(OUT, { recursive: true });
const errors = [], fails = []; let n = 0;
const ok = (c, m) => { if (!c) { fails.push(m); console.log('FAIL', m); } };
(async () => {
  const b = await chromium.launch(); const pg = await b.newPage({ viewport: { width: 1366, height: 768 } });
  pg.on('console', m => { if (m.type() === 'error' || m.type() === 'warning') errors.push(m.text()); }); pg.on('pageerror', e => errors.push(e.message));
  const shot = async name => { await pg.waitForTimeout(250); await pg.screenshot({ path: `${OUT}/${String(++n).padStart(2, '0')}-${name}.png` }); };
  const act = (a, arg) => `[data-act="${a}"]${arg !== undefined ? `[data-arg="${arg}"]` : ''}`;
  const click = async s => { await pg.locator(s).first().click(); await pg.waitForTimeout(120); };
  const text = () => pg.locator('#app').innerText();
  const age = () => pg.evaluate(() => { const v = P().versions; v[v.length - 1].t -= 120000; U.toasts = []; });
  await pg.goto('file://' + path.resolve(__dirname, '../index.html')); await pg.evaluate(() => localStorage.clear()); await pg.reload();
  await pg.evaluate(() => { S.wiz = { ...S.wiz, step: 7, game: { path: FOLDERS.fnv[0][1], ok: true }, store: { path: FOLDERS.store[1][1], free: 142 }, engine: true, fo3: { path: FOLDERS.fo3[0][1], ok: true }, ttw: 'added', prof: 'beaut', far: 'built', q: 'high' }; A.wizFinish(); U.toasts = []; render(); });
  await click(act('nav', 'mods')); await click(act('tab', 'versions')); await shot('versions-start');
  ok((await text()).includes('Current'), 'current version shown');
  await click(act('tab', 'list')); await click(act('modToggle', 'wfo')); await age();
  await click(act('modMove', 'fco:-1')).catch(async () => { await click('[data-act="rowOpen"][data-arg="fco"]'); await click(act('modMove', 'fco:-1')); }); await age();
  await click(act('tab', 'versions')); await shot('versions-after-changes');
  ok((await text()).includes('Wasteland Flora Overhaul off'), 'summary for switch');
  ok((await text()).includes('order changed'), 'summary for order');
  await click(act('nav', 'home')); await shot('home-rec-card-revert'); ok((await text()).includes('Revert to last stable'), 'home revert button');
  await click(act('play')); await pg.waitForFunction(() => document.querySelector('#app').innerText.includes('Before you play'), null, { timeout: 8000 }); await shot('play-checks-revert');
  await click(act('playCancel'));
  // benchmark on the new version, then the graph points
  await click(act('nav', 'mods')); await click(act('tab', 'test')); await shot('bench-card-button');
  await click(act('benchGo', '')); await pg.waitForFunction(() => document.querySelector('#app').innerText.includes('Benchmark done'), null, { timeout: 9000 });
  await pg.evaluate(() => { U.toasts = []; render(); }); await shot('bench-graph-labels');
  const pts = pg.locator('.chart [data-act="ptMenu"]'); const cnt = await pts.count();
  await pts.nth(1).click(); await shot('bench-point-menu');
  await click('.menu [data-act="vRevert"]'); await shot('revert-confirm'); await click('[data-k="dlgok"]'); await shot('after-revert-test-tab');
  await click(act('tab', 'versions')); await shot('versions-after-revert');
  ok((await text()).includes('Use this version'), 'later versions offered');
  await click(act('nav', 'far')); await shot('lod-after-revert'); ok((await text()).includes('Out of date'), 'LOD out of date after revert');
  await click(act('nav', 'mods')); await click(act('tab', 'versions'));
  await click('[data-act="vRevert"]'); await shot('pick-later-confirm'); await click('[data-k="dlgok"]'); await shot('versions-picked-later');
  // revert to last stable from Mods card
  await click(act('tab', 'list')); await click(act('vRevertStable')); await click('[data-k="dlgok"]'); await click(act('tab', 'versions')); await shot('versions-reverted-stable');
  ok((await text()).match(/Current/), 'current after stable revert');
  await click('[data-act="vexp"]'); await shot('version-details');
  // settings change makes a version
  await click(act('nav', 'settings')); await click(act('secS', 's_prof')); await click('[data-act="gfxSeg"][data-arg="up:Off"]'); await click(act('nav', 'mods')); await click(act('tab', 'versions'));
  ok((await text()).includes('settings changed'), 'settings make a version');
  // system wording
  await click(act('nav', 'system')); await pg.waitForFunction(() => document.querySelector('#app').innerText.includes('Checked'), null, { timeout: 5000 }); await shot('system-wording');
  const t = await text(); ok(t.includes('will be updated') && !t.includes('is ready') && !t.includes('updates with the app') && !t.includes('Update to 0.6.0'), 'system wording');
  console.log('points', cnt, 'console errors:', errors.length, errors, 'failed checks:', fails.length);
  await b.close(); process.exit(errors.length || fails.length ? 1 : 0);
})().catch(e => { console.error(e); process.exit(2); });
