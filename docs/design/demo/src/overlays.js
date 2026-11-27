/* SPDX-License-Identifier: GPL-3.0-only */
/* ===== menus, dialogs, toasts, search, test situations panel ===== */
function overlays() {
  let h = '';
  if (U.menu) h += menuHTML();
  if (U.dlg) h += `<div class="scrim" data-act="scrim" data-arg="">${dlgHTML()}</div>`;
  if (U.pal) h += `<div class="scrim" data-act="scrim" data-arg="" style="align-items:flex-start;padding-top:12vh">${palHTML()}</div>`;
  if (U.scen) h += scenHTML();
  h += `<div class="toasts" aria-live="polite">${(U.dlg ? [] : U.toasts).map(t => `<div class="toast" data-key="${t.id}"><span>${esc(t.text)}</span>${t.undo ? btn('Undo', { k: 'ghost', icon: 'undo', act: 'undo', arg: '', key: 'toastundo:' + t.id }) : ''}${btn('', { k: 'ghost', cls: 'icon', icon: 'x', act: 'toastX', arg: t.id, title: 'Dismiss' })}</div>`).join('')}</div>`;
  return h;
}
function menuHTML() {
  const m = U.menu; let items = [];
  if (m.id === 'prof') {
    const busy = !!U.run;
    items.push({ head: 'Profiles' });
    S.profiles.forEach(p => items.push({ label: p.name, icon: p.id === S.active ? 'check' : 'user', act: 'profSwitch', arg: p.id, dis: busy && p.id !== S.active, cur: p.id === S.active, note: stableNow(p) ? '' : 'not tested' }));
    items.push({ sep: 1 });
    items.push({ label: 'New profile…', icon: 'plus', act: 'profNew', arg: '' }, { label: `Duplicate “${P().name}”…`, icon: 'dup', act: 'profDup', arg: '' }, { label: `Rename “${P().name}”…`, icon: 'edit', act: 'profRename', arg: '' }, { label: `Delete “${P().name}”…`, icon: 'trash', act: 'profDelete', arg: '', danger: 1, dis: S.profiles.length < 2 || busy });
  }
  if (m.id === 'grp') {
    const p = P(), g = p.groups.find(x => x.id === m.gid), gi = p.groups.indexOf(g);
    if (g.id !== 'ug') items.push({ label: 'Rename…', icon: 'edit', act: 'grpRename', arg: g.id });
    items.push({ label: 'Move up', icon: 'up', act: 'grpMove', arg: g.id + ':-1', dis: gi === 0 }, { label: 'Move down', icon: 'down', act: 'grpMove', arg: g.id + ':1', dis: gi === p.groups.length - 1 });
    if (g.id !== 'ug') items.push({ sep: 1 }, { label: 'Delete group…', icon: 'trash', act: 'grpDelete', arg: g.id, danger: 1, title: 'Its mods move to Ungrouped' });
  }
  if (m.id === 'pt') {
    const p = P(), r = p.bench[m.run], v = p.versions.find(x => x.id === r.vid), cur = v && v.id === p.cur;
    items.push({ head: v ? `Version · ${vLabel(v.t)}` : fdate(r.t) });
    if (v) items.push({ label: v.summary, icon: 'info', act: 'noop', arg: '', dis: 1 });
    items.push({ label: `${r.avg} fps · ${r.low} fps low`, icon: 'bolt', act: 'noop', arg: '', dis: 1 }, { sep: 1 });
    items.push(v ? { label: cur ? 'This is the current version' : 'Revert to this version…', icon: 'undo', act: 'vRevert', arg: v.id, dis: cur } : { label: 'Version not kept', icon: 'off', act: 'noop', arg: '', dis: 1 });
  }
  return `<div class="menu" role="menu" aria-label="${m.id === 'prof' ? 'Profiles' : 'Group options'}">${items.map(it => it.head ? `<div class="mh">${it.head}</div>` : it.sep ? '<div class="sepm"></div>' : `<button type="button" role="${it.cur !== undefined ? 'menuitemradio' : 'menuitem'}" ${it.cur !== undefined ? `aria-checked="${!!it.cur}"` : ''} class="mi ${it.danger ? 'danger' : ''}" data-act="${it.act}" data-arg="${esc(it.arg)}" ${it.dis ? 'disabled' : ''} ${it.title ? `title="${esc(it.title)}"` : ''}>${ic(it.icon, 16, it.cur ? 'accc' : '')}<span>${esc(it.label)}</span>${it.note ? `<span class="muted" style="margin-left:auto;font-size:.8rem;padding-left:12px">${it.note}</span>` : ''}</button>`).join('')}</div>`;
}
function dlgHTML() {
  const d = U.dlg;
  const wrap = (title, body, acts, wide) => `<div class="dlg ${wide ? 'wide' : ''}" role="dialog" aria-modal="true" aria-labelledby="dlgT" data-act="noop" data-arg=""><h2 id="dlgT">${title}</h2>${body}<div class="cca">${acts}</div></div>`;
  if (d.t === 'confirm') return wrap(d.title, `<p>${d.text}</p>`, btn('Cancel', { k: 'ghost', act: 'dlgClose', arg: '' }) + btn(d.ok, { k: d.danger ? 'dangerp' : 'primary', act: d.act, arg: d.arg, key: 'dlgok' }));
  if (d.t === 'name') return wrap(d.title, `<div class="field"><label for="nameIn">${d.label || 'Name'}</label><input id="nameIn" class="inp" value="${esc(d.v)}" data-val="${esc(d.v)}" data-in="dlgName" data-k="nameIn" autocomplete="off" maxlength="40">${d.err ? `<span class="err-text" role="alert">${d.err}</span>` : ''}</div>${d.extra || ''}`, btn('Cancel', { k: 'ghost', act: 'dlgClose', arg: '' }) + btn(d.ok, { k: 'primary', act: 'nameOk', arg: '', dis: !d.v.trim() }));
  if (d.t === 'folder') {
    const list = FOLDERS[d.kind.replace(/^w/, '')];
    return wrap(d.kind.includes('store') ? 'Pick a storage folder' : d.kind.includes('fo3') ? 'Pick the Fallout 3 folder' : 'Pick the game folder', `<div class="rows picker" role="listbox" aria-label="Folders">${list.map((f, i) => `<div class="r" role="option" tabindex="0" aria-selected="${d.sel === i}" data-act="folderSel" data-arg="${i}" data-k="folder:${i}">${ic(d.kind.includes('store') ? 'drive' : 'folder', 18)}<div style="min-width:0"><div class="nm">${f[0]}</div><div class="sub mono">${esc(f[1])}</div></div><span class="end">${f[3] ? `<span class="muted">${f[3]} GB free</span>` : ''}</span></div>`).join('')}</div>`, btn('Cancel', { k: 'ghost', act: 'dlgClose', arg: '' }) + btn('Use this folder', { k: 'primary', act: 'folderOk', arg: '', dis: d.sel == null }));
  }
  if (d.t === 'shortcuts') return wrap('Shortcuts', `<div class="rows">${[['Ctrl K', 'Search or go to a page'], ['?', 'This list'], ['Alt ↑ / Alt ↓', 'Move the focused mod, group or plugin'], ['Space', 'Switch or tick the focused control'], ['Enter', 'Open the focused row'], ['Esc', 'Close a menu or dialog']].map(k => `<div class="r"><span class="kbd" style="margin:0;min-width:110px;justify-content:center">${k[0]}</span><span>${k[1]}</span></div>`).join('')}</div>`, btn('Close', { k: 'primary', act: 'dlgClose', arg: '' }));
  if (d.t === 'install') return installHTML(d, wrap);
  return '';
}
function installHTML(d, wrap) {
  const p = P(), have = new Set(p.mods.map(m => m.id));
  if (d.step === 'pick') return wrap('Install a mod', `<p>Archives in your download folder. Pick one.</p><div class="rows picker" role="listbox" aria-label="Archives" style="max-height:340px;overflow:auto">${ARCH.map((a, i) => { const inst = a.mod && have.has(a.mod); return `<div class="r" role="option" tabindex="${inst ? -1 : 0}" aria-selected="${d.sel === i}" aria-disabled="${!!inst}" data-act="${inst ? 'noop' : 'archSel'}" data-arg="${i}" data-k="arch:${i}" style="${inst ? 'opacity:.55;cursor:default' : ''}">${ic('archive', 18)}<div style="min-width:0"><div class="nm">${esc(a.file)}</div><div class="sub">${a.size >= 1000 ? (a.size / 1000).toFixed(1) + ' GB' : a.size + ' MB'}${a.fomod ? ' · has install options' : ''}</div></div><span class="end">${inst ? tag('Installed', 'ok') : ''}</span></div>`; }).join('')}</div>`,
    btn('Browse…', { k: 'ghost', icon: 'folder', act: 'openLink', arg: 'browse:archive' }) + btn('Cancel', { k: 'ghost', act: 'dlgClose', arg: '' }) + btn('Next', { k: 'primary', act: 'instNext', arg: '', dis: d.sel == null }), true);
  const a = ARCH[d.sel], c = a.mod ? CAT[a.mod] : null;
  if (d.step === 'opts') {
    const g = a.fomod[d.page];
    const prev = g.id === 'style' ? (d.ch.style === '2d' ? 'Flat scope overlay' : 'Real lens view') : g.id === 'ed' ? (d.ch.ed === 'full' ? 'Storms and dust wall' : 'Weather and skies') : 'Patches for mods you have';
    return wrap(`${esc(c.name)} · install options`, `<p>Page ${d.page + 1} of ${a.fomod.length} · ${g.t}</p><div class="fomod"><div role="${g.kind === 'one' ? 'radiogroup' : 'group'}" aria-label="${g.t}">${g.opts.map(o => { const on = g.kind === 'one' ? d.ch[g.id] === o[0] : (d.ch[g.id] || []).includes(o[0]); const rec = g.id === 'patch' && ((o[0] === 'wmx' && have.has('wmx')) || (o[0] === 'wrp' && have.has('wrp'))); return `<button type="button" class="opt" role="${g.kind === 'one' ? 'radio' : 'checkbox'}" aria-checked="${on}" data-act="optPick" data-arg="${g.id}:${o[0]}:${g.kind}" data-k="opt:${o[0]}">${g.kind === 'one' ? '<span class="rd"></span>' : `<span class="cb" aria-checked="${on}">${on ? ic('check', 14) : ''}</span>`}<span><b>${o[1]}</b>${rec ? ' ' + tag('You have this mod', 'acc') : ''}<div class="od">${o[2]}</div></span></button>`; }).join('')}</div>
      <div class="prev">${scene(g.id === 'ed' ? 'red' : 'gs', { q: 'balanced', trees: false, look: d.ch.ed === 'full' ? 1 : 0, seed: 'opt' + JSON.stringify(d.ch), label: prev }).replace('<svg ', '<svg style="width:100%;border-radius:8px;display:block" ')}<div class="sub" style="margin-top:6px">${prev}</div></div></div>`,
      btn('Back', { k: 'ghost', act: 'instBack', arg: '' }) + btn('Next', { k: 'primary', act: 'instNext', arg: '' }), true);
  }
  if (d.step === 'review') {
    const plugs = instPlugins(d), mm = plugs.flatMap(f => pinfo(f).masters.filter(x => !allActiveFiles(p).includes(x) && !plugs.includes(x)).map(x => [f, x]));
    return wrap(`Install ${esc(c.name)}`, `<dl class="kv" style="margin-bottom:12px"><dt>Version</dt><dd>${c.ver}</dd><dt>Plugins</dt><dd>${plugs.length ? plugs.map(f => `<span class="mono">${esc(f)}</span>`).join('<br>') : 'None · files only'}</dd>${d.ch && Object.keys(d.ch).length ? `<dt>Options</dt><dd>${esc(optLabels(d).join(' · ') || 'Defaults')}</dd>` : ''}<dt><label for="instG">Group</label></dt><dd><select id="instG" class="inp" data-in="instGroup" data-val="${d.g}" data-k="instG" style="height:34px">${p.groups.map(g => `<option value="${g.id}">${esc(g.name)}</option>`).join('')}</select></dd></dl>
      ${mm.length ? `<div class="banner warn" style="background:var(--raise2)"><span class="bi">${ic('warn', 20)}</span><div><div class="bt">${esc(mm[0][0])} needs ${esc(mm[0][1])}</div><div class="bs">It won’t load until that mod is on.</div></div></div>` : ''}<p>Plugins go to the end of the load order. Sort puts them in place.</p>`,
      (a.fomod ? btn('Back', { k: 'ghost', act: 'instBack', arg: '' }) : btn('Back', { k: 'ghost', act: 'instBack', arg: '' })) + btn('Install', { k: 'primary', icon: 'dl', act: 'instGo', arg: '' }), true);
  }
  if (d.step === 'busy') { const j = J.install; return wrap(`Installing ${esc(c ? c.name : a.file)}`, `<p>${j ? ['Unpacking', 'Checking files', 'Adding plugins'][Math.min(2, Math.floor(j.p * 3))] : 'Finishing'}</p>${pbar(j ? j.p : 1)}`, btn('Cancel', { k: 'ghost', act: 'instCancel', arg: '' }), true); }
  if (d.step === 'error') return wrap('This archive is damaged', `<div class="banner err" style="background:var(--raise2)"><span class="bi">${ic('err', 22)}</span><div><div class="bt">${esc(a.file)} can’t be opened</div><div class="bs">It looks like part of a split download. Download the whole file again, then install it.</div></div></div>`, btn('Pick another', { k: 'ghost', act: 'instBack', arg: 'pick' }) + btn('Close', { k: 'primary', act: 'dlgClose', arg: '' }), true);
  return '';
}
function instPlugins(d) {
  const a = ARCH[d.sel]; if (!a.mod) return [];
  let pl = CAT[a.mod].plugins.map(x => x[0]);
  if (a.mod === 'nsk' && d.ch.ed === 'full') pl = ['Nevada Skies - Full Edition.esp'];
  (d.ch.patch || []).forEach(x => pl.push({ wmx: 'B42 Optics - WMX.esp', wrp: 'B42 Optics - WRP.esp', jsue: 'The Living Desert - JSUE.esp' }[x]));
  return pl;
}
function optLabels(d) { const a = ARCH[d.sel]; if (!a.fomod) return []; return a.fomod.flatMap(g => g.opts.filter(o => g.kind === 'one' ? d.ch[g.id] === o[0] : (d.ch[g.id] || []).includes(o[0])).map(o => o[1])); }

/* search or go to */
function palItems() {
  const it = [['Home', 'home', 'nav', 'home'], ['Mods', 'mods', 'nav', 'mods'], ['Load order', 'load', 'nav', 'load'], ['LOD', 'lod', 'nav', 'far'], ['System', 'versions', 'nav', 'system'], ['Doctor', 'doctor', 'nav', 'doctor'], ['Settings', 'settings', 'nav', 'settings'],
    ['Play', 'play', 'play', ''], ['Install mod…', 'plus', 'install', ''], ['Sort the load order', 'load', 'sortGo', ''], ['Build the LOD', 'lod', 'farBuild', 'go'], ['Run a stability test', 'shield', 'stabGo', 'pal'], ['Run the benchmark', 'bolt', 'benchGo', 'pal'], ['Check for updates', 'refresh', 'checkUpdates', 'go'], ['New profile…', 'plus', 'profNew', '']];
  S.profiles.forEach(p => it.push([`Switch to ${p.name}`, 'user', 'profSwitch', p.id]));
  P().mods.forEach(m => it.push([CAT[m.id].name, 'mods', 'findMod', m.id]));
  const q = U.pal.q.toLowerCase();
  return it.filter(x => !q || x[0].toLowerCase().includes(q)).slice(0, 9);
}
function palHTML() {
  const items = palItems(); U.pal.i = Math.min(U.pal.i, Math.max(0, items.length - 1));
  return `<div class="dlg pal" role="dialog" aria-modal="true" aria-label="Search or go to" data-act="noop" data-arg=""><label class="srch">${ic('search', 16)}<span class="sr-only">Search</span><input id="palIn" type="search" placeholder="Pages, actions, profiles, mods" value="${esc(U.pal.q)}" data-val="${esc(U.pal.q)}" data-in="palQ" data-k="palIn" role="combobox" aria-expanded="true" aria-controls="palList" autocomplete="off"></label><div id="palList" role="listbox">${items.map((x, i) => `<button type="button" role="option" aria-selected="${i === U.pal.i}" class="pi ${i === U.pal.i ? 'on' : ''}" data-act="palGo" data-arg="${i}">${ic(x[1], 16)}<span>${esc(x[0])}</span><span class="k">${x[2] === 'nav' ? 'Page' : x[2] === 'findMod' ? 'Mod' : x[2] === 'profSwitch' ? 'Profile' : 'Action'}</span></button>`).join('') || '<div class="muted" style="padding:10px 12px">Nothing found.</div>'}</div></div>`;
}

/* hidden panel (Ctrl+Shift+D) that sets up failure and warning situations for testing */
const SCEN = [['game', 'Game folder missing', 'Play and Doctor fail on the game check'], ['engine', 'Engine files damaged', 'Fails the engine check; Repair fixes it'], ['disk', 'Low disk space', 'Storage warns; TTW install is blocked'], ['dl', 'Downloads fail', 'Update and engine install stop with an error'], ['stab', 'Next stability test crashes', 'The test fails in Freeside'], ['fo3', 'Fallout 3 not found', 'TTW step 1 fails until you browse']];
function scenHTML() {
  return `<div class="scen" role="dialog" aria-label="Test situations"><div style="display:flex;align-items:center;margin-bottom:8px"><b>Test situations</b><span style="margin-left:auto">${btn('', { k: 'ghost', cls: 'icon', icon: 'x', act: 'scenClose', arg: '', title: 'Close (Ctrl+Shift+D)' })}</span></div><div class="rows">${SCEN.map(([k, t, d]) => `<div class="r click" data-act="scen" data-arg="${k}"><div style="flex:1"><div class="nm" style="white-space:normal">${t}</div><div class="sub">${d}</div></div>${sw(S.g.scen[k], 'scen', k, t)}</div>`).join('')}</div>
  <div class="cca">${btn('Add play history', { k: 'ghost', cls: 'sm', act: 'scenHistory', arg: '' })}${btn('TTW update', { k: 'ghost', cls: 'sm', act: 'scenTtwUpd', arg: '' })}</div></div>`;
}
