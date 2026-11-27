// SPDX-License-Identifier: GPL-3.0-only
// usage: node test/abar.js — floating action bar: shots of System, Mods (row open, scrolled to end), LOD, wizard at 1366x768, plus measured button boxes
const { chromium } = require('playwright');
const path = require('path'); const fs = require('fs');
const OUT = path.resolve(__dirname, '../shots/abar'); fs.mkdirSync(OUT, { recursive: true });
(async () => {
  const b = await chromium.launch(); const pg = await b.newPage({ viewport: { width: 1366, height: 768 } }); const errs = [];
  pg.on('pageerror', e => errs.push(e.message)); pg.on('console', m => { if (m.type() === 'error') errs.push(m.text()); });
  await pg.goto('file://' + path.resolve(__dirname, '../index.html')); await pg.evaluate(() => localStorage.clear()); await pg.reload();
  const measure = async name => { const r = await pg.evaluate(() => { const bar = document.querySelector('.abar'); if (!bar) return null; const bb = bar.getBoundingClientRect(); return { bar: [Math.round(bb.height)], btns: [...bar.querySelectorAll('.btn')].map(x => { const r = x.getBoundingClientRect(); return [x.innerText.trim(), Math.round(r.top), Math.round(r.height), getComputedStyle(x).borderRadius]; }) }; }); console.log(name, JSON.stringify(r)); await pg.screenshot({ path: `${OUT}/${name}.png` }); };
  await pg.click('[data-act="wizNext"]'); await pg.waitForTimeout(2000); await measure('wizard-game');
  await pg.evaluate(() => { S.wiz = { ...S.wiz, step: 7, game: { path: FOLDERS.fnv[0][1], ok: true }, store: { path: FOLDERS.store[1][1], free: 142 }, engine: true, fo3: { path: FOLDERS.fo3[0][1], ok: true }, ttw: 'added', prof: 'beaut', far: 'built', q: 'high' }; render(); });
  await pg.waitForTimeout(300); await measure('wizard-done');
  await pg.evaluate(() => { A.wizFinish(); U.toasts = []; }); await pg.click('[data-act="nav"][data-arg="system"]'); await pg.waitForTimeout(2600); await measure('system');
  await pg.click('[data-act="nav"][data-arg="mods"]'); await pg.click('[data-act="rowOpen"][data-arg="wfo"]'); await pg.evaluate(() => { U.toasts = []; const m = document.querySelector('.mscroll'); m.scrollTop = m.scrollHeight; render(); }); await pg.waitForTimeout(300); await measure('mods-row-open-end');
  await pg.click('[data-act="nav"][data-arg="far"]'); await pg.waitForTimeout(300); await measure('lod');
  await pg.click('[data-act="ttwStart"]').catch(() => {}); await pg.evaluate(() => { A.ttwStart(); }); await pg.waitForTimeout(1800); await measure('ttw');
  console.log('errors', errs); await b.close();
})();
