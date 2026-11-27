// SPDX-License-Identifier: GPL-3.0-only
// usage: node scripts/tests.js — walks Mods > Test your setup, Versions, profiles and the Home stability banner
// writes screens/tests/NN-<step>.png and prints observations; exits non-zero on console errors or failed checks
const { chromium } = require('playwright');
const path = require('path'); const fs = require('fs');
const FILE = 'file://' + require('path').resolve(__dirname, '../../design/demo/index.html');
const OUT = path.resolve(__dirname, '../screens/tests'); fs.mkdirSync(OUT, { recursive: true });
const errors = []; const fails = []; const obs = []; let n = 0;
const ok = (c, m) => { obs.push(`${c ? 'OK  ' : 'FAIL'} ${m}`); if (!c) fails.push(m); };
const note = m => obs.push('    ' + m);
(async () => {
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1366, height: 768 } });
  const pg = await ctx.newPage();
  pg.on('console', m => { if (m.type() === 'error' || m.type() === 'warning') errors.push(m.text()); });
  pg.on('pageerror', e => errors.push('pageerror: ' + e.message));
  const shot = async name => { await pg.waitForTimeout(300); await pg.locator('#win').screenshot({ path: path.join(OUT, `${String(++n).padStart(2, '0')}-${name}.png`) }); };
  const click = async (sel, o) => { await pg.locator(sel).first().click(o); await pg.waitForTimeout(150); };
  const act = (a, arg) => `[data-act="${a}"]${arg !== undefined ? `[data-arg="${arg}"]` : ''}`;
  const text = () => pg.locator('#app').innerText();
  const has = async t => (await text()).toLowerCase().includes(t.toLowerCase());
  const toasts = async () => [...new Set(await pg.evaluate(() => [...document.querySelectorAll('[class*="toast"]')].map(e => e.innerText.trim()).filter(Boolean)))];
  const waitText = (t, ms = 15000) => pg.waitForFunction(t => document.querySelector('#app').innerText.includes(t), t, { timeout: ms });
  const st = () => pg.evaluate(() => JSON.parse(JSON.stringify(S)));
  const P = () => pg.evaluate(() => { const p = S.profiles.find(x => x.id === S.active); return { name: p.name, stab: p.stab.length, stable: p.stable.length, bench: p.bench.length, pics: p.pics.length, versions: p.versions.length, cur: p.cur, mods: p.mods.filter(m => m.on).length }; });

  await pg.goto(FILE); await pg.evaluate(() => localStorage.clear()); await pg.reload();
  await pg.evaluate(() => { S.wiz.done = true; S.g.ttwInstalled = true; commit(); });
  await click('[data-act="nav"][data-arg="mods"]'); await pg.waitForTimeout(300);

  // ---- Test your setup: initial
  await click(act('tab', 'test')); await shot('test-tab-initial');
  ok(await has('Stability test'), 'Test tab shows Stability test section');
  ok(await has('Benchmark'), 'Test tab shows Benchmark section');
  ok(await has('Location pictures'), 'Test tab shows Location pictures section');
  ok(await has('Log'), 'Test tab shows Log section');
  note('initial stability text: ' + ((await text()).match(/This exact mod set[^\n]*/) || ['none'])[0]);
  note('initial benchmark: ' + ((await text()).includes('No benchmark for') ? 'empty state' : 'has tiles'));
  const hasRec = await pg.locator('.banner.rec').count();
  note('recommendation banner on Test tab: ' + (hasRec ? 'shown' : 'hidden'));

  // stability: cancel path
  await click(act('stabGo', 'tab')); await pg.waitForTimeout(600); await shot('stab-running');
  ok(await has('Loading cells'), 'stability running shows "Loading cells · N of 40"');
  note('running sub-line: ' + ((await text()).match(/\d+ of 20 min[^\n]*/) || ['?'])[0]);
  note('recommendation banner while running: ' + ((await pg.locator('.banner.rec').count()) ? 'shown' : 'hidden'));
  await click(act('cancelJob', 'stab:' + (await pg.evaluate(() => S.active)))); await pg.waitForTimeout(200);
  note('toasts after cancel: ' + JSON.stringify(await toasts()));
  note('stab runs after cancel: ' + (await P()).stab);
  await shot('stab-cancelled');

  // stability: run to pass
  const before = (await P()).stab;
  await click(act('stabGo', 'tab')); await waitText('Stability test passed', 15000).catch(() => {}); await shot('stab-passed');
  ok((await P()).stab === before + 1, 'stability pass appends one run to history');
  note('toasts after pass: ' + JSON.stringify(await toasts()));
  ok(await has('Tested stable') || await has('passed a stability test'), 'pass text visible');

  // known stable list and Forget
  await click('[data-act="sec"][data-arg="stablelist"]'); await shot('known-stable-open');
  note('known stable rows: ' + ((await text()).match(/Passed [^\n]*/g) || []).length);
  await click(act('forgetStable', '0')); await shot('known-stable-forgotten');
  note('toasts after Forget: ' + JSON.stringify(await toasts()));

  // crashed path
  await pg.evaluate(() => { S.g.scen.stab = true; commit(); });
  await click(act('stabGo', 'tab')); await waitText('Stability test crashed', 15000).catch(() => {}); await shot('stab-crashed');
  ok(await has('Crashed'), 'crash row shows "Crashed"');
  note('after crash, full log open: ' + (await pg.evaluate(() => !!U.secs.fulllog)) + ', log filter: ' + (await pg.evaluate(() => U.logF)));
  note('toasts after crash: ' + JSON.stringify(await toasts()));
  await pg.evaluate(() => { S.g.scen.stab = false; commit(); });

  // ---- benchmark
  await click(act('benchGo')); await pg.waitForTimeout(800); await shot('bench-running');
  ok(await has('Running the benchmark route'), 'benchmark running text');
  note('bench running sub-line: ' + ((await text()).match(/(Loading Goodsprings|Walking to Primm|Crossing the Strip|Back to the start)[^\n]*/) || ['?'])[0]);
  await click(act('cancelJob', 'bench:' + (await pg.evaluate(() => S.active)))); await pg.waitForTimeout(200);
  note('bench runs after cancel: ' + (await P()).bench);
  await click(act('benchGo')); await waitText('Benchmark done', 12000).catch(() => {}); await shot('bench-result');
  note('bench tiles: ' + ((await text()).match(/Average\n[^\n]*\n1 % low\n[^\n]*/) || ['?'])[0].replace(/\n/g, ' | '));
  await shot('bench-history-graph');
  note('toasts after bench: ' + JSON.stringify(await toasts()));

  // ---- location pictures
  await click(act('picsGo')); await pg.waitForTimeout(800); await shot('pics-running');
  ok(await has('Taking pictures'), 'pictures running text');
  await waitText('New pictures taken', 12000).catch(() => {}); await shot('pics-side-by-side');
  note('pic count after take: ' + (await P()).pics + ' ; mode ' + (await pg.evaluate(() => U.picMode)));
  await click(act('picMode', 'one')); await shot('pics-single');
  await click(act('picMode', 'two'));
  await click(act('picAll')); await shot('pics-all'); note('all button label: ' + ((await text()).match(/(All \d+|Fewer)/) || ['?'])[0]);
  await click(act('picAll')); await click(act('picPlace', await pg.evaluate(() => PLACES[1][0])));
  await pg.locator('[data-in="picQ"]').fill('freeside'); await pg.waitForTimeout(300); await shot('pics-search');
  note('search: q=' + (await pg.evaluate(() => U.picQ)) + ' ; places shown: ' + (await pg.locator('.places button').allInnerTexts()).join(', ') + ' ; ' + ((await text()).match(/(\d+ matches?|No place matches\.)/) || ['no count text'])[0]);
  await pg.locator('[data-in="picQ"]').fill(''); await pg.waitForTimeout(200);
  await pg.evaluate(() => { U.picQ = ''; render(); });
  const vp = pg.locator('.pic .vp').first(); const box = await vp.boundingBox();
  await pg.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await pg.mouse.wheel(0, -300); await pg.waitForTimeout(200); await shot('pics-zoom-wheel');
  note('after wheel up: zoom=' + (await pg.evaluate(() => U.zoom)));
  await pg.mouse.down(); await pg.mouse.move(box.x + box.width / 2 + 60, box.y + box.height / 2 + 30, { steps: 5 }); await pg.mouse.up(); await shot('pics-pan');
  note('after drag: pan=' + JSON.stringify(await pg.evaluate(() => U.pan)));
  await vp.dblclick(); await pg.waitForTimeout(200); note('after dblclick: zoom=' + (await pg.evaluate(() => U.zoom)) + ' pan=' + JSON.stringify(await pg.evaluate(() => U.pan)));
  await shot('pics-reset');
  for (const z of ['2', '4', '8', '1']) { await click(act('zoom', z)); }
  await click(act('zoom', '8')); note('zoom chip 8 aria-pressed: ' + (await pg.locator(act('zoom', '8')).first().getAttribute('aria-pressed')));
  await click(act('zoom', '1')); await shot('pics-zoom-chips');

  // ---- log
  if (!(await pg.evaluate(() => U.secs.fulllog))) await click('[data-act="sec"][data-arg="fulllog"]'); await shot('log-open');
  await click(act('logF', 'warn')); await shot('log-warnings');
  await click(act('logF', 'err')); await shot('log-errors');
  await click(act('logF', 'all'));
  await click('[data-in="logQ"]'); await pg.keyboard.type('zzz-none'); await pg.waitForTimeout(150); await shot('log-search-none');
  note('no-match text: ' + ((await text()).includes('Nothing matches.') ? 'Nothing matches.' : 'absent'));
  await pg.evaluate(() => { U.logQ = ''; render(); });
  await click('[data-in="logQ"]'); await pg.keyboard.type('freeside'); await pg.waitForTimeout(150); await shot('log-search-freeside');
  await pg.evaluate(() => { U.logQ = ''; render(); });
  await click('.logbox .logl, .rows .r .btn').catch(() => {});

  // ---- Versions
  await click(act('tab', 'versions')); await shot('versions-list');
  note('version rows: ' + await pg.locator('.tlist .mrow').count());
  await click('.tlist .mrow .r.click >> nth=1'); await shot('versions-expanded');
  note('current chip present: ' + (await has('Current')));
  const reverts = await pg.locator('.tlist button', { hasText: /^(Revert|Use this version)/ }).count(); note('Revert/Use buttons: ' + reverts);
  await pg.locator('.tlist button', { hasText: /^(Revert|Use this version)/ }).first().click(); await pg.waitForTimeout(200); await shot('revert-dialog');
  note('dialog: ' + ((await pg.locator('.dlg').innerText().catch(() => '')) || '').replace(/\n/g, ' | '));
  await pg.keyboard.press('Escape'); await pg.waitForTimeout(150);
  ok((await pg.locator('.dlg').count()) === 0, 'Escape closes revert dialog without changing version');
  const curBefore = (await P()).cur;
  await pg.locator('.tlist button', { hasText: /^(Revert|Use this version)/ }).first().click();
  await pg.locator('.dlg [data-k="dlgok"]').click().catch(() => {}); await pg.waitForTimeout(200); await shot('versions-after-revert');
  ok((await P()).cur !== curBefore, 'revert moves current version'); note('toasts after revert: ' + JSON.stringify(await toasts()));
  note('versions count after revert: ' + (await P()).versions + ' (newer versions should remain)');

  // revert to last stable
  await click(act('tab', 'test')); await pg.evaluate(() => { U.tab = 'test'; render(); });
  const lsBtn = pg.locator('.banner.rec [data-act="vRevertStable"]');
  note('Revert to last stable on banner: ' + (await lsBtn.count()));

  // ---- recommendation rules: change the mod set
  await click('[data-act="nav"][data-arg="mods"]'); await click(act('tab', 'list'));
  await click('[data-act="modToggle"]'); await pg.waitForTimeout(200); await click(act('tab', 'test')); await shot('rec-after-toggle');
  note('toast after toggle: ' + JSON.stringify(await toasts()));
  ok(await has("hasn't been tested yet"), 'changing mod set shows "This mod setup hasn\'t been tested yet."');
  note('rec banner text: ' + ((await pg.locator('.banner.rec').innerText().catch(() => '')) || '').replace(/\n/g, ' | '));
  note('stability card text now: ' + ((await text()).match(/(This exact mod set[^\n]*|has no passed[^\n]*)/i) || ['?'])[0]);
  note('Known stable list entries after change: ' + await pg.evaluate(() => S.profiles.find(p => p.id === S.active).stable.length));
  await click('[data-act="tab"][data-arg="list"]'); await click('[data-act="modToggle"]'); await pg.waitForTimeout(200); await click(act('tab', 'test'));
  note('after toggling back, banner count: ' + (await pg.locator('.banner.rec').count()) + ' ; toasts ' + JSON.stringify(await toasts()));
  await shot('rec-after-toggle-back');

  // ---- Home stability banner
  await click('[data-act="nav"][data-arg="home"]'); await shot('home');
  const bannerHome = await pg.locator('.banner.rec').count(); note('Home rec banner count: ' + bannerHome);
  if (bannerHome) {
    await pg.locator('.banner.rec [data-act="stabGo"]').first().click(); await pg.waitForTimeout(400);
    const pageNow = await pg.evaluate(() => U.page + '/' + U.tab);
    ok(pageNow === 'mods/test', 'Home banner Run stability test lands on Mods > Test your setup (got ' + pageNow + ')');
    await shot('home-banner-run-lands-on-test'); await waitText('Stability test passed', 15000).catch(() => {});
    await click('[data-act="nav"][data-arg="home"]');
    const lsHome = pg.locator('.banner.rec [data-act="vRevertStable"]');
    if (await lsHome.count()) { await lsHome.first().click(); await pg.waitForTimeout(200); await shot('home-banner-revert-dialog'); note('home revert dialog: ' + ((await pg.locator('.dlg').innerText().catch(() => '')) || '').replace(/\n/g, ' | ')); await pg.keyboard.press('Escape'); }
    else note('no Revert to last stable on Home banner (no passing older version)');
  }

  // ---- Home revert-to-last-stable with an untested setup
  await click('[data-act="nav"][data-arg="mods"]'); await click(act('tab', 'list')); await click('[data-act="modToggle"]'); await pg.waitForTimeout(200);
  await click('[data-act="nav"][data-arg="home"]'); await pg.waitForTimeout(200); await shot('home-untested-banner');
  const lsH = pg.locator('.banner.rec [data-act="vRevertStable"]');
  note('Home banner Revert to last stable count (untested setup): ' + (await lsH.count()) + ' ; banner: ' + ((await pg.locator('.banner.rec').innerText().catch(() => '')) || '').replace(/\n/g, ' | '));
  if (await lsH.count()) { await lsH.first().click(); await pg.waitForTimeout(200); await shot('home-banner-revert-dialog'); note('dialog: ' + ((await pg.locator('.dlg').innerText().catch(() => '')) || '').replace(/\n/g, ' | ')); await pg.keyboard.press('Escape'); await pg.waitForTimeout(150); ok((await pg.locator('.dlg').count()) === 0, 'Escape closes Home revert dialog'); }
  note('Home page after revert click: ' + (await pg.evaluate(() => U.page + '/' + U.tab)));
  await click('[data-act="nav"][data-arg="mods"]'); await click(act('tab', 'list')); await click('[data-act="modToggle"]'); await pg.waitForTimeout(200);

  // ---- profiles
  const p0 = await P();
  await click(act('profMenu', 'top')); await shot('profile-menu-top');
  note('profile menu text: ' + ((await pg.locator('.menu').innerText().catch(() => '')) || '').replace(/\n/g, ' | '));
  await click('.menu [data-act="profNew"]'); await pg.waitForTimeout(200); await shot('profile-new-dialog');
  await pg.locator('#nameIn').fill(p0.name); await click(act('nameOk'));
  note('new with existing name error: ' + ((await pg.locator('.dlg').innerText().catch(() => '')) || '').replace(/\n/g, ' | '));
  await shot('profile-new-duplicate-name');
  await pg.locator('#nameIn').fill('Test new'); await click(act('nameOk')); await pg.waitForTimeout(200); await shot('profile-new-created');
  note('after create: ' + JSON.stringify(await P()) + ' ; toasts ' + JSON.stringify(await toasts()));
  await click(act('profMenu', 'top')); await click('.menu [data-act="profDup"]'); await pg.waitForTimeout(200); await shot('profile-duplicate-dialog');
  await pg.locator('#nameIn').fill('Test copy'); await click(act('nameOk')); await pg.waitForTimeout(200);
  note('after duplicate: ' + JSON.stringify(await P()));
  await click(act('profMenu', 'top')); await click('.menu [data-act="profRename"]'); await pg.waitForTimeout(200); await shot('profile-rename-dialog');
  await pg.locator('#nameIn').fill('Test renamed'); await click(act('nameOk')); await pg.waitForTimeout(200);
  note('after rename: ' + JSON.stringify(await P()));
  // switch from Load order and LOD
  await click('[data-act="nav"][data-arg="load"]'); await click(act('profMenu', 'pill')); await shot('profile-menu-from-load');
  await click('.menu [data-act="profSwitch"][data-arg="beaut"]'); await pg.waitForTimeout(200); note('switch from Load order to beaut: ' + JSON.stringify(await P()) + ' toasts ' + JSON.stringify(await toasts()));
  await click('[data-act="nav"][data-arg="far"]'); await click(act('profMenu', 'pill')); await shot('profile-menu-from-lod');
  const renamedId = await pg.evaluate(() => S.profiles.find(p => p.name === 'Test renamed').id);
  await click(`.menu [data-act="profSwitch"][data-arg="${renamedId}"]`); await pg.waitForTimeout(200); note('switch from LOD to Test renamed: ' + JSON.stringify(await P()) + ' ; page ' + (await pg.evaluate(() => U.page)));
  await click('[data-act="nav"][data-arg="mods"]');
  await click(act('profMenu', 'top')); await click('.menu [data-act="profSwitch"][data-arg="beaut"]'); await pg.waitForTimeout(200);
  await click(act('profMenu', 'top')); await click('.menu [data-act="profDup"]'); await pg.waitForTimeout(200);
  await pg.locator('#nameIn').fill('Beautiful copy'); await click(act('nameOk')); await pg.waitForTimeout(200); await shot('profile-duplicate-of-beautiful');
  note('duplicate of Beautiful: ' + JSON.stringify(await P()) + ' ; stable copied: ' + (await pg.evaluate(() => S.profiles.find(p => p.id === S.active).stable.length)) + ' ; mods on ' + (await P()).mods);
  note('duplicate versions[0].summary: ' + (await pg.evaluate(() => S.profiles.find(p => p.id === S.active).versions[0].summary)));
  // delete the copy
  await click(act('profMenu', 'top')); await click('.menu [data-act="profDelete"]'); await pg.waitForTimeout(200); await click(act('doDeleteProfile'));
  await pg.waitForTimeout(200); note('after delete: now on ' + (await P()).name + ' ; toasts ' + JSON.stringify(await toasts()));
  await shot('profile-after-delete');
  const names = await pg.evaluate(() => S.profiles.map(p => p.id + '=' + p.name)); note('profile ids: ' + names.join(', '));
  await click('[data-act="nav"][data-arg="mods"]');
  await click(act('profMenu', 'top')); await click('.menu [data-act="profDelete"]').catch(() => {}); await pg.waitForTimeout(200); await shot('profile-delete-confirm');
  note('delete dialog: ' + ((await pg.locator('.dlg').innerText().catch(() => '')) || '').replace(/\n/g, ' | '));
  await pg.keyboard.press('Escape'); await pg.waitForTimeout(150);
  ok((await pg.evaluate(() => S.profiles.length)) >= 2, 'Escape keeps profile');
  // shared vs per profile check
  note('library shared check: mods list count ' + await pg.evaluate(() => S.profiles.map(p => p.mods.length).join(',')));

  // ---- TTW world tabs, Not now
  await pg.evaluate(() => { const p = S.profiles.find(x => x.id === S.active); p.ttwOn = true; S.g.ttwInstalled = true; commit(); });
  await click('[data-act="nav"][data-arg="mods"]'); await click(act('tab', 'test')); await pg.waitForTimeout(200);
  await pg.locator('.pics').first().scrollIntoViewIfNeeded().catch(() => {});
  note('TTW on: world seg present: ' + (await has('Capital Wasteland')) + ' ; places: ' + (await pg.locator('.places button').allInnerTexts()).length);
  await click(act('picWorld', 'cw')); await pg.waitForTimeout(200); await shot('pics-capital-wasteland');
  note('after Capital Wasteland: place ' + (await pg.evaluate(() => U.picPlace)) + ' ; shown ' + (await pg.locator('.places button').allInnerTexts()).slice(0, 8).join(', '));
  await click(act('picWorld', 'nv')); await pg.waitForTimeout(150);
  await pg.evaluate(() => { const p = S.profiles.find(x => x.id === S.active); p.ttwOn = false; commit(); });
  await click('[data-act="nav"][data-arg="mods"]'); await click(act('tab', 'test'));
  note('TTW off: world seg present: ' + (await has('Capital Wasteland')));
  const notNow = pg.locator('.banner.rec [data-act="recDismiss"]');
  if (await notNow.count()) { await notNow.first().click(); await pg.waitForTimeout(200); note('after Not now: banner count ' + (await pg.locator('.banner.rec').count()) + ' ; toasts ' + JSON.stringify(await toasts())); await shot('rec-after-not-now'); }

  if (errors.length) { obs.push('CONSOLE ERRORS:'); errors.forEach(e => obs.push('  ' + e)); }
  fs.writeFileSync(path.resolve(__dirname, '../screens/tests/observations.txt'), obs.join('\n') + '\n');
  console.log(obs.join('\n'));
  await b.close();
  if (errors.length || fails.length) process.exit(1);
})().catch(e => { console.error('SCRIPT ERROR', e); process.exit(2); });
