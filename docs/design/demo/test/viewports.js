// SPDX-License-Identifier: GPL-3.0-only
// usage: node test/viewports.js — the window at 1920x1080, 1366x768 and 1024x600, plus a menu, dialog and the test panel
const { chromium } = require('playwright');
const path = require('path'); const fs = require('fs');
const OUT = path.resolve(__dirname, '../shots/window'); fs.mkdirSync(OUT, { recursive: true });
(async () => {
  const b = await chromium.launch(); const errs = [];
  for (const [w, h] of [[1920, 1080], [1366, 768], [1024, 600]]) {
    const pg = await b.newPage({ viewport: { width: w, height: h } }); pg.on('pageerror', e => errs.push(e.message)); pg.on('console', m => { if (m.type() === 'error') errs.push(m.text()); });
    await pg.goto('file://' + path.resolve(__dirname, '../index.html')); await pg.evaluate(() => localStorage.clear()); await pg.reload();
    await pg.screenshot({ path: `${OUT}/${w}x${h}-wizard.png` });
    await pg.evaluate(() => { S.wiz = { ...S.wiz, step: 7, game: { path: FOLDERS.fnv[0][1], ok: true }, store: { path: FOLDERS.store[1][1], free: 142 }, engine: true, fo3: { path: FOLDERS.fo3[0][1], ok: true }, ttw: 'added', prof: 'beaut', far: 'built', q: 'high' }; A.wizFinish(); A.scenHistory(); U.toasts = []; render(); });
    await pg.waitForTimeout(300); await pg.screenshot({ path: `${OUT}/${w}x${h}-home.png` });
    await pg.click('[data-act="nav"][data-arg="mods"]'); await pg.click('[data-act="profMenu"][data-arg="pill"]'); await pg.waitForTimeout(200); await pg.screenshot({ path: `${OUT}/${w}x${h}-mods-profile-menu.png` });
    await pg.keyboard.press('Escape'); await pg.click('[data-act="install"]'); await pg.waitForTimeout(200); await pg.screenshot({ path: `${OUT}/${w}x${h}-install-dialog.png` });
    await pg.keyboard.press('Escape'); await pg.keyboard.press('Control+Shift+D'); await pg.waitForTimeout(150); await pg.click('[data-act="modToggle"][data-arg="nmc"]'); await pg.waitForTimeout(200); await pg.screenshot({ path: `${OUT}/${w}x${h}-panel-and-toast.png` });
    await pg.keyboard.press('Control+Shift+D'); await pg.click('[data-act="quitAsk"]'); await pg.waitForTimeout(200); await pg.screenshot({ path: `${OUT}/${w}x${h}-quit.png` });
    await pg.close();
  }
  console.log('errors', errs.length, errs); await b.close();
})();
