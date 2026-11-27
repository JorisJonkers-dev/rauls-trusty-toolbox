/* SPDX-License-Identifier: GPL-3.0-only */
/* ===== app shell and pages ===== */
let ABAR = '';
const abar = h => { ABAR = h; };
const NAV = [['PLAY', [['home', 'Home', 'home'], ['mods', 'Mods', 'mods'], ['load', 'Load order', 'load'], ['far', 'LOD', 'lod']]], ['SYSTEM', [['system', 'System', 'versions'], ['doctor', 'Doctor', 'doctor'], ['settings', 'Settings', 'settings']]]];
const PAGES = {};
const pillP = () => { const p = P(); return `<button type="button" class="ppill" data-act="profMenu" data-arg="pill" data-k="profMenu:pill" aria-haspopup="menu" aria-expanded="${U.menu && U.menu.id === 'prof' && U.menu.anchor === 'profMenu:pill'}">${ic('user', 16)}<span class="lb">Profile</span>${esc(p.name)}${ic('chevd', 14)}</button>`; };
const stableChip = p => !p.mods.some(m => m.on) ? '' : stableNow(p) ? tag('Tested stable', 'ok', 'shield') : tag('Not tested yet', 'warn', 'warn');
const hdr = (icon, title, status = '', right = '') => `<div class="phdr"><span class="ico">${ic(icon, 26)}</span><h1>${title}</h1>${status ? `<span class="st">${status}</span>` : ''}<div class="rgt">${right}</div></div>`;
const gx = x => { const w = winBox(); return (x - w.left) / w.s; }, gy = y => { const w = winBox(); return (y - w.top) / w.s; };
const swRow = ctl => { const m = /^<button type="button" class="sw" role="switch" [^>]*data-act="([^"]+)" data-arg="([^"]*)"/.exec(ctl); return m ? ` click" data-act="${m[1]}" data-arg="${m[2]}` : ''; };
const pageWrap = (key, html) => `<div class="pg" data-key="${key}">${html}</div>`;

function appHTML() {
  ABAR = '';
  const page = (PAGES[U.page] || PAGES.home)();
  return `${topHTML()}${sideHTML()}<main class="main" id="main" aria-label="${esc(pageTitle())}"><div class="mscroll">${page}</div>${ABAR ? `<div class="abar" role="toolbar" aria-label="Page actions">${ABAR}</div>` : ''}</main>${footHTML()}${overlays()}`;
}
const pageTitle = () => ({ home: 'Home', mods: 'Mods', load: 'Load order', far: 'LOD', farset: 'LOD settings', ttw: 'Add Tale of Two Wastelands', system: 'System', doctor: 'Doctor', settings: 'Settings' })[U.page] || 'Home';

function topHTML() {
  const p = P(), run = U.run, chk = U.play && U.play.state === 'checking';
  let play;
  if (run) play = `<button type="button" class="btn sec" data-act="stopGame" data-k="stopGame:">${ic('stop', 16)}<span>Stop · ${hm(Math.floor((Date.now() - run.start) / 1000))}</span></button>`;
  else if (U.play && U.play.state === 'starting') play = btn('Starting…', { k: 'primary', icon: 'play', busy: true, dis: true });
  else if (chk) play = btn(`Checking ${Math.min(U.play.i + 1, U.play.list.length)}/${U.play.list.length}`, { k: 'primary', busy: true, icon: 'play', dis: true });
  else play = btn('Play', { k: 'primary', icon: 'play', act: 'play', arg: '', title: 'Checks your setup, then starts the game' });
  return `<header class="top"><div class="brand">${ic('app', 22)}Raul's Toolbox</div>
  <button type="button" class="tchip" data-act="nav" data-arg="settings" data-k="top:game" title="Game folder">${ic('folder', 16)}New Vegas${S.g.scen.game || !S.g.gameOk ? ' ' + tag('Not found', 'err') : ''}</button>
  <button type="button" class="tchip" data-act="nav" data-arg="system" data-k="top:engine" title="Engine build">${ic('bolt', 16)}Engine <b class="mono">${S.g.engine}</b></button>
  <button type="button" class="tchip" data-act="profMenu" data-arg="top" data-k="profMenu:top" aria-haspopup="menu" aria-expanded="${!!(U.menu && U.menu.anchor === 'profMenu:top')}">${ic('user', 16)}Profile <b>${esc(p.name)}</b>${ic('chevd', 14)}</button>
  <span class="tsp"></span>
  <button type="button" class="srchbtn" data-act="palette" data-arg="" data-k="palette:">${ic('search', 16)}Search or go to…<span class="kbd">Ctrl K</span></button>${play}</header>`;
}
function sideHTML() {
  const doc = docSummary();
  const p = P(), mm = missingMasters(p).length, ooo = outOfOrder(p), unt = stableNow(p) ? 0 : 1, fs = farState(p), att = p.mods.filter(m => modAttention(m, p)).length + (p.ttwOn && !S.g.ttwInstalled ? 1 : 0);
  const b = (n, k, t) => n ? `<span class="bdg ${k}" title="${t}" aria-label="${t}">${n}</span>` : '';
  const bdg = { load: b(mm + ooo + unt, mm ? 'err' : 'warn', [mm && plural(mm, 'missing master'), ooo && `${ooo} out of order`, unt && 'untested change'].filter(Boolean).join(', ')), far: fs === 'never' || fs.startsWith('stale') ? b(1, 'warn', fs === 'never' ? 'LOD not built' : 'LOD out of date') : '', mods: b(att, mm ? 'err' : 'warn', plural(att, 'mod') + (att === 1 ? ' needs' : ' need') + ' attention'), system: updAvail() ? `<span class="bdg" title="Update available">${ic('up', 13)}</span>` : '', doctor: doc ? `<span class="bdg ${doc[0]}" title="${doc[1]}">${doc[2]}</span>` : '' };
  return `<nav class="side" aria-label="Pages">${NAV.map(([h, items]) => `<div class="navh">${h}</div>${items.map(([id, t, icon]) => `<button type="button" class="nav ${U.page === id || (id === 'far' && U.page === 'farset') || (id === 'mods' && U.page === 'ttw') ? 'on' : ''}" data-act="nav" data-arg="${id}" data-k="nav:${id}" ${U.page === id ? 'aria-current="page"' : ''}>${ic(icon, 18)}<span>${t}</span>${bdg[id] || ''}</button>`).join('')}`).join('')}</nav>`;
}
function docSummary() { if (!S.g.docRun) return null; const L = checks(); const f = L.filter(c => c.st === 'fail').length, w = L.filter(c => c.st === 'warn').length; return f ? ['err', `${f} failed`, f] : w ? ['warn', `${w} warnings`, w] : null; }
function footHTML() {
  const busy = Object.entries(J).filter(([k]) => !k.startsWith('find') && k !== 'play');
  let txt = 'Ready', dot = '';
  if (U.run) { txt = `Playing · ${esc(prof(U.run.pid).name)} · ${hm(Math.floor((Date.now() - U.run.start) / 1000))}`; dot = 'busy'; }
  else if (busy.length) { const [k, j] = busy[0]; txt = `${j.label || 'Working'} · ${Math.round(j.p * 100)} %${busy.length > 1 ? ` · ${busy.length - 1} more` : ''}`; dot = 'busy'; }
  return `<footer class="foot"><span class="dot ${dot}"></span><span role="status">${txt}</span><span class="sp"></span><button type="button" class="btn ghost sm" data-act="shortcuts" data-arg="" data-k="shortcuts:"><span class="kbd" style="margin:0">?</span><span>Shortcuts</span></button></footer>`;
}

/* ---------- Home ---------- */
function recCard(p, where) {
  if (stableNow(p) || p.dismissed.includes(sig(p))) return '';
  const running = J['stab:' + p.id];
  return `<div class="banner warn rec" role="note"><span class="bi">${ic('flask', 22)}</span><div><div class="bt">This mod setup hasn't been tested yet.</div><div class="bs">${running ? `Stability test running · ${Math.round(running.p * 100)} %` : 'A 20-minute test checks this exact set and order.'}</div></div><div class="ba">${running ? btn('Show test', { k: 'sec', act: 'stabShow', arg: '' }) : btn('Not now', { k: 'ghost', act: 'recDismiss', arg: where }) + (lastStable(p) ? btn('Revert to last stable', { k: 'sec', icon: 'undo', act: 'vRevertStable', arg: '' }) : '') + btn('Run stability test', { k: 'primary', icon: 'play', act: 'stabGo', arg: where })}</div></div>`;
}
const homeRows = p => {
  /* short list: game, TTW, mods, load order, LOD; storage, engine and mod-setup rows only when they need attention */
  const L = checks(p), want = ['game', 'ttw', 'mods', 'load', 'far'];
  return L.filter(c => want.includes(c.id) || c.st !== 'ok').filter(c => c.id !== 'stab').map(c => shortRow(c, p));
};
const shortRow = (c, p) => c.id === 'ttw' && c.st === 'ok' ? { ...c, st: p.ttwOn ? 'ok' : 'off', t: p.ttwOn ? 'Installed' : S.g.ttwInstalled ? 'Installed, off here' : 'Not added' } : { ...c, n: c.id === 'far' ? 'LOD' : c.n, t: c.st === 'ok' ? 'OK' : c.t };
const ROWNAV = { game: 'settings', store: 'settings', engine: 'system', ttw: 'mods', mods: 'mods', load: 'load', far: 'far', stab: 'mods' };
PAGES.home = () => {
  const p = P(), pl = U.play;
  let rows = homeRows(p), card = '', status, big;
  if (pl && pl.state === 'checking') {
    rows = pl.list.map((c, i) => i < pl.i ? shortRow(c, p) : i === pl.i ? { ...c, st: 'run', t: 'Checking…' } : { ...c, st: 'wait', t: 'Waiting' });
    status = 'Checking before play'; big = [stt('run', 'Checking your setup'), `Check ${Math.min(pl.i + 1, pl.list.length)} of ${pl.list.length}`];
    card = `<div class="card">${pbar((pl.i) / pl.list.length)}<div class="cca">${btn('Cancel', { k: 'ghost', act: 'playCancel', arg: '' })}</div></div>`;
  } else if (pl && pl.state === 'failed') {
    const bad = pl.list.filter(c => c.st === 'fail');
    status = 'Play stopped by a check'; rows = pl.list.map((c, i) => i <= pl.stopAt ? shortRow(c, p) : { ...c, st: 'wait', t: 'Not checked' });
    card = `<div class="card" role="alert"><div class="phdr" style="margin:0 0 10px;min-height:0">${ic('err', 24, 'errc')}<h2 style="margin:0;font-size:1.15rem">Play stopped: ${plural(bad.length, 'check')} failed</h2></div><div class="rows">${bad.map(c => `<div class="r"><span class="nm">${c.n}</span><span class="end">${tag(c.why, 'err', 'err')}</span></div>`).join('')}</div><div class="cca">${btn('Check again', { k: 'ghost', icon: 'refresh', act: 'play', arg: '' })}${btn(bad[0].fix[0], { k: 'primary', act: bad[0].fix[1], arg: bad[0].fix[2] })}</div></div>`;
    big = [stt('fail', 'Play stopped'), 'Fix the failed check'];
  } else if (pl && pl.state === 'warned') {
    const w = pl.list.filter(c => c.st === 'warn');
    status = 'Ready, with warnings'; rows = pl.list.map(c => shortRow(c, p));
    card = `<div class="card" role="alert"><div class="phdr" style="margin:0 0 10px;min-height:0">${ic('warn', 24, 'warnc')}<h2 style="margin:0;font-size:1.15rem">Before you play: ${plural(w.length, 'warning')}</h2></div><div class="rows">${w.map(c => `<div class="r"><span class="nm">${c.n}</span><span class="end">${tag(c.why, 'warn', 'warn')}${c.id === 'stab' && lastStable(p) ? btn('Revert to last stable', { k: 'sec', cls: 'sm', icon: 'undo', act: 'vRevertStable', arg: '' }) : ''}${btn(c.fix[0], { k: 'sec', cls: 'sm', act: c.fix[1], arg: c.fix[2] })}</span></div>`).join('')}</div><div class="cca">${btn('Cancel', { k: 'ghost', act: 'playCancel', arg: '' })}${btn('Play anyway', { k: 'primary', icon: 'play', act: 'playAnyway', arg: '' })}</div></div>`;
    big = [stt('warn', 'Ready, with warnings'), 'You can play, or fix these first'];
  } else if (U.run) { status = 'Playing'; big = [stt('ok', 'Playing'), `${esc(prof(U.run.pid).name)} · ${hm(Math.floor((Date.now() - U.run.start) / 1000))}`]; }
  else if (pl && pl.state === 'starting') { status = 'Starting'; big = [stt('run', 'Starting the game'), 'Loading your mods']; }
  else {
    const L = checks(p), f = L.filter(c => c.st === 'fail').length, w = L.filter(c => c.st === 'warn').length;
    status = f ? 'Needs attention' : w ? 'Ready, with warnings' : 'Ready to play';
    big = f ? [stt('fail', 'Needs attention'), 'Play checks first and shows the fix'] : w ? [stt('warn', 'Ready, with warnings'), 'Play shows them before it starts'] : [stt('ok', 'Ready to play'), 'All set. Go on, the Mojave’s waiting.'];
  }
  const tot = S.g.sessions.reduce((a, s) => a + s.mins, 0), last = S.g.sessions[S.g.sessions.length - 1];
  const listH = rows.map(c => `<div class="r click" role="button" tabindex="0" data-act="nav" data-arg="${ROWNAV[c.id]}" data-k="hrow:${c.id}"><span class="nm">${c.n}</span><span class="end">${c.st === 'fail' && c.why ? tag(c.why, 'err') : ''}${stt(c.st, c.t)}${ic('chev', 16, 'muted')}</span></div>`).join('');
  return pageWrap('home', hdr('home', 'Home', status) + card + recCard(p, 'home') +
    `<div class="card" style="display:flex;gap:20px;align-items:center;flex-wrap:wrap"><div style="flex:1;min-width:260px"><div style="font-size:1.35rem;font-weight:700;margin-bottom:4px">${big[0]}</div><div class="muted">${big[1]}</div></div>
     <dl class="kv" style="grid-template-columns:auto auto;margin:0"><dt>Profile</dt><dd><span class="tag">${ic('user', 14)}${esc(p.name)}</span> ${stableChip(p)}</dd><dt>Last played</dt><dd>${last ? `${dayLabel(last.t)} · ${hm(last.mins)}` : 'Not yet'}</dd><dt>Playtime</dt><dd>${tot ? hm(tot) : '0 h'}</dd></dl></div>` +
    `<div class="sec">${sech('Setup')}<div class="rows">${listH}</div></div>`);
};

/* ---------- Mods ---------- */
PAGES.mods = () => {
  const p = P(), on = p.mods.filter(m => m.on).length;
  const head = hdr('mods', 'Mods', `${on} of ${p.mods.length} on`, stableChip(p) + pillP());
  const tabs = `<div class="bar" role="tablist" aria-label="Mods views"><div class="segc">${[['list', 'Mod list'], ['test', 'Test your setup'], ['versions', 'Versions']].map(([v, t]) => `<button type="button" role="tab" aria-selected="${U.tab === v}" data-act="tab" data-arg="${v}" data-k="tab:${v}">${t}</button>`).join('')}</div></div>`;
  return pageWrap('mods-' + U.tab, head + tabs + recCard(p, 'mods') + (U.tab === 'list' ? modList(p) : U.tab === 'versions' ? versionsTab(p) : testTab(p)));
};
function modMatches(m, q, filt, p) {
  const c = CAT[m.id];
  if (q && !(c.name.toLowerCase().includes(q.toLowerCase()) || c.plugins.some(x => x[0].toLowerCase().includes(q.toLowerCase())))) return false;
  if (filt === 'on' && !m.on) return false; if (filt === 'off' && m.on) return false;
  if (filt === 'att' && !modAttention(m, p)) return false;
  return true;
}
function modAttention(m, p) { const mm = missingMasters(p); const files = [...CAT[m.id].plugins.map(x => x[0]), ...p.plugins.filter(x => pinfo(x.f).mod === m.id).map(x => x.f)]; const hit = mm.find(x => files.includes(x[0])); return hit ? `Needs ${hit[1]}` : ''; }
function modList(p) {
  const q = U.q, filt = U.filt, mods = modsInOrder(p);
  const att = mods.filter(m => modAttention(m, p)).length;
  const cnt = { all: p.mods.length, on: p.mods.filter(m => m.on).length, off: p.mods.filter(m => !m.on).length, att };
  const bar = `<div class="bar"><label class="srch">${ic('search', 16)}<span class="sr-only">Search mods</span><input type="search" placeholder="Search mods" value="${esc(q)}" data-val="${esc(q)}" data-in="q" data-k="in:q"></label>${seg([['all', `All ${cnt.all}`], ['on', `On ${cnt.on}`], ['off', `Off ${cnt.off}`], ['att', `Attention ${cnt.att}`]], filt, 'filt', 'Filter mods')}</div>`;
  /* built into the engine */
  const bo = U.secs.builtin;
  const builtin = `<div class="sec">${sech('Built into the engine', `${BUILTIN.length} <span title="These ship inside the engine. Nothing to install.">${ic('info', 15)}</span>`, { fold: bo, click: ['sec', 'builtin'], plain: 'Built into the engine' })}${bo ? `<div class="rows">${BUILTIN.filter(b => !q || b[0].toLowerCase().includes(q.toLowerCase())).map(b => `<div class="r">${ic('lock', 16, 'muted')}<span class="nm">${hl(b[0], q)}</span><span class="sub">${b[2]}</span><span class="end">${tag(b[1], b[1] === 'Built in' ? 'ok' : 'info')}${btn('', { k: 'ghost', cls: 'icon', icon: 'ext', act: 'openLink', arg: 'orig:' + b[0], title: `Original mod: ${b[0]}` })}${btn('', { k: 'ghost', cls: 'icon', icon: 'code', act: 'openLink', arg: 'ours:' + b[0], title: `Our version of ${b[0]}` })}</span></div>`).join('')}</div>` : ''}</div>`;
  /* TTW */
  const tOpen = U.open.ttw;
  let ttw;
  if (!S.g.ttwInstalled) ttw = `<div class="sec"><div class="rows"><div class="r">${ic('pin', 16, 'muted')}${ic('ttw', 20)}<span class="nm">Tale of Two Wastelands</span><span class="sub">Fallout 3 inside New Vegas</span><span class="end">${tag('Not added')}${btn('Add…', { k: 'primary', cls: 'sm', icon: 'plus', act: 'ttwStart', arg: '' })}</span></div></div></div>`;
  else ttw = `<div class="sec"><div class="rows"><div class="mrow ${tOpen ? 'open' : ''}"><div class="r click" data-act="rowOpen" data-arg="ttw">${ic('pin', 16, 'muted')}<span class="hit" data-act="ttwToggle" data-arg="">${sw(p.ttwOn, 'ttwToggle', '', p.ttwOn ? 'Tale of Two Wastelands on. Off: plain New Vegas, TTW stays installed.' : 'Tale of Two Wastelands off')}</span>${ic('ttw', 20)}<button type="button" class="nm" style="text-align:left" data-act="rowOpen" data-arg="ttw" data-k="rowOpen:ttw" aria-expanded="${!!tOpen}">Tale of Two Wastelands</button><span class="end">${S.g.ttwUpdate ? tag('Update available', 'info') : ''}${tag('Bundle', '', 'pin')}<span class="ver">${S.g.ttwVer}</span><button type="button" class="chev ${tOpen ? 'on' : ''}" data-act="rowOpen" data-arg="ttw" aria-label="Details">${ic('chev', 16)}</button></span></div>
    ${tOpen ? `<div class="mdet"><dl class="kv"><dt>What it does</dt><dd>Fallout 3 and its DLC inside New Vegas</dd><dt>Plugins</dt><dd>${TTWP.length} · always load first</dd><dt>Off</dt><dd>Plain New Vegas. TTW stays installed.</dd></dl><div class="acts">${btn(S.g.ttwUpdate ? 'Update TTW…' : 'Check for TTW update', { k: S.g.ttwUpdate ? 'primary' : 'sec', act: S.g.ttwUpdate ? 'ttwStart' : 'ttwCheck', arg: 'update', busy: !!J.ttwcheck })}</div></div>` : ''}</div></div></div>`;
  /* groups */
  let n = 0; const filtered = !!q || filt !== 'all';
  const D = U.drag;
  const groupsH = p.groups.map((g, gi) => {
    const gm = mods.filter(m => m.g === g.id);
    const vis = gm.filter(m => modMatches(m, q, filt, p));
    const nums = gm.map(() => ++n);
    if (filtered && !vis.length) return '';
    const allOn = gm.length && gm.every(m => m.on), someOn = gm.some(m => m.on);
    const open = g.open || filtered;
    const gix = p.groups.filter(x => !D || x.id !== D.id).indexOf(g);
    const dropBefore = D && D.moved && D.kind === 'group' && D.id !== g.id && D.to === gix ? `<div class="dropline" style="margin:0 8px 8px"></div>` : '';
    const rowsH = vis.map(m => {
      const idx = gm.indexOf(m), c = CAT[m.id], o = U.open[m.id], a = modAttention(m, p), gmx = gm.filter(x => !D || x.id !== D.id);
      const drop = D && D.moved && D.kind === 'mod' && D.id !== m.id && D.toG === g.id && D.toI === gmx.indexOf(m) ? '<div class="dropline"></div>' : '';
      return `${drop}<div class="mrow ${o ? 'open' : ''} ${D && D.id === m.id ? 'dragsrc' : ''}" data-mod="${m.id}" data-group="${g.id}" data-idx="${idx}"><div class="r click" data-act="rowOpen" data-arg="${m.id}">
        <span class="grip" data-grip="mod" data-id="${m.id}" title="Drag to move. Keyboard: Alt+Up or Alt+Down on the name.">${ic('grip', 16)}</span><span class="num">${nums[idx]}</span>
        <span class="hit" data-act="modToggle" data-arg="${m.id}">${cbx(m.on, 'modToggle', m.id, `${c.name} ${m.on ? 'on' : 'off'}`)}</span>
        <button type="button" class="nm" style="text-align:left;${m.on ? '' : 'color:var(--muted)'}" data-act="rowOpen" data-arg="${m.id}" data-k="mod:${m.id}" data-move="mod" aria-expanded="${!!o}">${hl(c.name, q)}</button>
        <span class="end">${a ? tag(a, 'err', 'err') : ''}${c.kind === 'dll' ? tag('Engine plugin') : c.kind === 'assets' ? tag('Files only') : ''}<span class="ver">${c.ver}</span><button type="button" class="chev ${o ? 'on' : ''}" data-act="rowOpen" data-arg="${m.id}" aria-label="Details for ${esc(c.name)}" tabindex="-1">${ic('chev', 16)}</button></span></div>
        ${o ? modDetail(p, m) : ''}</div>`;
    }).join('');
    const endDrop = D && D.moved && D.kind === 'mod' && D.toG === g.id && D.toI === gm.filter(x => x.id !== D.id).length ? '<div class="dropline"></div>' : '';
    const isUG = g.id === 'ug';
    return `${dropBefore}<div class="sec grp ${D && D.kind === 'group' && D.id === g.id ? 'dragsrc' : ''}" data-groupcard="${g.id}" data-gi="${gi}"><div class="sech click" data-act="grpFold" data-arg="${g.id}">
      <span class="grip" data-grip="group" data-id="${g.id}" title="Drag to move the group">${ic('grip', 16)}</span>
      <button type="button" class="t" style="text-transform:uppercase" data-act="grpFold" data-arg="${g.id}" data-k="grp:${g.id}" data-move="group" aria-expanded="${open}">${esc(g.name)}</button>
      <span class="r8">${gm.length}${gm.length ? sw(allOn ? true : someOn ? 'mixed' : false, 'grpToggle', g.id, `All mods in ${g.name}`) : ''}${btn('', { k: 'ghost', cls: 'icon', icon: 'more', act: 'grpMenu', arg: g.id, title: `${g.name} options`, pop: 'menu' })}</span>${fold(open, 'grpFold', g.id, (open ? 'Fold ' : 'Unfold ') + g.name)}</div>
      ${open ? `<div class="rows">${rowsH || (gm.length ? '' : `<div class="empty">${isUG ? 'Mods without a group land here.' : 'Empty group. Drag mods here, or use Move to group in a mod’s details.'}</div>`)}${endDrop}</div>` : ''}</div>`;
  }).join('') + (D && D.moved && D.kind === 'group' && D.to === p.groups.length - 1 ? '<div class="dropline" style="margin:0 8px 8px"></div>' : '');
  const none = filtered && !mods.some(m => modMatches(m, q, filt, p)) ? `<div class="empty">No mods match${q ? ` “${esc(q)}”` : ''}. ${btn('Clear filters', { k: 'ghost', cls: 'sm', act: 'clearFilt', arg: '' })}</div>` : '';
  const ust = U.undo[p.id] || [];
  abar((ust.length ? btn('Undo', { k: 'ghost', icon: 'undo', act: 'undo', arg: '', title: `Undo: ${ust[ust.length - 1].label}` }) : '') + btn('New group', { k: 'sec', icon: 'plus', act: 'newGroup', arg: '' }) + btn('Install mod…', { k: 'primary', icon: 'plus', act: 'install', arg: '' }));
  const ghost = D && D.moved ? `<div class="dghost" style="left:${gx(D.x) + 12}px;top:${gy(D.y) - 18}px">${ic('grip', 16)}<b>${esc(D.kind === 'mod' ? CAT[D.id].name : p.groups.find(g => g.id === D.id).name)}</b>${tag(D.kind === 'mod' ? 'Moving' : 'Moving group', 'acc')}</div>` : '';
  return bar + (p.mods.length || filtered ? '' : `<div class="banner info"><span class="bi">${ic('mods', 22)}</span><div><div class="bt">No mods in ${esc(p.name)} yet</div><div class="bs">Install one from an archive. The engine already has the script extenders built in.</div></div><div class="ba">${btn('Install mod…', { k: 'primary', icon: 'plus', act: 'install', arg: '' })}</div></div>`) + builtin + ttw + groupsH + none + ghost;
}
function modDetail(p, m) {
  const c = CAT[m.id], pl = [...c.plugins.map(x => x[0]), ...p.plugins.filter(x => pinfo(x.f).mod === m.id && !c.plugins.some(y => y[0] === x.f)).map(x => x.f)];
  return `<div class="mdet"><dl class="kv"><dt>What it does</dt><dd>${c.what}</dd><dt>Plugins</dt><dd>${pl.length ? pl.map(f => `<span class="mono">${esc(f)}</span>`).join('<br>') : c.kind === 'dll' ? 'None · an engine plugin' : 'None · files only'}</dd>${m.opts && m.opts.length ? `<dt>Options</dt><dd>${m.opts.map(esc).join(' · ')}</dd>` : ''}<dt>Size</dt><dd>${c.size >= 1000 ? (c.size / 1000).toFixed(1) + ' GB' : c.size + ' MB'}</dd><dt>Installed</dt><dd>${fdate(m.at)}</dd>
    <dt><label for="mg-${m.id}">Group</label></dt><dd><select class="inp" id="mg-${m.id}" data-in="modGroup" data-id="${m.id}" data-val="${m.g}" data-k="mg:${m.id}" style="height:34px">${p.groups.map(g => `<option value="${g.id}" ${g.id === m.g ? 'selected' : ''}>${esc(g.name)}</option>`).join('')}</select></dd></dl>
    <div class="acts">${btn('Up', { k: 'ghost', cls: 'sm', icon: 'up', act: 'modMove', arg: m.id + ':-1', title: 'Move up (Alt+Up)' })}${btn('Down', { k: 'ghost', cls: 'sm', icon: 'down', act: 'modMove', arg: m.id + ':1', title: 'Move down (Alt+Down)' })}${btn('Remove…', { k: 'sec', cls: 'sm danger', icon: 'trash', act: 'modRemove', arg: m.id })}</div></div>`;
}

/* Test your setup */
function testTab(p) {
  const jb = J['bench:' + p.id], js = J['stab:' + p.id], jpix = J['pics:' + p.id];
  const last = p.bench[p.bench.length - 1], prev = p.bench[p.bench.length - 2];
  const tiles = last ? `<div class="stiles"><div class="stile"><div class="sl">Average</div><div class="sv okc">${last.avg} fps</div></div><div class="stile"><div class="sl">1 % low</div><div class="sv">${last.low} fps</div></div><div class="stile"><div class="sl">Frame time</div><div class="sv">${last.ft} ms</div><div class="sl">p99 ${last.p99} ms</div></div><div class="stile"><div class="sl">${prev ? `Against ${fdate(prev.t)}` : 'First run'}</div>${prev ? `<div class="sv ${last.avg >= prev.avg ? 'okc' : 'warnc'}">${last.avg >= prev.avg ? '+' : ''}${Math.round((last.avg - prev.avg) / prev.avg * 100)} %</div>` : '<div class="sv">–</div>'}</div></div>` : `<div class="empty">No benchmark for ${esc(p.name)} yet.</div>`;
  const benchRun = jb ? `<div class="card sunk"><div style="display:flex;gap:12px;align-items:center;margin-bottom:8px">${ic('refresh', 18, 'spin accc')}<b>Running the benchmark route</b><span class="muted">${['Loading Goodsprings', 'Walking to Primm', 'Crossing the Strip', 'Back to the start'][Math.min(3, Math.floor(jb.p * 4))]} · ${Math.round(55 + Math.sin(jb.p * 20) * 9)} fps</span></div>${pbar(jb.p)}</div>` : '';
  const bench = `<div class="sec">${sech(`${ic('bolt', 16)} Benchmark`, jb ? btn('Cancel', { k: 'ghost', cls: 'sm', act: 'cancelJob', arg: 'bench:' + p.id }) : btn('Run benchmark', { k: 'primary', cls: 'sm', icon: 'play', act: 'benchGo', arg: '' }))}${benchRun || tiles}
    <div style="margin-top:12px">${benchChart(p)}</div><div class="bar" style="margin:8px 0 0">${tag('Average fps', 'acc')}${tag('1 % low, dashed')}${p.events.length ? tag('Changes to the setup', 'info', 'mods') : ''}${p.bench.some(b => b.ex) ? tag('Includes example runs') : ''}</div>
    </div>`;
  /* stability */
  const lastS = p.stab[p.stab.length - 1], stn = stableNow(p);
  const sRun = js ? `<div class="card sunk"><div style="display:flex;gap:12px;align-items:center;margin-bottom:8px">${ic('refresh', 18, 'spin accc')}<b>${js.p < .55 ? `Loading cells · ${Math.floor(js.p / .55 * 40)} of 40` : js.p < .9 ? 'Travelling across the map' : 'Saving and reloading'}</b><span class="muted">${Math.round(js.p * 20)} of 20 min · crashes 0 · hitches ${Math.floor(js.p * 3)}</span></div>${pbar(js.p)}</div>` : '';
  const sRes = lastS ? `<div class="r"><span>${lastS.ok ? stt('ok', 'Passed') : stt('fail', 'Crashed')}</span><span class="sub">${fdate(lastS.t)} · ${lastS.ok ? `${lastS.cells} cells, ${lastS.mins} min, ${lastS.hitches} hitches` : `in ${lastS.where} after ${lastS.mins} min`}${lastS.ex ? ' · example run' : ''}</span><span class="end">${lastS.ok ? '' : btn('Show in log', { k: 'ghost', cls: 'sm', act: 'logFilter', arg: 'err' })}</span></div>` : '';
  const so = U.secs.stablelist;
  const stab = `<div class="sec">${sech(`${ic('shield', 16)} Stability test`, stableChip(p))}
    ${sRun || `<div class="rows"><div class="r"><span class="nm">${stn ? 'This exact mod set and order passed a stability test.' : 'This exact mod set and order has no passed test yet.'}</span></div>${sRes}</div>`}
    <div class="sech click" data-act="sec" data-arg="stablelist" style="min-height:40px;padding-top:12px"><span class="muted">Known stable setups · ${p.stable.length}</span><span class="r8"></span>${fold(so, 'sec', 'stablelist', 'Known stable setups')}</div>
    ${so ? `<div class="rows">${p.stable.length ? p.stable.map((s, i) => `<div class="r"><span class="nm">${s.sig.split('|')[0].replace(/^T?;/, '').split(',').filter(Boolean).length} mods${s.sig.startsWith('T') ? ' + TTW' : ''}</span><span class="sub">Passed ${fdate(s.t)}</span><span class="end">${s.sig === sig(p) ? tag('Matches now', 'ok') : ''}${btn('Forget', { k: 'ghost', cls: 'sm', act: 'forgetStable', arg: i })}</span></div>`).join('') : '<div class="empty">None yet. A passed test adds the setup here.</div>'}</div>` : ''}
    <div class="cca"><span class="note">Loads 40 places, travels, about 20 min</span>${js ? btn('Cancel', { k: 'ghost', act: 'cancelJob', arg: 'stab:' + p.id }) : btn('Run stability test', { k: stn ? 'sec' : 'primary', icon: 'play', act: 'stabGo', arg: 'tab' })}</div></div>`;
  /* pictures */
  const allPlaces = PLACES.filter(x => x[2] !== 'Capital Wasteland' || ttwActive(p)), worldOf = x => x[2] === 'Capital Wasteland' ? 'cw' : 'nv';
  if (U.picWorld === 'cw' && !ttwActive(p)) U.picWorld = 'nv';
  const q = (U.picQ || '').trim().toLowerCase();
  const places = allPlaces.filter(x => q ? (x[1].toLowerCase().includes(q) || x[2].toLowerCase().includes(q)) : worldOf(x) === U.picWorld);
  const showAll = U.picAll || !!q;
  if (!allPlaces.find(x => x[0] === U.picPlace)) U.picPlace = allPlaces[0][0];
  const curPlace = allPlaces.find(x => x[0] === U.picPlace);
  const runs = p.pics; const A = runs.find(r => r.id === U.picA) || runs[runs.length - 2] || runs[0], B = runs.find(r => r.id === U.picB) || runs[runs.length - 1];
  const opts = (sel, which) => `<select class="inp" style="height:32px" data-in="pic${which}" data-val="${sel ? sel.id : ''}" data-k="pic${which}" aria-label="Picture ${which}">${runs.map(r => `<option value="${r.id}">${fdate(r.t)} · ${QUAL[r.q] ? QUAL[r.q][0] : 'Custom'}${r.ex ? ' · example' : ''}</option>`).join('')}</select>`;
  const pic = (r, which) => r ? `<div class="pic"><div class="vp">${scene(U.picPlace, { q: r.q, trees: r.trees ?? p.mods.some(m => m.id === 'wfo' && m.on), look: r.look ?? (p.mods.some(m => m.id === 'nsk' && m.on) ? 1 : 0), seed: r.id, label: `${curPlace[1]}, ${fdate(r.t)}` }).replace('<svg ', `<svg style="transform:translate(${(U.pan.x * 100).toFixed(3)}%,${(U.pan.y * 100).toFixed(3)}%) scale(${U.zoom.toFixed(3)})" `)}</div><div class="cap"><b>${which}</b>${opts(r, which)}</div></div>` : '';
  const pics = `<div class="sec">${sech(`${ic('camera', 16)} Location pictures`, `<span class="muted">${runs.length} sets</span>`)}
    <div class="bar">${ttwActive(p) ? seg([['nv', 'New Vegas'], ['cw', 'Capital Wasteland']], q ? '' : U.picWorld, 'picWorld', 'World') : '<b class="muted">New Vegas</b>'}<label class="srch" style="min-width:200px">${ic('search', 16)}<span class="sr-only">Search places</span><input type="search" placeholder="Search places" value="${esc(U.picQ)}" data-val="${esc(U.picQ)}" data-in="picQ" data-k="in:picQ"></label></div>
    <div class="plrow"><div class="places${showAll ? '' : ' strip'}" role="group" aria-label="Place">${places.map(([id, n]) => `<button type="button" class="btn sm ${U.picPlace === id ? 'primary' : ''}" aria-pressed="${U.picPlace === id}" data-act="picPlace" data-arg="${id}" data-k="picPlace:${id}">${n}</button>`).join('') || '<span class="muted">No place matches.</span>'}</div>${q ? `<span class="muted nowrap">${plural(places.length, 'match', 'matches')}</span>` : btn(U.picAll ? 'Fewer' : `All ${places.length}`, { k: 'ghost', cls: 'sm', act: 'picAll' })}</div>
    <div class="bar"><b>${esc(curPlace[1])}</b><span class="muted">${esc(curPlace[2])}</span><span style="margin-left:auto" class="muted">Wheel zooms, drag pans, double-click resets</span>${seg([['two', 'Side by side'], ['one', 'Single']], U.picMode, 'picMode', 'Compare')}${seg([[1, '1x'], [2, '2x'], [4, '4x'], [8, '8x']].map(([v, t]) => [String(v), t]), [1, 2, 4, 8].includes(U.zoom) ? String(U.zoom) : '', 'zoom', 'Zoom')}</div>
    ${jpix ? `<div class="card sunk"><div style="display:flex;gap:12px;align-items:center;margin-bottom:8px">${ic('refresh', 18, 'spin accc')}<b>Taking pictures · ${Math.min(allPlaces.length, Math.floor(jpix.p * allPlaces.length) + 1)} of ${allPlaces.length} places</b></div>${pbar(jpix.p)}</div>` : runs.length ? (U.picMode === 'one' ? `<div class="pics one">${pic(B, 'Latest')}</div>` : `<div class="pics">${pic(A, 'A')}${pic(B, 'B')}</div>`) : `<div class="empty">No pictures yet.</div>`}
    <div class="cca"><span class="note">Same spot, same time of day, side by side</span>${jpix ? btn('Cancel', { k: 'ghost', act: 'cancelJob', arg: 'pics:' + p.id }) : btn('Take pictures', { k: 'primary', icon: 'camera', act: 'picsGo', arg: '' })}</div></div>`;
  /* log */
  const log = testLog(p), nw = log.filter(l => l.k === 'warn').length, ne = log.filter(l => l.k === 'err').length;
  const shown = log.filter(l => (U.logF === 'all' || l.k === U.logF) && (!U.logQ || (l.t + l.d).toLowerCase().includes(U.logQ.toLowerCase())));
  const fo = U.secs.fulllog;
  const logH = `<div class="sec">${sech('Log', `${nw ? tag(plural(nw, 'warning'), 'warn', 'warn') : ''}${ne ? tag(plural(ne, 'error'), 'err', 'err') : ''}${!nw && !ne ? tag('Clean', 'ok', 'ok') : ''}`)}
    <div class="rows">${log.filter(l => l.k !== 'info').slice(0, 3).map(l => logLine(l)).join('') || '<div class="empty">No warnings or errors from the last runs.</div>'}</div>
    <div class="sech click" data-act="sec" data-arg="fulllog" style="min-height:40px;padding-top:12px"><span class="muted">Full log · ${log.length} lines</span><span class="r8"></span>${fold(fo, 'sec', 'fulllog', 'Full log')}</div>
    ${fo ? `<div class="bar">${seg([['all', 'All'], ['warn', 'Warnings'], ['err', 'Errors']], U.logF, 'logF', 'Log filter')}<label class="srch">${ic('search', 16)}<span class="sr-only">Search the log</span><input type="search" placeholder="Search the log" value="${esc(U.logQ)}" data-val="${esc(U.logQ)}" data-in="logQ" data-k="in:logQ"></label></div><div class="logbox">${shown.map(l => logLine(l, true)).join('') || '<div class="logl muted">Nothing matches.</div>'}</div>` : ''}</div>`;
  return bench + stab + pics + logH;
}
function testLog(p) {
  const L = []; const lastS = p.stab[p.stab.length - 1];
  const has = id => p.mods.some(m => m.id === id && m.on);
  L.push({ k: 'info', tm: '00:00', t: 'Stability test started', d: `${p.mods.filter(m => m.on).length} mods, ${allActiveFiles(p).length} plugins` });
  if (has('wfo')) L.push({ k: 'warn', tm: '03:12', t: 'Missing texture in WFO.esp', d: 'A shrub near Primm uses a texture the mod does not ship. It draws grey; nothing breaks.' });
  if (has('nmc')) L.push({ k: 'warn', tm: '06:40', t: 'Slow place load: Freeside (2.1 s)', d: 'Large textures from NMC’s Texture Pack. Expected with 2K textures.' });
  if (has('fco')) L.push({ k: 'warn', tm: '09:05', t: 'Face tint mismatch for two characters', d: 'Fallout Character Overhaul changes faces that another plugin also edits.' });
  L.push({ k: 'info', tm: '10:00', t: 'Travel: Goodsprings → New Vegas Strip', d: 'Fast travel 6 times, 0 problems.' });
  if (lastS && !lastS.ok) L.push({ k: 'err', tm: `${String(lastS.mins).padStart(2, '0')}:20`, t: `Crash in ${lastS.where || 'Freeside'}`, d: `The game stopped while loading ${lastS.where || 'Freeside'}. Last plugin to touch the place: ${visPlugins(p).filter(x => x.on).slice(-1)[0]?.f || 'FalloutNV.esm'}. Try the stability test again after turning recent mods off.` });
  L.push({ k: 'info', tm: '20:00', t: 'Stability test finished', d: lastS && lastS.ok ? 'Passed' : 'See the error above' });
  return L;
}
const logLine = (l, full) => { const k = l.k + l.tm; const o = U.logOpen[k]; return `<div><button type="button" class="logl" aria-expanded="${!!o}" data-act="logOpen" data-arg="${k}" data-k="log:${k}${full ? 'f' : ''}">${ic(l.k === 'err' ? 'err' : l.k === 'warn' ? 'warn' : 'info', 16, l.k === 'err' ? 'errc' : l.k === 'warn' ? 'warnc' : 'muted')}<span class="tm">${l.tm}</span><span>${esc(l.t)}</span></button>${o ? `<div class="logd">${esc(l.d)}</div>` : ''}</div>`; };

/* ---------- Load order ---------- */
PAGES.load = () => {
  const p = P(), prev = U.sortPrev && U.sortPrev.pid === p.id ? U.sortPrev : null;
  const vis = visPlugins(p), act = allActiveFiles(p), mm = missingMasters(p), ooo = outOfOrder(p);
  const status = `${plural(BASE.length + (ttwActive(p) ? TTWP.length : 0) + vis.length, 'plugin')} · ${act.length} active · ${prev ? `preview, ${plural(prev.moves.length, 'move')}` : mm.length ? `<span class="errc">${plural(mm.length, 'missing master')}</span>` : ooo ? `<span class="warnc">${ooo} out of order</span>` : '<span class="okc">sorted</span>'}${stableNow(p) ? '' : ' · <span class="warnc">untested change</span>'}`;
  const q = U.lq, filt = U.lfilt;
  const order = prev ? prev.sorted.map(f => p.plugins.find(x => x.f === f)) : vis;
  const movedSet = new Set(prev ? prev.moves.map(m => m[0]) : []);
  const oooSet = new Set(sortMoves(p).moves.map(m => m[0]));
  const D = U.drag;
  let n = BASE.length + (ttwActive(p) ? TTWP.length : 0);
  const rowsH = order.map((x, i) => {
    const num = ++n, inf = pinfo(x.f), modName = inf.mod ? CAT[inf.mod].name : '';
    if (q && !(x.f.toLowerCase().includes(q.toLowerCase()) || modName.toLowerCase().includes(q.toLowerCase()))) return '';
    if (filt === 'ooo' && !oooSet.has(x.f)) return ''; if (filt === 'moved' && !movedSet.has(x.f)) return '';
    const miss = mm.filter(m => m[0] === x.f);
    const mv = prev && prev.moves.find(m => m[0] === x.f);
    const drop = D && D.moved && D.kind === 'plugin' && D.id !== x.f && D.to === order.filter(y => y.f !== D.id).indexOf(x) ? '<div class="dropline"></div>' : '';
    return `${drop}<div class="mrow ${mv ? 'moved' : ''} ${D && D.id === x.f ? 'dragsrc' : ''}" data-plugin="${esc(x.f)}" data-idx="${i}"><div class="r click" data-act="plugToggle" data-arg="${esc(x.f)}" style="align-items:flex-start;padding-top:10px;padding-bottom:10px">
      ${prev ? '<span class="grip"></span>' : `<span class="grip" data-grip="plugin" data-id="${esc(x.f)}" title="Drag to move. Keyboard: Alt+Up or Alt+Down.">${ic('grip', 16)}</span>`}<span class="num">${num}</span>${cbx(x.on, 'plugToggle', x.f, `${x.f} ${x.on ? 'active' : 'inactive'}`)}
      <div style="min-width:0;flex:1"><button type="button" class="mono" style="text-align:left;${x.on ? '' : 'color:var(--muted)'}" data-act="plugToggle" data-arg="${esc(x.f)}" aria-pressed="${x.on}" data-move="plugin" data-id="${esc(x.f)}" data-k="plug:${esc(x.f)}">${hl(x.f, q)}</button><div class="sub">${esc(modName)}</div>
      ${miss.map(m => `<div class="errc" style="font-size:.86rem;margin-top:2px">${ic('err', 14)} Needs ${esc(m[1])}, which is off or missing</div>`).join('')}${inf.note ? `<div class="sub" style="margin-top:2px">${ic('info', 14)} ${esc(inf.note)}</div>` : ''}</div>
      <span class="end">${mv ? tag(mv[1] > 0 ? `Up ${mv[1]}` : `Down ${-mv[1]}`, 'acc', mv[1] > 0 ? 'up' : 'down') : !prev && oooSet.has(x.f) ? tag('Out of order', 'warn') : ''}</span></div></div>`;
  }).join('') + (D && D.moved && D.kind === 'plugin' && D.to === order.length - 1 ? '<div class="dropline"></div>' : '');
  const lock = (key, title, files) => { const o = U.secs[key]; return `<div class="sec">${sech(title, `${files.length} · always on`, { fold: o, click: ['sec', key], plain: title })}${o ? `<div class="rows">${files.map((f, i) => `<div class="r dim"><span class="grip">${ic('lock', 14)}</span><span class="num">${(key === 'lttw' ? BASE.length : 0) + i + 1}</span><span class="mono">${f}</span></div>`).join('')}</div>` : ''}</div>`; };
  const ust = U.undo[p.id] || [];
  abar(prev ? btn('Cancel', { k: 'ghost', act: 'sortCancel', arg: '' }) + btn(prev.moves.length ? `Apply ${plural(prev.moves.length, 'move')}` : 'Nothing to move', { k: 'primary', icon: 'check', act: 'sortApply', arg: '', dis: !prev.moves.length })
    : (ust.length ? btn('Undo', { k: 'ghost', icon: 'undo', act: 'undo', arg: '', title: `Undo: ${ust[ust.length - 1].label}` }) : '') + btn('Sort…', { k: ooo ? 'primary' : 'sec', icon: 'load', act: 'sortGo', arg: '' }));
  const movesCard = prev ? `<div class="sec">${sech(prev.moves.length ? `The ${plural(prev.moves.length, 'move')}` : 'Already sorted')}<div class="rows">${prev.moves.length ? prev.moves.map(m => `<div class="r"><span class="mono">${esc(m[0])}</span><span class="end">${tag(m[1] > 0 ? `Up ${m[1]}` : `Down ${-m[1]}`, 'acc', m[1] > 0 ? 'up' : 'down')}</span></div>`).join('') : '<div class="r"><span class="nm">Every plugin is where the masterlist wants it.</span></div>'}</div><p class="muted" style="margin:8px 4px 2px">A restore point is made first, and Undo stays after.</p></div>` : '';
  return pageWrap('load', hdr('load', 'Load order', status, stableChip(p) + pillP()) + movesCard + recCard(p, 'load') +
    `<div class="bar"><label class="srch">${ic('search', 16)}<span class="sr-only">Search plugins</span><input type="search" placeholder="Search plugins" value="${esc(q)}" data-val="${esc(q)}" data-in="lq" data-k="in:lq"></label>${seg(prev ? [['all', `All ${order.length}`], ['moved', `Moved ${prev.moves.length}`]] : [['all', `All ${vis.length}`], ['ooo', `Out of order ${ooo}`]], filt, 'lfilt', 'Filter plugins')}</div>` +
    lock('lbase', 'Base game', BASE) + (ttwActive(p) ? lock('lttw', 'Tale of Two Wastelands', TTWP) : '') +
    `<div class="sec">${sech('Your plugins', `${vis.length}`)}<div class="rows">${rowsH || `<div class="empty">${vis.length ? 'No plugins match.' : 'No plugins yet. Mods with plugins add them here.'}</div>`}</div></div>` +
    (D && D.moved && D.kind === 'plugin' ? `<div class="dghost" style="left:${gx(D.x) + 12}px;top:${gy(D.y) - 18}px">${ic('grip', 16)}<b class="mono">${esc(D.id)}</b>${tag('Moving', 'acc')}</div>` : ''));
};

/* ---------- LOD ---------- */
PAGES.far = () => {
  const p = P(), st = farState(p), j = J['far:' + p.id], b = p.far.built;
  const STX = { ok: ['ok', 'Up to date', `Matches your ${p.mods.filter(m => m.on).length} enabled mods.`], never: ['warn', 'Not built', 'Far away looks flat until you build it.'], 'stale-mods': ['warn', 'Out of date', 'Mods changed since the last build.'], 'stale-q': ['warn', 'Out of date', 'Quality changed since the last build.'], building: ['run', 'Building', ''] };
  const sx = STX[st];
  const stages = ['Reading your mods', 'Terrain', 'Objects', 'Trees', 'Packing'];
  const right = j ? `<div style="font-size:1.25rem;font-weight:700;margin-bottom:6px">${stt('run', `Building · ${Math.round(j.p * 100)} %`)}</div><div class="muted" style="margin-bottom:8px">${stages[Math.min(4, Math.floor(j.p * 5))]} · step ${Math.min(5, Math.floor(j.p * 5) + 1)} of 5 · about ${Math.max(1, Math.round((1 - j.p) * QUAL[p.far.q][2]))} min left</div>${pbar(j.p)}`
    : `<div style="font-size:1.25rem;font-weight:700;margin-bottom:4px">${stt(sx[0], sx[1])}</div><div class="muted" style="margin-bottom:10px">${sx[2]}</div><div class="bar" style="margin:0">${b ? tag(`Built ${ago(b.t)}`, '', 'clock') + tag(`${b.size} GB`, '', 'box') + tag(QUAL[b.q] ? QUAL[b.q][0] : 'Custom') : ''}</div>`;
  const q = p.far.q;
  const det = U.secs.details;
  abar(btn('Fine-tune', { k: 'ghost', icon: 'sliders', act: 'nav', arg: 'farset' }) + (j ? btn('Cancel…', { k: 'sec', act: 'farCancel', arg: '' }) : btn(st === 'never' ? 'Build LOD' : 'Rebuild LOD', { k: st === 'ok' ? 'sec' : 'primary', icon: st === 'never' ? 'play' : 'refresh', act: 'farBuild', arg: '' })));
  return pageWrap('far', hdr('lod', `LOD <span title="Render distance" style="display:inline-flex;vertical-align:2px">${ic('info', 16, 'muted')}</span>`, stt(j ? 'run' : sx[0], j ? 'Building' : sx[1]), pillP()) +
    `<div class="card farhero"><div>${farHero(p, st)}</div><div>${right}</div></div>
    <div class="sec">${sech('Quality')}<div class="bar" style="margin:0 4px 6px">${seg(Object.entries(QUAL).map(([k, v]) => [k, v[0]]), p.far.fine ? '' : q, 'farQ', 'LOD quality')}<span class="muted">${farSize(p).toFixed(1)} GB · about ${QUAL[q][2]} min</span>${p.far.fine ? tag('Custom', 'info') : ''}<span style="margin-left:auto">${btn('Fine-tune', { k: 'ghost', cls: 'sm', act: 'nav', arg: 'farset' })}</span></div><p class="muted" style="margin:4px 6px 4px">${p.far.fine ? 'Your own values from Fine-tune.' : QUAL[q][3]}</p></div>
    <div class="sec">${sech('Details', '', { fold: det, click: ['sec', 'details'], plain: 'Details' })}${det ? `<dl class="kv" style="padding:4px 8px 8px"><dt>Profile</dt><dd>${esc(p.name)} · each profile keeps its own LOD</dd><dt>Built</dt><dd>${b ? `${fdate(b.t)} ${ftime(b.t)}` : 'Never'}</dd><dt>Quality</dt><dd>${b ? (QUAL[b.q] ? QUAL[b.q][0] : 'Custom') : '–'}</dd><dt>Size</dt><dd>${b ? b.size + ' GB' : '–'}</dd><dt>Worlds</dt><dd>Mojave Wasteland and the DLC worlds${ttwActive(p) ? ', plus the Capital Wasteland' : ''}</dd><dt>Mods in it</dt><dd>${b ? plural(b.n, 'mod') : '–'}</dd></dl>` : ''}</div>`);
};
PAGES.farset = () => {
  const p = P(), f = p.far.fine || { tex: { performance: 256, balanced: 512, high: 1024 }[p.far.q], dist: { performance: 2, balanced: 3, high: 5 }[p.far.q], trees: { performance: 'sparse', balanced: 'normal', high: 'dense' }[p.far.q], plants: p.far.q === 'high' };
  abar(btn('Restore defaults', { k: 'ghost', icon: 'undo', act: 'fineReset', arg: '', dis: !p.far.fine }) + btn('Done', { k: 'primary', act: 'nav', arg: 'far' }));
  const row = (label, help, ctl) => `<div class="r${swRow(ctl)}" style="min-height:64px"><div style="flex:1"><div class="nm">${label}</div><div class="sub">${help}</div></div><span class="end">${ctl}</span></div>`;
  return pageWrap('farset', hdr('sliders', 'LOD settings', `${qLabel(p)} · used by the next build`, pillP()) +
    `<div class="sec">${sech('Fine-tune')}<div class="rows">
    ${row('Texture size', 'Bigger looks sharper and uses more video memory.', seg([['256', '256'], ['512', '512'], ['1024', '1024']], String(f.tex), 'fine', 'Texture size').replace(/data-arg="(\d+)"/g, 'data-arg="tex:$1"'))}
    ${row('Distance', `How far LOD reaches · ${['', 'Near', 'Short', 'Normal', 'Far', 'Very far'][f.dist]}`, `<input type="range" class="rng" min="1" max="5" step="1" value="${f.dist}" data-val="${f.dist}" data-in="fineDist" data-k="fineDist" aria-label="Distance">`)}
    ${row('Tree density', 'Trees and big shrubs in the distance.', seg([['sparse', 'Sparse'], ['normal', 'Normal'], ['dense', 'Dense']], f.trees, 'fine', 'Tree density').replace(/data-arg="(sparse|normal|dense)"/g, 'data-arg="trees:$1"'))}
    ${row('Far desert plants', 'Small desert plants in the distance. Costs some frames.', sw(f.plants, 'fine', 'plants:' + (!f.plants), 'Far desert plants'))}
    </div></div>`);
};

/* ---------- TTW flow ---------- */
const TTW_STEPS = ['Fallout 3', 'TTW file', 'Install', 'Done'];
PAGES.ttw = () => {
  const F = U.ttwFlow || (U.ttwFlow = { step: 0 }); const s = F.step;
  const fo3 = S.g.fo3Path && !S.g.scen.fo3, j = J.ttw;
  const subs = [fo3 ? 'Found' : 'Not found', 'Found', j ? `${Math.round(j.p * 100)} %` : s > 2 ? 'Done' : 'Waiting', s === 3 ? (S.g.ttwUpdate ? 'Update available' : 'Up to date') : 'Waiting'];
  const stepper = `<div class="stp" aria-label="Steps">${TTW_STEPS.map((t, i) => `${i ? `<span class="ln ${i <= s ? 'on' : ''}"></span>` : ''}<div class="s ${i < s ? 'done' : i === s ? 'cur' : ''} ${i === 0 && !fo3 && s === 0 ? 'bad' : ''}" ${i === s ? 'aria-current="step"' : ''}><span class="n">${i < s ? ic('check', 16) : i + 1}</span><span class="lb"><b>${t}</b><span>${subs[i]}</span></span></div>`).join('')}</div>`;
  let body = '';
  if (s === 0) body = `<div class="rows"><div class="r" style="min-height:64px"><div style="flex:1"><div class="nm">Fallout 3</div><div class="sub mono">${fo3 ? esc(S.g.fo3Path) : 'No Fallout 3 folder found'}</div></div><span class="end">${J['find:fo3'] ? stt('run', 'Looking…') : fo3 ? stt('ok', 'Found') : stt('fail', 'Not found')}${btn('Browse…', { k: 'ghost', icon: 'folder', act: 'pickFolder', arg: 'fo3' })}${btn('Find', { k: 'ghost', icon: 'search', act: 'findFolder', arg: 'fo3', busy: !!J['find:fo3'] })}</span></div></div>${fo3 ? '' : '<p class="muted" style="margin:10px 4px">TTW needs Fallout 3 GOTY installed. Steam and GOG copies both work.</p>'}`;
  if (s === 1) body = `<div class="rows"><div class="r" style="min-height:64px"><div style="flex:1"><div class="nm">TTW installer file</div><div class="sub mono">Downloads › ${esc(S.g.ttwFile)}</div></div><span class="end">${stt('ok', 'Found')}${btn('Browse…', { k: 'ghost', icon: 'folder', act: 'openLink', arg: 'browse:ttw' })}</span></div></div><div class="cca" style="justify-content:flex-start">${btn('Get the file', { k: 'ghost', icon: 'link', act: 'openLink', arg: 'ttwsite' })}</div>`;
  if (s === 2) {
    const low = !ttwRoom(S.g.storeFree);
    body = j ? `<div class="card sunk"><div style="display:flex;gap:12px;align-items:center;margin-bottom:8px">${ic('refresh', 18, 'spin accc')}<b>${['Checking the Fallout 3 files', 'Converting sound', 'Building archives', 'Patching plugins', 'Adding TTW to your mods'][Math.min(4, Math.floor(j.p * 5))]}</b><span class="muted">${Math.round(j.p * 100)} % · about ${Math.max(1, Math.round((1 - j.p) * 22))} min left</span></div>${pbar(j.p)}</div>`
      : `<div class="rows"><div class="r"><span class="nm">Space needed</span><span class="end">${low ? stt('fail', `25 GB, only ${S.g.storeFree} GB free`) : stt('ok', `25 GB of ${S.g.storeFree} GB free`)}</span></div><div class="r"><span class="nm">Takes about</span><span class="end">22 min</span></div><div class="r"><span class="nm">Lands in</span><span class="end">Mods, pinned at the top</span></div></div>${low ? `<div class="banner err" style="margin-top:12px"><span class="bi">${ic('err', 22)}</span><div><div class="bt">Not enough space</div><div class="bs">Free some space or move the Toolbox storage.</div></div><div class="ba">${btn('Free up space…', { k: 'sec', act: 'cleanCache', arg: '' })}${btn('Change storage', { k: 'sec', act: 'pickFolder', arg: 'store' })}</div></div>` : ''}`;
  }
  if (s === 3) body = `<div class="banner ok"><span class="bi">${ic('ok', 22)}</span><div><div class="bt">Tale of Two Wastelands ${S.g.ttwVer} is in your mods</div><div class="bs">Switch it on or off per profile in Mods.</div></div></div>`;
  const next = s === 0 ? btn('Next: TTW file', { k: 'primary', act: 'ttwNext', arg: '', dis: !fo3 }) : s === 1 ? btn('Next: Install', { k: 'primary', act: 'ttwNext', arg: '' }) : s === 2 ? (j ? '' : btn('Install', { k: 'primary', icon: 'dl', act: 'ttwInstall', arg: '', dis: !ttwRoom(S.g.storeFree) })) : btn('Done', { k: 'primary', act: 'ttwClose', arg: '' });
  abar((j ? btn('Cancel', { k: 'ghost', act: 'cancelJob', arg: 'ttw' }) : btn('Close', { k: 'ghost', icon: 'x', act: 'ttwClose', arg: '' }) + (s > 0 && s < 3 ? btn('Back', { k: 'ghost', act: 'ttwBack', arg: '' }) : '')) + next);
  return pageWrap('ttw', hdr('ttw', 'Add Tale of Two Wastelands', `Step ${s + 1} of 4`) + stepper + body);
};

/* ---------- System ---------- */
const vcmp = (a, b) => { const x = a.split('.').map(Number), y = b.split('.').map(Number); for (let i = 0; i < 3; i++) if (x[i] !== y[i]) return x[i] - y[i]; return 0; };
const parts = () => PARTS.map(c => S.g.tbVer === TB().nw ? { ...c, cur: c.nw } : c);
const updLine = () => `Updates Raul's Toolbox ${TB().cur} → ${TB().nw}, including ${PARTS.filter(c => c.id !== 'toolbox' && vcmp(c.nw, c.cur) > 0).map(c => `${c.n} ${c.nw}`).join(', ')}`;
PAGES.system = () => {
  const chk = J.check, up = J.update, ml = J.ml, avail = updAvail();
  const checked = chk ? '' : S.g.lastCheck ? `<span class="muted" style="margin-left:8px">Checked ${ago(S.g.lastCheck)}</span>` : '';
  const status = (chk ? stt('run', 'Checking for updates…') : up ? stt('run', 'Updating') : avail ? stt('info', 'Update available') : stt('ok', 'Up to date')) + checked;
  let top = '';
  if (chk) top = `<div class="banner acc"><span class="bi">${ic('refresh', 22, 'spin')}</span><div style="flex:1"><div class="bt">Checking for updates…</div><div style="margin-top:8px">${pbar(0, 'ind')}</div></div></div>`;
  else if (up) top = `<div class="banner acc"><span class="bi">${ic('refresh', 22, 'spin')}</span><div style="flex:1"><div class="bt">Updating to ${TB().nw} · ${Math.round(up.p * 100)} %</div><div class="bs">${['Downloading', 'Checking the download', 'Installing the parts', 'Restarting Raul’s Toolbox'][Math.min(3, Math.floor(up.p * 4))]}</div><div style="margin-top:8px">${pbar(up.p)}</div></div></div>`;
  else if (U.updErr) top = `<div class="banner err" role="alert"><span class="bi">${ic('err', 22)}</span><div><div class="bt">The download stopped</div><div class="bs">The server didn't answer. Your current version is untouched.</div></div></div>`;
  else if (avail) top = `<div class="banner info"><span class="bi">${ic('dl', 22)}</span><div><div class="bt">Update available</div><div class="bs">${updLine()}</div></div></div>`;
  else if (U.justUpdated) top = `<div class="banner ok"><span class="bi">${ic('ok', 22)}</span><div><div class="bt">Updated to ${TB().nw}</div><div class="bs">The change list is under Raul's Toolbox below. ${TB().cur} is kept, so you can go back.</div></div></div>`;
  abar(chk ? '' : up ? btn('Cancel', { k: 'ghost', act: 'cancelJob', arg: 'update' }) : btn('Check again', { k: 'sec', icon: 'refresh', act: 'checkUpdates', arg: '' }) + (avail ? btn(U.updErr ? 'Try again' : 'Update', { k: 'primary', icon: 'dl', act: 'update', arg: '', title: updLine() }) : ''));
  const rowsH = parts().map(c => {
    const upd = vcmp(c.nw, c.cur) > 0, o = U.vopen === c.id;
    return `<div class="mrow ${o ? 'open' : ''}"><div class="r click" role="button" tabindex="0" aria-expanded="${o}" data-act="vopen" data-arg="${c.id}" data-k="vopen:${c.id}">${ic(c.icon, 20)}<span class="nm">${c.n}</span><span class="sub">${c.role}</span><span class="end">${upd ? `<span class="muted">${c.nw} · will be updated</span>` : ''}<span class="ver">${c.id === 'engine' ? S.g.engine : c.cur}</span><span class="chev ${o ? 'on' : ''}">${ic('chev', 16)}</span></span></div>
    ${o ? `<div class="mdet" style="display:block;padding-left:48px">${c.v.map(v => `<div style="display:flex;gap:10px;align-items:center;margin:8px 0 4px"><b class="mono">${v[0]}</b><span class="muted">${v[1]}</span>${vcmp(v[0], c.cur) > 0 ? tag('With the update', 'info') : v[0] === c.cur ? tag('Installed', 'ok') : ''}</div>${v[2].map(l => `<div style="display:flex;gap:8px;align-items:center;padding:3px 0">${l[0] === 'feat' ? tag('Feature', 'acc') : tag('Fix')}<span>${l[1]}</span></div>`).join('')}`).join('')}</div>` : ''}</div>`;
  }).join('');
  const eo = U.secs.engines;
  const eng = `<div class="sec">${sech('Engine builds', `${S.g.builds.length}`, { fold: eo, click: ['sec', 'engines'], plain: 'Engine builds' })}${eo ? `<div class="rows">${S.g.builds.map(v => `<div class="r">${ic('bolt', 18)}<span class="nm">Engine <span class="mono">${v}</span></span><span class="end">${v === S.g.engine ? tag('In use', 'ok') : btn('Use this build…', { k: 'sec', cls: 'sm', act: 'useBuild', arg: v }) + btn('Remove…', { k: 'ghost', cls: 'sm', act: 'removeBuild', arg: v })}</span></div>`).join('')}</div><p class="muted" style="margin:8px 6px 2px">Older builds stay so you can go back if a new one misbehaves.</p>` : ''}</div>`;
  const mlH = `<div class="sec">${sech('Masterlist', `<span class="muted">${ml ? 'Updating…' : `Updated ${ago(S.g.mlAt)}`}</span>`)}<div class="rows"><div class="r"><span class="nm">Load order rules and plugin messages</span><span class="sub">${S.g.mlNew ? plural(S.g.mlNew, 'new message') : 'Used by Sort on Load order'}</span><span class="end">${ml ? `<div style="width:140px">${pbar(ml.p)}</div>` : btn('Update masterlist', { k: 'sec', cls: 'sm', icon: 'refresh', act: 'mlUpdate', arg: '' })}</span></div></div></div>`;
  return pageWrap('system', hdr('versions', 'System', status) + top + `<div class="sec">${sech('Toolbox and its parts', `<span title="The parts ship inside the app and update with it.">${ic('info', 15)}</span>`)}<div class="rows">${rowsH}</div></div>` + eng + mlH);
};

/* ---------- Doctor ---------- */
const DOC = [['Game and storage', ['game', 'gamefiles', 'store']], ['Engine and Toolbox', ['engine', 'tb']], ['Mods and plugins', ['masters', 'sorted', 'count', 'ext']], ['This profile', ['far', 'stab', 'ttw']]];
function docItems(p) {
  const C = Object.fromEntries(checks(p).map(c => [c.id, c])), mm = missingMasters(p), ooo = outOfOrder(p), n = allActiveFiles(p).length;
  return {
    game: { ...C.game, n: 'Game folder' }, gamefiles: gameOk() ? { n: 'Game files', st: 'ok', t: 'Intact' } : { n: 'Game files', st: 'wait', t: 'Needs the game folder' }, store: { ...C.store, t: C.store.st === 'ok' ? `${S.g.storeFree} GB free` : C.store.t },
    engine: { ...C.engine, t: C.engine.st === 'ok' ? `${S.g.engine} · OK` : C.engine.t }, tb: updAvail() ? { n: "Raul's Toolbox", st: 'info', t: 'Update available', fix: ['Open System', 'nav', 'system'] } : { n: "Raul's Toolbox", st: 'ok', t: `${TB().nw} · newest` },
    masters: mm.length ? { n: 'Masters', st: 'fail', t: `${mm[0][0]} needs ${mm[0][1]}`, fix: ['Show load order', 'nav', 'load'] } : { n: 'Masters', st: 'ok', t: 'All present' },
    sorted: ooo ? { n: 'Load order', st: 'warn', t: `${ooo} out of order`, fix: ['Sort…', 'sortGo', ''] } : { n: 'Load order', st: 'ok', t: 'Sorted' },
    count: { n: 'Plugin count', st: n > 230 ? 'warn' : 'ok', t: `${n} of 255` }, ext: { n: 'Script extenders', st: 'ok', t: 'Built into the engine' },
    far: { ...C.far, n: 'LOD' }, stab: { ...C.stab, n: 'Mod setup' }, ttw: { ...C.ttw }
  };
}
PAGES.doctor = () => {
  const p = P(), d = U.doc, it = docItems(p), keys = DOC.flatMap(g => g[1]);
  const ran = S.g.docRun; const j = J.doctor;
  const vals = keys.map(k => it[k]), f = vals.filter(v => v.st === 'fail').length, w = vals.filter(v => v.st === 'warn').length;
  const status = j ? stt('run', `Checking ${Math.min(keys.length, Math.floor(j.p * keys.length) + 1)} of ${keys.length}`) : !ran ? '<span class="muted">Not run yet</span>' : f ? stt('fail', `${plural(f, 'problem')}`) + (w ? ' ' + stt('warn', plural(w, 'warning')) : '') : w ? stt('warn', plural(w, 'warning')) : stt('ok', 'All good');
  let idx = 0;
  const groups = DOC.map(([title, ks], gi) => `<div class="sec">${sech(title, `${ks.length}`, { fold: U.secs['doc' + gi] !== false, click: ['docFold', gi], plain: title })}<div class="rows" ${U.secs['doc' + gi] === false ? 'hidden' : ''}>${ks.map(k => {
    const i = idx++, v = it[k]; const done = ran && (!j || i < Math.floor(j.p * keys.length)); const cur = j && i === Math.floor(j.p * keys.length);
    const s = cur ? stt('run', 'Checking…') : !done ? stt('wait', 'Not checked') : stt(v.st, v.t);
    return `<div class="r"><span class="nm">${v.n}</span><span class="end">${s}${done && v.fix && v.st !== 'ok' ? btn(v.fix[0], { k: v.st === 'fail' ? 'primary' : 'sec', cls: 'sm', act: v.fix[1], arg: v.fix[2] }) : ''}</span></div>`;
  }).join('')}</div></div>`).join('');
  abar(j ? btn('Cancel', { k: 'ghost', act: 'cancelJob', arg: 'doctor' }) : btn(ran ? 'Run all checks again' : 'Run all checks', { k: 'primary', icon: 'refresh', act: 'docRun', arg: '' }));
  return pageWrap('doctor', hdr('doctor', 'Doctor', status + (ran && !j ? `<span class="muted" style="margin-left:8px">Ran ${ago(ran)} · ${esc(p.name)}</span>` : '')) + groups);
};

/* ---------- Settings ---------- */
PAGES.settings = () => {
  const p = P(), gp = S.g, o = U.secs;
  const row = (label, help, ctl) => `<div class="r${swRow(ctl)}" style="min-height:60px;flex-wrap:wrap"><div style="flex:1;min-width:200px"><div class="nm">${label}</div>${help ? `<div class="sub">${help}</div>` : ''}</div><span class="end">${ctl}</span></div>`;
  const pathRow = (label, help, path, ok, kind, okText) => row(label, help, `<span class="inp mono" style="display:inline-flex;align-items:center;width:340px;overflow:hidden;white-space:nowrap;text-overflow:ellipsis" title="${esc(path || '')}">${path ? esc(path) : '<span class="muted">Not set</span>'}</span>${btn('Browse…', { k: 'ghost', icon: 'folder', act: 'pickFolder', arg: kind })}${kind !== 'store' ? btn('Find', { k: 'ghost', icon: 'search', act: 'findFolder', arg: kind, busy: !!J['find:' + kind] }) : ''}<span style="min-width:110px">${J['find:' + kind] ? stt('run', 'Looking…') : ok === true ? stt('ok', okText || 'Found') : ok === 'warn' ? stt('warn', okText) : stt('fail', okText || 'Not found')}</span>`);
  const sec = (key, title, body, extra = '') => `<div class="sec">${sech(title, extra, { fold: o[key] !== false, click: ['secS', key], plain: title })}${o[key] !== false ? `<div class="rows">${body}</div>` : ''}</div>`;
  if (o.s_game === undefined) Object.assign(o, { s_game: true, s_store: false, s_prof: false, s_look: false, s_upd: false, adv: false });
  const ttwRow = row('Tale of Two Wastelands', '', `${gp.ttwInstalled ? stt('ok', 'Added') : stt('wait', 'Not added')}${btn(gp.ttwInstalled ? 'Show in Mods' : 'Add…', { k: 'ghost', act: gp.ttwInstalled ? 'nav' : 'ttwStart', arg: gp.ttwInstalled ? 'mods' : '' })}`);
  return pageWrap('settings', hdr('settings', 'Settings', 'Changes save as you go') +
    sec('s_game', 'Game installations', pathRow('Fallout: New Vegas', 'Required', gp.gamePath, gameOk(), 'fnv', gameOk() ? 'Found' : gp.gamePath && !gp.scen.game ? 'No game here' : 'Not found') + pathRow('Fallout 3', 'For TTW', gp.fo3Path, gp.fo3Path && !gp.scen.fo3 ? true : false, 'fo3') + ttwRow) +
    sec('s_store', 'Toolbox storage', pathRow('Storage folder', 'Mods, LOD and backups live here', gp.storePath, storeLow() ? 'warn' : true, 'store', storeLow() ? `${gp.storeFree} GB free, low on space` : `${gp.storeFree} GB free`) + row('Download cache', `${gp.cache.toFixed(1)} GB of archives you already installed`, btn('Clean up…', { k: 'sec', cls: 'sm', act: 'cleanCache', arg: '', dis: gp.cache < 0.1 }))) +
    sec('s_prof', `Game settings for ${esc(p.name)}`, row('Resolution', '', `<select class="inp" data-in="gfx" data-id="res" data-val="${p.gfx.res}" data-k="gfx:res" aria-label="Resolution">${['1280 × 720', '1920 × 1080', '2560 × 1440', '3840 × 2160'].map(v => `<option>${v}</option>`).join('')}</select>`) +
      row('Frame rate limit', '', `<select class="inp" data-in="gfx" data-id="cap" data-val="${p.gfx.cap}" data-k="gfx:cap" aria-label="Frame rate limit">${['60', '90', '120', '144', 'Off'].map(v => `<option>${v}</option>`).join('')}</select>`) +
      row('Upscaling', 'Renders smaller and sharpens. More frames.', seg([['Off', 'Off'], ['FSR', 'FSR']], p.gfx.up, 'gfxSeg', 'Upscaling').replace(/data-arg="(Off|FSR)"/g, 'data-arg="up:$1"')) +
      row('Look', 'Original keeps the game’s own light.', seg([['Original', 'Original'], ['Modern', 'Modern']], p.gfx.look, 'gfxSeg', 'Look').replace(/data-arg="(Original|Modern)"/g, 'data-arg="look:$1"')) +
      row('V-sync', '', sw(p.gfx.vsync, 'gfxVsync', '', 'V-sync')), pillP()) +
    sec('s_look', 'Appearance', row('Reduce motion', 'Spinners pulse, bars stay still.', sw(!gp.motion, 'gset', 'motion', 'Reduce motion')) + row('High contrast', 'Black background, outlines on every block.', sw(gp.hc, 'gset', 'hc', 'High contrast')) + row('Larger text', '', sw(gp.big, 'gset', 'big', 'Larger text'))) +
    sec('s_upd', 'Updates', row('Check for updates when System opens', 'At most once every 5 minutes.', sw(gp.autoCheck, 'gset', 'autoCheck', 'Check for updates automatically'))) +
    sec('adv', 'Advanced', row('Start setup again', 'Forgets every choice and profile change, then runs setup from the start.', btn('Start setup again…', { k: 'sec', cls: 'sm danger', icon: 'refresh', act: 'resetAll', arg: '' }))));
};

/* Versions tab: every change makes a version; revert or pick a later one */
function versionsTab(p) {
  const vs = p.versions.slice().sort((a, b) => b.t - a.t), ls = lastStable(p), curV = p.versions.find(v => v.id === p.cur);
  const chip = st => st === 'pass' ? tag('Stable', 'ok', 'shield') : st === 'fail' ? tag('Crashed', 'err', 'err') : tag('Not tested', '', 'off');
  const rows = vs.map(v => {
    const st = vStatus(p, v), cur = v.id === p.cur, b = p.bench.filter(r => r.vid === v.id).slice(-1)[0], o = U.vexp === v.id;
    const later = curV && v.t > curV.t;
    const sn = v.snap, on = sn.mods.filter(m => m.on).length;
    const runs = p.stab.filter(r => r.vid === v.id), bens = p.bench.filter(r => r.vid === v.id);
    return `<div class="tl ${st}"><span class="tld"></span><div class="mrow ${o ? 'open' : ''} ${cur ? 'curv' : ''}"><div class="r click" role="button" tabindex="0" aria-expanded="${o}" data-act="vexp" data-arg="${v.id}" data-k="vexp:${v.id}">
      <div style="min-width:0;flex:1"><div class="nm">${vLabel(v.t)}</div><div class="sub">${esc(v.summary)}</div></div>
      <span class="end">${cur ? tag('Current', 'acc') : ''}${b ? tag(`${b.avg} fps`) : ''}${chip(st)}${cur ? '' : btn(later ? 'Use this version…' : 'Revert…', { k: 'sec', cls: 'sm', icon: later ? 'redo' : 'undo', act: 'vRevert', arg: v.id })}</span></div>
      ${o ? `<div class="mdet" style="padding-left:16px;display:block"><dl class="kv" style="grid-template-columns:120px 1fr"><dt>Mods</dt><dd>${on} of ${sn.mods.length} on${sn.ttwOn ? ' · TTW' : ''}</dd><dt>Plugins</dt><dd>${sn.plugins.filter(x => x.on).length} active</dd><dt>Game settings</dt><dd>${esc(sn.gfx.res)} · ${esc(sn.gfx.cap)} fps · ${esc(sn.gfx.up)} · ${esc(sn.gfx.look)}</dd><dt>Stability</dt><dd>${runs.length ? runs.map(r => `${r.ok ? 'Passed' : 'Crashed'} ${fdate(r.t)}`).join(' · ') : st === 'pass' ? 'Same setup passed before' : 'No test'}</dd><dt>Benchmark</dt><dd>${bens.length ? bens.map(r => `${r.avg} fps ${fdate(r.t)}`).join(' · ') : 'No run'}</dd></dl></div>` : ''}</div></div>`;
  }).join('');
  return `<div class="sec">${sech('Versions', `<span class="muted">${p.versions.length} · newest first</span>`)}<div class="rows tlist">${rows}</div></div>`;
}
