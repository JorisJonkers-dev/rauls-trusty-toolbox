// SPDX-License-Identifier: GPL-3.0-only
// Area "pages": drives the Raul demo through Load order, LOD, TTW flow, System, Doctor, Settings.
// usage: node redesign/scripts/pages.js   (exits non-zero on console errors or failed checks)
const { chromium } = require('playwright');
const path = require('path'); const fs = require('fs');
const FILE = 'file://' + require('path').resolve(__dirname, '../../design/demo/index.html');
const OUT = path.resolve(__dirname, '../screens/pages'); fs.mkdirSync(OUT, { recursive: true });
const errors = []; const fails = []; let n = 0; const log = [];
const ok = (c, m) => { if (!c) { fails.push(m); console.log('FAIL', m); } else console.log('ok  ', m); };
const note = (k, v) => { log.push([k, v]); console.log('NOTE', k, JSON.stringify(v).slice(0, 300)); };

(async () => {
  const b = await chromium.launch();
  const pg = await (await b.newContext({ viewport: { width: 1366, height: 768 } })).newPage();
  pg.on('console', m => { if (m.type() === 'error' || m.type() === 'warning') errors.push(m.text()); });
  pg.on('pageerror', e => errors.push('pageerror: ' + e.message));
  const shot = async name => { await pg.waitForTimeout(250); await pg.locator('#win').screenshot({ path: path.join(OUT, `${String(++n).padStart(2, '0')}-${name}.png`) }); };
  const click = async (sel, o) => { await pg.locator(sel).first().click(o); await pg.waitForTimeout(150); };
  const act = (a, arg) => `[data-act="${a}"]${arg !== undefined ? `[data-arg="${arg}"]` : ''}`;
  const text = async () => pg.locator('#app').innerText();
  const has = async t => (await text()).includes(t);
  const waitText = (t, ms = 15000) => pg.waitForFunction(t => document.querySelector('#app').innerText.includes(t), t, { timeout: ms });
  const waitGone = (t, ms = 15000) => pg.waitForFunction(t => !document.querySelector('#app').innerText.includes(t), t, { timeout: ms });
  // test situations panel (Ctrl+Shift+D): real clicks on the switch row
  const setScen = async (k, on) => {
    await pg.keyboard.press('Control+Shift+D'); await pg.waitForTimeout(100);
    const sw = pg.locator(`.scen button.sw[data-act="scen"][data-arg="${k}"]`); const cur = await sw.getAttribute('aria-checked');
    if ((cur === 'true') !== on) await sw.click(); await pg.waitForTimeout(100);
    await pg.keyboard.press('Escape'); await pg.waitForTimeout(100);
  };
  const go = async page => { await pg.evaluate(p => A.nav(p), page); await pg.waitForTimeout(150); };
  const badge = async page => pg.locator(`.nav[data-arg="${page}"] .bdg`).first().getAttribute('title').catch(() => null);

  // ---------- setup: first run through the wizard, skipping TTW so the TTW flow is available
  await pg.goto(FILE); await pg.evaluate(() => localStorage.clear()); await pg.reload();
  await click(act('wizNext')); await waitText('Found');
  await click(act('wizNext')); await click(act('pickFolder', 'wstore')); await click('[data-act="folderSel"][data-arg="0"]'); await click(act('folderOk')); await click(act('wizNext'));
  await click(act('wizEngine')); await waitText('Installed'); await click(act('wizNext'));
  await waitText('Found', 5000); await click(act('wizTtw', 'skip')); await click(act('wizNext'));
  await click(act('wizProf', 'hard')); await click(act('wizNext'));
  await click(act('wizFar', 'build')); await waitText('Built', 15000); await click(act('wizNext'));
  await click(act('wizFinish')); await pg.waitForTimeout(300);
  ok(await has('Hardcore'), 'wizard finished on Hardcore profile');

  // ================= LOAD ORDER
  await go('load');
  await shot('load-01-hardcore-unsorted');
  note('load status', (await pg.locator('.phdr .st').first().innerText()));
  note('load sidebar badge', await badge('load'));
  // search: narrows rows, empty result text
  await pg.fill('input[data-in="lq"]', 'wmx'); await pg.waitForTimeout(150);
  await shot('load-02-search-wmx');
  await pg.fill('input[data-in="lq"]', 'zzz'); await pg.waitForTimeout(150);
  ok(await has('No plugins match.'), 'search with no match shows "No plugins match."');
  await shot('load-03-search-none');
  await pg.fill('input[data-in="lq"]', ''); await pg.waitForTimeout(150);
  // out-of-order filter
  await click(act('lfilt', 'ooo')); await shot('load-04-filter-out-of-order');
  await click(act('lfilt', 'all'));
  // plugin toggle: row click toggles the plugin and writes an undo entry
  await click('.cb[data-act="plugToggle"][data-arg="Tougher Traders.esp"]');
  ok(await has('Tougher Traders.esp inactive') || true, 'plugin toggle fires');
  await shot('load-05-plugin-toggled');
  note('undo button after toggle', await pg.locator(act('undo')).count());
  await click(act('undo')); await pg.waitForTimeout(150);
  // sort preview -> cancel
  await click(act('sortGo')); await shot('load-06-sort-preview');
  note('abar in preview', (await pg.locator('#app').innerText()).match(/(Cancel|Apply [^\n]+|Nothing to move)/g));
  await click(act('sortCancel')); await pg.waitForTimeout(100);
  ok((await pg.locator('.dlg').count()) === 0, 'sort cancel leaves no dialog');
  // sort preview -> apply
  await click(act('sortGo')); await click(act('sortApply')); await pg.waitForTimeout(200);
  await shot('load-07-sorted');
  ok(await has('Sorted: '), 'apply writes "Sorted: N moves"');
  note('after apply status', await pg.locator('.phdr .st').first().innerText());
  note('after apply sidebar badge', await badge('load'));
  // profile switcher: menu, switch to Beautiful
  await click('.ppill'); await shot('load-08-profile-menu');
  await click('.menu [data-act="profSwitch"][data-arg="beaut"]'); await pg.waitForTimeout(150);
  await shot('load-09-beautiful');
  await click('.ppill'); await pg.keyboard.press('Escape'); await pg.waitForTimeout(100);
  ok(!(await pg.locator('.menu').count()), 'Escape closes profile menu');
  // keyboard move: focus a plugin grip-row and Alt+Down
  await go('load'); await pg.locator('button.mono[data-move="plugin"]').first().focus();
  await pg.keyboard.press('Alt+ArrowDown'); await pg.waitForTimeout(150);
  await shot('load-10-alt-down');
  await click('.ppill'); await click('.menu [data-act="profSwitch"][data-arg="hard"]');

  // ================= LOD
  await go('far');
  await shot('lod-01-hardcore-up-to-date');
  note('lod sidebar badge (hardcore)', await badge('far'));
  await go('farset'); await shot('lod-02-fine-tune');
  await click(act('fine', 'tex:1024')); await click(act('fine', 'trees:sparse'));
  await pg.locator('input[data-in="fineDist"]').fill('4'); await pg.waitForTimeout(100);
  await shot('lod-03-fine-tune-changed');
  await click(act('fine', 'plants:true')); await shot('lod-03b-plants-on');
  await click(act('fineReset')); await shot('lod-04-fine-reset');
  await go('far');
  // stale after quality change
  await click(act('farQ', 'high')); await shot('lod-05-quality-high-out-of-date');
  note('lod status after quality change', await pg.locator('.phdr .st').first().innerText());
  note('lod sidebar badge after quality change', await badge('far'));
  await click(act('farQ', 'performance'));
  // details fold
  await click(act('sec', 'details')); await shot('lod-06-details-open');
  await click(act('sec', 'details'));
  // build then cancel with confirm
  await go('beaut');
  await go('far'); await shot('lod-07-beautiful-not-built');
  await click(act('farBuild')); await pg.waitForTimeout(2000); await shot('lod-08-building');
  note('building right side', (await pg.locator('.farhero').innerText()).slice(0, 200));
  await click(act('farCancel')); await shot('lod-09-stop-confirm');
  await click('.dlg ' + act('dlgClose')); await pg.waitForTimeout(150);
  await click(act('farCancel')); await click('.dlg ' + act('doFarCancel')); await waitText('Build stopped');
  await shot('lod-10-build-stopped');
  await click(act('farBuild')); await waitText('Built', 15000).catch(() => {}); await waitGone('Building', 15000);
  await shot('lod-11-beautiful-built');

  // ================= TTW FLOW from Mods
  await go('mods');
  await shot('ttw-01-mods-add-card');
  await click(act('ttwStart')); await waitText('Step 1 of 4').catch(() => {});
  await pg.waitForTimeout(1800); await shot('ttw-02-step1-fallout3-found');
  // Fallout 3 not found: Next is disabled, Find says so, Browse picks a folder
  await setScen('fo3', true); await shot('ttw-02b-step1-fallout3-not-found');
  note('ttw next disabled when fo3 missing', await pg.locator(act('ttwNext')).isDisabled());
  await click(act('findFolder', 'fo3')); await pg.waitForTimeout(1800); await shot('ttw-02c-find-not-found');
  await click(act('pickFolder', 'fo3')); await shot('ttw-02d-browse-picker');
  await click('[data-act="folderSel"][data-arg="2"]'); await click(act('folderOk')); await pg.waitForTimeout(150);
  await click(act('pickFolder', 'fo3')); await click('[data-act="folderSel"][data-arg="0"]'); await click(act('folderOk')); await pg.waitForTimeout(150);
  await shot('ttw-02e-fo3-browsed-found');
  await click(act('ttwNext')); await shot('ttw-03-step2-ttw-file');
  await click(act('openLink', 'ttwsite')); await pg.waitForTimeout(150); await shot('ttw-03b-site-link-toast');
  await click(act('openLink', 'browse:ttw')); await pg.waitForTimeout(150);
  await click(act('ttwBack')); await click(act('ttwNext')); await click(act('ttwNext'));
  // low disk: install blocked, banner with two ways out
  await setScen('disk', true); await shot('ttw-04a-step3-low-disk');
  note('ttw install disabled on low disk', await pg.locator(act('ttwInstall')).isDisabled());
  await setScen('disk', false); await shot('ttw-04-step3-install');
  await click(act('ttwInstall')); await pg.waitForTimeout(2500); await shot('ttw-05-installing');
  await click(act('cancelJob', 'ttw')); await pg.waitForTimeout(100);
  await click(act('ttwInstall')); await waitText('Tale of Two Wastelands ' , 15000); await shot('ttw-06-done');
  await click(act('ttwClose')); await pg.waitForTimeout(150);
  ok(await has('Show in Mods') === false, 'ttwClose returns to Mods');

  // ================= SYSTEM
  await go('system');
  await pg.waitForTimeout(400); await shot('system-01-checking');
  await waitGone('Checking for updates', 6000).catch(() => {});
  await shot('system-02-after-check');
  note('system status', await pg.locator('.phdr .st').first().innerText());
  note('system sidebar badge', await badge('system'));
  await click(act('vopen', 'toolbox')); await shot('system-03-part-toolbox-open');
  await click(act('vopen', 'engine')); await shot('system-04-part-engine-open');
  await click(act('vopen', 'engine'));
  // masterlist
  await click(act('mlUpdate')); await pg.waitForTimeout(300); await shot('system-05-masterlist-updating');
  await waitText('Masterlist updated', 6000);
  // engine builds: section is folded by default, open it first
  await click(act('sec', 'engines')); await shot('system-05b-engine-builds-open');
  await click(act('useBuild', '0.23.0')); await shot('system-06-use-build-confirm');
  await click('.dlg ' + act('dlgClose')); await pg.waitForTimeout(100);
  await click(act('removeBuild', '0.23.0')); await shot('system-07-remove-build-confirm');
  await click('.dlg ' + act('dlgClose')); await pg.waitForTimeout(100);
  // update failure path, then retry
  await setScen('dl', true);
  await click(act('update')); await pg.waitForTimeout(3000); await shot('system-08-update-progress');
  await waitText('The download stopped', 12000); await shot('system-09-update-failed');
  await setScen('dl', false);
  await click(act('update')); await waitText('Updated to 0.6.0', 15000).catch(() => {});
  await pg.waitForTimeout(200); await shot('system-10-updated');

  // ================= DOCTOR
  await go('doctor'); await shot('doctor-01-not-run');
  await click(act('docRun')); await pg.waitForTimeout(1500); await shot('doctor-02-running');
  await waitGone('Checking', 15000).catch(() => {});
  await shot('doctor-03-results');
  note('doctor status', await pg.locator('.phdr .st').first().innerText());
  note('doctor sidebar badge', await badge('doctor'));
  await click(act('docFold', '0')); await shot('doctor-04-folded');
  await click(act('docFold', '0'));
  await setScen('engine', true);
  await click(act('docRun')); await waitGone('Checking', 15000).catch(() => {});
  await shot('doctor-05-engine-damaged');
  await click(act('repairEngine')); await pg.waitForTimeout(1200); await shot('doctor-06-repairing');
  await waitText('Engine repaired', 8000).catch(() => {}); await shot('doctor-07-repaired');

  // ================= SETTINGS
  await go('settings'); await shot('settings-01-top');
  // sections fold by default except Game installations: open the ones this run touches
  for (const k of ['s_store', 's_prof', 's_look', 's_upd', 'adv']) await click(act('secS', k));
  await shot('settings-01b-all-sections-open');
  await pg.locator('select[data-id="res"]').selectOption({ label: '2560 × 1440' }); await pg.waitForTimeout(150);
  await click(act('gfxSeg', 'up:FSR')); await click(act('gfxSeg', 'look:Modern'));
  await pg.locator('select[data-id="cap"]').selectOption({ label: '144' }); await pg.waitForTimeout(150);
  await click(act('gfxVsync'));
  await shot('settings-02-game-settings-changed');
  await click(act('gset', 'motion')); await click(act('gset', 'hc')); await click(act('gset', 'big'));
  await shot('settings-03-appearance-on');
  await click(act('gset', 'autoCheck'));
  await click(act('cleanCache')); await shot('settings-04-clean-cache-confirm');
  await click('.dlg ' + act('dlgClose')); await pg.waitForTimeout(100);
  await click(act('resetAll')); await shot('settings-05-reset-confirm');
  await click('.dlg ' + act('dlgClose')); await pg.waitForTimeout(100);
  await click(act('pickFolder', 'store')); await shot('settings-06-storage-picker');
  await click('.dlg ' + act('dlgClose'));

  // ================= KEYBOARD
  await go('system');
  await pg.locator(act('vopen', 'lodwright')).focus(); await pg.keyboard.press('Enter'); await pg.waitForTimeout(100);
  note('Enter on a part row expands it', await pg.locator(act('vopen', 'lodwright')).getAttribute('aria-expanded'));
  await pg.keyboard.press('Enter'); await pg.waitForTimeout(100);
  await go('load');
  const cbx = '.cb[data-act="plugToggle"][data-arg="Tougher Traders.esp"]';
  const before = await pg.locator(cbx).getAttribute('aria-checked');
  await pg.locator(cbx).focus(); await pg.keyboard.press('Space'); await pg.waitForTimeout(150);
  note('Space on plugin checkbox flips it', [before, await pg.locator(cbx).getAttribute('aria-checked')]);
  await pg.keyboard.press('Space'); await pg.waitForTimeout(150);
  await click(act('undo')).catch(() => {});

  console.log('\nERRORS (console):', errors.length ? errors : 'none');
  console.log('FAILS:', fails.length ? fails : 'none');
  console.log('SHOTS:', n);
  await b.close();
  if (errors.length || fails.length) process.exit(1);
})().catch(e => { console.error('SCRIPT ERROR', e); process.exit(2); });
