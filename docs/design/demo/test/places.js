// SPDX-License-Identifier: GPL-3.0-only
// usage: node test/places.js — sidebar badges, location-picture zoom and pan, the place list with areas and search; writes shots/places/
const { chromium } = require('playwright');
const path = require('path'); const fs = require('fs');
const OUT = path.resolve(__dirname, '../shots/places'); fs.mkdirSync(OUT, { recursive: true });
const fails = [], errs = []; const ok = (c, m) => { console.log(c ? 'ok  ' : 'FAIL', m); if (!c) fails.push(m); };
(async () => {
  const b = await chromium.launch(); const pg = await b.newPage({ viewport: { width: 1366, height: 768 } });
  pg.on('pageerror', e => errs.push(e.message)); pg.on('console', m => { if (m.type() === 'error' || m.type() === 'warning') errs.push(m.text()); });
  await pg.goto('file://' + path.resolve(__dirname, '../index.html')); await pg.evaluate(() => localStorage.clear()); await pg.reload();
  await pg.evaluate(() => { S.wiz = { ...S.wiz, step: 7, game: { path: FOLDERS.fnv[0][1], ok: true }, store: { path: FOLDERS.store[1][1], free: 142 }, engine: true, fo3: { path: FOLDERS.fo3[0][1], ok: true }, ttw: 'added', prof: 'beaut', far: 'built', q: 'high' }; A.wizFinish(); U.toasts = []; U.page = 'mods'; render(); });
  const ev = (f, a) => pg.evaluate(f, a); const w = (n = 200) => pg.waitForTimeout(n);
  const badge = id => ev(id => { const e = document.querySelector(`.nav[data-arg="${id}"] .bdg`); return e ? [e.textContent, e.className] : null; }, id);
  await w();
  // badges
  ok(!(await badge('load')) && !(await badge('far')) && !(await badge('mods')), 'clean profile: no Load order, LOD or Mods badge');
  await pg.click('.hit[data-arg="wfo"]'); await w();
  const bl = await badge('load'), bf = await badge('far'); ok(bl && bl[1].includes('warn') && bf, `after a change: Load order ${JSON.stringify(bl)} and LOD ${JSON.stringify(bf)} badges`);
  await ev(() => { U.toasts = []; render(); }); await w(); await pg.screenshot({ path: `${OUT}/badges-after-change.png` });
  await pg.click('[data-act="undo"]'); await w(); ok(!(await badge('load')) && !(await badge('far')), 'undo clears the badges');
  await pg.click('[data-act="profMenu"][data-arg="top"]'); await pg.click('.menu [data-act="profSwitch"][data-arg="hard"]'); await w();
  await pg.click('[data-act="tab"][data-arg="list"]'); await pg.click('.hit[data-arg="wmx"]'); await w();
  const bl2 = await badge('load'), bm = await badge('mods'); ok(bl2 && bl2[1].includes('err') && bm && bm[1].includes('err'), `missing master: error badges ${JSON.stringify([bl2, bm])}`);
  await ev(() => { U.toasts = []; render(); }); await w(); await pg.screenshot({ path: `${OUT}/badges-missing-master.png` });
  await pg.click('.hit[data-arg="wmx"]'); await w(); ok(!(await badge('mods')), 'Mods badge clears when fixed');
  // places with TTW on (Hardcore)
  await pg.click('[data-act="tab"][data-arg="test"]'); await w();
  await ev(() => { const s = [...document.querySelectorAll('.sec')].find(x => x.innerText.includes('LOCATION PICTURES')); s.scrollIntoView(); }); await w();
  const n1 = await ev(() => document.querySelectorAll('[data-act="picPlace"]').length); const dc = await ev(() => !!document.querySelector('[data-act="picWorld"][data-arg="cw"]'));
  const strip = await ev(() => { const s = document.querySelector('.places.strip'); return s && s.scrollWidth > s.clientWidth && s.getBoundingClientRect().height < 50; });
  ok(n1 >= 25 && dc && strip, `TTW on: New Vegas tab ${n1} places in one side-scroll row, Capital Wasteland tab present`);
  ok(await ev(() => !document.querySelector('[data-act="picArea"]')), 'no South/Vegas/East area tabs');
  await pg.screenshot({ path: `${OUT}/places-strip.png` });
  await pg.click('[data-act="picAll"]'); await w(); ok(await ev(() => !document.querySelector('.places.strip') && document.querySelector('.places').getBoundingClientRect().height > 60), 'All expands to wrapped list');
  await pg.screenshot({ path: `${OUT}/places-all.png` }); await pg.click('[data-act="picAll"]'); await w();
  await pg.click('[data-act="picWorld"][data-arg="cw"]'); await pg.click('[data-act="picPlace"][data-arg="rivet"]'); await w();
  await ev(() => { const s = [...document.querySelectorAll('.sec')].find(x => x.innerText.includes('LOCATION PICTURES')); s.scrollIntoView(); }); await w();
  await pg.screenshot({ path: `${OUT}/places-capital-wasteland.png` });
  await pg.fill('[data-in="picQ"]', 'vault'); await w();
  const vs = await ev(() => [...document.querySelectorAll('[data-act="picPlace"]')].map(e => e.textContent)); ok(vs.length === 2 && vs.includes('Vault 22') && vs.includes('Vault 101'), `search "vault" spans both worlds: ${vs}`);
  await pg.screenshot({ path: `${OUT}/places-search.png` });
  await pg.fill('[data-in="picQ"]', ''); await pg.click('[data-act="picWorld"][data-arg="nv"]'); await w();
  // zoom and pan
  await pg.click('[data-act="picPlace"][data-arg="strip"]'); await w();
  await ev(() => { const s = [...document.querySelectorAll('.sec')].find(x => x.innerText.includes('LOCATION PICTURES')); s.scrollIntoView({ block: 'end' }); }); await w();
  const vp = pg.locator('.pic .vp').first(); const vb = await vp.boundingBox();
  await pg.mouse.move(vb.x + vb.width * 0.3, vb.y + vb.height * 0.6); for (let i = 0; i < 4; i++) { await pg.mouse.wheel(0, -100); await w(60); }
  const z1 = await ev(() => [U.zoom, U.pan.x, U.pan.y, ...[...document.querySelectorAll('.pic .vp svg')].map(s => s.style.transform)]);
  ok(z1[0] > 2 && z1[3] === z1[4], `wheel zooms around the cursor, both sides the same ${JSON.stringify(z1)}`);
  await pg.screenshot({ path: `${OUT}/zoomed-side-by-side.png` });
  await pg.mouse.move(vb.x + vb.width * 0.5, vb.y + vb.height * 0.5); await pg.mouse.down(); await pg.mouse.move(vb.x + vb.width * 0.5 - 80, vb.y + vb.height * 0.5 - 30, { steps: 6 }); await pg.mouse.up(); await w();
  const z2 = await ev(() => [U.pan.x, U.pan.y]); ok(z2[0] !== z1[1], `drag pans ${JSON.stringify(z2)}`);
  await pg.screenshot({ path: `${OUT}/panned.png` });
  await pg.click('[data-act="picMode"][data-arg="one"]'); await w(); await pg.screenshot({ path: `${OUT}/single.png` });
  const vp1 = await pg.locator('.pic .vp').first().boundingBox(); await pg.mouse.move(vp1.x + 50, vp1.y + 50); await pg.mouse.wheel(0, -100); await w();
  ok(await ev(() => document.querySelectorAll('.pic .vp').length === 1 && U.zoom > 2), 'single picture zooms too');
  await pg.mouse.dblclick(vp1.x + 100, vp1.y + 100); await w(); ok(await ev(() => U.zoom === 1 && U.pan.x === 0 && U.pan.y === 0), 'double-click resets');
  // TTW off hides the Capital Wasteland
  await pg.click('[data-act="profMenu"][data-arg="top"]'); await pg.click('.menu [data-act="profSwitch"][data-arg="beaut"]'); await w();
  await pg.click('[data-act="tab"][data-arg="test"]'); await w();
  ok(await ev(() => !document.querySelector('[data-act="picWorld"]') && document.querySelectorAll('[data-act="picPlace"]').length >= 25), 'TTW off: no Capital Wasteland, 25+ places');
  await ev(() => { const s = [...document.querySelectorAll('.sec')].find(x => x.innerText.includes('LOCATION PICTURES')); s.scrollIntoView(); }); await w();
  await pg.screenshot({ path: `${OUT}/places-new-vegas.png` });
  console.log('errors', errs.length, errs, 'failed', fails.length); await b.close(); process.exit(errs.length || fails.length ? 1 : 0);
})().catch(e => { console.error(e); process.exit(2); });
