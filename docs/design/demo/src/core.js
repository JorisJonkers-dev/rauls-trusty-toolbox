/* SPDX-License-Identifier: GPL-3.0-only */
/* ===== core: helpers, stored state, model rules, jobs, rendering loop ===== */
const ic = (n, s = 18, cls = '') => `<svg class="i ${cls}" width="${s}" height="${s}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${I[n] || ''}</svg>`;
I.ext = '<path d="M14 4h6v6M20 4l-9 9M18 14v4.5c0 .8-.7 1.5-1.5 1.5h-11C4.7 20 4 19.3 4 18.5v-11C4 6.7 4.7 6 5.5 6H10"/>';
I.code = '<path d="M8.5 7L3.5 12l5 5M15.5 7l5 5-5 5M13.5 4.5l-3 15"/>';
I.flask = '<path d="M9 3.5h6M10 3.5v5.2L4.6 18.2c-.6 1.1.2 2.3 1.4 2.3h12c1.2 0 2-1.2 1.4-2.3L14 8.7V3.5M7.4 14h9.2"/>';
I.camera = '<rect x="3" y="7" width="18" height="13" rx="2.4"/><circle cx="12" cy="13.5" r="3.6"/><path d="M8.5 7l1.4-2.5h4.2L15.5 7"/>';
I.trash = '<path d="M4.5 7h15M9.5 7V4.5h5V7M6.5 7l1 12.5c.1.8.7 1.5 1.5 1.5h6c.8 0 1.4-.7 1.5-1.5L17.5 7M10 11v6M14 11v6"/>';
I.edit = '<path d="M4 20h4.2L19.4 8.8a2.4 2.4 0 00-3.4-3.4L4.8 16.6 4 20z"/>';
I.dup = '<rect x="8" y="8" width="12" height="12" rx="2.2"/><path d="M16 8V5.8C16 4.8 15.2 4 14.2 4H5.8C4.8 4 4 4.8 4 5.8v8.4c0 1 .8 1.8 1.8 1.8H8"/>';
I.shield = '<path d="M12 3l7.5 3v5.4c0 4.6-3.2 8.2-7.5 9.6-4.3-1.4-7.5-5-7.5-9.6V6z"/><path d="M8.6 12.2l2.4 2.4 4.4-4.8"/>';
I.drive = '<rect x="3" y="13" width="18" height="7" rx="2"/><path d="M5 13l2.2-7.4c.2-.6.8-1.1 1.5-1.1h6.6c.7 0 1.3.5 1.5 1.1L19 13M16.5 16.5h.01"/>';
I.archive = '<rect x="3.5" y="4" width="17" height="5" rx="1.4"/><path d="M5 9v9.2c0 .8.6 1.3 1.3 1.3h11.4c.7 0 1.3-.5 1.3-1.3V9M10 13h4"/>';
I.sliders = '<path d="M4 7h10M18 7h2M4 17h4M12 17h8"/><circle cx="16" cy="7" r="2.2"/><circle cx="10" cy="17" r="2.2"/>';
I.user = '<circle cx="12" cy="8.5" r="3.8"/><path d="M4.5 20.5c.9-3.8 3.9-6 7.5-6s6.6 2.2 7.5 6"/>';
I.redo = '<path d="M15 7l4.5 4.5L15 16M19 11.5H9.5a5 5 0 000 10H12"/>';
I.folderplus = I.folder + '<path d="M12 11v5M9.5 13.5h5"/>';

const esc = s => String(s == null ? '' : s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
const attrs = o => Object.entries(o).filter(([, v]) => v !== undefined && v !== null && v !== false).map(([k, v]) => v === true ? k : `${k}="${esc(v)}"`).join(' ');
/* a button: btn(label, {k: primary|sec|ghost|danger, icon, act, arg, dis, title, busy, key, cls, aria}) */
const btn = (label, o = {}) => `<button type="button" ${attrs({ class: `btn ${o.k || ''} ${o.cls || ''} ${o.busy ? 'busy' : ''}`, 'data-act': o.act, 'data-arg': o.arg, 'data-k': o.key || (o.act ? `${o.act}:${o.arg ?? ''}` : undefined), disabled: !!o.dis, title: o.title, 'aria-label': o.aria || (!label ? o.title : undefined), 'aria-haspopup': o.pop, 'aria-expanded': o.exp })}>${o.icon ? ic(o.busy ? 'refresh' : o.icon, 18) : ''}${label ? `<span>${label}</span>` : ''}</button>`;
const tag = (t, k = '', icon) => `<span class="tag ${k}">${icon ? ic(icon, 14) : ''}${t}</span>`;
const ST = { ok: ['ok', 'okc'], warn: ['warn', 'warnc'], fail: ['err', 'errc'], err: ['err', 'errc'], info: ['info', 'infoc'], off: ['off', 'muted'], run: ['refresh', 'accc'], wait: ['off', 'muted'] };
const stt = (k, t) => `<span class="stt ${ST[k][1]}">${ic(ST[k][0], 16, k === 'run' ? 'spin' : '')}${t}</span>`;
const sw = (on, act, arg, label, dis) => `<button type="button" class="sw" role="switch" ${attrs({ 'aria-checked': on === 'mixed' ? 'mixed' : String(!!on), 'data-act': act, 'data-arg': arg, 'data-k': `${act}:${arg}`, 'aria-label': label, title: label, disabled: !!dis })}></button>`;
const cbx = (on, act, arg, label) => `<button type="button" class="cb" role="checkbox" ${attrs({ 'aria-checked': String(!!on), 'data-act': act, 'data-arg': arg, 'data-k': `${act}:${arg}`, 'aria-label': label })}>${on ? ic('check', 15) : ''}</button>`;
const seg = (items, cur, act, label) => `<div class="segc" role="group" aria-label="${esc(label)}">${items.map(([v, t]) => `<button type="button" aria-pressed="${v === cur}" data-act="${act}" data-arg="${v}" data-k="${act}:${v}">${t}</button>`).join('')}</div>`;
const pbar = (p, cls = '') => `<div class="pb ${cls}" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow="${Math.round(p * 100)}"><i style="width:${Math.max(2, Math.round(p * 100))}%"></i></div>`;
const fold = (on, act, arg, label) => `<button type="button" class="fold ${on ? 'on' : ''}" aria-expanded="${!!on}" aria-label="${esc(label)}" data-act="${act}" data-arg="${arg}" data-k="${act}:${arg}">${ic('chev', 16)}</button>`;
const sech = (title, right = '', o = {}) => `<div class="sech ${o.click ? 'click' : ''}" ${o.click ? `data-act="${o.click[0]}" data-arg="${o.click[1]}"` : ''}><span class="t">${title}</span><span class="r8">${right}</span>${o.fold !== undefined ? fold(o.fold, o.click[0], o.click[1], (o.fold ? 'Fold ' : 'Unfold ') + o.plain) : ''}</div>`;
const plural = (n, a, b) => `${n} ${n === 1 ? a : (b || a + 's')}`;
const hl = (text, q) => { const t = esc(text); if (!q) return t; const i = text.toLowerCase().indexOf(q.toLowerCase()); if (i < 0) return t; return esc(text.slice(0, i)) + '<mark>' + esc(text.slice(i, i + q.length)) + '</mark>' + esc(text.slice(i + q.length)); };
const MON = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
const fdate = t => { const d = new Date(t); return `${d.getDate()} ${MON[d.getMonth()]}`; };
const ftime = t => { const d = new Date(t); return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`; };
const ago = t => { const m = Math.round((Date.now() - t) / 60000); if (m < 1) return 'just now'; if (m < 60) return `${m} min ago`; const h = Math.round(m / 60); if (h < 24) return `${h} h ago`; const d = Math.round(h / 24); return d === 1 ? 'yesterday' : `${d} days ago`; };
const dayLabel = t => { const d0 = new Date(); d0.setHours(0, 0, 0, 0); const dd = Math.floor((d0 - new Date(t).setHours(0, 0, 0, 0)) / 864e5); return dd <= 0 ? 'Today' : dd === 1 ? 'Yesterday' : fdate(t); };
const hm = mins => mins < 1 ? `${Math.max(1, Math.round(mins * 60))} s` : (mins = Math.round(mins)) < 60 ? `${mins} min` : `${Math.floor(mins / 60)} h${mins % 60 ? ' ' + (mins % 60) + ' min' : ''}`;
const DAY = 864e5;
const clone = o => JSON.parse(JSON.stringify(o));
const uid = () => Math.random().toString(36).slice(2, 8);
const rnd = seed => () => { seed = (seed * 16807) % 2147483647; return (seed - 1) / 2147483646; };
const hash = s => { let h = 2166136261; for (const c of s) h = Math.imul(h ^ c.charCodeAt(0), 16777619); return (h >>> 0) % 2147483646 + 1; };

/* ===== plugin facts ===== */
const pinfo = f => {
  for (const [id, m] of Object.entries(CAT)) { const p = m.plugins.find(x => x[0] === f); if (p) return { f, mod: id, rank: p[1], masters: p[2], note: p[3] }; }
  if (EXTRA_PLUGINS[f]) { const e = EXTRA_PLUGINS[f]; return { f, mod: e[3], rank: e[0], masters: e[1], note: e[2] }; }
  return { f, mod: null, rank: 99, masters: [], note: '' };
};

/* ===== stored state ===== */
const KEY = 'rauls-toolbox-state-1';
const store = {
  load() { try { const s = localStorage.getItem(KEY); return s ? JSON.parse(s) : null; } catch (e) { return null; } },
  save() { try { localStorage.setItem(KEY, JSON.stringify(S)); } catch (e) { /* storage blocked: state stays in memory */ } },
  clear() { try { localStorage.removeItem(KEY); } catch (e) { /* ignore */ } }
};

let LIBG = null; /* shared group ids for the seeded library */
function seedProfile(sd) {
  const now = Date.now();
  if (!LIBG) LIBG = LIB.map(([name]) => ({ id: name === 'Ungrouped' ? 'ug' : uid(), name }));
  const groups = LIBG.map(g => ({ ...g, open: sd.open.includes(g.name) }));
  const mods = [];
  LIB.forEach(([, ids], i) => ids.forEach(id => mods.push({ id, on: sd.on.includes(id), g: groups[i].id, at: now - (i === 4 ? 4 : 9) * DAY, opts: id === 'b42' ? ['3D scopes', 'Weapon Mods Expanded patch'] : [] })));
  let plugins = [];
  mods.forEach(m => CAT[m.id].plugins.forEach(p => plugins.push({ f: p[0], on: true })));
  plugins.push({ f: 'B42 Optics - WMX.esp', on: true });
  plugins.sort((a, b) => pinfo(a.f).rank - pinfo(b.f).rank);
  const vis = plugins.filter(x => sd.on.includes(pinfo(x.f).mod));
  sd.swap.forEach(([a, b]) => { const ia = plugins.indexOf(vis[a]), ib = plugins.indexOf(vis[b]); [plugins[ia], plugins[ib]] = [plugins[ib], plugins[ia]]; });
  const p = { id: sd.id, name: sd.name, blurb: sd.blurb, groups, mods, plugins, ttwOn: sd.ttw, gfx: { ...sd.gfx, vsync: true }, far: { q: sd.q, fine: null, built: null },
    stable: [], dismissed: [], bench: [], stab: [], pics: [], events: [], created: now - 12 * DAY, example: true };
  const n = sd.bench.length;
  sd.bench.forEach((avg, i) => p.bench.push({ t: now - (n - i) * 2 * DAY, avg, low: Math.round(avg * 0.64), ft: +(1000 / avg).toFixed(1), p99: +(1000 / avg * 1.7).toFixed(1), ex: true }));
  if (sd.id === 'beaut') { p.events.push({ t: now - 5 * DAY, label: 'Wasteland Flora Overhaul added' }); p.events.push({ t: now - 3 * DAY, label: 'Engine 0.24.0' }); }
  if (sd.id === 'vplus') p.events.push({ t: now - 3 * DAY, label: 'Engine 0.24.0' });
  if (sd.id === 'hard') p.events.push({ t: now - 3 * DAY, label: 'Tougher Traders added' });
  if (sd.stable) { const sg = sig(p, false); p.stable.push({ sig: sg, t: now - 2 * DAY }); p.stab.push({ t: now - 2 * DAY, ok: true, cells: 40, crashes: 0, hitches: 2, mins: 20, ex: true }); }
  else p.stab.push({ t: now - 2 * DAY, ok: false, cells: 23, crashes: 1, hitches: 4, mins: 11, where: 'Freeside', ex: true });
  seedVersions(p, sd);
  p.pics.push({ id: uid(), t: now - 4 * DAY, label: 'Earlier', q: sd.q === 'high' ? 'balanced' : sd.q, look: 0, ex: true });
  p.pics.push({ id: uid(), t: now - 1 * DAY, label: 'Latest', q: sd.q, look: 1, ex: true });
  return p;
}
/* versions: every change to mods, order or game settings makes one; reverting moves "current", newer ones stay */
const vsnap = p => clone({ mods: p.mods.map(m => ({ id: m.id, on: m.on, g: m.g })), plugins: p.plugins, ttwOn: p.ttwOn, gfx: p.gfx, groups: p.groups });
/* a version holds the enabled set and order; the library (installed mods, groups) is shared and stays */
function applySnap(p, sn) {
  const mi = new Map(sn.mods.map((m, i) => [m.id, i])), pi = new Map(sn.plugins.map((x, i) => [x.f, i]));
  p.mods.forEach(m => { const sm = sn.mods.find(x => x.id === m.id); m.on = sm ? sm.on : false; });
  p.mods.sort((a, b) => (mi.has(a.id) ? mi.get(a.id) : 1e6) - (mi.has(b.id) ? mi.get(b.id) : 1e6));
  p.plugins.forEach(x => { const sx = sn.plugins.find(y => y.f === x.f); if (sx) x.on = sx.on; });
  p.plugins.sort((a, b) => (pi.has(a.f) ? pi.get(a.f) : 1e6) - (pi.has(b.f) ? pi.get(b.f) : 1e6));
  p.ttwOn = sn.ttwOn; p.gfx = clone(sn.gfx);
}
/* shared library: installed mods, group names, membership and group order follow the profile that changed them */
function syncShared(src) {
  S.profiles.forEach(q => {
    if (q === src) return;
    const open = new Map(q.groups.map(g => [g.id, g.open]));
    q.groups = src.groups.map(g => ({ ...g, open: open.has(g.id) ? open.get(g.id) : g.open }));
    const lib = new Map(src.mods.map(m => [m.id, m]));
    const gone = q.mods.filter(m => !lib.has(m.id)).map(m => m.id);
    q.mods = q.mods.filter(m => lib.has(m.id));
    q.mods.forEach(m => { const l = lib.get(m.id); m.g = l.g; m.opts = clone(l.opts); m.at = l.at; });
    src.mods.forEach(l => { if (!q.mods.some(m => m.id === l.id)) { q.mods.push({ id: l.id, on: false, g: l.g, at: l.at, opts: clone(l.opts) }); src.plugins.filter(x => pinfo(x.f).mod === l.id).forEach(x => { if (!q.plugins.some(y => y.f === x.f)) q.plugins.push({ f: x.f, on: true }); }); } });
    const keep = new Set(src.plugins.map(x => x.f));
    q.plugins = q.plugins.filter(x => keep.has(x.f) || !gone.includes(pinfo(x.f).mod));
  });
}
function libraryCopy() { const src = P(); return { groups: clone(src.groups), mods: src.mods.map(m => ({ ...clone(m), on: false })), plugins: clone(src.plugins) }; }
function seedVersions(p, sd) {
  const now = Date.now(), full = vsnap(p);
  const drop = { vplus: 'lrf', beaut: 'wfo', hard: 'tt' }[sd.id];
  const first = clone(full); first.mods.find(m => m.id === drop).on = false;
  const tAdd = { vplus: 6, beaut: 5, hard: 3 }[sd.id];
  const v1 = { id: 'v' + uid(), t: now - 10 * DAY, summary: `Started with ${first.mods.filter(m => m.on).length} mods on`, snap: first };
  const v2 = { id: 'v' + uid(), t: now - tAdd * DAY, summary: `${CAT[drop].name} on`, snap: full };
  p.versions = [v1, v2]; p.cur = v2.id;
  const vAt = t => (t >= v2.t ? v2 : v1).id;
  p.bench.forEach(r => { r.vid = vAt(r.t); }); p.stab.forEach(r => { r.vid = vAt(r.t); });
  if (sd.id === 'hard') { const sg1 = sig({ ...p, ...first }, true); p.stable.push({ sig: sg1, t: now - 8 * DAY }); p.stab.unshift({ t: now - 8 * DAY, ok: true, cells: 40, crashes: 0, hitches: 3, mins: 20, ex: true, vid: v1.id }); }
}
function ensureVersions(p) { if (!p.versions || !p.versions.length) { const v = { id: 'v' + uid(), t: Date.now(), summary: p.mods.length ? `${plural(p.mods.length, 'mod')}` : 'Empty profile', snap: vsnap(p) }; p.versions = [v]; p.cur = v.id; } }
function vdiff(a, b) {
  const out = [], A = new Map(a.mods.map(m => [m.id, m])), B = new Map(b.mods.map(m => [m.id, m]));
  const add = b.mods.filter(m => !A.has(m.id)).length, rem = a.mods.filter(m => !B.has(m.id)).length;
  if (add) out.push(`+${plural(add, 'mod')}`); if (rem) out.push(`−${plural(rem, 'mod')}`);
  const on = b.mods.filter(m => A.has(m.id) && A.get(m.id).on !== m.on);
  if (on.length) out.push(on.length === 1 ? `${CAT[on[0].id].name} ${on[0].on ? 'on' : 'off'}` : `${on.length} mods switched`);
  const common = x => x.filter(id => A.has(id) && B.has(id));
  const ord = p => modsInOrder(p).map(m => m.id), pl = p => p.plugins.map(x => x.f);
  const pa = new Set(a.plugins.map(x => x.f)), pb = new Set(b.plugins.map(x => x.f));
  if (common(ord(a)).join() !== common(ord(b)).join() || pl(a).filter(f => pb.has(f)).join() !== pl(b).filter(f => pa.has(f)).join()) out.push('order changed');
  const pOn = b.plugins.filter(x => pa.has(x.f) && a.plugins.find(y => y.f === x.f).on !== x.on).length; if (pOn) out.push(`${plural(pOn, 'plugin')} switched`);
  if (a.ttwOn !== b.ttwOn) out.push(`TTW ${b.ttwOn ? 'on' : 'off'}`);
  if (JSON.stringify(a.gfx) !== JSON.stringify(b.gfx)) out.push('settings changed');
  return out.slice(0, 3).join(', ');
}
function recordVersion(p) {
  ensureVersions(p);
  const now = Date.now(), s = vsnap(p), cur = p.versions.find(v => v.id === p.cur) || p.versions[p.versions.length - 1];
  const last = p.versions[p.versions.length - 1];
  if (cur === last && last.auto && now - last.t < 30000) { /* quick edits fold into one version */
    const sum = vdiff(last.base, s);
    if (!sum) { p.versions.pop(); p.cur = last.baseId; return; }
    Object.assign(last, { snap: s, summary: sum, t: now }); return;
  }
  const sum = vdiff(cur.snap, s); if (!sum) return;
  const v = { id: 'v' + uid(), t: now, summary: sum, snap: s, base: cur.snap, baseId: cur.id, auto: true };
  p.versions.push(v); p.cur = v.id;
}
const vLabel = t => `${dayLabel(t)} ${ftime(t)}`;
function vStatus(p, v) {
  const runs = p.stab.filter(r => r.vid === v.id);
  if (runs.some(r => r.ok) || p.stable.some(x => x.sig === sig({ ...p, ...v.snap }))) return 'pass';
  return runs.length ? 'fail' : 'untested';
}
const lastStable = p => (p.versions || []).slice().sort((a, b) => b.t - a.t).find(v => v.id !== p.cur && vStatus(p, v) === 'pass');
function emptyProfile(name) {
  const L = S && S.profiles && S.profiles.length ? libraryCopy() : { groups: [{ id: 'ug', name: 'Ungrouped', open: true }], mods: [], plugins: [] };
  return { id: 'p' + uid(), name, blurb: '', groups: L.groups, mods: L.mods, plugins: L.plugins, ttwOn: false, gfx: { res: '1920 × 1080', cap: '60', up: 'Off', look: 'Original', vsync: true },
    far: { q: 'balanced', fine: null, built: null }, stable: [], dismissed: [], bench: [], stab: [], pics: [], events: [], created: Date.now(), versions: null };
}
function fresh() {
  return {
    v: 1,
    wiz: { step: 0, done: false, game: null, store: null, engine: false, fo3: null, ttw: null, prof: 'beaut', far: null, q: 'high' },
    g: { gamePath: null, gameOk: false, fo3Path: null, storePath: null, storeFree: 0, cache: 3.2, ttwInstalled: false, ttwVer: '3.4', ttwFile: 'TTW_3.4.0.mpi', ttwUpdate: false,
      engine: '0.24.0', builds: ['0.24.0', '0.23.0'], tbVer: '0.5.0', lastCheck: 0, autoCheck: true, motion: true, hc: false, big: false, mlAt: Date.now() - 2 * DAY, mlNew: 0,
      sessions: [], docRun: null, scen: { game: false, engine: false, disk: false, dl: false, stab: false, fo3: false } },
    profiles: SEED.map(seedProfile), active: 'beaut'
  };
}
let S = null;

/* transient UI state */
const U = { page: 'home', tab: 'list', q: '', filt: 'all', lq: '', lfilt: 'all', open: {}, secs: { builtin: false, details: false, sessions: false, stablelist: false, fulllog: false, adv: false, lbase: false, lttw: false, engines: false }, menu: null, dlg: null, toasts: [],
  pal: null, scen: false, drag: null, undo: {}, play: null, run: null, sortPrev: null, ttwFlow: null, vopen: null, picPlace: 'gs', picWorld: 'nv', picAll: false, picQ: '', picMode: 'two', pan: { x: 0, y: 0 }, picA: null, picB: null, zoom: 1, logF: 'all', logQ: '', logOpen: {}, doc: null, focus: null, wizFind: null };

const P = () => S.profiles.find(p => p.id === S.active) || S.profiles[0];
const prof = id => S.profiles.find(p => p.id === id);

/* ===== model rules ===== */
const ttwActive = p => S.g.ttwInstalled && p.ttwOn;
const groupOrder = p => p.groups;
/* mods in display order (group by group); row numbers follow this order */
const modsInOrder = p => { const out = []; p.groups.forEach(g => p.mods.forEach(m => { if (m.g === g.id) out.push(m); })); return out; };
const visPlugins = p => p.plugins.filter(x => { const i = pinfo(x.f); const m = p.mods.find(mm => mm.id === i.mod); return m ? m.on : true; });
const allActiveFiles = p => [...BASE, ...(ttwActive(p) ? TTWP : []), ...visPlugins(p).filter(x => x.on).map(x => x.f)];
function sig(p, ttw = ttwActive(p)) { return (ttw ? 'T;' : ';') + modsInOrder(p).filter(m => m.on).map(m => m.id).join(',') + '|' + visPlugins(p).filter(x => x.on).map(x => x.f).join(','); }
const farSig = p => (ttwActive(p) ? 'T;' : ';') + p.mods.filter(m => m.on && CAT[m.id].kind !== 'dll').map(m => m.id).sort().join(',');
const stableNow = p => !p.mods.some(m => m.on) || p.stable.some(s => s.sig === sig(p));
const missingMasters = p => { const act = new Set(allActiveFiles(p)); const out = []; visPlugins(p).filter(x => x.on).forEach(x => pinfo(x.f).masters.forEach(m => { if (!act.has(m)) out.push([x.f, m]); })); return out; };
const sortedPlugins = p => { const arr = p.plugins.slice(); return arr.map((x, i) => [x, i]).sort((a, b) => (pinfo(a[0].f).rank - pinfo(b[0].f).rank) || (a[1] - b[1])).map(x => x[0]); };
const sortMoves = p => { const vis = visPlugins(p).map(x => x.f); const sorted = sortedPlugins(p).map(x => x.f).filter(f => vis.includes(f)); const moves = []; vis.forEach((f, i) => { const j = sorted.indexOf(f); if (j !== i) moves.push([f, i - j]); }); return { moves, sorted }; };
const outOfOrder = p => sortMoves(p).moves.length;
function farState(p) {
  if (J['far:' + p.id]) return 'building';
  const b = p.far.built; if (!b) return 'never';
  if (b.sig !== farSig(p)) return 'stale-mods';
  if (b.q !== p.far.q || JSON.stringify(b.fine) !== JSON.stringify(p.far.fine)) return 'stale-q';
  return 'ok';
}
const qLabel = p => p.far.fine ? 'Custom' : QUAL[p.far.q][0];
const farSize = p => { let s = QUAL[p.far.q][1]; if (p.far.fine) s = +(s * (0.6 + p.far.fine.tex / 1024 * 0.4 + p.far.fine.dist * 0.08)).toFixed(1); return s + (p.mods.some(m => m.on && m.id === 'wfo') ? 0.3 : 0) + (ttwActive(p) ? 0.8 : 0); };
const storeLow = () => S.g.storeFree < 20;
const TTW_GB = 25, ttwRoom = free => free >= TTW_GB;
const gameOk = () => S.g.gameOk && !S.g.scen.game;
const TB = () => PARTS.find(c => c.id === 'toolbox'), updAvail = () => S.g.tbVer !== TB().nw;

/* play checks: one list feeds Home, the Play flow and the Doctor */
function checks(p = P()) {
  const mm = missingMasters(p), ooo = outOfOrder(p), fs = farState(p), on = p.mods.filter(m => m.on).length;
  const L = [];
  L.push(gameOk() ? { id: 'game', n: 'Fallout: New Vegas', st: 'ok', t: 'Found' } : { id: 'game', n: 'Fallout: New Vegas', st: 'fail', t: 'Not found', why: 'Game folder not found', fix: ['Find the game…', 'pickFolder', 'fnv'] });
  L.push(storeLow() ? { id: 'store', n: 'Toolbox storage', st: 'warn', t: `${S.g.storeFree} GB free`, why: 'Low on space', fix: ['Free up space…', 'cleanCache', ''] } : { id: 'store', n: 'Toolbox storage', st: 'ok', t: 'OK', chip: `${S.g.storeFree} GB free` });
  L.push(S.g.scen.engine ? { id: 'engine', n: 'Engine', st: 'fail', t: 'Damaged', why: '3 engine files changed', fix: ['Repair engine', 'repairEngine', ''] } : { id: 'engine', n: 'Engine', st: 'ok', t: 'OK', chip: S.g.engine });
  if (!p.ttwOn) L.push({ id: 'ttw', n: 'Tale of Two Wastelands', st: 'ok', t: 'Not used' });
  else if (!S.g.ttwInstalled) L.push({ id: 'ttw', n: 'Tale of Two Wastelands', st: 'warn', t: 'Not added', why: 'This profile uses TTW', fix: ['Add TTW…', 'ttwStart', ''] });
  else L.push({ id: 'ttw', n: 'Tale of Two Wastelands', st: 'ok', t: 'Added', chip: S.g.ttwVer });
  L.push({ id: 'mods', n: 'Mods', st: 'ok', t: 'OK', chip: `${on} of ${p.mods.length} on` });
  if (mm.length) L.push({ id: 'load', n: 'Load order', st: 'fail', t: 'Missing master', why: `${mm[0][0]} needs ${mm[0][1]}`, fix: ['Show load order', 'nav', 'load'] });
  else if (ooo) L.push({ id: 'load', n: 'Load order', st: 'warn', t: `${ooo} out of order`, why: 'Not sorted', fix: ['Sort…', 'sortGo', ''] });
  else L.push({ id: 'load', n: 'Load order', st: 'ok', t: 'OK', chip: plural(allActiveFiles(p).length, 'plugin') });
  if (fs === 'ok') L.push({ id: 'far', n: 'LOD', st: 'ok', t: 'Up to date' });
  else if (fs === 'building') L.push({ id: 'far', n: 'LOD', st: 'info', t: 'Building' });
  else L.push({ id: 'far', n: 'LOD', st: 'warn', t: fs === 'never' ? 'Not built' : 'Out of date', why: fs === 'never' ? 'Far away looks flat' : fs === 'stale-q' ? 'Quality changed' : 'Mods changed since the build', fix: [fs === 'never' ? 'Build LOD' : 'Rebuild LOD', 'farBuild', 'go'] });
  if (stableNow(p)) L.push({ id: 'stab', n: 'Mod setup', st: 'ok', t: p.mods.length ? 'Tested stable' : 'Nothing to test' });
  else L.push({ id: 'stab', n: 'Mod setup', st: 'warn', t: 'Not tested yet', why: 'This exact setup has no stability test', fix: ['Run stability test', 'stabGo', ''] });
  return L;
}

/* ===== undo, events ===== */
const snap = p => clone({ groups: p.groups, mods: p.mods, plugins: p.plugins, ttwOn: p.ttwOn });
function change(label, fn, o = {}) {
  const p = P(); const s = snap(p);
  fn(p);
  syncShared(p);
  recordVersion(p);
  (U.undo[p.id] = U.undo[p.id] || []).push({ label, s });
  if (U.undo[p.id].length > 30) U.undo[p.id].shift();
  if (o.event) p.events.push({ t: Date.now(), label: o.event });
  if (o.toast !== false) toast(o.toast || label, true);
  commit();
}
function undo() {
  const p = P(); const st = U.undo[p.id]; if (!st || !st.length) return;
  const u = st.pop(); U.toasts = U.toasts.filter(t => t.text !== u.label); Object.assign(p, clone(u.s)); syncShared(p); recordVersion(p); toast(`Undone: ${u.label}`); commit();
}

/* ===== jobs: timed mock work with progress ===== */
const J = {};
let tickT = null;
function job(key, dur, done, extra = {}) {
  J[key] = { start: Date.now(), dur, p: 0, done, ...extra };
  if (!tickT) tickT = setInterval(tick, 120);
  render();
}
function cancelJob(key) { delete J[key]; render(); }
function tick() {
  const now = Date.now(); let any = false;
  for (const k of Object.keys(J)) {
    const j = J[k]; j.p = Math.min(1, (now - j.start) / j.dur);
    if (j.failAt && j.p >= j.failAt) { delete J[k]; j.fail && j.fail(); continue; }
    if (j.p >= 1) { delete J[k]; j.done && j.done(); } else any = true;
  }
  if (U.run) any = true;
  if (!any && !Object.keys(J).length) { clearInterval(tickT); tickT = null; }
  render();
}
const jp = k => J[k] ? J[k].p : 0;

/* ===== toasts ===== */
function toast(text, withUndo = false) {
  const id = uid(); U.toasts.push({ id, text, undo: withUndo }); if (U.toasts.length > 2) U.toasts.shift();
  setTimeout(() => { U.toasts = U.toasts.filter(t => t.id !== id); render(); }, withUndo ? 5000 : 3500);
}

/* ===== render loop with a small DOM morph (keeps focus, inputs and running animations) ===== */
function syncAttrs(a, b) {
  for (const at of [...a.attributes]) if (!b.hasAttribute(at.name)) a.removeAttribute(at.name);
  for (const at of [...b.attributes]) if (a.getAttribute(at.name) !== at.value) a.setAttribute(at.name, at.value);
}
function morph(a, b) {
  const bc = [...b.childNodes];
  for (let i = 0; i < bc.length; i++) {
    const bn = bc[i], an = a.childNodes[i];
    if (!an) { a.appendChild(bn); continue; }
    const key = n => n.nodeType === 1 ? n.getAttribute('data-key') : null;
    if (an.nodeType !== bn.nodeType || an.nodeName !== bn.nodeName || key(an) !== key(bn)) { a.replaceChild(bn, an); continue; }
    if (an.nodeType !== 1) { if (an.nodeValue !== bn.nodeValue) an.nodeValue = bn.nodeValue; continue; }
    syncAttrs(an, bn);
    morph(an, bn);
  }
  while (a.childNodes.length > bc.length) a.removeChild(a.lastChild);
}
let rq = false;
function render() { if (rq) return; rq = true; requestAnimationFrame(() => { rq = false; draw(); }); }
function commit() { store.save(); render(); }
function draw() {
  const root = document.getElementById('app');
  const html = U.quit ? `<div class="wiz"><div class="inner" style="text-align:center;padding-top:160px"><h1>Raul’s Toolbox has closed</h1><p class="lead" style="margin:0 auto 18px">Your setup is saved.</p><button type="button" class="btn primary" data-act="reopen" data-arg="" data-k="reopen:">Open again</button></div></div>` : S.wiz.done ? appHTML() : wizHTML();
  const t = document.createElement('div'); t.innerHTML = html;
  morph(root, t);
  document.documentElement.dataset.hc = S.g.hc ? '1' : '0';
  document.documentElement.dataset.motion = S.g.motion ? '1' : '0';
  document.documentElement.dataset.fs = S.g.big ? 'l' : 'm';
  document.querySelectorAll('[data-val]').forEach(el => { if (el !== document.activeElement && el.value !== el.dataset.val) el.value = el.dataset.val; });
  placeMenu();
  if (typeof afterDraw === 'function') afterDraw();
  if (U.focus) { const el = document.querySelector(`[data-k="${CSS.escape(U.focus)}"]`); U.focus = null; if (el) el.focus(); }
  if (U.scrollTop) { const m = document.querySelector('.mscroll,.wiz'); if (m) m.scrollTop = 0; U.scrollTop = false; }
}
function placeMenu() {
  const m = document.querySelector('.menu'); if (!m || !U.menu) return;
  const a = document.querySelector(`[data-k="${CSS.escape(U.menu.anchor)}"]`); if (!a) return;
  const w = winBox(), r0 = a.getBoundingClientRect(), r = { left: (r0.left - w.left) / w.s, right: (r0.right - w.left) / w.s, top: (r0.top - w.top) / w.s, bottom: (r0.bottom - w.top) / w.s }, mw = m.offsetWidth, mh = m.offsetHeight;
  let left = U.menu.alignLeft ? r.left : r.right - mw; left = Math.max(8, Math.min(left, 1280 - mw - 8));
  let top = r.bottom + 6; if (top + mh > 720 - 8) top = Math.max(8, r.top - mh - 6);
  m.style.left = left + 'px'; m.style.top = top + 'px';
  if (U.menu.fresh) { U.menu.fresh = false; const f = m.querySelector('.mi:not(:disabled)'); if (f) f.focus(); }
}

/* the app lives in a 1280x720 desktop window; fixed overlays are positioned inside it (the window is transformed) */
function winBox() { const el = document.getElementById('win'); const r = el.getBoundingClientRect(); return { left: r.left, top: r.top, s: r.width / 1280 }; }
function fitWin() { const s = Math.min(1, (innerWidth - 32) / 1280, (innerHeight - 32) / 720); document.getElementById('win').style.setProperty('--ws', Math.max(0.3, s)); }
addEventListener('resize', fitWin);
