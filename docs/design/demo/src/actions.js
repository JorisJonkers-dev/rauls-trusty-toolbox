/* SPDX-License-Identifier: GPL-3.0-only */
/* ===== actions, input, drag, keyboard, start-up ===== */
const A = {};
const IN = {};
const nav = page => { U.menu = null; if (U.page !== page) { U.page = page; U.scrollTop = true; } if (page === 'system') autoCheck(); render(); };
const openDlg = d => { U.ret = document.activeElement && document.activeElement.dataset ? document.activeElement.dataset.k : null; U.dlg = d; U.menu = null; U.dlgFresh = true; render(); };
const closeDlg = () => { U.dlg = null; U.focus = U.ret; render(); };
const confirmDlg = (title, text, ok, act, arg = '', danger = false) => openDlg({ t: 'confirm', title, text, ok, act, arg, danger });
const ensureTick = () => { if (!tickT) tickT = setInterval(tick, 120); };
const modName = id => CAT[id].name;

/* navigation and chrome */
A.nav = a => { if (a === 'ttw' && !U.ttwFlow) U.ttwFlow = { step: 0 }; nav(a); };
A.noop = () => {};
A.tab = a => { U.tab = a; render(); };
A.sec = a => { U.secs[a] = !U.secs[a]; render(); };
A.secS = a => { U.secs[a] = !U.secs[a]; render(); };
A.docFold = a => { U.secs['doc' + a] = U.secs['doc' + a] === false; render(); };
A.rowOpen = a => { U.open[a] = !U.open[a]; render(); };
A.vopen = a => { U.vopen = U.vopen === a ? null : a; render(); };
A.filt = a => { U.filt = a; render(); };
A.lfilt = a => { U.lfilt = a; render(); };
A.clearFilt = () => { U.q = ''; U.filt = 'all'; render(); };
A.logF = a => { U.logF = a; render(); };
A.logOpen = a => { U.logOpen[a] = !U.logOpen[a]; render(); };
A.logFilter = a => { U.secs.fulllog = true; U.logF = a; render(); };
A.zoom = a => { zoomAt(+a, 0.5, 0.5); };
A.picWorld = a => { U.picWorld = a; U.picQ = ''; const f = PLACES.find(x => (x[2] === 'Capital Wasteland') === (a === 'cw')); if (f) U.picPlace = f[0]; render(); };
A.picAll = () => { U.picAll = !U.picAll; render(); };
A.picMode = a => { U.picMode = a; render(); };
IN.picQ = v => { U.picQ = v; render(); };
/* zoom around a point (fractions of the picture), pan clamped so the picture always fills the frame; both pictures share it */
function clampPan() { const lim = 1 - U.zoom; U.pan.x = Math.min(0, Math.max(lim, U.pan.x)); U.pan.y = Math.min(0, Math.max(lim, U.pan.y)); }
function zoomAt(z, fx, fy) { z = Math.max(1, Math.min(8, z)); const k = z / U.zoom; U.pan.x = fx - (fx - U.pan.x) * k; U.pan.y = fy - (fy - U.pan.y) * k; U.zoom = z; clampPan(); render(); }
document.addEventListener('wheel', ev => { const st = ev.target.closest && ev.target.closest('.places.strip'); if (st && Math.abs(ev.deltaY) > Math.abs(ev.deltaX)) { ev.preventDefault(); st.scrollLeft += ev.deltaY; return; } const vp = ev.target.closest && ev.target.closest('.pic .vp'); if (!vp) return; ev.preventDefault(); const r = vp.getBoundingClientRect(); zoomAt(U.zoom * (ev.deltaY < 0 ? 1.25 : 0.8), (ev.clientX - r.left) / r.width, (ev.clientY - r.top) / r.height); }, { passive: false });
let panD = null;
document.addEventListener('pointerdown', ev => { const vp = ev.target.closest && ev.target.closest('.pic .vp'); if (!vp || ev.button !== 0) return; ev.preventDefault(); const r = vp.getBoundingClientRect(); panD = { x: ev.clientX, y: ev.clientY, w: r.width, h: r.height, px: U.pan.x, py: U.pan.y }; vp.classList.add('panning'); });
document.addEventListener('pointermove', ev => { if (!panD) return; U.pan.x = panD.px + (ev.clientX - panD.x) / panD.w; U.pan.y = panD.py + (ev.clientY - panD.y) / panD.h; clampPan(); render(); });
document.addEventListener('pointerup', () => { if (panD) { panD = null; document.querySelectorAll('.vp.panning').forEach(v => v.classList.remove('panning')); } });
document.addEventListener('dblclick', ev => { if (ev.target.closest && ev.target.closest('.pic .vp')) { U.zoom = 1; U.pan = { x: 0, y: 0 }; render(); } });
A.picPlace = a => { U.picPlace = a; render(); };
A.toastX = a => { U.toasts = U.toasts.filter(t => t.id !== a); render(); };
A.scrim = () => { if (U.pal) { U.pal = null; U.focus = U.ret; render(); return; } if (U.dlg && !(U.dlg.t === 'install' && U.dlg.step === 'busy')) closeDlg(); };
A.dlgClose = () => closeDlg();
A.shortcuts = () => openDlg({ t: 'shortcuts' });
A.palette = () => { U.ret = document.activeElement && document.activeElement.dataset ? document.activeElement.dataset.k : null; U.pal = { q: '', i: 0 }; U.menu = null; U.focus = 'palIn'; render(); };
A.palGo = a => { const it = palItems()[+a]; U.pal = null; if (!it) return render(); run(it[2], it[3]); };
A.openLink = a => { toast(a.startsWith('browse') ? 'Your file browser opened' : a === 'ttwsite' ? 'The TTW download page opened in your browser' : a.startsWith('orig') ? `${a.slice(5)}: the original mod’s page opened in your browser` : `${a.slice(5)}: our version’s source opened in your browser`); render(); };
A.findMod = a => { U.tab = 'list'; U.q = ''; U.filt = 'all'; U.open[a] = true; U.focus = 'mod:' + a; nav('mods'); };

/* profiles */
A.profMenu = a => { const anchor = 'profMenu:' + a; U.menu = U.menu && U.menu.anchor === anchor ? null : { id: 'prof', anchor, fresh: true, alignLeft: a === 'top' }; render(); };
A.profSwitch = a => { U.menu = null; if (a === S.active) return render(); S.active = a; U.sortPrev = null; U.open = {}; U.picA = U.picB = null; U.play = null; toast(`Switched to ${P().name}: mods, load order, settings, LOD and tests`); commit(); };
const nameDlg = (title, v, ok, kind, extra) => openDlg({ t: 'name', title, v, ok, kind, extra });
A.profNew = () => nameDlg('New profile', '', 'Create', 'profNew', '<p>Starts empty. Use Duplicate to start from this one.</p>');
A.profDup = () => nameDlg(`Duplicate “${P().name}”`, `${P().name} copy`, 'Duplicate', 'profDup', '<p>Copies mods, groups, load order, settings, LOD and known stable setups. Test history starts fresh.</p>');
A.profRename = () => nameDlg('Rename profile', P().name, 'Rename', 'profRename');
A.profDelete = () => confirmDlg(`Delete “${esc(P().name)}”?`, 'Its mod choices, load order, settings and test history go. Installed mod files stay for other profiles.', 'Delete profile', 'doDeleteProfile', '', true);
A.doDeleteProfile = () => { const n = P().name; S.profiles = S.profiles.filter(p => p.id !== S.active); S.active = S.profiles[0].id; U.dlg = null; toast(`Deleted ${n}. Now on ${P().name}`); commit(); };
A.newGroup = () => nameDlg('New group', '', 'Create group', 'grpNew');
A.grpRename = a => { U.menu = null; const g = P().groups.find(x => x.id === a); nameDlg('Rename group', g.name, 'Rename', 'grpRename:' + a); };
A.nameOk = () => {
  const d = U.dlg, v = d.v.trim(), p = P(); if (!v) return;
  if (d.kind.startsWith('prof') && S.profiles.some(x => x.name.toLowerCase() === v.toLowerCase() && (d.kind !== 'profRename' || x.id !== p.id))) { d.err = 'A profile with that name exists.'; return render(); }
  if (d.kind.startsWith('grp') && p.groups.some(x => x.name.toLowerCase() === v.toLowerCase() && !d.kind.endsWith(':' + x.id))) { d.err = 'A group with that name exists.'; return render(); }
  U.dlg = null;
  if (d.kind === 'profNew') { const np = emptyProfile(v); ensureVersions(np); S.profiles.push(np); S.active = np.id; toast(`Created ${v}`); U.open = {}; return commit(); }
  if (d.kind === 'profDup') { const np = clone(p); Object.assign(np, { id: 'p' + uid(), name: v, bench: [], stab: [], pics: [], events: [], dismissed: [], example: false, created: Date.now(), versions: null }); ensureVersions(np); np.versions[0].summary = `Copied from ${p.name}`; S.profiles.push(np); S.active = np.id; toast(`Duplicated as ${v}`); return commit(); }
  if (d.kind === 'profRename') { p.name = v; toast('Renamed'); return commit(); }
  if (d.kind === 'grpNew') { const id = uid(); change(`New group ${v}`, pp => { const ui = pp.groups.findIndex(g => g.id === 'ug'); pp.groups.splice(ui < 0 ? pp.groups.length : ui, 0, { id, name: v, open: true }); }); U.focus = 'grp:' + id; return; }
  if (d.kind.startsWith('grpRename')) { const id = d.kind.split(':')[1]; change(`Renamed group to ${v}`, pp => { pp.groups.find(g => g.id === id).name = v; }); }
};
IN.dlgName = v => { U.dlg.v = v; U.dlg.err = ''; render(); };

/* mods */
A.modToggle = a => { const m = P().mods.find(x => x.id === a); change(`${modName(a)} ${m.on ? 'off' : 'on'}`, p => { p.mods.find(x => x.id === a).on = !m.on; }, { event: `${modName(a)} turned ${m.on ? 'off' : 'on'}` }); };
A.grpToggle = a => { const p = P(), gm = p.mods.filter(m => m.g === a), on = !gm.every(m => m.on), g = p.groups.find(x => x.id === a); change(`${g.name}: all ${on ? 'on' : 'off'}`, pp => pp.mods.forEach(m => { if (m.g === a) m.on = on; }), { event: `${g.name} turned ${on ? 'on' : 'off'}` }); };
A.grpFold = a => { const g = P().groups.find(x => x.id === a); g.open = !g.open; commit(); };
A.grpMenu = a => { const anchor = 'grpMenu:' + a; U.menu = U.menu && U.menu.anchor === anchor ? null : { id: 'grp', gid: a, anchor, fresh: true }; render(); };
A.grpMove = a => { const [id, d] = a.split(':'); U.menu = null; moveGroup(id, P().groups.findIndex(g => g.id === id) + +d); U.focus = 'grp:' + id; };
function moveGroup(id, to) { const p = P(), from = p.groups.findIndex(g => g.id === id); if (to < 0 || to >= p.groups.length || to === from) return render(); const g = p.groups[from]; change(`Moved group ${g.name}`, pp => { const [x] = pp.groups.splice(from, 1); pp.groups.splice(to, 0, x); }, { event: 'Mod order changed' }); }
A.grpDelete = a => { U.menu = null; const g = P().groups.find(x => x.id === a); confirmDlg(`Delete the group “${esc(g.name)}”?`, 'Its mods move to Ungrouped. Nothing is uninstalled.', 'Delete group', 'doDeleteGroup', a, true); };
A.doDeleteGroup = a => { U.dlg = null; const g = P().groups.find(x => x.id === a); change(`Deleted group ${g.name}`, p => { p.mods.forEach(m => { if (m.g === a) m.g = 'ug'; }); p.groups = p.groups.filter(x => x.id !== a); }); };
A.modMove = a => { const [id, d] = a.split(':'); moveModBy(id, +d); };
function moveModBy(id, d) {
  const p = P(), list = modsInOrder(p), i = list.findIndex(m => m.id === id), j = i + d;
  if (j < 0 || j >= list.length) return;
  const target = list[j];
  change(`Moved ${modName(id)}`, pp => { const l = modsInOrder(pp), m = l.find(x => x.id === id), rest = l.filter(x => x.id !== id), t = rest.find(x => x.id === target.id), ti = rest.indexOf(t); m.g = t.g; rest.splice(d > 0 ? ti + 1 : ti, 0, m); pp.mods = rest; }, { event: 'Mod order changed', toast: false });
  U.focus = 'mod:' + id;
}
function dropMod(id, gid, k) {
  const p = P(); const cur = modsInOrder(p); const m0 = cur.find(x => x.id === id);
  const inG = cur.filter(x => x.g === gid && x.id !== id);
  if (m0.g === gid && inG.indexOf(cur.filter(x => x.g === gid)[k]) === -1 && cur.filter(x => x.g === gid).indexOf(m0) === k) return;
  const gname = p.groups.find(g => g.id === gid).name;
  change(m0.g === gid ? `Moved ${modName(id)}` : `Moved ${modName(id)} to ${gname}`, pp => {
    const l = modsInOrder(pp), m = l.find(x => x.id === id), rest = l.filter(x => x.id !== id), mem = rest.filter(x => x.g === gid);
    m.g = gid; let at;
    if (k < mem.length) at = rest.indexOf(mem[k]); else if (mem.length) at = rest.indexOf(mem[mem.length - 1]) + 1; else at = rest.length;
    rest.splice(at, 0, m); pp.mods = rest;
  }, { event: 'Mod order changed' });
}
IN.modGroup = (v, el) => { const id = el.dataset.id, p = P(); const g = p.groups.find(x => x.id === v); change(`Moved ${modName(id)} to ${g.name}`, pp => { const l = modsInOrder(pp), m = l.find(x => x.id === id), rest = l.filter(x => x.id !== id); m.g = v; rest.push(m); pp.mods = rest; }, { event: 'Mod order changed' }); U.focus = 'mg:' + id; };
A.modRemove = a => confirmDlg(`Remove ${esc(modName(a))} from all profiles?`, 'Mods are shared by every profile. Its plugins leave every load order. The archive stays in your downloads, and Undo brings it back.', 'Remove from all', 'doRemoveMod', a, true);
A.doRemoveMod = a => { U.dlg = null; const files = new Set([...CAT[a].plugins.map(x => x[0]), ...Object.keys(EXTRA_PLUGINS).filter(f => EXTRA_PLUGINS[f][3] === a)]); change(`Removed ${modName(a)}`, p => { p.mods = p.mods.filter(m => m.id !== a); p.plugins = p.plugins.filter(x => !files.has(x.f)); }, { event: `${modName(a)} removed` }); delete U.open[a]; };
A.undo = () => undo();
A.ttwToggle = () => { const on = !P().ttwOn; change(`TTW ${on ? 'on' : 'off'}`, p => { p.ttwOn = on; }, { event: `TTW turned ${on ? 'on' : 'off'}` }); };
A.ttwCheck = () => job('ttwcheck', 1500, () => { toast(S.g.ttwUpdate ? 'TTW 3.4.1 is ready' : `TTW ${S.g.ttwVer} is the newest`); }, { label: 'Checking TTW' });
A.recDismiss = () => { const p = P(); p.dismissed.push(sig(p)); toast('Hidden until the mod setup changes'); commit(); };
A.stabShow = () => { U.tab = 'test'; nav('mods'); };

/* install a mod */
A.install = () => { const p = P(); openDlg({ t: 'install', step: 'pick', sel: null, ch: {}, page: 0, g: (p.groups.find(g => g.id !== 'ug') || p.groups[0]).id }); };
A.archSel = a => { U.dlg.sel = +a; render(); };
A.instNext = () => {
  const d = U.dlg, a = ARCH[d.sel];
  if (d.step === 'pick') {
    if (a.broken) { d.step = 'error'; return render(); }
    if (a.fomod) { d.step = 'opts'; d.page = 0; const have = new Set(P().mods.map(m => m.id)); d.ch = {}; a.fomod.forEach(g => { d.ch[g.id] = g.kind === 'one' ? g.opts[0][0] : g.opts.filter(o => have.has(o[0])).map(o => o[0]); }); }
    else d.step = 'review';
  } else if (d.step === 'opts') { if (d.page < a.fomod.length - 1) d.page++; else d.step = 'review'; }
  U.dlgFresh = true; render();
};
A.instBack = arg => { const d = U.dlg, a = ARCH[d.sel]; if (arg === 'pick' || d.step === 'error') d.step = 'pick'; else if (d.step === 'review') { if (a.fomod) { d.step = 'opts'; d.page = a.fomod.length - 1; } else d.step = 'pick'; } else if (d.step === 'opts') { if (d.page > 0) d.page--; else d.step = 'pick'; } U.dlgFresh = true; render(); };
A.optPick = arg => { const [g, o, kind] = arg.split(':'), d = U.dlg; if (kind === 'one') d.ch[g] = o; else { const s = new Set(d.ch[g] || []); s.has(o) ? s.delete(o) : s.add(o); d.ch[g] = [...s]; } U.focus = 'opt:' + o; render(); };
IN.instGroup = v => { U.dlg.g = v; };
A.instGo = () => {
  const d = U.dlg, a = ARCH[d.sel]; d.step = 'busy'; U.dlgFresh = true;
  job('install', 2600, () => {
    const plugs = instPlugins(d), labels = optLabels(d);
    U.dlg = null;
    change(`Installed ${modName(a.mod)}`, p => { p.mods.push({ id: a.mod, on: true, g: d.g, at: Date.now(), opts: labels }); const tg = p.groups.find(x => x.id === d.g); if (tg) tg.open = true; plugs.forEach(f => { if (!p.plugins.some(x => x.f === f)) p.plugins.push({ f, on: true }); }); }, { event: `${modName(a.mod)} added`, toast: `Installed ${modName(a.mod)}` });
    U.open[a.mod] = true; U.tab = 'list'; U.q = ''; U.filt = 'all'; U.focus = 'mod:' + a.mod; nav('mods');
  }, { label: 'Installing' });
};
A.instCancel = () => { delete J.install; U.dlg.step = 'review'; render(); };

/* load order */
A.plugToggle = a => { const x = P().plugins.find(y => y.f === a); change(`${a} ${x.on ? 'inactive' : 'active'}`, p => { const y = p.plugins.find(z => z.f === a); y.on = !y.on; }, { event: 'Load order changed' }); };
A.sortGo = () => { U.dlg = null; const p = P(); U.sortPrev = { pid: p.id, ...sortMoves(p) }; U.lfilt = 'all'; nav('load'); };
A.sortCancel = () => { U.sortPrev = null; render(); };
A.sortApply = () => { const pr = U.sortPrev; U.sortPrev = null; change(`Sorted: ${plural(pr.moves.length, 'move')}`, p => applyVisOrder(p, pr.sorted), { event: 'Load order sorted' }); };
function applyVisOrder(p, files) { const vis = new Set(visPlugins(p).map(x => x.f)); const queue = files.map(f => p.plugins.find(x => x.f === f)); p.plugins = p.plugins.map(x => vis.has(x.f) ? queue.shift() : x); }
function movePlugin(f, to) { const p = P(), vis = visPlugins(p).map(x => x.f), from = vis.indexOf(f); if (to < 0 || to >= vis.length || to === from) return render(); vis.splice(from, 1); vis.splice(to, 0, f); change(`Moved ${f}`, pp => applyVisOrder(pp, vis), { event: 'Load order changed', toast: false }); U.focus = 'plug:' + f; }

/* LOD */
A.farQ = a => { const p = P(); p.far.q = a; p.far.fine = null; commit(); };
A.fine = a => { const p = P(), [k, v] = a.split(':'); const base = p.far.fine || { tex: { performance: 256, balanced: 512, high: 1024 }[p.far.q], dist: { performance: 2, balanced: 3, high: 5 }[p.far.q], trees: { performance: 'sparse', balanced: 'normal', high: 'dense' }[p.far.q], plants: p.far.q === 'high' }; base[k] = k === 'tex' ? +v : k === 'plants' ? v === 'true' : v; p.far.fine = base; U.focus = `fine:${k === 'plants' ? 'plants:' + !base.plants : a}`; commit(); };
IN.fineDist = v => { const p = P(); A.fine('dist:' + v); p.far.fine.dist = +v; commit(); };
A.fineReset = () => { P().far.fine = null; commit(); };
A.farBuild = a => {
  const p = P(); if (a === 'go') nav('far'); if (J['far:' + p.id]) return;
  const pid = p.id;
  job('far:' + pid, 9000, () => { const pp = prof(pid); if (!pp) return; pp.far.built = { sig: farSig(pp), q: pp.far.q, fine: clone(pp.far.fine), t: Date.now(), size: +farSize(pp).toFixed(1), n: pp.mods.filter(m => m.on).length }; toast(`LOD for ${pp.name} is ready`); commit(); }, { label: 'Building LOD' });
};
A.farCancel = () => confirmDlg('Stop the LOD build?', 'The LOD you had stays in place.', 'Stop build', 'doFarCancel', '', true);
A.doFarCancel = () => { U.dlg = null; delete J['far:' + P().id]; toast('Build stopped'); render(); };
A.cancelJob = a => { delete J[a]; toast('Stopped'); render(); };

/* tests */
A.stabGo = a => {
  const p = P(); U.tab = 'test'; if (U.page !== 'mods') nav('mods'); if (J['stab:' + p.id]) return render();
  const pid = p.id, sg = sig(p), fail = S.g.scen.stab, vid = p.cur;
  job('stab:' + pid, 9000, () => { const pp = prof(pid); pp.stab.push({ vid, t: Date.now(), ok: true, cells: 40, crashes: 0, hitches: 1 + Math.floor(Math.random() * 3), mins: 20 }); if (!pp.stable.some(s => s.sig === sg)) pp.stable.push({ sig: sg, t: Date.now() }); toast('Stability test passed. This setup is now Tested stable'); commit(); },
    { label: 'Stability test', failAt: fail ? 0.55 : null, fail: () => { const pp = prof(pid); pp.stab.push({ vid, t: Date.now(), ok: false, cells: 22, crashes: 1, hitches: 3, mins: 11, where: 'Freeside' }); toast('Stability test crashed in Freeside. See the log'); U.secs.fulllog = true; U.logF = 'err'; commit(); } });
};
A.benchGo = a => {
  const p = P(); if (a === 'pal') { U.tab = 'test'; nav('mods'); } if (J['bench:' + p.id]) return; const pid = p.id, vid = p.cur;
  job('bench:' + pid, 6000, () => { const pp = prof(pid); const last = pp.bench[pp.bench.length - 1]; const base = last ? last.avg : 70 - pp.mods.filter(m => m.on).length; const avg = Math.max(30, Math.round(base + (Math.random() * 6 - 3))); pp.bench.push({ vid, t: Date.now(), avg, low: Math.round(avg * (0.6 + Math.random() * 0.08)), ft: +(1000 / avg).toFixed(1), p99: +(1000 / avg * 1.7).toFixed(1) }); toast(`Benchmark done: ${avg} fps average`); commit(); }, { label: 'Benchmark' });
};
A.picsGo = () => { const p = P(), pid = p.id; if (J['pics:' + pid]) return; job('pics:' + pid, 5000, () => { const pp = prof(pid); const prevLast = pp.pics[pp.pics.length - 1]; const r = { id: uid(), t: Date.now(), q: pp.far.q, trees: pp.mods.some(m => m.id === 'wfo' && m.on), look: pp.mods.some(m => m.id === 'nsk' && m.on) ? 1 : 0 }; pp.pics.push(r); U.picA = prevLast ? prevLast.id : null; U.picB = r.id; toast('New pictures taken'); commit(); }, { label: 'Taking pictures' }); };
IN.picA = v => { U.picA = v; render(); };
IN.picB = v => { U.picB = v; render(); };
IN.logQ = v => { U.logQ = v; render(); };
A.forgetStable = a => { const p = P(); p.stable.splice(+a, 1); toast('Forgotten'); commit(); };

/* play */
let playT = null;
A.play = () => {
  if (U.run || (U.play && U.play.state === 'checking')) return;
  const list = checks(P()); U.play = { state: 'checking', i: 0, list }; nav('home');
  clearInterval(playT);
  playT = setInterval(() => {
    const pl = U.play; if (!pl || pl.state !== 'checking') { clearInterval(playT); return; }
    const c = pl.list[pl.i];
    if (c.st === 'fail') { pl.state = 'failed'; pl.stopAt = pl.i; clearInterval(playT); return render(); }
    pl.i++;
    if (pl.i >= pl.list.length) { clearInterval(playT); if (pl.list.some(x => x.st === 'warn')) pl.state = 'warned'; else startGame(); }
    render();
  }, 380);
};
A.playCancel = () => { clearInterval(playT); U.play = null; render(); };
A.playAnyway = () => startGame();
function startGame() { U.play = { state: 'starting' }; render(); setTimeout(() => { if (!U.play || U.play.state !== 'starting') return; U.play = null; U.run = { pid: S.active, start: Date.now() }; ensureTick(); toast('New Vegas is running'); render(); }, 1500); }
A.stopGame = () => { const r = U.run; if (!r) return; const mins = Math.max(1, Math.floor((Date.now() - r.start) / 1000)); S.g.sessions.push({ t: Date.now(), mins, pid: r.pid }); U.run = null; toast(`Played ${hm(mins)}. Added to your playtime`); commit(); };

/* TTW flow */
A.ttwStart = a => { U.dlg = null; U.ttwFlow = { step: a === 'update' ? 2 : 0 }; if (!S.g.fo3Path && !J['find:fo3']) A.findFolder('fo3'); nav('ttw'); };
A.ttwNext = () => { U.ttwFlow.step++; render(); };
A.ttwBack = () => { U.ttwFlow.step = Math.max(0, U.ttwFlow.step - 1); render(); };
A.ttwClose = () => { U.ttwFlow = null; nav('mods'); };
A.ttwInstall = () => job('ttw', 8000, () => { S.g.ttwInstalled = true; S.g.ttwUpdate = false; P().ttwOn = true; if (U.ttwFlow) U.ttwFlow.step = 3; toast('Tale of Two Wastelands added'); commit(); }, { label: 'Installing TTW' });

/* folders, storage */
A.pickFolder = a => openDlg({ t: 'folder', kind: a, sel: null });
A.folderSel = a => { U.dlg.sel = +a; U.focus = 'folder:' + a; render(); };
A.folderOk = () => {
  const d = U.dlg, k = d.kind, f = FOLDERS[k.replace(/^w/, '')][d.sel]; U.dlg = null; U.focus = U.ret;
  if (k === 'fnv') { S.g.gamePath = f[1]; S.g.gameOk = f[2]; if (f[2]) { S.g.scen.game = false; toast('Game folder saved'); } else toast('No FalloutNV.exe in that folder'); }
  if (k === 'fo3') { if (f[2]) { S.g.fo3Path = f[1]; S.g.scen.fo3 = false; toast('Fallout 3 folder saved'); } else toast('No Fallout 3 in that folder'); }
  if (k === 'store') { S.g.storePath = f[1]; S.g.storeFree = f[3]; S.g.scen.disk = f[3] < 20; toast(`Storage moved. ${f[3]} GB free`); }
  if (k === 'wfnv') { S.wiz.game = { path: f[1], ok: f[2] }; if (f[2]) S.g.scen.game = false; }
  if (k === 'wfo3') { S.wiz.fo3 = { path: f[1], ok: f[2] }; if (f[2]) S.g.scen.fo3 = false; }
  if (k === 'wstore') S.wiz.store = { path: f[1], free: f[3] };
  commit();
};
A.findFolder = a => job('find:' + a, 1500, () => {
  if (a === 'fnv') { if (S.g.scen.game) toast('Not found in your Steam or GOG libraries. Try Browse'); else { S.g.gamePath = FOLDERS.fnv[0][1]; S.g.gameOk = true; toast('Found Fallout: New Vegas'); } }
  if (a === 'fo3') { if (S.g.scen.fo3) toast('Fallout 3 not found. Try Browse'); else { S.g.fo3Path = FOLDERS.fo3[0][1]; } }
  if (a === 'wfnv') S.wiz.game = S.g.scen.game ? { path: null, ok: false } : { path: FOLDERS.fnv[0][1], ok: true };
  if (a === 'wfo3') S.wiz.fo3 = S.g.scen.fo3 ? { path: null, ok: false } : { path: FOLDERS.fo3[0][1], ok: true };
  commit();
}, { label: 'Looking for the folder' });
A.cleanCache = () => { if (S.g.cache < 0.1) { toast('The download cache is already empty'); return render(); } confirmDlg('Clean up the download cache?', `Frees ${S.g.cache.toFixed(1)} GB. These are archives of mods you already installed; you can download them again.`, 'Clean up', 'doClean'); };
A.doClean = () => { U.dlg = null; const c = S.g.cache; S.g.cache = 0; S.g.storeFree = Math.round(S.g.storeFree + c); toast(`Freed ${c.toFixed(1)} GB`); commit(); };
A.repairEngine = () => job('repair', 3000, () => { S.g.scen.engine = false; toast('Engine repaired'); commit(); }, { label: 'Repairing the engine' });

/* system */
function autoCheck() { if (S.g.autoCheck && Date.now() - S.g.lastCheck > 5 * 60000 && !J.check) A.checkUpdates(); }
A.checkUpdates = a => { if (a === 'go') { U.page = 'system'; U.scrollTop = true; } job('check', 1800, () => { S.g.lastCheck = Date.now(); commit(); }, { label: 'Checking for updates' }); };
A.update = () => { U.updErr = false; job('update', 7000, () => { S.g.tbVer = '0.6.0'; U.justUpdated = true; U.vopen = 'toolbox'; toast("Updated to Raul's Toolbox 0.6.0"); commit(); }, { label: 'Updating', failAt: S.g.scen.dl ? 0.35 : null, fail: () => { U.updErr = true; toast('The download stopped'); render(); } }); };
A.useBuild = a => confirmDlg(`Use engine ${a}?`, `The game runs on ${a} from the next start. You can switch back here.`, `Use ${a}`, 'doUseBuild', a);
A.doUseBuild = a => { U.dlg = null; S.g.engine = a; toast(`Engine ${a} in use`); commit(); };
A.removeBuild = a => confirmDlg(`Remove engine ${a}?`, 'You can’t switch back to it after this.', 'Remove', 'doRemoveBuild', a, true);
A.doRemoveBuild = a => { U.dlg = null; S.g.builds = S.g.builds.filter(b => b !== a); toast(`Engine ${a} removed`); commit(); };
A.mlUpdate = () => job('ml', 2500, () => { S.g.mlAt = Date.now(); S.g.mlNew = 4; toast('Masterlist updated: 4 new messages'); commit(); }, { label: 'Updating the masterlist' });
A.docRun = () => { S.g.docRun = Date.now(); job('doctor', 11 * 320, () => { commit(); }, { label: 'Doctor' }); };

/* settings */
IN.gfx = (v, el) => { P().gfx[el.dataset.id] = v; recordVersion(P()); toast(`Saved for ${P().name}`); commit(); };
A.gfxSeg = a => { const [k, v] = a.split(':'); P().gfx[k] = v; recordVersion(P()); commit(); };
A.gfxVsync = () => { P().gfx.vsync = !P().gfx.vsync; recordVersion(P()); commit(); };
A.gset = a => { if (a === 'motion') S.g.motion = !S.g.motion; else S.g[a] = !S.g[a]; commit(); };
A.resetAll = () => confirmDlg('Start setup again?', 'Raul’s Toolbox forgets every choice, profile change and test result, then runs setup from the start. Your game folder is not touched.', 'Start setup again', 'doReset', '', true);
A.doReset = () => { store.clear(); for (const k of Object.keys(J)) delete J[k]; S = fresh(); Object.assign(U, { page: 'home', dlg: null, menu: null, play: null, run: null, undo: {}, sortPrev: null, ttwFlow: null, open: {}, justUpdated: false, updErr: false }); commit(); };

/* wizard */
const W = () => S.wiz;
A.wizNext = () => { const w = W(); w.step = Math.min(WSTEPS.length - 1, w.step + 1); wizEnter(); U.focus = null; commit(); };
A.wizBack = () => { W().step = Math.max(0, W().step - 1); commit(); };
function wizEnter() {
  const w = W();
  if (w.step === 1 && !w.game && !J['find:wfnv']) A.findFolder('wfnv');
  if (w.step === 2 && !w.store) { const f = FOLDERS.store[1]; w.store = { path: f[1], free: S.g.scen.disk ? 11 : f[3] }; }
  if (w.step === 4 && !w.fo3 && !J['find:wfo3']) A.findFolder('wfo3');
  if (w.step === 6 && w.prof !== 'empty' && !w.qSet) { w.q = SEED.find(x => x.id === w.prof).q; }
}
A.wizEngine = () => { U.wEngErr = false; job('wengine', 4200, () => { W().engine = true; commit(); }, { label: 'Installing the engine', failAt: S.g.scen.dl ? 0.4 : null, fail: () => { U.wEngErr = true; render(); } }); };
A.wizTtw = a => { const w = W(); if (a === 'skip') w.ttw = 'skipped'; if (a === 'undo') w.ttw = null; if (a === 'add') { job('wttw', 6000, () => { W().ttw = 'added'; commit(); }, { label: 'Installing TTW' }); return; } commit(); };
A.wizProf = a => { W().prof = a; W().qSet = false; U.focus = 'wizProf:' + a; commit(); };
A.wizQ = a => { W().q = a; W().qSet = true; commit(); };
A.wizFar = a => { const w = W(); if (a === 'later') w.far = 'later'; if (a === 'undo') w.far = null; if (a === 'build') { job('wfar', 6000, () => { W().far = 'built'; commit(); }, { label: 'Building LOD' }); return; } commit(); };
A.wizFinish = () => {
  const w = W(), g = S.g;
  g.gamePath = w.game.path; g.gameOk = true; g.storePath = w.store.path; g.storeFree = w.store.free; g.ttwInstalled = w.ttw === 'added'; g.fo3Path = w.fo3 && w.fo3.ok ? w.fo3.path : null;
  if (w.prof === 'empty') { const np = emptyProfile('My profile'); ensureVersions(np); S.profiles.push(np); S.active = np.id; } else S.active = w.prof;
  const p = P(); p.far.q = w.q;
  if (w.far === 'built') p.far.built = { sig: farSig(p), q: p.far.q, fine: null, t: Date.now(), size: +farSize(p).toFixed(1), n: p.mods.filter(m => m.on).length };
  w.done = true; U.page = 'home'; U.scrollTop = true;
  toast(`Welcome in. ${p.name} is your profile`);
  commit();
};

/* test situations panel */
A.scenClose = () => { U.scen = false; render(); };
A.scen = a => { const s = S.g.scen; s[a] = !s[a]; if (a === 'disk') { if (s.disk) { S.g.storeFreeWas = S.g.storeFree; S.g.storeFree = 11; } else S.g.storeFree = S.g.storeFreeWas || 142; } if (a === 'game' && !s.game) S.g.gameOk = !!S.g.gamePath || S.g.gameOk; U.focus = 'scen:' + a; commit(); };
A.scenHistory = () => { const now = Date.now(); const ids = S.profiles.map(p => p.id); [[9, 134], [6, 88], [4, 41], [2, 152], [1, 74]].forEach(([d, m], i) => S.g.sessions.push({ t: now - d * DAY, mins: m, pid: ids[i % ids.length] })); S.g.sessions.sort((a, b) => a.t - b.t); toast('Play history added'); commit(); };
A.scenTtwUpd = () => { S.g.ttwUpdate = true; toast('TTW reports an update'); commit(); };

/* dispatch */
function run(act, arg, el, ev) { if (A[act]) A[act](arg, el, ev); }
document.addEventListener('click', ev => {
  const el = ev.target.closest('[data-act]');
  if (U.menu && !ev.target.closest('.menu') && !(el && el.dataset.k === U.menu.anchor)) { U.menu = null; if (!el) return render(); }
  if (!el || el.disabled) return;
  /* whole-row actions: nested form fields and drag handles keep their own behaviour */
  if (ev.target.closest('[data-grip]')) return;
  const fld = ev.target.closest('input,select,textarea,label'); if (fld && el.contains(fld) && fld !== el) return;
  if (el.dataset.act === 'scrim' && ev.target !== el) return;
  if (el.closest('.menu')) U.menu = null;
  run(el.dataset.act, el.dataset.arg, el, ev);
});
document.addEventListener('input', ev => { const el = ev.target; if (el.dataset && el.dataset.in && el.tagName !== 'SELECT') { if (IN[el.dataset.in]) IN[el.dataset.in](el.value, el); else { U[el.dataset.in] = el.value; render(); } } });
document.addEventListener('change', ev => { const el = ev.target; if (el.tagName === 'SELECT' && el.dataset.in && IN[el.dataset.in]) IN[el.dataset.in](el.value, el); });
IN.palQ = v => { if (!U.pal) return; U.pal.q = v; U.pal.i = 0; render(); };
IN.q = v => { U.q = v; render(); };
IN.lq = v => { U.lq = v; render(); };

function afterDraw() {
  if (U.picSeen !== U.picPlace + U.picWorld) { const st = document.querySelector('.places.strip'), b = st && st.querySelector('[aria-pressed="true"]'); if (b) { U.picSeen = U.picPlace + U.picWorld; const l = b.offsetLeft - st.offsetLeft; if (l < st.scrollLeft || l + b.offsetWidth > st.scrollLeft + st.clientWidth) st.scrollLeft = l - 24; } }
  if (U.dlgFresh) { U.dlgFresh = false; const d = document.querySelector('.dlg'); if (d) { const f = ['input', '[aria-selected="true"]', '.picker [tabindex="0"]', '.opt', '[data-k="dlgok"]', '.cca .btn.primary:not(:disabled)', '.cca .btn.dangerp', '.btn'].map(q => d.querySelector(q)).find(Boolean); if (f) f.focus(); } }
}

/* keyboard */
document.addEventListener('keydown', ev => {
  const k = ev.key, t = ev.target, typing = t.tagName === 'INPUT' || t.tagName === 'SELECT';
  if (ev.ctrlKey && ev.shiftKey && (k === 'D' || k === 'd')) { ev.preventDefault(); U.scen = !U.scen; return render(); }
  if (ev.ctrlKey && (k === 'k' || k === 'K')) { ev.preventDefault(); if (S.wiz.done) U.pal ? (U.pal = null, render()) : A.palette(); return; }
  if (k === 'Escape') {
    if (U.menu) { const a = U.menu.anchor; U.menu = null; U.focus = a; return render(); }
    if (U.pal) { U.pal = null; U.focus = U.ret; return render(); }
    if (U.dlg) { if (!(U.dlg.t === 'install' && U.dlg.step === 'busy')) closeDlg(); return; }
    if (U.scen) { U.scen = false; return render(); }
    if (U.drag) { U.drag = null; return render(); }
  }
  if (U.pal) {
    const n = palItems().length;
    if (k === 'ArrowDown') { ev.preventDefault(); U.pal.i = (U.pal.i + 1) % Math.max(1, n); return render(); }
    if (k === 'ArrowUp') { ev.preventDefault(); U.pal.i = (U.pal.i - 1 + n) % Math.max(1, n); return render(); }
    if (k === 'Enter') { ev.preventDefault(); return A.palGo(U.pal.i); }
  }
  if (U.menu && (k === 'ArrowDown' || k === 'ArrowUp' || k === 'Home' || k === 'End')) {
    ev.preventDefault(); const items = [...document.querySelectorAll('.menu .mi:not(:disabled)')]; const i = items.indexOf(document.activeElement);
    const j = k === 'Home' ? 0 : k === 'End' ? items.length - 1 : (i + (k === 'ArrowDown' ? 1 : -1) + items.length) % items.length; items[j] && items[j].focus(); return;
  }
  if (U.menu && k === 'Tab') { U.menu = null; render(); }
  if (k === 'Tab' && (U.dlg || U.pal)) {
    const box = document.querySelector('.dlg'); if (!box) return; const f = [...box.querySelectorAll('button:not(:disabled),input,select,[tabindex="0"]')]; if (!f.length) return;
    const i = f.indexOf(document.activeElement);
    if (ev.shiftKey && (i <= 0)) { ev.preventDefault(); f[f.length - 1].focus(); } else if (!ev.shiftKey && (i === f.length - 1 || i < 0)) { ev.preventDefault(); f[0].focus(); }
    return;
  }
  if ((k === 'ArrowDown' || k === 'ArrowUp') && t.closest && t.closest('.picker') && t.getAttribute('role') === 'option') { ev.preventDefault(); const opts = [...t.closest('.picker').querySelectorAll('[role="option"][tabindex="0"]')]; const i = opts.indexOf(t); const n = opts[i + (k === 'ArrowDown' ? 1 : -1)]; if (n) { n.click(); n.focus(); } return; }
  /* Enter is the dialog's primary action (Space still picks an option; a focused plain button keeps its own Enter) */
  if (k === 'Enter' && U.dlg && !ev.shiftKey && t.tagName !== 'TEXTAREA') {
    const box = document.querySelector('.dlg');
    if (box && box.contains(t)) {
      const plainBtn = t.tagName === 'BUTTON' && !t.getAttribute('role');
      if (!plainBtn) {
        ev.preventDefault();
        if (U.dlg.t === 'name') { A.nameOk(); return; }
        if ((t.getAttribute('role') === 'option') && t.dataset.act) t.click();
        const go = () => { const p = [...document.querySelectorAll('.dlg .cca .btn.primary:not(:disabled), .dlg .cca .btn.dangerp:not(:disabled)')].pop(); if (p) p.click(); };
        if (t.getAttribute('role') === 'option') requestAnimationFrame(() => requestAnimationFrame(go)); else go();
        return;
      }
    }
  }
  if (ev.altKey && (k === 'ArrowUp' || k === 'ArrowDown') && t.dataset && t.dataset.move) {
    ev.preventDefault(); const d = k === 'ArrowUp' ? -1 : 1;
    if (t.dataset.move === 'mod') moveModBy(t.dataset.k.slice(4), d);
    if (t.dataset.move === 'group') { const id = t.dataset.k.slice(4); moveGroup(id, P().groups.findIndex(g => g.id === id) + d); U.focus = 'grp:' + id; }
    if (t.dataset.move === 'plugin' && !U.sortPrev) { const f = t.dataset.id; movePlugin(f, visPlugins(P()).map(x => x.f).indexOf(f) + d); }
    return;
  }
  if ((k === 'Enter' || k === ' ') && t.getAttribute && t.getAttribute('role') !== null && t.tagName !== 'BUTTON' && t.dataset && t.dataset.act) { ev.preventDefault(); t.dispatchEvent(new MouseEvent('click', { bubbles: true })); return; }
  if (k === '?' && !typing && !U.dlg && S.wiz.done) { ev.preventDefault(); A.shortcuts(); }
});

/* drag: pointer events, a lifted ghost and a drop line; Alt+Up/Down does the same from the keyboard */
document.addEventListener('pointerdown', ev => {
  const g = ev.target.closest('[data-grip]'); if (!g || ev.button !== 0 || U.sortPrev) return;
  ev.preventDefault();
  U.drag = { kind: g.dataset.grip, id: g.dataset.id, sx: ev.clientX, sy: ev.clientY, x: ev.clientX, y: ev.clientY, moved: false };
});
document.addEventListener('pointermove', ev => {
  const D = U.drag; if (!D) return;
  D.x = ev.clientX; D.y = ev.clientY;
  if (!D.moved && Math.hypot(D.x - D.sx, D.y - D.sy) < 5) return;
  D.moved = true;
  if (D.kind === 'mod') {
    const els = document.elementsFromPoint(D.x, D.y); const card = els.map(e => e.closest && e.closest('[data-groupcard]')).find(Boolean);
    if (card) { D.toG = card.dataset.groupcard; const rows = [...card.querySelectorAll('.mrow[data-mod]')].filter(r => r.dataset.mod !== D.id); D.toI = rows.filter(r => { const b = r.getBoundingClientRect(); return b.top + b.height / 2 < D.y; }).length; }
  } else if (D.kind === 'group') {
    const cards = [...document.querySelectorAll('[data-groupcard]')].filter(c => c.dataset.groupcard !== D.id);
    D.to = cards.filter(c => { const b = c.getBoundingClientRect(); return b.top + Math.min(b.height / 2, 40) < D.y; }).length;
  } else if (D.kind === 'plugin') {
    const rows = [...document.querySelectorAll('.mrow[data-plugin]')].filter(r => r.dataset.plugin !== D.id);
    D.to = rows.filter(r => { const b = r.getBoundingClientRect(); return b.top + b.height / 2 < D.y; }).length;
  }
  const sc = document.querySelector('.mscroll'); if (sc) { const b = sc.getBoundingClientRect(); if (D.y < b.top + 24) sc.scrollTop -= 10; if (D.y > b.bottom - 24) sc.scrollTop += 10; }
  render();
});
document.addEventListener('pointerup', () => {
  const D = U.drag; if (!D) return; U.drag = null;
  if (!D.moved) return render();
  if (D.kind === 'mod' && D.toG) dropMod(D.id, D.toG, D.toI);
  else if (D.kind === 'group' && D.to != null) moveGroup(D.id, D.to);
  else if (D.kind === 'plugin' && D.to != null) movePlugin(D.id, D.to);
  else render();
});

/* versions: revert or pick a later one; the list stays */
A.vexp = a => { U.vexp = U.vexp === a ? null : a; render(); };
A.vRevert = a => {
  U.menu = null; const p = P(), v = p.versions.find(x => x.id === a), cur = p.versions.find(x => x.id === p.cur);
  const later = cur && v.t > cur.t;
  confirmDlg(later ? `Go to the version from ${vLabel(v.t)}?` : `Revert to the version from ${vLabel(v.t)}?`, `Mods, order and game settings become “${esc(v.summary)}” again. Other versions stay in the list.`, later ? 'Use this version' : 'Revert', 'doRevert', a);
};
A.vRevertStable = () => { const v = lastStable(P()); if (v) A.vRevert(v.id); };
A.doRevert = a => {
  U.dlg = null; const p = P(), v = p.versions.find(x => x.id === a);
  applySnap(p, v.snap); p.cur = v.id; U.undo[p.id] = []; U.sortPrev = null;
  toast(`Now on the version from ${vLabel(v.t)}`); commit();
};
A.ptMenu = a => { const anchor = 'pt:' + a; U.menu = U.menu && U.menu.anchor === anchor ? null : { id: 'pt', run: +a, anchor, fresh: true }; render(); };

/* start */
A.quitAsk = () => confirmDlg('Quit Raul’s Toolbox?', 'Running jobs stop. Your setup is saved.', 'Quit', 'dlgClose', '');
document.getElementById('tbi').innerHTML = ic('app', 16);
fitWin();
S = store.load();
if (!S || S.v !== 1) S = fresh();
S.profiles.forEach(ensureVersions);
if (S.wiz.done && S.g.autoCheck && U.page === 'system') autoCheck();
if (!S.wiz.done) wizEnter();
render();
