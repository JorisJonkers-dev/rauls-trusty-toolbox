// SPDX-License-Identifier: GPL-3.0-only
// usage: node test/home-shot.js <out.png> — Home in a ready state (setup done, Beautiful, far view built, some play history)
const { chromium } = require('playwright');
const path = require('path');
(async () => {
  const b = await chromium.launch(); const pg = await b.newPage({ viewport: { width: 1366, height: 768 } });
  await pg.goto('file://' + path.resolve(__dirname, '../index.html')); await pg.evaluate(() => localStorage.clear()); await pg.reload();
  await pg.evaluate(() => { S.wiz = { ...S.wiz, step: 7, game: { path: FOLDERS.fnv[0][1], ok: true }, store: { path: FOLDERS.store[1][1], free: 142 }, engine: true, fo3: { path: FOLDERS.fo3[0][1], ok: true }, ttw: 'added', prof: 'beaut', far: 'built', q: 'high' }; A.wizFinish(); A.scenHistory(); U.toasts = []; render(); });
  await pg.waitForTimeout(400); await pg.screenshot({ path: process.argv[2] }); await b.close();
})();
