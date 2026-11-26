// SPDX-License-Identifier: GPL-3.0-only
// usage: node test/profiles.js — shared library: same rows and groups under two profiles; an install shows up off elsewhere; writes shots/profiles/
const { chromium } = require('playwright');
const path = require('path'); const fs = require('fs');
const OUT = path.resolve(__dirname, '../shots/profiles'); fs.mkdirSync(OUT, { recursive: true });
const fails = [], errs = []; const ok = (c, m) => { if (!c) { fails.push(m); console.log('FAIL', m); } };
(async () => {
  const b = await chromium.launch(); const pg = await b.newPage({ viewport: { width: 1366, height: 768 } });
  pg.on('pageerror', e => errs.push(e.message)); pg.on('console', m => { if (m.type() === 'error' || m.type() === 'warning') errs.push(m.text()); });
  await pg.goto('file://' + path.resolve(__dirname, '../index.html')); await pg.evaluate(() => localStorage.clear()); await pg.reload();
  await pg.evaluate(() => { S.wiz = { ...S.wiz, step: 7, game: { path: FOLDERS.fnv[0][1], ok: true }, store: { path: FOLDERS.store[1][1], free: 142 }, engine: true, fo3: { path: FOLDERS.fo3[0][1], ok: true }, ttw: 'added', prof: 'vplus', far: 'built', q: 'balanced' }; A.wizFinish(); S.profiles.forEach(p => p.groups.forEach(g => g.open = true)); U.toasts = []; U.page = 'mods'; render(); });
  const rows = () => pg.evaluate(() => [...document.querySelectorAll('.mrow[data-mod]')].map(r => r.dataset.mod).sort().join());
  const groups = () => pg.evaluate(() => [...document.querySelectorAll('[data-groupcard] .sech .t')].map(e => e.textContent).join('|'));
  await pg.waitForTimeout(300); await pg.screenshot({ path: `${OUT}/1-mods-vanilla-plus.png` });
  const r1 = await rows(), g1 = await groups();
  await pg.click('[data-act="profMenu"][data-arg="pill"]'); await pg.click('.menu [data-act="profSwitch"][data-arg="hard"]'); await pg.evaluate(() => { U.toasts = []; render(); }); await pg.waitForTimeout(300);
  await pg.screenshot({ path: `${OUT}/2-mods-hardcore.png` });
  ok(r1 === await rows(), 'same mod rows after switch'); ok(g1 === await groups(), 'same groups after switch');
  const on = await pg.evaluate(() => [prof('vplus').mods.filter(m => m.on).length, prof('hard').mods.filter(m => m.on).length]); ok(on[0] !== on[1], 'switches differ');
  // install in Hardcore, then it is off in Vanilla+
  await pg.click('[data-act="install"]'); await pg.click('[data-act="archSel"][data-arg="3"]'); await pg.click('[data-act="instNext"]'); await pg.click('[data-act="instGo"]');
  await pg.waitForFunction(() => document.querySelector('#app').innerText.includes('Installed Weapon Animation'), null, { timeout: 6000 });
  const st = await pg.evaluate(() => [prof('hard').mods.find(m => m.id === 'war').on, prof('vplus').mods.find(m => m.id === 'war')?.on]);
  ok(st[0] === true && st[1] === false, 'new install is off in other profiles');
  // a group made in one profile exists in the other
  await pg.evaluate(() => { U.dlg = { t: 'name', kind: 'grpNew', v: 'Animations', ok: 'Create group' }; A.nameOk(); });
  ok(await pg.evaluate(() => prof('vplus').groups.some(g => g.name === 'Animations')), 'new group is shared');
  await pg.click('[data-act="profMenu"][data-arg="pill"]'); await pg.click('.menu [data-act="profSwitch"][data-arg="vplus"]'); await pg.evaluate(() => { U.toasts = []; S.profiles.forEach(p => p.groups.forEach(g => g.open = true)); render(); const m = document.querySelector('.mscroll'); m.scrollTop = m.scrollHeight; }); await pg.waitForTimeout(300);
  await pg.screenshot({ path: `${OUT}/3-mods-vanilla-plus-after-install.png` });
  // remove asks for all profiles
  await pg.evaluate(() => A.modRemove('war')); await pg.waitForTimeout(200); await pg.screenshot({ path: `${OUT}/4-remove-all-profiles.png` });
  await pg.click('[data-k="dlgok"]'); ok(await pg.evaluate(() => !prof('hard').mods.some(m => m.id === 'war')), 'remove is shared');
  console.log('errors', errs.length, errs, 'failed', fails.length); await b.close(); process.exit(errs.length || fails.length ? 1 : 0);
})().catch(e => { console.error(e); process.exit(2); });
