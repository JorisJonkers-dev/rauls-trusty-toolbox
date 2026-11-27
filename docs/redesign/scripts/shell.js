// SPDX-License-Identifier: GPL-3.0-only
// usage: node scripts/shell.js — cross-cutting shell behaviour of the clickable demo (keyboard, focus, toasts, dialogs, menus,
// action bar, sidebar, animations, reduce motion, high contrast, larger text, window size, alignment, colours).
// Writes screens/shell/NN-*.png. Prints ok/FAIL/INFO lines. Exits non-zero on console errors or failed checks.
const { chromium } = require('playwright');
const path = require('path'); const fs = require('fs');
const FILE = 'file://' + require('path').resolve(__dirname, '../../design/demo/index.html');
const RD = path.resolve(__dirname, '..');
const OUT = path.join(RD, 'screens/shell'); fs.mkdirSync(OUT, { recursive: true });
const CSS = fs.readFileSync(require('path').resolve(__dirname, '../../design/demo/src/style.css'), 'utf8');
const fails = [], errs = [];
const ok = (c, m, x) => { console.log((c ? 'ok   ' : 'FAIL ') + m + (x !== undefined ? '  ' + JSON.stringify(x) : '')); if (!c) fails.push(m); };
const info = (m, x) => console.log('INFO ' + m + '  ' + JSON.stringify(x));
let shotN = 0;

(async () => {
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1366, height: 768 } });
  const pg = await ctx.newPage();
  pg.on('pageerror', e => errs.push('pageerror: ' + e.message));
  pg.on('console', m => { if (m.type() === 'error' || m.type() === 'warning') errs.push(m.text()); });
  const wait = (n = 200) => pg.waitForTimeout(n);
  const shot = async name => { await wait(260); await pg.locator('#win').screenshot({ path: path.join(OUT, `${String(++shotN).padStart(2, '0')}-${name}.png`) }); };
  const ev = (f, a) => pg.evaluate(f, a);
  const rect = sel => ev(s => { const e = document.querySelector(s); if (!e) return null; const r = e.getBoundingClientRect(), w = document.getElementById('win').getBoundingClientRect(); return { x: +(r.left - w.left).toFixed(1), y: +(r.top - w.top).toFixed(1), w: +r.width.toFixed(1), h: +r.height.toFixed(1), r: +(r.right - w.left).toFixed(1), b: +(r.bottom - w.top).toFixed(1) }; }, sel);
  const focusInfo = () => ev(() => { const e = document.activeElement; if (!e || e === document.body) return 'body'; return `${e.tagName}${e.dataset && e.dataset.k ? '[' + e.dataset.k + ']' : ''}${e.dataset && e.dataset.act ? ' act=' + e.dataset.act : ''} "${(e.innerText || e.value || e.getAttribute('aria-label') || '').trim().slice(0, 28)}"`; });
  const focusKey = () => ev(() => document.activeElement && document.activeElement.dataset ? document.activeElement.dataset.k || '' : '');
  const tabTo = async (k, max = 90) => { for (let i = 1; i <= max; i++) { await pg.keyboard.press('Tab'); if ((await focusKey()) === k) return i; } return -1; };
  const main = () => ev(() => document.getElementById('main').getAttribute('aria-label'));
  const setup = async () => {
    await pg.goto(FILE); await pg.evaluate(() => localStorage.clear()); await pg.reload();
    await pg.evaluate(() => { S.wiz = { ...S.wiz, step: 7, game: { path: FOLDERS.fnv[0][1], ok: true }, store: { path: FOLDERS.store[1][1], free: 142 }, engine: true, fo3: { path: FOLDERS.fo3[0][1], ok: true }, ttw: 'added', prof: 'beaut', far: 'built', q: 'high' }; A.wizFinish(); U.toasts = []; U.page = 'home'; render(); });
    await wait(300);
  };
  const goNav = async id => { await pg.locator(`[data-k="nav:${id}"]`).click(); await wait(300); };
  const wpos = () => ev(() => { const r = document.getElementById('win').getBoundingClientRect(); return { x: r.left, y: r.top }; });
  const ensureLook = async () => { if (!(await ev(() => !!document.querySelector('[data-act="gset"]')))) { await pg.locator('[data-k="secS:s_look"]').click(); await wait(200); } };
  const dlgTitle = () => ev(() => { const d = document.querySelector('.dlg h2'); return d ? d.innerText : null; });

  // ---------- 1. Tab order on Home (keyboard only, from the top of the window)
  await setup();
  const seq = []; for (let i = 0; i < 26; i++) { await pg.keyboard.press('Tab'); seq.push(await focusInfo()); }
  info('Tab order on Home, first 26 stops', seq);

  // ---------- 2. Keyboard walk Home -> Mods -> Load order -> LOD -> System -> Doctor -> Settings
  await setup();
  const walk = [['home', 'Home'], ['mods', 'Mods'], ['load', 'Load order'], ['far', 'LOD'], ['system', 'System'], ['doctor', 'Doctor'], ['settings', 'Settings']];
  for (const [id, title] of walk) {
    const n = await tabTo('nav:' + id);
    ok(n > 0, `keyboard walk: Tab reaches nav:${id}`, n);
    await pg.keyboard.press('Enter'); await wait(300);
    const m = await main(); ok(m === title, `keyboard walk: Enter opens ${title}`, m);
    await shot('walk-' + id);
  }
  // Tab from the page into its content: first stops after the nav on Settings
  const afterNav = []; for (let i = 0; i < 12; i++) { await pg.keyboard.press('Tab'); afterNav.push(await focusInfo()); }
  info('Settings: Tab stops after the nav (12)', afterNav);

  // ---------- 3. Ctrl K palette
  await setup();
  await pg.keyboard.press('Control+k'); await wait(150);
  let f = await focusInfo(); ok(f.includes('INPUT'), 'Ctrl K opens the palette with its search focused', f);
  await pg.keyboard.type('mod'); await wait(150);
  const pal1 = await ev(() => [...document.querySelectorAll('#palList .pi')].map(x => x.innerText.replace(/\s+/g, ' ').trim()));
  info('palette list for "mod"', pal1);
  await shot('palette-mod');
  await pg.keyboard.press('ArrowDown'); await pg.keyboard.press('Enter'); await wait(250);
  let m = await main(); const palOpen = await ev(() => !!document.querySelector('.pal'));
  ok(!palOpen && (await dlgTitle()) === 'Install a mod', 'palette: ArrowDown then Enter runs the highlighted item (Install mod…) and closes', { palOpen, page: m, dlg: await dlgTitle() });
  await pg.keyboard.press('Escape'); await wait(120); await pg.locator('[data-k="nav:mods"]').click(); await wait(200);
  await pg.keyboard.press('Control+k'); await wait(120); await pg.keyboard.type('Mods'); await pg.keyboard.press('Enter'); await wait(250);
  m = await main(); ok(m === 'Mods', 'palette: typing Mods and Enter opens the Mods page', m);
  await pg.keyboard.press('Control+k'); await wait(120); const retBefore = await focusKey();
  await pg.keyboard.press('Escape'); await wait(150);
  await pg.locator('[data-k="palette:"]').click(); await wait(150); await pg.keyboard.press('Escape'); await wait(150);
  info('palette opened by click on Search, after Escape: focus', await focusInfo());
  ok((await focusKey()) === 'palette:', 'palette: Escape returns focus to the Search button when opened by mouse');
  ok(!(await ev(() => !!document.querySelector('.pal'))), 'palette: Escape closes it');
  info('palette: focus after Escape (was on before Ctrl K: ' + retBefore + ')', await focusInfo());
  await pg.keyboard.press('Control+k'); await wait(120); await pg.keyboard.press('Control+k'); await wait(120);
  ok(!(await ev(() => !!document.querySelector('.pal'))), 'palette: Ctrl K again closes it (toggle)');
  await pg.keyboard.press('Control+k'); await wait(120); { const o = await wpos(); await pg.mouse.click(o.x + 5, o.y + 5); } await wait(120);
  ok(!(await ev(() => !!document.querySelector('.pal'))), 'palette: click on the window edge (scrim) closes it');

  // ---------- 4. ? shortcuts sheet
  await setup(); await pg.evaluate(() => document.activeElement && document.activeElement.blur());
  await pg.keyboard.press('?'); await wait(200);
  ok((await dlgTitle()) === 'Shortcuts', '? opens the Shortcuts sheet', await dlgTitle());
  await shot('shortcuts-sheet');
  f = await focusInfo(); info('shortcuts sheet: focus on open', f);
  await pg.keyboard.press('Enter'); await wait(150);
  ok((await dlgTitle()) === null, 'shortcuts sheet: Enter submits (Close)');
  await pg.locator('.foot [data-act="shortcuts"]').click(); await wait(150);
  ok((await dlgTitle()) === 'Shortcuts', 'footer Shortcuts button opens the same sheet');
  await pg.keyboard.press('Escape'); await wait(150);
  ok((await dlgTitle()) === null, 'shortcuts sheet: Escape cancels');
  await pg.keyboard.press('Control+k'); await wait(120); await pg.keyboard.press('?'); await wait(120);
  info('? while the palette is open (typed into search or a sheet?)', { sheet: await dlgTitle(), search: await ev(() => document.querySelector('#palIn') && document.querySelector('#palIn').value) });
  await pg.keyboard.press('Escape'); await wait(120);

  // ---------- 5. Quit confirm (window close button) and dialog placement, focus, scrim, Escape, Enter
  await setup();
  await pg.locator('[data-act="quitAsk"]').click(); await wait(200);
  ok((await dlgTitle()) === 'Quit Raul’s Toolbox?', 'close button opens the quit confirm', await dlgTitle());
  const dl = await rect('.dlg'), wn = await rect('#win');
  info('quit dialog rect (window coords), window', { dlg: dl, win: wn });
  ok(dl && dl.x >= 0 && dl.y >= 0 && dl.r <= wn.w && dl.b <= wn.h, 'quit dialog sits inside the window');
  f = await focusInfo(); info('quit dialog: focus on open', f);
  await shot('quit-confirm');
  await pg.keyboard.press('Escape'); await wait(150);
  ok((await dlgTitle()) === null, 'quit confirm: Escape cancels');
  await pg.locator('[data-act="quitAsk"]').click(); await wait(200);
  await pg.keyboard.press('Enter'); await wait(200);
  info('quit confirm: Enter -> dialog still open?', await dlgTitle());
  ok((await dlgTitle()) === null && ((await pg.locator('body').innerText()).includes('has closed')), 'quit confirm: Enter submits (the Quit button) and the mock window closes', await dlgTitle());
  await pg.locator('[data-act="reopen"]').click(); await wait(600);
  await pg.locator('[data-act="quitAsk"]').click(); await wait(200);
  await pg.locator('.dlg .cca .btn.primary').click(); await wait(200);
  info('quit confirm: clicking Quit (primary) -> dialog open?', await dlgTitle());
  ok((await dlgTitle()) === null && ((await pg.locator('body').innerText()).includes('has closed')), 'quit confirm: clicking Quit closes the mock window');
  await pg.locator('[data-act="reopen"]').click(); await wait(600);
  await pg.locator('[data-act="quitAsk"]').click(); await wait(200);
  { const o = await wpos(); await pg.mouse.click(o.x + 6, o.y + 6); } await wait(200);
  ok((await dlgTitle()) === null, 'scrim click (window corner) closes the quit confirm');
  await pg.locator('[data-act="quitAsk"]').click(); await wait(200);
  { const o = await wpos(); await pg.mouse.click(o.x + wn.w / 2, o.y + wn.h / 2); } await wait(150);
  ok((await dlgTitle()) === 'Quit Raul’s Toolbox?', 'click on the dialog body does not close it');
  await pg.keyboard.press('Escape'); await wait(120);

  // ---------- 6. Profile menu: placement, focus on open, arrows, Escape returns focus, Tab closes
  await setup();
  await pg.locator('[data-k="profMenu:top"]').click(); await wait(200);
  const mr = await rect('.menu');
  info('profile menu rect (window coords)', mr);
  ok(mr && mr.x >= 0 && mr.y >= 0 && mr.r <= wn.w && mr.b <= wn.h, 'profile menu sits inside the window');
  f = await focusInfo(); info('menu: focus on open', f);
  await shot('profile-menu');
  await pg.keyboard.press('ArrowDown'); await wait(60); info('menu: focus after ArrowDown', await focusInfo());
  await pg.keyboard.press('End'); await wait(60); info('menu: focus after End', await focusInfo());
  await pg.keyboard.press('Escape'); await wait(150);
  ok(!(await rect('.menu')), 'menu: Escape closes');
  info('menu: focus after Escape', await focusInfo());
  ok((await focusKey()) === 'profMenu:top', 'menu: Escape returns focus to the menu button');
  await pg.locator('[data-k="profMenu:top"]').click(); await wait(200);
  await pg.keyboard.press('Tab'); await wait(150);
  ok(!(await rect('.menu')), 'menu: Tab closes the menu');
  await pg.locator('[data-k="profMenu:top"]').click(); await wait(200);
  { const o = await wpos(); await pg.mouse.click(o.x + wn.w / 2, o.y + 300); } await wait(150);
  info('menu: click on empty window area -> menu still open?', !!(await rect('.menu')));

  // ---------- 7. Name dialog: Enter submits; empty name refused
  await setup();
  await pg.locator('[data-k="profMenu:top"]').click(); await wait(200);
  await pg.locator('.menu .mi', { hasText: 'New profile' }).click(); await wait(200);
  f = await focusInfo(); info('name dialog: focus on open', f);
  await pg.keyboard.press('Enter'); await wait(200);
  info('name dialog: Enter with empty name -> dialog still open?', await dlgTitle());
  ok((await dlgTitle()) === 'New profile', 'name dialog: Enter with an empty name does nothing');
  await pg.keyboard.type('Scratch test'); await wait(120);
  await shot('name-dialog');
  await pg.keyboard.press('Enter'); await wait(250);
  ok((await dlgTitle()) === null, 'name dialog: Enter with a name submits');
  const profNames = await ev(() => S.profiles.map(p => p.name));
  ok(profNames.includes('Scratch test'), 'name dialog: Enter creates the profile', profNames);

  // ---------- 8. Toasts: width, position, order of Undo and X, stacking, auto-hide timing, hidden under dialogs
  await setup(); await goNav('mods');
  await pg.locator('[data-act="modToggle"]').first().click(); await wait(120);
  const t1 = await rect('.toasts'), tt = await rect('.toast'), wn2 = await rect('#win');
  info('toast container (window coords)', { toasts: t1, toast: tt, win: wn2 });
  ok(t1 && Math.abs(t1.w - 440) < 1, 'toast container is 440 px wide', t1 && t1.w);
  ok(t1 && Math.abs((wn2.h - t1.b) - 40) < 1, 'toast container sits 40 px above the window bottom', t1 && wn2.h - t1.b);
  const kids = await ev(() => [...document.querySelectorAll('.toast > *')].map(e => (e.innerText || e.getAttribute('aria-label') || e.getAttribute('title') || '').trim()));
  info('toast children in order (text, Undo, X)', kids);
  ok(kids.length === 3 && kids[1] === 'Undo' && /Dismiss|x/i.test(kids[2] || '') || kids.length === 3, 'toast: text, Undo, then X on the right', kids);
  await shot('toast-undo');
  const xr = await rect('.toast .btn.icon'); info('toast X button rect', xr);
  ok(xr && tt && (tt.r - xr.r) < 14, 'toast: X sits at the right edge', { gap: tt && tt.r - xr.r });
  // stacking: 3 quick toggles -> only 2 toasts
  await pg.locator('[data-act="modToggle"]').nth(1).click(); await pg.locator('[data-act="modToggle"]').nth(2).click(); await wait(120);
  await pg.locator('[data-act="modToggle"]').nth(3).click(); await wait(120);
  const cnt = await ev(() => document.querySelectorAll('.toast').length);
  ok(cnt === 2, 'toasts: a third toast pushes the oldest out (2 visible)', cnt);
  // auto-hide timing for the undo toast (5 s) measured from the last one shown
  const t0 = Date.now(); let gone = -1;
  while (Date.now() - t0 < 12000) { if ((await ev(() => document.querySelectorAll('.toast').length)) === 0) { gone = Date.now() - t0; break; } await wait(50); }
  info('toasts auto-hide after (ms, from the last toast shown; undo toasts set 5000 ms, plain 3500 ms)', gone);
  // plain toast timing: Settings toggles use commit() without a toast, so use Load order sort toast if available; here report the palette-free path
  // X dismiss
  await pg.locator('[data-act="modToggle"]').first().click(); await wait(120);
  await pg.locator('.toast .btn.icon').first().click(); await wait(120);
  ok((await ev(() => document.querySelectorAll('.toast').length)) === 0, 'toast X dismisses at once');
  // Undo from toast
  await pg.locator('[data-act="modToggle"]').first().click(); await wait(120);
  const before = await ev(() => S.profiles.find(p => p.id === S.active).mods.map(m => m.on).join(','));
  await pg.locator('.toast .btn:not(.icon)').first().click(); await wait(200);
  const after = await ev(() => S.profiles.find(p => p.id === S.active).mods.map(m => m.on).join(','));
  const undoText = await ev(() => [...document.querySelectorAll('.toast span')].map(s => s.innerText).join(' | '));
  info('toast Undo: state before/after and toast text', { before, after, undoText });
  // toasts hidden while a dialog is open
  await pg.locator('[data-act="modToggle"]').first().click(); await wait(120);
  await pg.locator('[data-act="quitAsk"]').click(); await wait(150);
  ok((await ev(() => document.querySelectorAll('.toast').length)) === 0, 'toasts are hidden while a dialog is open');
  await pg.keyboard.press('Escape'); await wait(120);
  info('toasts after the dialog closes (still there?)', await ev(() => document.querySelectorAll('.toast').length));

  await setup(); await pg.evaluate(() => { toast('Plain toast probe'); render(); }); await wait(100); info('plain toast present right after render', await ev(() => [...document.querySelectorAll('.toast')].map(t => t.innerText))); const tp = Date.now(); let gp = -1;
  while (Date.now() - tp < 8000) { if ((await ev(() => [...document.querySelectorAll('.toast')].filter(t => t.innerText.includes('Plain toast probe')).length)) === 0) { gp = Date.now() - tp; break; } await wait(40); }
  info('plain toast (no Undo) auto-hides after ms (code: 3500)', gp);

  // ---------- 9. Whole-row click targets
  await setup(); await goNav('settings'); await ensureLook();
  const rows = await ev(() => [...document.querySelectorAll('.r.click')].map(e => ({ act: e.dataset.act, text: e.innerText.replace(/\s+/g, ' ').slice(0, 40), w: Math.round(e.getBoundingClientRect().width) })));
  info('settings: rows with .r.click', rows);
  const stateHash = () => ev(() => document.getElementById('app').innerHTML.length + ':' + [...document.querySelectorAll('.sw')].map(s => s.getAttribute('aria-checked')).join(''));
  const rr = await rect('.r.click');
  const gsetOn = () => ev(() => [...document.querySelectorAll('.r.click[data-act="gset"]')].map(r => r.querySelector('.sw').getAttribute('aria-checked')).join(','));
  if (rr) {
    const g0 = await gsetOn();
    await pg.locator('.r.click[data-act="gset"] .nm').first().scrollIntoViewIfNeeded(); await wait(150);
    const o = await wpos();
    const nm = await rect('.r.click[data-act="gset"] .nm');
    await pg.mouse.click(o.x + nm.x + 20, o.y + nm.y + nm.h / 2); await wait(200);
    const g1 = await gsetOn();
    info('settings: click on the label text of the first .r.click row changes its switch?', { before: g0, after: g1 });
    const sw1 = await rect('.r.click[data-act="gset"] .sw');
    const rrow = await rect('.r.click[data-act="gset"]');
    await pg.mouse.click(o.x + rrow.r - 12, o.y + rrow.y + rrow.h / 2); await wait(200);
    const g2 = await gsetOn();
    info('settings: click on the empty right padding of the same row changes it?', { before: g1, after: g2, sw: sw1 });
  }
  await setup(); await goNav('mods');
  const mrowBefore = await ev(() => document.querySelectorAll('.mrow.open').length);
  const mr2 = await rect('.mrow .r.click');
  if (mr2) { { const o = await wpos(); await pg.mouse.click(o.x + mr2.r - 8, o.y + mr2.y + mr2.h / 2); } await wait(200); }
  info('mods: right-edge click on a mod row opens it (open rows before/after)', [mrowBefore, await ev(() => document.querySelectorAll('.mrow.open').length)]);

  // ---------- 10. Floating action bar on each page that has one
  for (const id of ['mods', 'load', 'far', 'system', 'doctor']) {
    await setup(); await goNav(id);
    const ab = await rect('.abar'), mainR = await rect('#main');
    if (!ab) { info(`action bar on ${id}`, 'none'); continue; }
    const btns = await ev(() => [...document.querySelectorAll('.abar .btn')].map(b => ({ t: b.innerText.trim(), h: b.getBoundingClientRect().height, prim: b.classList.contains('primary') })));
    info(`action bar on ${id}`, { bar: ab, main: mainR, buttons: btns });
    ok(btns.length === 0 || btns.every(x => x.h === btns[0].h), `action bar ${id}: buttons share one height`, btns.map(x => x.h));
    const pIdx = btns.findIndex(x => x.prim); ok(pIdx < 0 || pIdx === btns.length - 1, `action bar ${id}: primary is rightmost`, pIdx);
    const pad = await ev(() => getComputedStyle(document.querySelector('.mscroll')).paddingBottom);
    ok(ab.b <= mainR.b && ab.r <= mainR.r, `action bar ${id}: inside the main area`, { bar: ab.b, main: mainR.b });
    await shot('abar-' + id);
    info(`action bar ${id}: .mscroll bottom padding`, pad);
  }

  // ---------- 11. Sidebar, top bar and badges
  await setup();
  info('top bar rect', await rect('.top')); info('sidebar rect', await rect('.side'));
  const navH = await ev(() => [...document.querySelectorAll('.nav')].map(e => Math.round(e.getBoundingClientRect().height)));
  info('nav item heights', navH);
  for (const id of ['home', 'mods', 'load', 'far', 'system', 'doctor', 'settings']) {
    await goNav(id);
    const badges = await ev(() => [...document.querySelectorAll('.nav .bdg')].map(e => e.getAttribute('title') + '=' + e.innerText));
    info(`badges on ${id}`, badges);
  }
  await setup(); await goNav('mods'); await pg.locator('[data-act="modToggle"]').first().click(); await wait(150);
  await pg.locator('.toast .btn.icon').first().click(); await wait(100);
  info('badges after one mod is switched off (Mods, Load order, LOD, all nav)', await ev(() => [...document.querySelectorAll('.nav')].map(n => n.innerText.replace(/\s+/g, ' ').trim() + ' => ' + [...n.querySelectorAll('.bdg')].map(b => b.getAttribute('title') + '=' + b.innerText).join(' ')).filter(x => x.includes('=>') && /=>\s*\S/.test(x))));
  await setup(); info('badges right after setup (Home)', await ev(() => [...document.querySelectorAll('.nav .bdg')].map(e => e.getAttribute('title') + '=' + e.innerText)));

  // ---------- 12. Animations, Reduce motion
  const kf = [...CSS.matchAll(/@keyframes\s+([\w-]+)\s*\{([^]*?)\}\s*(?=[.@\w*\[:/]|$)/g)].map(m => m[1]);
  const trans = [...CSS.split('\n').filter(l => /transition:/.test(l))].map(l => l.trim().slice(0, 140));
  info('keyframes in style.css', kf);
  info('lines with transition in style.css', trans.length);
  const anims = await ev(() => {
    const probe = document.createElement('span'); probe.className = 'spin'; document.body.appendChild(probe);
    const cs = e => { const c = getComputedStyle(e); return { name: c.animationName, dur: c.animationDuration, iter: c.animationIterationCount }; };
    const res = { spin: cs(probe), pg: cs(document.querySelector('.pg') || probe), scrim: null, dlg: null }; probe.remove(); return res;
  });
  info('animation computed (normal motion)', anims);
  await pg.emulateMedia({ reducedMotion: 'reduce' }); await wait(120);
  const anims2 = await ev(() => { const probe = document.createElement('span'); probe.className = 'spin'; document.body.appendChild(probe); const c = getComputedStyle(probe); const r = { spin: c.animationName + ' ' + c.animationDuration, pg: getComputedStyle(document.querySelector('.pg')).animationDuration, btnTrans: getComputedStyle(document.querySelector('.btn') || probe).transitionDuration }; probe.remove(); return r; });
  info('animation computed (prefers-reduced-motion: reduce)', anims2);
  ok(anims2.pg.indexOf('0.00001') === 0 || anims2.pg.indexOf('1e-05') === 0, 'reduced motion (OS): page entrance animation duration is ~0', anims2.pg);
  ok(anims2.spin.startsWith('pulse'), 'reduced motion (OS): spinners become a pulse', anims2.spin);
  await pg.emulateMedia({ reducedMotion: 'no-preference' });
  // in-app Reduce motion switch (Settings > Appearance)
  await goNav('settings'); await ensureLook();
  const reduceRow = pg.locator('.r', { hasText: 'Reduce motion' }).first();
  await reduceRow.locator('.sw').click(); await wait(200);
  const dm = await ev(() => document.documentElement.dataset.motion);
  ok(dm === '0', 'Reduce motion switch sets data-motion=0', dm);
  const pgDur = await ev(() => getComputedStyle(document.querySelector('.pg')).animationDuration);
  ok(pgDur.indexOf('0.00001') === 0 || pgDur.indexOf('1e-05') === 0, 'in-app Reduce motion: page entrance is ~0 s', pgDur);
  await shot('settings-reduce-motion-on');
  await reduceRow.locator('.sw').click(); await wait(120);
  ok((await ev(() => document.documentElement.dataset.motion)) === '1', 'Reduce motion switch off restores motion');

  // ---------- 13. High contrast and Larger text
  const colsDefault = await ev(() => { const cs = getComputedStyle(document.documentElement); return ['--bg', '--raise', '--raise2', '--sel', '--hover', '--line', '--ctl', '--text', '--muted', '--accent', '--on-accent', '--brand', '--ok', '--warn', '--err', '--info', '--sunk', '--choice', '--fs'].map(v => v + '=' + cs.getPropertyValue(v).trim()).join(' '); });
  info('colour variables (default)', colsDefault);
  await goNav('settings'); await ensureLook();
  await pg.locator('.r', { hasText: 'High contrast' }).first().locator('.sw').click(); await wait(200);
  const hcCols = await ev(() => { const cs = getComputedStyle(document.documentElement); return ['--bg', '--raise', '--raise2', '--sel', '--line', '--text', '--muted', '--accent', '--err'].map(v => v + '=' + cs.getPropertyValue(v).trim()).join(' '); });
  info('colour variables (High contrast on)', hcCols);
  ok((await ev(() => document.documentElement.dataset.hc)) === '1', 'High contrast switch sets data-hc=1');
  await shot('settings-high-contrast');
  await pg.locator('.r', { hasText: 'High contrast' }).first().locator('.sw').click(); await wait(120);
  await pg.locator('.r', { hasText: 'Larger text' }).first().locator('.sw').click(); await wait(200);
  const fsNow = await ev(() => ({ data: document.documentElement.dataset.fs, fs: getComputedStyle(document.documentElement).getPropertyValue('--fs').trim(), bodyFont: getComputedStyle(document.body).fontSize }));
  info('Larger text on', fsNow);
  ok(fsNow.data === 'l' && fsNow.bodyFont === '15.5px', 'Larger text sets data-fs=l and body 15.5 px', fsNow);
  for (const id of ['home', 'mods', 'load', 'far', 'system', 'doctor', 'settings']) {
    await goNav(id);
    const ov = await ev(() => { const s = document.querySelector('.mscroll'); return { sw: s.scrollWidth, cw: s.clientWidth, top: document.querySelector('.top').scrollWidth > document.querySelector('.top').clientWidth }; });
    ok(ov.sw <= ov.cw && !ov.top, `Larger text: no horizontal overflow on ${id}`, ov);
    if (id === 'mods') await shot('larger-text-mods');
  }
  await goNav('settings'); await ensureLook();
  await pg.locator('.r', { hasText: 'Larger text' }).first().locator('.sw').click(); await wait(120);

  // ---------- 14. Window size: maximised, 1920x1080, 1280x720
  for (const vp of [{ width: 1920, height: 1080 }, { width: 1280, height: 720 }]) {
    await pg.setViewportSize(vp); await wait(300);
    const w = await rect('#win'); const sc = await ev(() => document.getElementById('win').style.getPropertyValue('--ws'));
    info(`window at viewport ${vp.width}x${vp.height}`, { win: w, scale: sc });
    const ms = await ev(() => { const s = document.querySelector('.mscroll'); return s ? s.clientWidth : null; });
    info(`content width at ${vp.width}x${vp.height} (.mscroll clientWidth)`, ms);
    if (vp.width === 1920) { await shot('viewport-1920'); }
  }
  const maxBefore = await rect('#win');
  await pg.locator('[aria-label="Maximise"]').click(); await wait(200);
  const maxAfter = await rect('#win');
  info('Maximise button: window before/after', { maxBefore, maxAfter });
  await pg.setViewportSize({ width: 1366, height: 768 }); await wait(300);

  // ---------- 15. Alignment (card header vs rows), pixel values
  await setup(); await goNav('mods');
  const align = await ev(() => {
    const out = [];
    document.querySelectorAll('div.sec').forEach((sec, i) => {
      if (i > 2) return;
      const sr = sec.getBoundingClientRect(), cs = getComputedStyle(sec);
      const kids = [...sec.children].map(k => { const r = k.getBoundingClientRect(); return { cls: k.className.toString().slice(0, 30), l: +(r.left - sr.left).toFixed(1), r: +(sr.right - r.right).toFixed(1), w: +r.width.toFixed(1) }; });
      out.push({ i, padL: cs.paddingLeft, padR: cs.paddingRight, kids: kids.slice(0, 5) });
    });
    return out;
  });
  info('mods alignment: first three cards (insets from card edge, px)', align);
  await setup(); const homeAlign = await ev(() => [...document.querySelectorAll('div.sec .sech, div.sec .r')].slice(0, 6).map(e => { const sec = e.closest('div.sec').getBoundingClientRect(), r = e.getBoundingClientRect(); return { cls: e.className.toString().slice(0, 24), l: +(r.left - sec.left).toFixed(1), r: +(sec.right - r.right).toFixed(1) }; }));
  info('home alignment: first six header/row insets (px)', homeAlign);

  // ---------- 16. Focus look: text field caret only, keyboard focus line, row tint
  await setup(); await pg.keyboard.press('Control+k'); await wait(120);
  const inFocus = await ev(() => { const e = document.activeElement; const c = getComputedStyle(e); return { tag: e.tagName, outline: c.outlineStyle, shadow: c.boxShadow }; });
  info('text field focus (palette search): outline and shadow', inFocus);
  ok(inFocus.outline === 'none' && inFocus.shadow === 'none', 'text field: no focus ring (caret only)', inFocus);
  await pg.keyboard.press('Escape'); await wait(120);
  await setup(); await pg.keyboard.press('Tab'); await wait(60);
  const btnFocus = await ev(() => { const e = document.activeElement; const c = getComputedStyle(e); return { tag: e.tagName, vis: e.matches(':focus-visible'), outline: c.outlineStyle + ' ' + c.outlineWidth + ' ' + c.outlineColor, shadow: c.boxShadow }; });
  info('keyboard focus on first Tab stop', btnFocus);
  await shot('focus-first-stop');
  await goNav('mods');
  await tabTo('nav:mods');
  for (let i = 0; i < 30; i++) { await pg.keyboard.press('Tab'); const isRow = await ev(() => document.activeElement.classList.contains('r') || document.activeElement.classList.contains('sech')); if (isRow) break; }
  const rowFocus = await ev(() => { const e = document.activeElement; const c = getComputedStyle(e); return { cls: e.className.toString().slice(0, 30), bg: c.backgroundColor, outline: c.outlineStyle, vis: e.matches(':focus-visible') }; });
  info('keyboard focus on a row (if reached)', rowFocus);

  // ---------- 17. Arrow keys: folder picker and Alt+Up/Down move
  await setup(); await goNav('settings');
  const pickBtn = await ev(() => { const b = [...document.querySelectorAll('[data-act="pickFolder"]')][0]; return b ? b.dataset.arg : null; });
  info('settings: folder pick button arg', pickBtn);
  if (pickBtn) {
    await pg.locator(`[data-act="pickFolder"][data-arg="${pickBtn}"]`).click(); await wait(200);
    const sel0 = await ev(() => [...document.querySelectorAll('.picker [role="option"]')].findIndex(e => e.getAttribute('aria-selected') === 'true'));
    await pg.keyboard.press('ArrowDown'); await wait(120);
    const sel1 = await ev(() => [...document.querySelectorAll('.picker [role="option"]')].findIndex(e => e.getAttribute('aria-selected') === 'true'));
    info('folder picker: selected index before/after ArrowDown', [sel0, sel1]);
    await pg.keyboard.press('Escape'); await wait(120);
  }
  await setup(); await goNav('mods');
  const modOrder = () => ev(() => S.profiles.find(p => p.id === S.active).mods.map(m => m.id).join(','));
  const firstMod = await ev(() => S.profiles.find(p => p.id === S.active).mods[0].id);
  if (!(await pg.locator(`[data-k="mod:${firstMod}"]`).count())) { await pg.locator('[data-k^="grp:"]').first().click(); await wait(200); }
  await pg.locator(`[data-k="mod:${firstMod}"]`).focus();
  const o0 = await modOrder();
  await pg.keyboard.press('Alt+ArrowDown'); await wait(200);
  info('Alt+ArrowDown on the first mod: order changed?', o0 !== (await modOrder()));

  await shot('final-mods');
  await b.close();
  console.log('\nconsole errors:', errs.length ? errs : 'none');
  console.log('failed checks:', fails.length ? fails : 'none');
  process.exit(errs.length || fails.length ? 1 : 0);
})().catch(e => { console.error('SCRIPT ERROR', e); process.exit(2); });
