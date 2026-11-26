// SPDX-License-Identifier: GPL-3.0-only
// usage: node test/run.js  — walks every flow at 1280x720 and writes ../screenshots/ ; exits non-zero on console errors or failed checks
const { chromium } = require('playwright');
const path = require('path'); const fs = require('fs');
const FILE = 'file://' + path.resolve(__dirname, '../index.html');
const OUT = path.resolve(__dirname, '../../screenshots'); fs.mkdirSync(OUT, { recursive: true });
const errors = []; const fails = []; let n = 0;
const ok = (c, m) => { if (!c) { fails.push(m); console.log('FAIL', m); } };
(async () => {
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1366, height: 768 } });
  const pg = await ctx.newPage();
  pg.on('console', m => { if (m.type() === 'error' || m.type() === 'warning') errors.push(m.text()); });
  pg.on('pageerror', e => errors.push('pageerror: ' + e.message));
  const shot = async name => { await pg.waitForTimeout(250); await pg.screenshot({ path: path.join(OUT, `${String(++n).padStart(2, '0')}-${name}.png`) }); };
  const click = async (sel, o) => { await pg.locator(sel).first().click(o); await pg.waitForTimeout(120); };
  const act = (a, arg) => `[data-act="${a}"]${arg !== undefined ? `[data-arg="${arg}"]` : ''}`;
  const text = async () => pg.locator('#app').innerText();
  const waitText = async (t, ms = 15000) => pg.waitForFunction(t => document.querySelector('#app').innerText.includes(t), t, { timeout: ms });
  await pg.goto(FILE); await pg.evaluate(() => localStorage.clear()); await pg.reload();

  // ---- wizard
  await shot('wizard-1-welcome');
  await click(act('wizNext')); await shot('wizard-2-game-finding'); await waitText('Found'); await shot('wizard-2-game-found');
  // reload resumes on the same step
  await pg.reload(); await pg.waitForTimeout(300); ok((await text()).includes('step 2 of 8'), 'wizard resumes after reload');
  // fail path: scenario game missing then browse fixes
  await pg.keyboard.press('Control+Shift+D'); await shot('scenario-panel'); await click(act('scen', 'game')); await pg.keyboard.press('Escape');
  await click(act('findFolder', 'wfnv')); await waitText('was not found'); await shot('wizard-2-game-notfound');
  await click(act('pickFolder', 'wfnv')); await click('[data-act="folderSel"][data-arg="2"]'); await click(act('folderOk')); await shot('wizard-2-game-wrongfolder');
  await click(act('pickFolder', 'wfnv')); await click('[data-act="folderSel"][data-arg="0"]'); await click(act('folderOk'));
  ok((await text()).includes('Found'), 'browse fixes game');
  await click(act('wizNext')); await shot('wizard-3-storage');
  await click(act('pickFolder', 'wstore')); await click('[data-act="folderSel"][data-arg="2"]'); await click(act('folderOk')); await shot('wizard-3-storage-low');
  await click(act('pickFolder', 'wstore')); await click('[data-act="folderSel"][data-arg="0"]'); await click(act('folderOk'));
  await click(act('wizNext')); await shot('wizard-4-engine');
  await click(act('wizEngine')); await pg.waitForTimeout(1500); await shot('wizard-4-engine-installing'); await waitText('Installed'); await click(act('wizNext'));
  await waitText('Found', 5000); await shot('wizard-5-ttw');
  await click(act('wizTtw', 'add')); await pg.waitForTimeout(2000); await shot('wizard-5-ttw-installing'); await waitText('added', 10000); await shot('wizard-5-ttw-added');
  await click(act('wizNext')); await shot('wizard-6-profile'); await click(act('wizProf', 'hard')); await click(act('wizNext'));
  await shot('wizard-7-farview'); await click(act('wizFar', 'build')); await pg.waitForTimeout(2500); await shot('wizard-7-farview-building'); await waitText('Built', 10000);
  await click(act('wizNext')); await shot('wizard-8-done');
  await click(act('wizFinish')); await shot('home-after-setup');
  ok((await text()).includes('Hardcore'), 'chosen profile active');

  // ---- home + play paths
  await click(act('profMenu', 'top')); await shot('profile-menu'); await click('.menu [data-act="profSwitch"][data-arg="beaut"]');
  await shot('home-beautiful');
  await click(act('play')); await pg.waitForTimeout(900); await shot('play-checking');
  await waitText('Before you play', 6000).catch(() => {}); await shot('play-result-beautiful');
  // beautiful far view was never built -> warn
  ok((await text()).includes('Before you play'), 'play warns when far view not built');
  await click(act('playAnyway')); await waitText('Stop', 4000); await pg.waitForTimeout(2200); await shot('play-running');
  await click(act('stopGame')); await shot('home-after-play');
  ok((await text()).includes('Last played'), 'playtime recorded');
  // fail path: engine damaged
  await pg.keyboard.press('Control+Shift+D'); await click(act('scen', 'engine')); await pg.keyboard.press('Control+Shift+D');
  await click(act('play')); await waitText('Play stopped', 6000); await shot('play-failed');
  await click(act('repairEngine')); await waitText('Engine repaired', 6000);

  // ---- far view build
  await click(act('nav', 'far')); await shot('far-never');
  await click(act('farQ', 'balanced')); await click(act('farBuild', '')); await pg.waitForTimeout(3000); await shot('far-building');
  await waitText('Up to date', 12000); await shot('far-done');
  await click(act('farQ', 'high')); await shot('far-stale-quality');
  await click(act('nav', 'farset')); await click('[data-act="fine"][data-arg="tex:1024"]'); await shot('far-finetune'); await click(act('nav', 'far'));
  await click(act('sec', 'details')); await shot('far-details');

  // ---- mods
  await click(act('nav', 'mods')); await shot('mods-list');
  await click(act('sec', 'builtin')); await shot('mods-builtin'); await click(act('sec', 'builtin'));
  await click('[data-act="rowOpen"][data-arg="wfo"]'); await shot('mods-row-open');
  await click(act('modToggle', 'wfo')); await shot('mods-untested-card');
  ok((await text()).includes("hasn't been tested"), 'recommendation card after change');
  await click(act('undo')); ok(!(await text()).includes("hasn't been tested"), 'undo restores tested stable');
  await pg.fill('[data-in="q"]', 'tex'); await shot('mods-search'); await pg.fill('[data-in="q"]', '');
  await click(act('filt', 'off')); await click(act('filt', 'all'));
  // group menu, rename, new group, drag
  const gid = await pg.locator('[data-act="grpMenu"]').first().getAttribute('data-arg');
  await click(`[data-act="grpMenu"][data-arg="${gid}"]`); await shot('mods-group-menu'); await click('.menu [data-act="grpRename"]'); await pg.fill('#nameIn', 'Bug fixes'); await click(act('nameOk'));
  await click(act('newGroup')); await pg.fill('#nameIn', 'Characters'); await shot('mods-new-group'); await click(act('nameOk'));
  const src = pg.locator('[data-grip="mod"][data-id="fco"]'); const dst = pg.locator('[data-groupcard]').filter({ has: pg.locator('button.t', { hasText: /^Characters$/ }) });
  await dst.evaluate(e => e.scrollIntoView({ block: 'center' })); await pg.waitForTimeout(100); const sb = await src.boundingBox(); const db = await dst.boundingBox();
  await pg.mouse.move(sb.x + 5, sb.y + 5); await pg.mouse.down(); await pg.mouse.move(sb.x + 5, sb.y + 40, { steps: 4 }); await pg.mouse.move(db.x + 300, db.y + db.height / 2 + 10, { steps: 8 }); await shot('mods-drag'); await pg.mouse.up(); await pg.waitForTimeout(200);
  ok((await dst.innerText()).includes('Fallout Character Overhaul'), 'drag moves mod into group');
  await shot('mods-after-drag');
  // keyboard move
  await pg.locator('[data-k="mod:nmc"]').focus(); await pg.keyboard.press('Alt+ArrowDown'); await pg.waitForTimeout(150);
  // install with options
  await click(act('install')); await shot('install-pick'); await click('[data-act="archSel"][data-arg="1"]'); await click(act('instNext')); await shot('install-options-1');
  await click(act('instNext')); await shot('install-options-2'); await click(act('instNext')); await shot('install-review'); await click(act('instGo')); await pg.waitForTimeout(900); await shot('install-busy');
  await waitText('Installed The Living Desert', 6000); await shot('install-done');
  await click(act('install')); await click('[data-act="archSel"][data-arg="7"]'); await click(act('instNext')); await shot('install-broken'); await click(act('dlgClose'));
  await pg.evaluate(() => { U.open.tld = true; render(); }); await click(act('modRemove', 'tld')); await shot('mods-remove-confirm'); await click('[data-k="dlgok"]');
  // test your setup
  await click(act('tab', 'test')); await shot('test-tab');
  await click(act('stabGo', 'tab')); await pg.waitForTimeout(2500); await shot('test-stability-running'); await waitText('Tested stable', 12000); await shot('test-stability-passed');
  await click(act('benchGo', '')); await pg.waitForTimeout(2000); await shot('test-bench-running'); await waitText('Benchmark done', 9000); await shot('test-bench-done');
  await click(act('picsGo')); await waitText('New pictures', 8000); await click(act('zoom', '4')); await shot('test-pictures-zoom');
  await click(act('zoom', '1')); await click(act('sec', 'fulllog')); await click(act('logF', 'warn')); await shot('test-log');
  // stability fail path
  await click(act('tab', 'list')); await click(act('modToggle', 'nvtf')); await click(act('tab', 'test'));
  await pg.keyboard.press('Control+Shift+D'); await click(act('scen', 'stab')); await pg.keyboard.press('Control+Shift+D');
  await click(act('stabGo', 'tab')); await waitText('crashed in Freeside', 12000); await shot('test-stability-crashed');
  await pg.keyboard.press('Control+Shift+D'); await click(act('scen', 'stab')); await pg.keyboard.press('Control+Shift+D');

  // ---- load order
  await click(act('profMenu', 'top')); await click('.menu [data-act="profSwitch"][data-arg="hard"]');
  const lib = await pg.evaluate(() => { const [a, b] = [prof('beaut'), prof('hard')]; const k = p => JSON.stringify({ g: p.groups.map(g => [g.id, g.name]), m: p.mods.map(m => [m.id, m.g]).sort() }); return [k(a) === k(b), b.mods.find(m => m.id === 'tld') ? b.mods.find(m => m.id === 'tld').on : 'missing', b.groups.some(g => g.name === 'Characters')]; });
  ok(lib[0] === true, 'profiles share groups and library'); ok(lib[2] === true, 'new group shared'); 
  await click(act('nav', 'mods')); await click(act('tab', 'list')); await shot('mods-hardcore-shared-library');
  await click(act('nav', 'load')); await shot('load-hardcore');
  await click(act('sortGo')); await shot('load-sort-preview'); await click(act('sortApply')); await shot('load-sorted');
  await click(act('nav', 'mods')); await click(act('tab', 'list')); await click(act('modToggle', 'wmx')); await click(act('nav', 'load')); await shot('load-missing-master');
  await click(act('nav', 'home')); await shot('home-missing-master');
  await click(act('nav', 'mods')); await click(act('modToggle', 'wmx'));

  // ---- profiles
  await click(act('profMenu', 'top')); await click('.menu [data-act="profDup"]'); await shot('profile-duplicate'); await click(act('nameOk'));
  ok((await text()).includes('Hardcore copy'), 'duplicate profile');
  await click(act('profMenu', 'top')); await click('.menu [data-act="profRename"]'); await pg.fill('#nameIn', 'Hardcore test'); await click(act('nameOk'));
  await click(act('profMenu', 'top')); await click('.menu [data-act="profDelete"]'); await click('[data-k="dlgok"]');
  await click(act('profMenu', 'top')); await click('.menu [data-act="profNew"]'); await pg.fill('#nameIn', 'Fresh start'); await click(act('nameOk')); await click(act('nav', 'mods')); await shot('mods-empty-profile');

  // ---- TTW flow from Not added: reset TTW
  await pg.evaluate(() => { S.g.ttwInstalled = false; S.g.scen.fo3 = true; commit(); });
  await click(act('profMenu', 'top')); await click('.menu [data-act="profSwitch"][data-arg="hard"]');
  await click(act('nav', 'mods')); await shot('mods-ttw-not-added');
  await click(act('ttwStart')); await pg.waitForTimeout(1800); await shot('ttw-1-notfound');
  await click(act('pickFolder', 'fo3')); await click('[data-act="folderSel"][data-arg="1"]'); await click(act('folderOk')); await shot('ttw-1-found');
  await click(act('ttwNext')); await shot('ttw-2'); await click(act('ttwNext')); await shot('ttw-3'); await click(act('ttwInstall')); await pg.waitForTimeout(2500); await shot('ttw-3-installing');
  await waitText('is in your mods', 10000); await shot('ttw-4-done'); await click(act('ttwClose'));

  // ---- system
  await click(act('nav', 'system')); await pg.waitForTimeout(500); await shot('system-checking'); await waitText('Checked', 4000); await shot('system');
  await click(act('vopen', 'toolbox')); await shot('system-changes');
  await pg.keyboard.press('Control+Shift+D'); await click(act('scen', 'dl')); await pg.keyboard.press('Control+Shift+D');
  await click(act('update')); await waitText('download stopped', 6000); await shot('system-update-failed');
  await pg.keyboard.press('Control+Shift+D'); await click(act('scen', 'dl')); await pg.keyboard.press('Control+Shift+D');
  await click(act('update')); await pg.waitForTimeout(2500); await shot('system-updating'); await waitText('Updated to 0.6.0', 9000); await shot('system-updated');
  await click(act('sec', 'engines')); await click(act('mlUpdate')); await shot('system-engines'); await waitText('4 new messages', 5000);

  // ---- doctor
  await pg.keyboard.press('Control+Shift+D'); await click(act('scen', 'disk')); await pg.keyboard.press('Control+Shift+D');
  await click(act('nav', 'doctor')); await shot('doctor-not-run'); await click(act('docRun')); await pg.waitForTimeout(1500); await shot('doctor-running'); await pg.waitForTimeout(3000); await shot('doctor-results');

  // ---- settings
  await click(act('nav', 'settings')); await shot('settings');
  await click(act('secS', 's_store')); await click(act('secS', 's_prof')); await shot('settings-storage-profile');
  await click(act('secS', 'adv')); await click(act('secS', 's_look')); await click(act('gset', 'hc')); await shot('settings-high-contrast'); await click(act('gset', 'hc'));
  await pg.keyboard.press('Control+Shift+D'); await click(act('scen', 'disk')); await pg.keyboard.press('Control+Shift+D');

  // ---- palette, shortcuts, keyboard
  await pg.keyboard.press('Control+k'); await pg.keyboard.type('load'); await shot('palette'); await pg.keyboard.press('Enter');
  ok((await text()).includes('Load order'), 'palette navigates');
  await pg.locator('body').press('?'); await shot('shortcuts'); await pg.keyboard.press('Escape');

  // ---- reset to setup
  await click(act('nav', 'settings')); await click(act('resetAll')); await click('[data-k="dlgok"]'); await shot('reset-to-wizard');
  ok((await text()).includes("Let's get New Vegas ready"), 'reset runs setup again');
  // skip TTW + later far view path
  await click(act('wizNext')); await waitText('Found'); await click(act('wizNext')); await click(act('wizNext')); await click(act('wizEngine')); await waitText('Installed', 8000); await click(act('wizNext'));
  await click(act('wizTtw', 'skip')); await click(act('wizNext')); await click(act('wizProf', 'vplus')); await click(act('wizNext')); await click(act('wizFar', 'later')); await click(act('wizNext')); await click(act('wizFinish'));
  await shot('home-skipped-ttw-later-far'); await click(act('nav', 'mods')); ok((await text()).includes('Not added'), 'TTW not added after skip');
  await click(act('nav', 'far')); ok((await text()).includes('Not built'), 'far view not built after later');
  // reduced motion
  await pg.emulateMedia({ reducedMotion: 'reduce' }); await click(act('farBuild', '')); await pg.waitForTimeout(800); await shot('far-building-reduced-motion');

  console.log('console errors:', errors.length); errors.forEach(e => console.log('  ', e));
  console.log('failed checks:', fails.length);
  await b.close(); process.exit(errors.length || fails.length ? 1 : 0);
})().catch(e => { console.error(e); console.log('errors so far', errors); process.exit(2); });
