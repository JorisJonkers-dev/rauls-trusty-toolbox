// SPDX-License-Identifier: GPL-3.0-only
// usage: node test/insets.js — measures that rows line up with their section header, shoots System, Doctor, Mods and a toast; writes shots/insets/
const { chromium } = require('playwright');
const path = require('path'); const fs = require('fs');
const OUT = path.resolve(__dirname, '../shots/insets'); fs.mkdirSync(OUT, { recursive: true });
const fails = []; const ok = (c, m) => { console.log(c ? 'ok  ' : 'FAIL', m); if (!c) fails.push(m); };
(async () => {
  const b = await chromium.launch(); const pg = await b.newPage({ viewport: { width: 1366, height: 768 } }); const errs = [];
  pg.on('pageerror', e => errs.push(e.message));
  await pg.goto('file://' + path.resolve(__dirname, '../index.html')); await pg.evaluate(() => localStorage.clear()); await pg.reload();
  await pg.evaluate(() => { S.wiz = { ...S.wiz, step: 7, game: { path: FOLDERS.fnv[0][1], ok: true }, store: { path: FOLDERS.store[1][1], free: 142 }, engine: true, fo3: { path: FOLDERS.fo3[0][1], ok: true }, ttw: 'added', prof: 'beaut', far: 'built', q: 'high' }; A.wizFinish(); U.toasts = []; render(); });
  const check = async name => { const bad = await pg.evaluate(() => { const out = []; document.querySelectorAll('.sec').forEach(sec => { const h = sec.querySelector(':scope>.sech'); if (!h) return; const hb = h.getBoundingClientRect(); sec.querySelectorAll(':scope>.rows>.r, :scope>.rows>.mrow').forEach(r => { const rb = r.getBoundingClientRect(); if (Math.abs(rb.left - hb.left) > 0.5 || Math.abs(rb.right - hb.right) > 0.5) out.push([h.innerText.split('\n')[0], Math.round(rb.left - hb.left), Math.round(rb.right - hb.right)]); }); }); return out; }); ok(!bad.length, `${name}: rows line up with headers ${JSON.stringify(bad.slice(0, 3))}`); };
  await pg.click('[data-act="nav"][data-arg="system"]'); await pg.waitForTimeout(2200); await pg.click('.sech[data-arg="engines"]'); await pg.evaluate(() => { document.querySelector('.mscroll').scrollTop = 9999; }); await pg.waitForTimeout(200);
  await check('System'); await pg.screenshot({ path: `${OUT}/system-engine-builds-masterlist.png` });
  await pg.click('[data-act="nav"][data-arg="doctor"]'); await pg.click('[data-act="docRun"]'); await pg.waitForTimeout(4000); await check('Doctor'); const dh = await pg.evaluate(() => [...document.querySelectorAll('div.sec')].map(sec => { const c = sec.getBoundingClientRect(), h = sec.querySelector(':scope>.sech').getBoundingClientRect(); const rs = [...sec.querySelectorAll(':scope>.rows>.r')].map(r => Math.round(r.getBoundingClientRect().height)); return [Math.round(h.left - c.left), Math.round(c.right - h.right), Math.round(h.height), ...rs]; })); ok(dh.every(d => d[0] === 12 && d[1] === 12 && d.slice(2).every(x => x === 48)), `Doctor cards: 12 px insets, header and rows 48 px ${JSON.stringify(dh)}`); ok((await pg.locator('#app').innerText()).includes('Update available'), 'Doctor says Update available'); await pg.screenshot({ path: `${OUT}/doctor-groups.png` });
  await pg.click('[data-act="nav"][data-arg="mods"]'); await pg.click('.sech[data-arg="builtin"]'); await pg.waitForTimeout(200); await check('Mods'); await pg.screenshot({ path: `${OUT}/mods-groups.png` });
  await pg.click('[data-act="nav"][data-arg="load"]'); await check('Load order'); await pg.click('[data-act="nav"][data-arg="settings"]'); await check('Settings');
  await pg.click('[data-act="nav"][data-arg="mods"]'); await pg.click('.hit[data-arg="nmc"]'); await pg.evaluate(() => toast('Saved')); await pg.waitForTimeout(250); await pg.screenshot({ path: `${OUT}/toasts.png` });
  const tb = await pg.evaluate(() => [...document.querySelectorAll('.toast')].map(t => { const r = t.getBoundingClientRect(), x = t.querySelector('.btn.icon').getBoundingClientRect(); return [Math.round(r.width), Math.round(r.right - x.right), Math.round((x.top + x.height / 2) - (r.top + r.height / 2))]; }));
  ok(tb.every(t => t[0] === tb[0][0] && t[1] === 8 && Math.abs(t[2]) <= 1), `toasts same width, X at the right edge, centred ${JSON.stringify(tb)}`);
  console.log('errors', errs, 'failed', fails.length); await b.close(); process.exit(errs.length || fails.length ? 1 : 0);
})();
