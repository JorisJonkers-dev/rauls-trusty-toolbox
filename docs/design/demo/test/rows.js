// SPDX-License-Identifier: GPL-3.0-only
// usage: node test/rows.js — whole-row clicks: a click on a row's text folds, expands or toggles; nested controls keep their own action
const { chromium } = require('playwright');
const path = require('path'); const fs = require('fs');
const OUT = path.resolve(__dirname, '../shots/rows'); fs.mkdirSync(OUT, { recursive: true });
const fails = [], errs = []; const ok = (c, m) => { console.log(c ? 'ok  ' : 'FAIL', m); if (!c) fails.push(m); };
(async () => {
  const b = await chromium.launch(); const pg = await b.newPage({ viewport: { width: 1366, height: 768 } });
  pg.on('pageerror', e => errs.push(e.message)); pg.on('console', m => { if (m.type() === 'error' || m.type() === 'warning') errs.push(m.text()); });
  await pg.goto('file://' + path.resolve(__dirname, '../index.html')); await pg.evaluate(() => localStorage.clear()); await pg.reload();
  await pg.evaluate(() => { S.wiz = { ...S.wiz, step: 7, game: { path: FOLDERS.fnv[0][1], ok: true }, store: { path: FOLDERS.store[1][1], free: 142 }, engine: true, fo3: { path: FOLDERS.fo3[0][1], ok: true }, ttw: 'added', prof: 'beaut', far: 'built', q: 'high' }; A.wizFinish(); U.toasts = []; U.page = 'mods'; render(); });
  const ev = (f, a) => pg.evaluate(f, a); const w = () => pg.waitForTimeout(150);
  await w();
  await pg.locator('.mrow[data-mod="nmc"] .ver').click(); await w(); ok(await ev(() => !!U.open.nmc), 'mod row: click on version text opens');
  await pg.screenshot({ path: `${OUT}/mod-row-open.png` });
  await pg.locator('.mrow[data-mod="nmc"] .num').click(); await w(); ok(await ev(() => !U.open.nmc), 'mod row: click on number closes');
  const before = await ev(() => P().mods.find(m => m.id === 'nmc').on);
  const hb = await pg.locator('.mrow[data-mod="nmc"] .hit').boundingBox(); ok(hb.width >= 44 && hb.height >= 44, `switch cell ${hb.width}x${hb.height}`);
  await pg.mouse.click(hb.x + 3, hb.y + 3); await w(); ok(await ev(b0 => P().mods.find(m => m.id === 'nmc').on !== b0, before), 'mod row: edge of switch cell toggles');
  ok(await ev(() => !U.open.nmc), 'switch cell does not open the row');
  const gid = await ev(() => P().groups.find(g => g.name === 'Textures').id);
  await pg.locator(`[data-groupcard="${gid}"] .sech .r8`).click({ position: { x: 4, y: 10 } }); await w(); ok(await ev(id => !P().groups.find(g => g.id === id).open, gid), 'group header: click on count folds');
  await pg.locator(`[data-groupcard="${gid}"] .sech .grip`).click(); await w(); ok(await ev(id => !P().groups.find(g => g.id === id).open, gid), 'grip does not fold');
  await pg.locator('.sech[data-arg="builtin"] .t').click(); await w(); ok(await ev(() => U.secs.builtin), 'section header: click on title unfolds');
  await pg.click('[data-act="tab"][data-arg="versions"]'); await pg.locator('.tl .r .sub').first().click(); await w(); ok(await ev(() => !!U.vexp), 'version row: click on summary expands');
  await pg.locator('.tl .r').first().focus(); await pg.keyboard.press(' '); await w(); ok(await ev(() => !U.vexp), 'version row: Space toggles');
  await pg.click('[data-act="nav"][data-arg="load"]'); const f = await ev(() => visPlugins(P())[0].f); const on0 = await ev(f => P().plugins.find(x => x.f === f).on, f);
  await pg.locator(`.mrow[data-plugin="${f}"] .sub`).first().click(); await w(); ok(await ev(([f, o]) => P().plugins.find(x => x.f === f).on !== o, [f, on0]), 'plugin row: click on mod name toggles');
  await pg.screenshot({ path: `${OUT}/plugin-row-toggled.png` });
  await pg.click('[data-act="nav"][data-arg="system"]'); await pg.waitForTimeout(2200); await pg.locator('[data-arg="nvmod"] .sub').click(); await w(); ok(await ev(() => U.vopen === 'nvmod'), 'System part row: click on role text opens');
  await pg.click('[data-act="nav"][data-arg="doctor"]'); await pg.locator('.sech[data-act="docFold"] .t').first().click(); await w(); ok(await ev(() => U.secs.doc0 === false), 'Doctor group: click on title folds');
  await pg.click('[data-act="nav"][data-arg="settings"]'); await pg.locator('.sech[data-arg="s_look"] .t').click(); await w(); ok(await ev(() => U.secs.s_look), 'settings section: click on title unfolds');
  await pg.locator('.r', { hasText: 'Reduce motion' }).locator('.nm').click(); await w(); ok(await ev(() => S.g.motion === false), 'switch row: click on label toggles');
  await pg.locator('.r', { hasText: 'Reduce motion' }).hover(); await pg.screenshot({ path: `${OUT}/settings-switch-row-hover.png` });
  await pg.locator('.r', { hasText: 'Reduce motion' }).locator('.sw').click(); await w(); ok(await ev(() => S.g.motion === true), 'the switch itself toggles once');
  console.log('errors', errs.length, errs, 'failed', fails.length); await b.close(); process.exit(errs.length || fails.length ? 1 : 0);
})().catch(e => { console.error(e); process.exit(2); });
