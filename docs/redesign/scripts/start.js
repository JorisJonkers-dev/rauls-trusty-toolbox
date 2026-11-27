// SPDX-License-Identifier: GPL-3.0-only
// usage: node scripts/start.js  — drives the start area of the demo (setup wizard, Home, Play, top bar, Start setup again)
// writes screens/start/NN-*.png; prints PASS/FAIL lines and TEXT lines; exits 1 on any console/page error or failed check
const { chromium } = require('playwright');
const path = require('path'), fs = require('fs');
const ROOT = path.resolve(__dirname, '..');
const FILE = 'file://' + require('path').resolve(__dirname, '../../design/demo/index.html');
const OUT = path.join(ROOT, 'screens', 'start'); fs.mkdirSync(OUT, { recursive: true });
const errors = [], fails = [];
let n = 0;
const ok = (c, m) => { console.log((c ? 'PASS ' : 'FAIL ') + m); if (!c) fails.push(m); };
const say = (k, v) => console.log(`TEXT ${k}: ${v}`);
const act = (a, arg) => `[data-act="${a}"]${arg !== undefined ? `[data-arg="${arg}"]` : ''}`;

async function newRun(b) {
  const ctx = await b.newContext({ viewport: { width: 1366, height: 768 } });
  const pg = await ctx.newPage();
  pg.on('console', m => { if (m.type() === 'error' || m.type() === 'warning') errors.push(m.text()); });
  pg.on('pageerror', e => errors.push('pageerror: ' + e.message));
  const r = {
    pg,
    shot: async name => { await pg.waitForTimeout(300); await pg.locator('#win').screenshot({ path: path.join(OUT, `${String(++n).padStart(2, '0')}-${name}.png`) }); },
    click: async (sel, o) => { await pg.locator(sel).first().click(o); await pg.waitForTimeout(150); },
    text: () => pg.locator('#app').innerText(),
    wait: async (t, ms = 20000) => pg.waitForFunction(t => document.querySelector('#app') && document.querySelector('#app').innerText.includes(t), t, { timeout: ms }),
    gone: async (t, ms = 20000) => pg.waitForFunction(t => !document.querySelector('#app').innerText.includes(t), t, { timeout: ms }),
    fresh: async () => { await pg.goto(FILE); await pg.evaluate(() => localStorage.clear()); await pg.reload(); await pg.waitForTimeout(300); },
    reload: async () => { await pg.reload(); await pg.waitForTimeout(400); },
    scen: async key => { await pg.keyboard.press('Control+Shift+D'); await pg.locator(act('scen', key)).first().click(); await pg.keyboard.press('Control+Shift+D'); await pg.waitForTimeout(150); },
    toasts: async () => pg.locator('.toast').allInnerTexts(),
    main: async () => pg.locator('#main').getAttribute('aria-label'),
  };
  return { ctx, ...r };
}

async function wizardGameAndStorage(r, storeIdx) {
  await r.click(act('wizNext'));                       // Welcome -> Game
  await r.wait('Found');
  await r.click(act('wizNext'));                       // Game -> Storage
  await r.click(act('pickFolder', 'wstore'));
  await r.click(`[data-act="folderSel"][data-arg="${storeIdx}"]`);
  await r.click(act('folderOk'));
}

(async () => {
  const b = await chromium.launch();

  // ================= RUN A: happy path through every step, then Home =================
  {
    const r = await newRun(b); const pg = r.pg;
    await r.fresh();
    await r.shot('wizard-welcome');
    ok((await r.text()).includes("Let's get New Vegas ready."), 'A: welcome heading');
    await r.click(act('wizNext'));
    await r.wait('Found');
    await r.shot('game-found');
    say('A game step', (await r.text()).split('\n').slice(0, 12).join(' | '));
    await r.reload();
    ok((await r.text()).includes('step 2 of 8'), 'A: reload on game step keeps step 2 of 8');
    await r.click(act('wizNext'));
    await r.shot('storage-default');
    ok((await r.text()).includes('Game drive'.length ? '142 GB free' : ''), 'A: default storage is 142 GB free');
    await r.click(act('wizNext'));
    await r.shot('engine-not-installed');
    await r.click(act('wizEngine'));
    await pg.waitForTimeout(1500);
    await r.shot('engine-installing');
    say('A engine installing', (await r.text()).split('\n').slice(0, 20).join(' | '));
    await r.reload();
    ok((await r.text()).includes('Not installed'), 'A: reload during engine install drops the job, shows Not installed');
    await r.click(act('wizEngine'));
    await r.wait('Installed');
    await r.shot('engine-installed');
    await r.click(act('wizNext'));
    await r.wait('Found');
    await r.shot('ttw-decision');
    say('A ttw step', (await r.text()).split('\n').slice(0, 25).join(' | '));
    ok(await pg.locator(act('wizNext') + '[disabled]').count() > 0, 'A: TTW Next disabled until decided');
    await r.click(act('wizTtw', 'add'));
    await pg.waitForTimeout(2000);
    await r.shot('ttw-installing');
    await r.wait('added');
    await r.shot('ttw-added');
    await r.click(act('wizNext'));
    await r.shot('profile-cards');
    say('A profile cards', (await r.text()).split('\n').slice(0, 30).join(' | '));
    await r.click(act('wizProf', 'hard'));
    await r.shot('profile-hard-selected');
    await r.click(act('wizNext'));
    await r.shot('lod-default');
    say('A lod step', (await r.text()).split('\n').slice(0, 20).join(' | '));
    await r.click(act('wizQ', 'balanced'));
    await r.shot('lod-balanced');
    await r.click(act('wizFar', 'build'));
    await pg.waitForTimeout(2500);
    await r.shot('lod-building');
    await r.wait('Built');
    await r.click(act('wizNext'));
    await r.shot('done-step');
    say('A done step', (await r.text()).split('\n').slice(0, 25).join(' | '));
    await r.click(act('wizFinish'));
    await r.wait('Hardcore');
    await r.shot('home-after-setup');
    say('A home', (await r.text()).split('\n').slice(0, 60).join(' | '));
    ok((await r.main()) === 'Home', 'A: finish lands on Home');
    await r.click(act('recDismiss', 'home'));
    await r.gone("hasn't been tested");
    ok(true, 'A: Not now hides the stability banner');
    await r.reload();
    ok(!(await r.text()).includes("hasn't been tested"), 'A: Not now persists across reload');
    await r.ctx.close();
  }

  // ================= RUN B: branches (not found, wrong folder, low space, engine fail, TTW later, LOD later, empty profile) =================
  let homeB = null;
  {
    const r = await newRun(b); const pg = r.pg;
    await r.fresh();
    await r.scen('game');
    await r.click(act('wizNext'));
    await r.wait('was not found');
    await r.shot('game-not-found');
    ok((await r.text()).includes('Fallout: New Vegas was not found'), 'B: not-found banner text');
    await r.click(act('findFolder', 'wfnv'));
    await r.wait('was not found');
    await r.click(act('pickFolder', 'wfnv'));
    await r.shot('picker-game');
    await r.click(act('dlgClose'));
    ok(!(await r.text()).includes('Pick the game folder'), 'B: Cancel closes the picker');
    await r.click(act('pickFolder', 'wfnv'));
    await pg.keyboard.press('Escape');
    await pg.waitForTimeout(150);
    ok(!(await r.text()).includes('Pick the game folder'), 'B: Escape closes the picker');
    await r.click(act('pickFolder', 'wfnv'));
    await r.click('[data-act="folderSel"][data-arg="2"]');
    await r.click(act('folderOk'));
    await r.wait('no FalloutNV.exe');
    await r.shot('game-wrong-folder');
    say('B wrong folder', (await r.text()).split('\n').slice(0, 14).join(' | '));
    ok(await pg.locator(act('wizNext') + '[disabled]').count() > 0, 'B: Next disabled on wrong folder');
    await r.scen('game');
    await r.click(act('pickFolder', 'wfnv'));
    await r.click('[data-act="folderSel"][data-arg="0"]');
    await r.click(act('folderOk'));
    await r.wait('Found');
    await r.shot('game-found-after-browse');
    await r.click(act('wizNext'));
    await r.click(act('pickFolder', 'wstore'));
    await r.click('[data-act="folderSel"][data-arg="2"]');
    await r.click(act('folderOk'));
    await r.wait('Low on space');
    await r.shot('storage-low');
    say('B low storage', (await r.text()).split('\n').slice(0, 20).join(' | '));
    await r.click(act('pickFolder', 'wstore'));
    await r.click('[data-act="folderSel"][data-arg="0"]');
    await r.click(act('folderOk'));
    await r.gone('Low on space');
    await r.click(act('wizNext'));
    await r.scen('dl');
    await r.click(act('wizEngine'));
    await r.wait('The download stopped', 15000);
    await r.shot('engine-failed');
    say('B engine failed', (await r.text()).split('\n').slice(0, 22).join(' | '));
    ok((await r.text()).includes('Try again'), 'B: failed engine install offers Try again');
    await r.scen('dl');
    await r.click(act('wizEngine'));
    await r.wait('Installed');
    await r.shot('engine-retry-ok');
    await r.click(act('wizNext'));
    await r.scen('fo3');
    await r.wait('Not found', 8000);
    await r.shot('ttw-fo3-not-found');
    say('B ttw fo3 missing', (await r.text()).split('\n').slice(0, 25).join(' | '));
    await r.click(act('pickFolder', 'wfo3'));
    await r.click('[data-act="folderSel"][data-arg="0"]');
    await r.click(act('folderOk'));
    await r.wait('Found', 8000);
    await r.click(act('wizTtw', 'skip'));
    await r.shot('ttw-skipped');
    ok((await r.text()).includes('Mods shows TTW as Not added'), 'B: skipped TTW banner text');
    await r.click(act('wizTtw', 'undo'));
    await r.click(act('wizTtw', 'skip'));
    await r.click(act('wizNext'));
    await r.click(act('wizProf', 'empty'));
    await r.shot('profile-empty');
    await r.click(act('wizNext'));
    await r.shot('lod-default-empty');
    say('B lod empty default', (await r.text()).split('\n').slice(0, 14).join(' | '));
    await r.click(act('wizFar', 'later'));
    await r.shot('lod-later');
    await r.click(act('wizFar', 'undo'));
    await r.click(act('wizFar', 'later'));
    await r.click(act('wizNext'));
    await r.click(act('wizFinish'));
    await r.wait('Home');
    await r.shot('home-skipped-setup');
    homeB = await r.text();
    say('B home', homeB.split('\n').slice(0, 60).join(' | '));
    await r.ctx.close();
  }

  // ================= RUN C: Home, top bar, Play flow, stability, Start setup again (on a finished setup, profile Beautiful) =================
  {
    const r = await newRun(b); const pg = r.pg;
    await r.fresh();
    // quick finish: Welcome -> game -> storage -> engine -> TTW skipped -> profile Beautiful -> LOD later -> Done
    await r.click(act('wizNext')); await r.wait('Found');
    await r.click(act('wizNext')); await r.click(act('wizNext'));
    await r.click(act('wizEngine')); await r.wait('Installed');
    await r.click(act('wizNext')); await r.wait('Found');
    await r.click(act('wizTtw', 'skip')); await r.click(act('wizNext'));
    await r.click(act('wizProf', 'hard'));           // Hardcore: not tested yet, uses TTW (skipped here)
    await r.click(act('wizNext'));
    await r.click(act('wizFar', 'later')); await r.click(act('wizNext'));
    await r.click(act('wizFinish')); await r.wait('Hardcore');

    // top bar from Home
    await r.shot('home-hardcore');
    say('C home hardcore', (await r.text()).split('\n').slice(0, 50).join(' | '));
    await r.click(act('profMenu', 'top'));
    await r.shot('profile-menu-open');
    say('C profile menu', (await r.text()).split('\n').slice(0, 25).join(' | '));
    await pg.keyboard.press('Escape'); await pg.waitForTimeout(150);
    ok(!(await r.text()).includes('New profile'), 'C: Escape closes profile menu');
    await r.click(act('palette'));
    await r.shot('search-palette');
    ok(await pg.locator('.dlg.pal').count() === 1, 'C: search field opens palette');
    await pg.keyboard.press('Escape'); await pg.waitForTimeout(150);
    ok(await pg.locator('.dlg.pal').count() === 0, 'C: Escape closes search');
    await pg.keyboard.press('Control+k'); await pg.waitForTimeout(150);
    ok(await pg.locator('.dlg.pal').count() === 1, 'C: Ctrl K opens palette from Home');
    await pg.keyboard.press('Escape');

    // stability banner: three buttons on this setup (Not now, Run stability test, and Revert to last stable when one exists)
    const banner = pg.locator('.banner.rec');
    ok(await banner.count() === 1, 'C: stability banner shown for Hardcore');
    say('C banner buttons', (await banner.first().innerText()).replace(/\n/g, ' | '));
    await r.shot('stability-banner');
    await r.click(act('vRevertStable'));
    await pg.locator('.dlg').first().waitFor({ timeout: 5000 });
    await r.shot('revert-to-last-stable-dialog');
    say('C revert dialog', (await pg.locator('.dlg').first().innerText()).replace(/\n/g, ' | '));
    await r.click(act('dlgClose'));
    await r.click(act('stabGo'));
    await pg.waitForTimeout(600);
    say('C after Run stability test, page', await r.main());
    await r.shot('stability-test-tab');
    await r.click(act('nav', 'home'));
    await pg.waitForTimeout(300);
    say('C home while running banner', (await pg.locator('.banner.rec').innerText()).replace(/\n/g, ' | '));
    await r.shot('stability-running-banner');
    await r.click(act('stabShow'));
    say('C Show test goes to', await r.main());
    await r.click(act('nav', 'home'));
    await r.gone("hasn't been tested", 20000);
    await r.shot('stability-passed-home');
    ok(true, 'C: passed stability test removes the banner');
    await r.click('[data-k="top:game"]');
    say('C top New Vegas chip goes to', await r.main());
    await r.click(act('nav', 'home'));
    await r.click('[data-k="top:engine"]');
    say('C top Engine chip goes to', await r.main());
    await r.click(act('nav', 'home'));
    await r.click(act('profMenu', 'top'));
    await r.click('.menu [data-act="profSwitch"][data-arg="beaut"]');
    await r.wait('Beautiful');
    await r.shot('home-after-profile-switch');
    say('C after switch to Beautiful', (await r.text()).split('\n').slice(0, 45).join(' | '));

    // status card + setup rows: click each row and note where it lands
    const rows = await pg.locator('.rows .r.click').evaluateAll(els => els.map(e => e.innerText.replace(/\n/g, ' | ')));
    say('C setup rows', JSON.stringify(rows));
    for (const id of ['game', 'ttw', 'mods', 'load', 'far']) {
      await r.click(`[data-k="hrow:${id}"]`);
      say('C row ' + id + ' lands on', await r.main());
      await r.shot('row-' + id);
      await r.click(act('nav', 'home'));
    }
    await r.click(`[data-k="hrow:game"]`);
    await pg.keyboard.press('Escape');
    await r.click(act('nav', 'home'));

    // Play flow: warnings path first
    await r.click(act('play'));
    await pg.waitForTimeout(400);
    await r.shot('play-checking');
    say('C checking top', (await pg.locator('.top').innerText()).replace(/\n/g, ' | '));
    await r.click(act('playCancel'));
    ok(!(await r.text()).includes('Checking your setup'), 'C: Cancel stops checking');
    await r.click(act('play'));
    await r.wait('Before you play', 15000).catch(() => {});
    await r.shot('play-warned');
    say('C warned card', (await r.text()).split('\n').slice(0, 40).join(' | '));
    await r.click(act('playCancel'));
    await r.click(act('play'));
    await r.wait('Before you play', 15000).catch(() => {});
    await r.click(act('playAnyway'));
    await r.wait('Playing', 6000);
    await pg.waitForTimeout(3200);
    await r.shot('play-running');
    say('C running top', (await pg.locator('.top').innerText()).replace(/\n/g, ' | '));
    say('C running footer', (await pg.locator('.foot').innerText()).replace(/\n/g, ' | '));
    await r.click(act('stopGame'));
    await r.wait('Added to your playtime');
    await r.shot('play-stopped');
    say('C toast after stop', (await r.toasts()).join(' || '));
    say('C home after stop', (await r.text()).split('\n').slice(0, 40).join(' | '));
    ok((await r.text()).includes('Last played'), 'C: last played shown after stop');

    // failed check: engine damaged
    await r.scen('engine');
    await r.click(act('play'));
    await r.wait('Play stopped', 15000);
    await r.shot('play-failed-engine');
    say('C failed engine', (await r.text()).split('\n').slice(0, 30).join(' | '));
    await r.click(act('repairEngine'));
    await r.wait('Engine repaired', 15000);
    await r.shot('engine-repaired');

    // failed check: game missing (button opens folder picker)
    await r.scen('game');
    await r.click(act('play'));
    await r.wait('Play stopped', 15000);
    await r.shot('play-failed-game');
    say('C failed game', (await r.text()).split('\n').slice(0, 30).join(' | '));
    await r.click('.card [data-act="pickFolder"]');
    await r.shot('picker-from-play');
    await r.click(act('dlgClose'));
    await r.scen('game');

    // Settings: Start setup again (cancel, then confirm)
    await r.click(act('nav', 'settings'));
    await r.click(act('secS', 'adv'));
    say('C settings advanced', (await r.text()).split('\n').slice(-14).join(' | '));
    await r.click(act('resetAll'));
    await r.shot('start-again-confirm');
    say('C reset dialog', (await r.text()).split('\n').slice(-12).join(' | '));
    await r.click(act('dlgClose'));
    ok(!(await pg.locator('.dlg').count()), 'C: Cancel keeps setup');
    await r.click(act('resetAll'));
    await r.click('[data-k="dlgok"]');
    await r.wait('Let');
    await r.shot('after-start-again');
    ok((await r.text()).includes('Welcome') || (await r.text()).includes('Let'), 'C: start again returns to welcome');
    await r.ctx.close();
  }

  // ================= RUN D: TTW add while storage is low (scenario says TTW install is blocked) =================
  {
    const r = await newRun(b); const pg = r.pg;
    await r.fresh();
    await r.click(act('wizNext')); await r.wait('Found');
    await r.click(act('wizNext'));
    await r.click(act('pickFolder', 'wstore')); await r.click('[data-act="folderSel"][data-arg="2"]'); await r.click(act('folderOk'));
    await r.wait('Low on space');
    await r.click(act('wizNext'));
    await r.click(act('wizEngine')); await r.wait('Installed');
    await r.click(act('wizNext')); await r.wait('Found');
    say('D TTW step with 11 GB free', (await r.text()).split('\n').slice(0, 30).join(' | '));
    ok(await pg.locator(act('wizTtw', 'add') + '[disabled]').count() > 0, 'D: Add TTW is disabled with 11 GB free');
    ok((await r.text()).includes('Not enough space for TTW'), 'D: the TTW step says "Not enough space for TTW"');
    await r.shot('ttw-blocked-low-space');
    await r.ctx.close();
  }

  await b.close();
  const uniqueErrors = [...new Set(errors)];
  console.log(`\nSUMMARY screens=${n} fails=${fails.length} console_errors=${uniqueErrors.length}`);
  uniqueErrors.forEach(e => console.log('CONSOLE ' + e));
  if (fails.length || uniqueErrors.length) process.exit(1);
})().catch(e => { console.error('SCRIPT ERROR', e); process.exit(2); });
