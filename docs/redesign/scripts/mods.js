// User-story walk for the Mods area of the Toolbox demo (docs/design/demo/index.html).
// usage: node mods.js [storyPrefix]
// Exits 1 on console/page errors or failed checks. Screens go to ../screens/mods/.
const { chromium } = require('playwright');
const path = require('path');
const fs = require('fs');
const FILE = 'file://' + require('path').resolve(__dirname, '../../design/demo/index.html');
const OUT = path.join(__dirname, '..', 'screens', 'mods');
fs.mkdirSync(OUT, { recursive: true });
const KEY = 'rauls-toolbox-state-1';
const only = process.argv[2] || '';
const fails = []; const errors = []; const log = [];
const check = (c, m) => { if (!c) { fails.push(m); console.log('  FAIL', m); } else console.log('  ok  ', m); };
const note = m => { log.push(m); console.log('  note', m); };

let browser, ctx, pg, savedState = null;
const act = (a, arg) => `[data-act="${a}"]${arg !== undefined ? `[data-arg="${arg}"]` : ''}`;
const click = async (sel, o) => { try { await pg.locator(sel).first().click(o, { timeout: 5000 }); } catch (e) { throw new Error('click failed: ' + sel); } await pg.waitForTimeout(140); };
const text = () => pg.locator('#app').innerText();
const has = async t => { for (let i = 0; i < 15; i++) { if ((await text()).toLowerCase().includes(t.toLowerCase())) return true; await pg.waitForTimeout(100); } return false; };
const waitText = (t, ms = 15000) => pg.waitForFunction(t => (document.querySelector('#app') || document.body).innerText.includes(t), t, { timeout: ms });
const shot = async name => { await pg.waitForTimeout(300); await pg.locator('#win').screenshot({ path: path.join(OUT, name + '.png') }); };
const type = async (sel, v) => { await pg.locator(sel).fill(v); await pg.waitForTimeout(120); };
const groupIds = () => pg.evaluate(() => [...document.querySelectorAll('[data-groupcard]')].map(c => [c.dataset.groupcard, c.querySelector('.t').innerText.trim()]));
const setState = async s => { await pg.evaluate(([k, v]) => { localStorage.setItem(k, v); }, [KEY, s]); await pg.reload(); await pg.waitForTimeout(250); };
const freshFrom = async () => { await pg.goto(FILE); await setState(savedState); };
const dragTo = async (grip, dx, dy) => {
  const b = await grip.boundingBox(); const x = b.x + b.width / 2, y = b.y + b.height / 2;
  await pg.mouse.move(x, y); await pg.mouse.down();
  for (let i = 1; i <= 12; i++) { await pg.mouse.move(x + dx * i / 12, y + dy * i / 12); await pg.waitForTimeout(20); }
  return { up: async () => { await pg.mouse.up(); await pg.waitForTimeout(200); } };
};
const modOrder = () => pg.evaluate(() => [...document.querySelectorAll('.mrow[data-mod]')].map(r => r.dataset.mod));

// ---- one setup run, state saved and re-used by every story
async function buildSetup(profile, ttwMode = 'add') {
  ctx = await browser.newContext({ viewport: { width: 1366, height: 768 } });
  pg = await ctx.newPage();
  pg.on('console', m => { if (m.type() === 'error') errors.push(m.text()); });
  pg.on('pageerror', e => errors.push('pageerror: ' + e.message));
  await pg.goto(FILE); await pg.evaluate(() => localStorage.clear()); await pg.reload();
  await click(act('wizNext')); await waitText('Found');
  await click(act('wizNext'));
  await click(act('pickFolder', 'wstore')); await click('[data-act="folderSel"][data-arg="0"]'); await click(act('folderOk'));
  await click(act('wizNext'));
  await click(act('wizEngine')); await waitText('Installed', 15000); await click(act('wizNext'));
  await waitText('Found', 5000);
  await click(act('wizTtw', ttwMode)); if (ttwMode === 'add') await waitText('added', 15000); await click(act('wizNext'));
  await click(act('wizProf', profile)); await click(act('wizNext'));
  await click(act('wizFar', 'build')); await waitText('Built', 15000); await click(act('wizNext'));
  await click(act('wizFinish')); await pg.waitForTimeout(250);
  const saved = await pg.evaluate(k => localStorage.getItem(k), KEY);
  if (ttwMode !== 'add') { await ctx.close(); return saved; }
  savedState = saved;
  // give the groups readable ids so the stories can name them (ids are random per run)
  const st0 = JSON.parse(savedState); const pr0 = st0.profiles.find(x => x.id === st0.active) || st0.profiles[0];
  const names = { 'FIXES': 'fixes', 'TEXTURES': 'textures', 'LIGHT AND SKY': 'lightsky', 'GAMEPLAY': 'gameplay', 'UNGROUPED': 'ug' };
  for (const g of pr0.groups) { if (names[g.name.toUpperCase()] && g.id !== 'ug') savedState = savedState.split('"' + g.id + '"').join('"' + names[g.name.toUpperCase()] + '"'); }
  await ctx.close();
}
async function start(profileState) {
  ctx = await browser.newContext({ viewport: { width: 1366, height: 768 } });
  pg = await ctx.newPage();
  pg.on('console', m => { if (m.type() === 'error') errors.push(m.text()); });
  pg.on('pageerror', e => errors.push('pageerror: ' + e.message));
  await pg.goto(FILE); await setState(profileState || savedState);
}
const goMods = async () => { await click(act('nav', 'mods')); await pg.waitForTimeout(150); for (const g of await groupIds()) { const f = pg.locator(`button.t[data-act="grpFold"][data-arg="${g[0]}"][aria-expanded="false"]`); if (await f.count()) await f.first().click(); } await pg.waitForTimeout(150); };
const stateObj = () => pg.evaluate(k => JSON.parse(localStorage.getItem(k)), KEY);

module.exports = {};
const STORIES = {};
const story = (id, fn) => { STORIES[id] = fn; };

// ================= stories =================
story('01', async () => { // search
  await goMods();
  await type('input[data-in="q"]', 'zzz');
  check(await has('No mods match “zzz”.'), 'search with no hit shows "No mods match “zzz”."');
  check(await has('Clear filters'), 'empty search result offers "Clear filters"');
  await shot('01-search-no-match');
  await type('input[data-in="q"]', 'nmc');
  check((await modOrder()).join() === 'nmc', 'search "nmc" leaves only NMC\'s Texture Pack in the list');
  await shot('02-search-match-by-name');
  await type('input[data-in="q"]', 'WFO.esp');
  const o = await modOrder(); check(o.join() === 'wfo', 'search matches a plugin file name (WFO.esp) case-insensitively');
  await shot('03-search-match-by-plugin');
  await type('input[data-in="q"]', '');
  check(await has('All 15') && !(await has('No mods match')) && await has('FIXES'), 'clearing the search restores the grouped list with all 15 mods counted');
});

story('02', async () => { // filters + clear
  await goMods();
  check(await has('All 15') && await has('On 7') && await has('Off 8') && await has('Attention 0'), 'filter chips show All 15, On 7, Off 8, Attention 0 (Hardcore profile)');
  await click(act('filt', 'off')); check((await modOrder()).length === 8, 'Off filter shows 8 mods'); await shot('04-filter-off');
  await click(act('filt', 'on')); check((await modOrder()).length === 7, 'On filter shows 7 mods');
  await click(act('filt', 'att')); check(await has('No mods match.'), 'Attention filter with 0 mods shows "No mods match." (no quoted search)'); await shot('05-filter-attention-empty');
  await click(act('clearFilt')); check(!(await has('No mods match')) && await has('FIXES'), 'Clear filters resets to All and clears the search');
  await type('input[data-in="q"]', 'Nevada'); await click(act('filt', 'on')); 
  check(await has('No mods match “Nevada”.'), 'search and On filter combine: Nevada Skies is off, so no match');
  await click(act('clearFilt')); check(await has('FIXES') && !(await has('No mods match')), 'Clear filters also clears the search box');
});

story('03', async () => { // built into the engine
  await goMods();
  check(!(await has('xNVSE')), 'Built into the engine list is collapsed by default');
  await click(act('sec', 'builtin')); await shot('06-builtin-open');
  for (const n of ['xNVSE', 'JIP LN NVSE', 'JohnnyGuitar NVSE', 'ShowOff xNVSE', 'kNVSE', "lStewieAl's Tweaks", 'Mod Configuration Menu', 'UI Organizer', 'TTW NVSE plugin']) check(await has(n), `built-in row "${n}" present`);
  check((await pg.locator('.r:has-text("xNVSE") .tag').first().innerText()) === 'Built in', 'xNVSE tagged "Built in"');
  check((await pg.locator('.r:has-text("JohnnyGuitar NVSE") .tag').first().innerText()) === 'In progress', 'JohnnyGuitar NVSE tagged "In progress"');
  await click('[data-act="openLink"][data-arg="orig:xNVSE"]'); check(await has('xNVSE: the original mod’s page opened in your browser'), 'Original link toast: "xNVSE: the original mod’s page opened in your browser"');
  await click('[data-act="openLink"][data-arg="ours:xNVSE"]'); check(await has('xNVSE: our version’s source opened in your browser'), 'Our-version link toast: "xNVSE: our version’s source opened in your browser"');
  await pg.waitForTimeout(5200);
  await type('input[data-in="q"]', 'kNVSE');
  const rows = await pg.evaluate(() => [...document.querySelectorAll('.sec')].filter(sc => sc.innerText.toUpperCase().startsWith('BUILT INTO THE ENGINE')).flatMap(sc => [...sc.querySelectorAll('.r .nm')].map(x => x.innerText)));
  check(rows.join() === 'kNVSE', 'search "kNVSE" leaves only the kNVSE built-in row in the engine group (got ' + rows.join(' / ') + ')');
  await type('input[data-in="q"]', '');
  await click(act('sec', 'builtin')); check(!(await has('Mod Configuration Menu')), 'built-in section folds closed again');
});

story('04', async () => { // bundled plugins shown in mod details
  await goMods();
  await click('button.nm[data-act="rowOpen"][data-arg="nmc"]');
  check(await has('Files only'), 'asset-only mod shows "Files only" tag');
  check(await has('None · files only'), 'mod details say "None · files only" when it has no plugins');
  await click('button.nm[data-act="rowOpen"][data-arg="nvac"]');
  check(await has('None · an engine plugin'), 'DLL mod details say "None · an engine plugin"');
  await click('button.nm[data-act="rowOpen"][data-arg="lrf"]');
  check(await has('LonesomeRoadFixes.esp'), 'plugin file shown in mod details (LonesomeRoadFixes.esp)');
  check(!(await has('Loads after LonesomeRoad.esm, as the masterlist asks.')), 'plugin note (4th field of the plugin entry) is NOT shown in details');
  await shot('07-bundled-plugins-details');
  await click('button.nm[data-act="rowOpen"][data-arg="ilo"]');
  check(await has('Interior Lighting Overhaul - Core.esm') && await has('ILO - New Vegas.esp'), 'a mod with two plugins lists both files');
});

story('05', async () => { // TTW row
  await goMods();
  check(await has('Tale of Two Wastelands') && await has('Bundle'), 'TTW row shows name and "Bundle" tag');
  const sw = pg.locator('button[data-act="ttwToggle"]').first();
  check((await sw.getAttribute('aria-checked')) === 'true', 'TTW switch is on in this profile');
  await click('button[data-act="ttwToggle"]'); check(await has('TTW off'), 'TTW switch off shows toast "TTW off" with Undo');
  await shot('08-ttw-off');
  await click('button[data-act="ttwToggle"]');
  await click('button.nm[data-act="rowOpen"][data-arg="ttw"]');
  check(await has('Plain New Vegas. TTW stays installed.'), 'TTW details explain "Off: plain New Vegas"');
  await click('[data-act="ttwCheck"]');
  await pg.waitForTimeout(1700);
  check(await has('is the newest'), 'Check for TTW update reports "is the newest" when no update');
  await shot('09-ttw-checked');
  await pg.evaluate(() => {}); 
  // update available state via the demo's scenario panel
  await pg.keyboard.press('Control+Shift+D'); await click('[data-act="scenTtwUpd"]'); await pg.keyboard.press('Escape');
  check(await has('Update available'), 'TTW row shows "Update available" after the update scenario');
  check(await has('Update TTW…'), 'details offer "Update TTW…" when an update exists');
  await shot('10-ttw-update-available');
});

story('06', async () => { // create group
  await goMods();
  await click(act('newGroup'));
  check(await has('New group'), 'New group dialog opens');
  check(await pg.locator('.dlg [data-act="nameOk"]').isDisabled(), 'Create group is disabled while the name is empty');
  await shot('11-new-group-empty');
  await type('#nameIn', 'Gameplay');
  await click('.dlg [data-act="nameOk"]'); check(await has('A group with that name exists.'), 'duplicate group name shows "A group with that name exists." under the field');
  await shot('12-new-group-duplicate');
  await type('#nameIn', 'Tests group');
  await pg.locator('#nameIn').press('Enter');
  check(!(await pg.locator('.dlg').count()), 'Enter in the name field closes dialog? (checked)');
  check(await has('Created') || await has('New group Tests group') || await has('Tests group'), 'new group appears in the list');
  await shot('13-new-group-created');
  await click(act('newGroup')); await type('#nameIn', 'Abc'); await click('[data-act="dlgClose"]'); check(!(await pg.locator('.dlg').count()), 'Cancel closes the New group dialog without creating');
  check(!(await has('Abc')), 'cancelled name is not created');
});

story('07', async () => { // rename group
  await goMods();
  await click('[data-act="grpMenu"][data-arg="fixes"]');
  await shot('14-group-menu');
  await click('.menu [data-act="grpRename"][data-arg="fixes"]');
  check(await has('Rename group'), 'Rename group dialog opens');
  await type('#nameIn', 'Textures');
  await click('.dlg [data-act="nameOk"]'); check(await has('A group with that name exists.'), 'rename to a taken name is refused');
  await type('#nameIn', 'Core fixes');
  await click('.dlg [data-act="nameOk"]');
  check(await has('Core fixes'), 'renamed group shows the new name');
  check(await has('Renamed group to Core fixes'), 'rename toast "Renamed group to Core fixes"');
  await shot('15-group-renamed');
});

story('08', async () => { // delete group
  await goMods();
  await click('[data-act="grpMenu"][data-arg="gameplay"]');
  await click('.menu [data-act="grpDelete"][data-arg="gameplay"]');
  check(await has('Delete the group “Gameplay”?'), 'delete confirm asks "Delete the group “Gameplay”?"');
  check(await has('Its mods move to Ungrouped. Nothing is uninstalled.'), 'delete confirm explains mods move to Ungrouped');
  await shot('16-group-delete-confirm');
  await click('.dlg [data-act="dlgClose"]');
  check(!(await pg.locator('.dlg').count()) && await pg.locator('[data-groupcard="gameplay"]').count() === 1, 'Cancel keeps the group');
  await click('[data-act="grpMenu"][data-arg="gameplay"]');
  await click('.menu [data-act="grpDelete"][data-arg="gameplay"]');
  await click('.dlg [data-act="doDeleteGroup"]');
  check(await pg.locator('[data-groupcard="gameplay"]').count() === 0, 'Delete group removes the group card');
  const st = await stateObj(); const pr = st.profiles.find(x => x.id === st.active);
  check(pr.mods.filter(m => m.g === 'ug').map(m => m.id).includes('jsue'), 'former group mods now sit in Ungrouped (jsue checked in state)');
  check(!pr.groups.some(g => g.id === 'gameplay'), 'group removed from the profile state');
  await shot('17-group-deleted');
});

story('09', async () => { // fold and toggle all, from the default fold state
  await click('.nav[data-arg="mods"]'); await pg.waitForTimeout(150);
  const r = async () => pg.evaluate(() => { const b = document.querySelector('[data-groupcard="textures"]'); return b ? [...b.querySelectorAll('.mrow[data-mod]')].length : -1; });
  check(await r() === 0, 'Textures starts folded in the Hardcore profile (only Gameplay is open)');
  await click('button.t[data-act="grpFold"][data-arg="textures"]');
  check(await r() === 3, 'unfolding Textures shows its 3 mods');
  await shot('18-group-unfolded');
  await click('button.t[data-act="grpFold"][data-arg="textures"]');
  check(await r() === 0, 'folding Textures hides its mods');
  await click('button.t[data-act="grpFold"][data-arg="textures"]');
  await click('[data-act="grpToggle"][data-arg="textures"]');
  check(await has('Textures: all on'), 'group switch on a group with none on: toast "Textures: all on"');
  check((await pg.locator('[data-act="grpToggle"][data-arg="textures"]').first().getAttribute('aria-checked')) === 'true', 'group switch reads on');
  await shot('19-group-all-on');
  await click('[data-act="grpToggle"][data-arg="textures"]');
  check(await has('Textures: all off'), 'group switch again: toast "Textures: all off"');
});

story('10', async () => { // move group up/down
  await goMods();
  const ord = async () => pg.evaluate(() => [...document.querySelectorAll('[data-groupcard]')].map(c => c.dataset.groupcard).join(','));
  const before = await ord();
  await click('[data-act="grpMenu"][data-arg="fixes"]');
  check(await pg.locator('.menu .mi', { hasText: 'Move up' }).isDisabled(), 'Move up is disabled on the first group');
  await click('.menu [data-act="grpMove"][data-arg="fixes:1"]');
  check((await ord()) !== before, 'Move down reorders the group list');
  await shot('20-group-moved-down');
  await click('[data-act="grpMenu"][data-arg="fixes"]');
  await click('.menu [data-act="grpMove"][data-arg="fixes:-1"]');
  check((await ord()) === before, 'Move up puts the group back');
  await click('[data-act="grpMenu"][data-arg="ug"]');
  check(!(await pg.locator('.menu [data-act="grpRename"]').count()) && !(await pg.locator('.menu [data-act="grpDelete"]').count()), 'Ungrouped menu has no Rename or Delete');
  await pg.keyboard.press('Escape'); await pg.waitForTimeout(150);
  await pg.locator('button.t[data-act="grpFold"][data-arg="fixes"]').focus();
  await pg.keyboard.press('Alt+ArrowDown'); await pg.waitForTimeout(150);
  check((await ord()) === 'textures,fixes,lightsky,gameplay,ug', 'Alt+Down on a focused group name moves that group down one place');
  await pg.keyboard.press('Escape');
});

story('11', async () => { // drag group reorder
  await goMods();
  for (const g of ['fixes', 'textures', 'lightsky', 'gameplay']) { const f = pg.locator(`button.t[data-act="grpFold"][data-arg="${g}"][aria-expanded="true"]`); if (await f.count()) await f.first().click(); }
  await pg.waitForTimeout(150);
  const ord = async () => pg.evaluate(() => [...document.querySelectorAll('[data-groupcard]')].map(c => c.dataset.groupcard).join(','));
  const before = await ord();
  const d = await dragTo(pg.locator('[data-grip="group"][data-id="fixes"]').first(), 0, 120);
  await pg.waitForTimeout(100); await shot('21-group-drag-ghost');
  check(await pg.locator('.dghost').count() === 1, 'dragging a group header shows the lifted ghost "Moving group"');
  await d.up();
  check((await ord()) !== before, 'dragging a group header below another reorders the groups');
  check((await ord()).startsWith('textures,fixes') || (await ord()).split(',')[1] === 'fixes', 'Fixes lands after Textures');
});

story('12', async () => { // group menu close
  await goMods();
  await click('[data-act="grpMenu"][data-arg="gameplay"]');
  check(await pg.locator('.menu').count() === 1, 'group menu opens');
  await pg.keyboard.press('Escape'); await pg.waitForTimeout(200);
  check(await pg.locator('.menu').count() === 0, 'Escape closes the group menu');
});

story('13', async () => { // mod switch + undo
  await goMods();
  await click('[data-act="modToggle"][data-arg="tt"]');
  check(await has('Tougher Traders off'), 'switching a mod off shows toast "Tougher Traders off" (with Undo)');
  check(await pg.locator('.toast [data-act="undo"]').count() === 1, 'toast carries an Undo button');
  await shot('22-mod-off-toast');
  await click('.toast [data-act="undo"]');
  check(await has('Undone: Tougher Traders off'), 'Undo from toast shows "Undone: Tougher Traders off"');
  { const st = await stateObj(); const pr = st.profiles.find(x => x.id === st.active); check(pr.mods.find(m => m.id === 'tt').on === true, 'Undo restores the mod to on (checked in state)'); }
  await click('[data-act="modToggle"][data-arg="tt"]');
  await pg.waitForTimeout(5600);
  check(!(await pg.locator('.toast').count()), 'the Undo toast goes away after about 5 s');
});

story('14', async () => { // details fold
  await goMods();
  const cards = pg.locator('.mrow.open');
  check(await cards.count() === 0, 'no mod details open at start');
  await click('button.nm[data-act="rowOpen"][data-arg="wrp"]');
  check(await has('What it does'), 'details show "What it does"');
  check(await has('Installed'), 'details show "Installed" date');
  check(await has('Group'), 'details show Group select');
  await shot('23-mod-details-open');
  await click('button.nm[data-act="rowOpen"][data-arg="wrp"]');
  check(!(await has('What it does')), 'clicking the name again folds the details');
  await click('button.nm[data-act="rowOpen"][data-arg="wrp"]');
  await pg.keyboard.press('Escape'); // details stay open
});

story('15', async () => { // move up/down
  await goMods();
  const o0 = await modOrder();
  await click('button.nm[data-act="rowOpen"][data-arg="ojo"]');
  await click('[data-act="modMove"][data-arg="ojo:-1"]');
  const o1 = await modOrder();
  check(o1.indexOf('ojo') === o0.indexOf('ojo') - 1, 'Up moves the mod one place up in the list');
  check(await pg.evaluate(() => document.activeElement && document.activeElement.dataset.k === 'mod:ojo'), 'focus stays on the moved mod name');
  await shot('24-mod-moved-up');
  await pg.locator('button.nm[data-act="rowOpen"][data-arg="ojo"]').focus();
  await pg.keyboard.press('Alt+ArrowDown'); await pg.waitForTimeout(120);
  check((await modOrder()).indexOf('ojo') === o1.indexOf('ojo') + 1, 'Alt+Down on the focused name moves the mod down');
  await pg.locator('[data-act="modMove"][data-arg="ojo:1"]').first().click().catch(() => {});
});

story('16', async () => { // drag a mod onto another group
  await goMods();
  for (const g of ['fixes', 'textures', 'lightsky', 'gameplay']) { const f = pg.locator(`button.t[data-act="grpFold"][data-arg="${g}"][aria-expanded="true"]`); if (await f.count()) await f.first().click(); }
  await click('button.t[data-act="grpFold"][data-arg="textures"]');
  const grip = pg.locator('[data-grip="mod"][data-id="wrp"]').first();
  await grip.scrollIntoViewIfNeeded(); await pg.waitForTimeout(100);
  const tb = await pg.locator('[data-groupcard="lightsky"] .sech').first().boundingBox();
  const gb = await grip.boundingBox();
  const d = await dragTo(grip, 0, tb.y + tb.height / 2 - (gb.y + gb.height / 2));
  await pg.waitForTimeout(100); await shot('25-mod-drag-ghost');
  await d.up();
  const stG = await stateObj(); const prG = stG.profiles.find(x => x.id === stG.active);
  check(prG.mods.find(m => m.id === 'wrp').g === 'lightsky', 'dragging a mod onto the Light and sky header moves it into that group (checked in state; group stays folded)');
  await shot('26-mod-dropped-in-group');
});

story('17', async () => { // move to group via select
  await goMods();
  await click('button.nm[data-act="rowOpen"][data-arg="fco"]');
  const opts = await pg.locator('#mg-fco option').allInnerTexts();
  check(opts.length === 5, 'group select lists all 5 groups');
  await pg.locator('#mg-fco').selectOption({ label: 'Gameplay' });
  check(await has('Moved Fallout Character Overhaul to Gameplay'), 'select a group: toast "Moved Fallout Character Overhaul to Gameplay"');
  await shot('27-mod-group-select');
});

story('18', async () => { // remove mod
  await goMods();
  await click('button.nm[data-act="rowOpen"][data-arg="tt"]');
  await click('[data-act="modRemove"][data-arg="tt"]');
  check(await has('Remove Tougher Traders from all profiles?'), 'remove confirm title');
  check(await has('Mods are shared by every profile.'), 'remove confirm explains shared mods');
  check(await has('The archive stays in your downloads, and Undo brings it back.'), 'remove confirm mentions archive and Undo');
  await shot('28-remove-confirm');
  await click('.dlg [data-act="dlgClose"]');
  check(!(await pg.locator('.dlg').count()) && await has('Tougher Traders'), 'Cancel keeps the mod');
  await click('[data-act="modRemove"][data-arg="tt"]');
  await click('.dlg [data-act="doRemoveMod"]');
  check(await pg.locator('.mrow[data-mod="tt"]').count() === 0, 'Remove from all takes the mod out of the list');
  check(await has('Removed Tougher Traders'), 'toast "Removed Tougher Traders" with Undo');
  await shot('29-mod-removed');
  await click('.toast [data-act="undo"]');
  check(await has('Tougher Traders'), 'Undo brings the removed mod back');
});

story('19', async () => { // install with a patch whose plugin needs a mod that is off
  await goMods();
  await click('[data-act="install"]');
  await click('[data-act="archSel"][data-arg="1"]'); await click('[data-act="instNext"]');
  await click('[data-act="instNext"]'); await click('[data-act="instNext"]');
  check(await has('Options'), 'review step reached with options summary');
  check(await has('Normal · JSawyer Ultimate patch'), 'review options line "Normal · JSawyer Ultimate patch" (patch pre-ticked: JSawyer is installed)');
  check(await has('The Living Desert - JSUE.esp') && !(await has('B42 Optics - WRP.esp')), 'review lists the patch plugin "The Living Desert - JSUE.esp", not a B42 file');
  await shot('30-install-review-missing-master');
  await click('[data-act="instGo"]'); await waitText('Installed The Living Desert', 10000);
  await shot('31-installed-with-missing-master');
  await shot('32-missing-master');
});

story('20', async () => { // untested change card on the hard profile
  await goMods();
  check(await has('This mod setup hasn\'t been tested yet.'), 'untested change card shows on a profile that is not stable');
  await pg.evaluate(() => { const m = document.querySelector('.mscroll'); if (m) m.scrollTop = 0; }); await pg.waitForTimeout(150);
  check(await has('Not now') && await has('Run stability test'), 'card offers Not now and Run stability test');
  await shot('33-untested-card');
  await click('[data-act="recDismiss"]');
  check(await has('Hidden until the mod setup changes'), 'Not now: toast "Hidden until the mod setup changes"');
  check(!(await has('This mod setup hasn\'t been tested yet.')), 'card hides after Not now');
  await click('[data-act="modToggle"][data-arg="nmc"]');
  check(await has('This mod setup hasn\'t been tested yet.'), 'card returns once the mod setup changes');
  await click('[data-act="modToggle"][data-arg="nmc"]');
});

story('21', async () => { // undo from header and toasts
  await goMods();
  check(!(await pg.locator('[data-act="undo"]').count()), 'no Undo button before any change');
  await click('[data-act="modToggle"][data-arg="tt"]');
  check(await pg.locator('[data-act="undo"]').first().isVisible(), 'Undo button appears in the list header after a change');
  await pg.locator('[data-act="undo"]').first().click(); await pg.waitForTimeout(160);
  check(await has('Undone: Tougher Traders off'), 'header Undo reverts the last change ("Undone: Tougher Traders off")');
  await click('[data-act="modToggle"][data-arg="tt"]');
  while (await pg.locator('.toast [data-act="toastX"]').count()) { await pg.locator('.toast [data-act="toastX"]').first().click(); await pg.waitForTimeout(120); }
  check(!(await pg.locator('.toast').count()), 'toast X dismisses the toast');
});

story('22', async () => { // empty profile: remove every mod from the Hardcore profile
  await goMods();
  const ids = await pg.evaluate(() => [...document.querySelectorAll('.mrow[data-mod]')].map(r => r.dataset.mod));
  for (const id of ids) {
    const row = pg.locator(`.mrow[data-mod="${id}"] button.nm`).first();
    if (!(await row.count())) { await goMods(); }
    await pg.locator(`.mrow[data-mod="${id}"] button.nm[data-act="rowOpen"]`).first().click();
    await pg.waitForTimeout(120);
    await click(`[data-act="modRemove"][data-arg="${id}"]`);
    await click('.dlg [data-act="doRemoveMod"]');
  }
  check(await has('No mods in Hardcore yet'), 'with every mod removed the list shows "No mods in Hardcore yet"');
  check(await has('Install one from an archive. The engine already has the script extenders built in.'), 'empty banner explains installing from an archive');
  check(await pg.locator('.banner [data-act="install"]').count() === 1, 'empty banner has an "Install mod…" button');
  await shot('34-empty-profile');
  await click('.banner [data-act="install"]');
  check(await has('Archives in your download folder'), 'empty banner Install mod… opens the picker');
  await click('[data-act="dlgClose"]');
});

story('23', async () => { // install with options
  await goMods();
  await click('[data-act="install"]'); await shot('35-install-pick');
  check(await has('Archives in your download folder. Pick one.'), 'picker says "Archives in your download folder. Pick one."');
  check(await pg.locator('[data-act="instNext"]').isDisabled(), 'Next is disabled until an archive is picked');
  await pg.locator('[data-k="arch:0"]').first().click({ force: true }); await pg.waitForTimeout(140);
  check(await pg.locator('[data-act="instNext"]').isDisabled(), 'an archive already installed (B42 Optics) cannot be picked: Next stays disabled');
  check(await pg.locator('.r[aria-disabled="true"]').count() >= 1, 'installed archives are marked aria-disabled');
  await click('[data-act="archSel"][data-arg="1"]');
  check(!(await pg.locator('[data-act="instNext"]').isDisabled()), 'Next enables after picking');
  await click('[data-act="instNext"]'); await shot('36-install-options-1');
  check(await has('Page 1 of 2'), 'options page shows "Page 1 of 2"');
  await click('[data-act="optPick"][data-arg="enc:more:one"]');
  await click('[data-act="instBack"]'); check(await has('Install a mod'), 'Back from options page 1 returns to the picker');
  await click('[data-act="archSel"][data-arg="1"]'); await click('[data-act="instNext"]');
  await click('[data-act="optPick"][data-arg="enc:more:one"]');
  await click('[data-act="instNext"]'); await shot('37-install-options-2');
  check(await has('Page 2 of 2'), 'second options page');
  await click('[data-act="instBack"]'); check(await has('Page 1 of 2'), 'Back from page 2 returns to page 1');
  await click('[data-act="instNext"]'); await click('[data-act="instNext"]');
  await shot('38-install-review');
  check(await has('Install The Living Desert'), 'review dialog title "Install The Living Desert"');
  await click('[data-act="instGo"]');
  check(await pg.locator('.dlg [data-act="instCancel"]').count() === 1, 'install progress shows Cancel');
  await shot('39-install-progress');
  await waitText('Installed The Living Desert', 10000);
  await shot('40-install-done');
  check(await has('Installed The Living Desert'), 'install done toast "Installed The Living Desert"');
});

story('24', async () => { // no-options archive goes straight to review
  await goMods();
  await click('[data-act="install"]');
  await click('[data-act="openLink"][data-arg="browse:archive"]');
  check(!(await has('Your file browser opened')), 'Browse… while the picker is open: no toast is drawn (toasts are hidden under dialogs)');
  await click('.dlg [data-act="dlgClose"]');
  check(await has('Your file browser opened'), 'after closing the picker the queued toast "Your file browser opened" shows');
  check(!(await pg.locator('.dlg').count()), 'Cancel on the picker closes the dialog');
  await click('[data-act="install"]');
  await click('[data-act="archSel"][data-arg="3"]'); await click('[data-act="instNext"]');
  check(await has('Plugins go to the end of the load order. Sort puts them in place.'), 'archive without options goes straight to review');
  check(await has('Defaults') === false, 'no options line on a plain archive');
  await click('[data-act="instBack"]'); check(await has('Archives in your download folder'), 'Back on review goes to the picker for plain archives');
  await click('[data-act="dlgClose"]');
});

story('25', async () => { // broken archive
  await goMods();
  await click('[data-act="install"]');
  await click('[data-act="archSel"][data-arg="7"]'); await click('[data-act="instNext"]'); await shot('41-broken-archive');
  check(await has('This archive is damaged'), 'broken archive title "This archive is damaged"');
  check(await has('It looks like part of a split download. Download the whole file again, then install it.'), 'broken archive text explains split download');
  await click('[data-act="instBack"][data-arg="pick"]'); check(await has('Archives in your download folder'), 'Pick another returns to the picker');
  await click('[data-act="dlgClose"]');
});

story('26', async () => { // cancel during install
  await goMods();
  await click('[data-act="install"]');
  await click('[data-act="archSel"][data-arg="3"]'); await click('[data-act="instNext"]');
  await click('[data-act="instGo"]');
  await pg.waitForTimeout(300);
  await pg.locator('.scrim').click({ position: { x: 5, y: 5 } });
  check(await has('Installing Weapon Animation Replacers'), 'clicking outside does not close the dialog while installing');
  await pg.keyboard.press('Escape'); await pg.waitForTimeout(150);
  check(await has('Installing Weapon Animation Replacers'), 'Escape does not close the dialog while installing');
  await click('[data-act="instCancel"]');
  check(await has('Install Weapon Animation Replacers'), 'Cancel returns to the review step');
  await shot('42-install-cancelled-review');
  await pg.keyboard.press('Escape'); await pg.waitForTimeout(150);
  check(!(await pg.locator('.dlg').count()), 'Escape closes the review step');
});

story('27', async () => { // palette finds a mod by name and jumps
  await goMods();
  await pg.keyboard.press('Control+k'); await pg.waitForTimeout(200);
  check(await pg.locator('#palIn').count() === 1, 'Ctrl K opens search or go to');
  await pg.keyboard.type('wasteland'); await pg.waitForTimeout(200);
  check(await pg.locator('#palList [role="option"]').count() === 1, 'typing "wasteland" leaves one result');
  check((await pg.locator('#palList [role="option"]').first().innerText()).includes('Wasteland Flora Overhaul'), 'palette lists Wasteland Flora Overhaul (under the Mod kind)');
  check((await pg.locator('#palList [role="option"] .k').first().innerText()) === 'Mod', 'palette result is labelled "Mod"');
  await shot('43-palette-mod');
  await pg.keyboard.press('Enter'); await pg.waitForTimeout(250);
  check(await pg.locator('.mrow[data-mod="wfo"]').count() === 1, 'choosing the mod opens the Mods list with that row');
  check(await pg.locator('.mrow.open[data-mod="wfo"]').count() === 1, 'the row opens its details');
  await shot('44-palette-jumped');
  await pg.keyboard.press('Control+k'); await pg.waitForTimeout(150);
  await pg.keyboard.type('zzzz'); await pg.waitForTimeout(150);
  check(await has('Nothing found.'), 'palette shows "Nothing found." for no match');
  await pg.keyboard.press('Escape'); await pg.waitForTimeout(150);
});

story('28', async () => { // sidebar badge: appears with a missing master, clears when fixed
  await goMods();
  check(!(await pg.locator('.nav[data-arg="mods"] .bdg').count()), 'no Mods badge when nothing needs attention');
  await click('[data-act="install"]');
  await click('[data-act="archSel"][data-arg="1"]'); await click('[data-act="instNext"]');
  await click('[data-act="instNext"]'); await click('[data-act="instNext"]');
  await click('[data-act="instGo"]'); await waitText('Installed The Living Desert', 10000);
  await click('[data-act="modToggle"][data-arg="jsue"]'); // JSawyer off: the new patch loses its master
  const badge = pg.locator('.nav[data-arg="mods"] .bdg');
  check(await badge.count() === 1, 'Mods sidebar badge appears when a mod needs attention');
  check((await badge.first().getAttribute('title')) === '1 mod needs attention', 'badge tooltip reads "1 mod needs attention"');
  check((await badge.first().innerText()).trim() === '1', 'badge shows the count 1');
  await shot('45-badge-appears');
  await click('[data-act="modToggle"][data-arg="jsue"]');
  check(!(await pg.locator('.nav[data-arg="mods"] .bdg').count()), 'Mods badge clears when the missing master is turned on');
  await shot('46-badge-cleared');
});

story('29', async () => { // TTW row when Tale of Two Wastelands was never added
  await ctx.close();
  const noTtw = await buildSetup('hard', 'skip');
  await start(noTtw);
  await goMods();
  check(await has('Not added'), 'TTW row shows the tag "Not added"');
  check(await has('Fallout 3 inside New Vegas'), 'TTW row subtitle "Fallout 3 inside New Vegas"');
  check(await pg.locator('.r button', { hasText: 'Add…' }).count() === 1, 'TTW row offers an "Add…" button');
  check(await pg.locator('button[data-act="ttwToggle"]').count() === 0, 'no TTW switch while TTW is not added');
  await shot('47-ttw-not-added');
  await click('.r button:has-text("Add…")');
  await pg.waitForTimeout(300);
  check(await has('Add Tale of Two Wastelands'), 'Add… opens the TTW add flow (page title "Add Tale of Two Wastelands")');
});

story('30', async () => { // New profile: the dialog says "Starts empty" - what does the Mods list show?
  await goMods();
  await click(act('profMenu', 'top'));
  await click('.menu [data-act="profNew"]');
  check(await has('Starts with every mod off. Your installed mods are shared.'), 'New profile dialog text: "Starts with every mod off. Your installed mods are shared."');
  await pg.locator('#nameIn').fill('Fresh one');
  await click('.dlg [data-act="nameOk"]');
  await goMods();
  const hdr = await pg.locator('.hdr, header').first().innerText().catch(() => '');
  const count = await pg.evaluate(() => [...document.querySelectorAll('.mrow[data-mod]')].length);
  note('new profile header/list: ' + (await text()).replace(/\n+/g, ' | ').slice(0, 160));
  check(await has('of 15 on') && await has('No mods in Fresh one yet') === false, 'the new profile is NOT empty: header shows "x of 15 on" (it copies the shared mod library)');
  await shot('48-new-profile-not-empty');
});

const CACHE = require('path').join(require('os').tmpdir(), 'redesign-mods-setup-state.json');
(async () => {
  browser = await chromium.launch();
  if (process.env.REUSE && fs.existsSync(CACHE)) savedState = fs.readFileSync(CACHE, 'utf8');
  else { await buildSetup('hard'); fs.writeFileSync(CACHE, savedState); }
  const ids = Object.keys(STORIES).filter(k => k.startsWith(only));
  for (const id of ids) {
    console.log('STORY ' + id);
    try {
      await start(savedState);
      await STORIES[id]();
    } catch (e) { fails.push(id + ': ' + e.message.split('\n')[0]); console.log('  EXC', id, e.message.split('\n')[0]); }
    await ctx.close();
  }
  await browser.close();
  console.log(JSON.stringify({ fails: fails.length, errors }, null, 1));
  process.exit(fails.length || errors.length ? 1 : 0);
})();
