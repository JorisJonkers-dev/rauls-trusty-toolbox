// SPDX-License-Identifier: GPL-3.0-only
// usage: node test/keys.js — focus indicators (no ring on text fields or mouse focus, one subtle line on keyboard focus) and Enter submits every dialog; writes shots/keys/
const { chromium } = require('playwright');
const path = require('path'); const fs = require('fs');
const OUT = path.resolve(__dirname, '../shots/keys'); fs.mkdirSync(OUT, { recursive: true });
const fails = [], errs = []; const ok = (c, m) => { console.log(c ? 'ok  ' : 'FAIL', m); if (!c) fails.push(m); };
(async () => {
  const b = await chromium.launch(); const pg = await b.newPage({ viewport: { width: 1366, height: 768 } });
  pg.on('pageerror', e => errs.push(e.message)); pg.on('console', m => { if (m.type() === 'error' || m.type() === 'warning') errs.push(m.text()); });
  await pg.goto('file://' + path.resolve(__dirname, '../index.html')); await pg.evaluate(() => localStorage.clear()); await pg.reload();
  await pg.evaluate(() => { S.wiz = { ...S.wiz, step: 7, game: { path: FOLDERS.fnv[0][1], ok: true }, store: { path: FOLDERS.store[1][1], free: 142 }, engine: true, fo3: { path: FOLDERS.fo3[0][1], ok: true }, ttw: 'added', prof: 'beaut', far: 'built', q: 'high' }; A.wizFinish(); U.toasts = []; U.page = 'mods'; render(); });
  const ev = (f, a) => pg.evaluate(f, a); const w = (n = 150) => pg.waitForTimeout(n);
  const ring = sel => ev(s => { const e = document.querySelector(s); const c = getComputedStyle(e); return { ow: c.outlineStyle === 'none' ? 0 : parseFloat(c.outlineWidth), bs: c.boxShadow }; }, sel);
  const ringOf = () => ev(() => { const e = document.activeElement; const c = getComputedStyle(e); return { tag: e.tagName, ow: c.outlineStyle === 'none' ? 0 : parseFloat(c.outlineWidth), bs: c.boxShadow, vis: e.matches(':focus-visible') }; });
  // text field
  await w(300); await pg.click('[data-in="q"]'); await pg.keyboard.type('nev'); await w();
  const r1 = await ring('[data-in="q"]'), r1b = await ring('.srch'); ok(r1.ow === 0 && r1b.ow === 0 && (r1.bs === 'none') && (r1b.bs === 'none'), `search field: no outline on the input or its pill ${JSON.stringify([r1, r1b])}`);
  await pg.screenshot({ path: `${OUT}/search-focused.png` });
  await pg.fill('[data-in="q"]', '');
  // mouse focus on a button: no ring
  await pg.click('[data-act="filt"][data-arg="on"]'); await w(); const r2 = await ringOf(); ok(r2.tag === 'BUTTON' && !r2.vis && r2.ow === 0, `mouse-focused button: no ring ${JSON.stringify(r2)}`);
  await pg.click('[data-act="filt"][data-arg="all"]');
  // keyboard focus: one 1px line, no double ring
  await pg.keyboard.press('Tab'); await w(); const r3 = await ringOf(); ok(r3.vis && r3.ow <= 1 && (r3.bs === 'none' || !r3.bs.includes(',')), `keyboard focus: single subtle indicator ${JSON.stringify(r3)}`);
  await pg.screenshot({ path: `${OUT}/keyboard-focus-chip.png` });
  for (let i = 0; i < 6; i++) await pg.keyboard.press('Tab'); await w(); const r4 = await ringOf(); ok(r4.ow <= 1, `keyboard focus further on: ${JSON.stringify(r4)}`);
  // select: keyboard only
  await pg.click('.mrow[data-mod="nmc"] .ver'); await w(); await pg.click('#mg-nmc'); await pg.keyboard.press('Escape'); await w(); const r5 = await ringOf(); ok(r5.ow <= 1, `select focus: at most one 1px line ${JSON.stringify(r5)}`);
  // Enter submits: new group
  await pg.click('[data-act="newGroup"]'); await w(); await pg.keyboard.type('Weather'); await pg.screenshot({ path: `${OUT}/dialog-new-group.png` }); await pg.keyboard.press('Enter'); await w(300);
  ok(await ev(() => !U.dlg && P().groups.some(g => g.name === 'Weather')), 'Enter creates the group');
  const r6 = await ring('.srch input'); ok(r6.ow === 0, 'search input still no outline');
  // rename group via menu, Enter
  const gid = await ev(() => P().groups.find(g => g.name === 'Weather').id);
  await pg.click(`[data-act="grpMenu"][data-arg="${gid}"]`); await pg.click('.menu [data-act="grpRename"]'); await w(); await pg.keyboard.press('Control+a'); await pg.keyboard.type('Weather mods'); await pg.keyboard.press('Enter'); await w(300);
  ok(await ev(() => P().groups.some(g => g.name === 'Weather mods')), 'Enter renames the group');
  // profile name
  await pg.click('[data-act="profMenu"][data-arg="top"]'); await pg.click('.menu [data-act="profNew"]'); await w(); await pg.keyboard.type('Keyboard'); await pg.keyboard.press('Enter'); await w(300);
  ok(await ev(() => P().name === 'Keyboard'), 'Enter creates the profile');
  // Escape cancels
  await pg.click('[data-act="profMenu"][data-arg="top"]'); await pg.click('.menu [data-act="profRename"]'); await w(); await pg.keyboard.type(' x'); await pg.keyboard.press('Escape'); await w();
  ok(await ev(() => !U.dlg && P().name === 'Keyboard'), 'Escape cancels rename');
  // install: archive row Enter -> options, Enter -> next page, Enter -> review, Enter -> install
  await pg.click('[data-act="profMenu"][data-arg="top"]'); await pg.click('.menu [data-act="profSwitch"][data-arg="beaut"]'); await w();
  await pg.click('[data-act="install"]'); await w(); await pg.locator('[data-act="archSel"][data-arg="1"]').focus(); await pg.keyboard.press('Enter'); await w(300);
  ok(await ev(() => U.dlg && U.dlg.step === 'opts'), 'Enter on an archive goes to its options');
  await pg.keyboard.press('Enter'); await w(250); ok(await ev(() => U.dlg.step === 'opts' && U.dlg.page === 1), 'Enter on options goes to the next page');
  await pg.keyboard.press('Enter'); await w(250); ok(await ev(() => U.dlg.step === 'review'), 'Enter reaches review');
  await pg.screenshot({ path: `${OUT}/dialog-install-review.png` });
  await pg.keyboard.press('Enter'); await w(250); ok(await ev(() => U.dlg && U.dlg.step === 'busy'), 'Enter installs');
  await pg.waitForFunction(() => !U.dlg, null, { timeout: 6000 });
  // confirm: remove
  await ev(() => A.modRemove('tld')); await w(); await pg.screenshot({ path: `${OUT}/dialog-confirm.png` }); await pg.keyboard.press('Enter'); await w(300);
  ok(await ev(() => !P().mods.some(m => m.id === 'tld')), 'Enter confirms remove');
  await ev(() => A.resetAll()); await w(); await pg.keyboard.press('Escape'); await w(); ok(await ev(() => !U.dlg && S.wiz.done), 'Escape cancels a confirm');
  console.log('errors', errs.length, errs, 'failed', fails.length); await b.close(); process.exit(errs.length || fails.length ? 1 : 0);
})().catch(e => { console.error(e); process.exit(2); });
