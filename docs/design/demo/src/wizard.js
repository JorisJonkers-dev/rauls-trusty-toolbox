/* SPDX-License-Identifier: GPL-3.0-only */
/* ===== setup wizard: runs on first open; its stage is stored, so a reload resumes ===== */
const WSTEPS = ['Welcome', 'Game', 'Storage', 'Engine', 'TTW', 'Profile', 'LOD', 'Done'];
function wizHTML() {
  ABAR = '';
  const W = S.wiz, s = W.step;
  const stepper = `<div class="stp" aria-label="Setup steps" style="gap:8px;padding:12px 14px">${WSTEPS.map((t, i) => `${i ? `<span class="ln ${i <= s ? 'on' : ''}"></span>` : ''}<div class="s ${i < s ? 'done' : i === s ? 'cur' : ''}" ${i === s ? 'aria-current="step"' : ''}><span class="n" style="width:28px;height:28px;font-size:.85rem">${i < s ? ic('check', 14) : i + 1}</span><span class="lb"><b style="font-size:.86rem">${t}</b></span></div>`).join('')}</div>`;
  const body = WIZ[s]();
  return `<header class="top"><div class="brand">${ic('app', 22)}Raul's Toolbox</div><span class="tchip" style="cursor:default">Setup · step ${s + 1} of ${WSTEPS.length}</span><span class="tsp"></span></header>
  <main class="main wizm" id="main" aria-label="Setup"><div class="wiz"><div class="inner">${stepper}<div class="pg" data-key="wiz${s}">${body}</div></div></div>${ABAR ? `<div class="abar" role="toolbar" aria-label="Setup actions">${ABAR}</div>` : ''}</main>
  <footer class="foot"><span class="dot ${Object.keys(J).length ? 'busy' : ''}"></span><span role="status">${Object.keys(J).length ? 'Working…' : 'Setup'}</span><span class="sp"></span></footer>${overlays()}`;
}
const wbar = (next, o = {}) => abar((S.wiz.step > 0 && !o.noBack ? btn('Back', { k: 'ghost', act: 'wizBack', arg: '', dis: o.busy }) : '') + (o.extra || '') + next);
const wrow = (name, sub, end) => `<div class="r" style="min-height:64px"><div style="flex:1;min-width:0"><div class="nm">${name}</div>${sub ? `<div class="sub mono" style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${sub}</div>` : ''}</div><span class="end">${end}</span></div>`;
const WIZ = [
  () => { wbar(btn('Start', { k: 'primary', act: 'wizNext', arg: '' })); return `<div class="hero">${welcomeMark()}<div><h1>Let's get New Vegas ready.</h1><p class="lead">A few steps: find the game, install the engine, pick a starting setup. Nothing in your game folder changes.</p><p class="muted" style="margin:0">About 5 minutes, plus the LOD build if you want it now.</p></div></div>`; },
  () => {
    const W = S.wiz, f = J['find:wfnv'], ok = W.game && W.game.ok;
    wbar(btn('Next', { k: 'primary', act: 'wizNext', arg: '', dis: !ok || !!f }));
    const st = f ? stt('run', 'Looking…') : ok ? stt('ok', 'Found') : W.game ? stt('fail', W.game.path ? 'No game in this folder' : 'Not found') : stt('wait', 'Not checked');
    return `<h2 style="margin:4px 0 12px">Find Fallout: New Vegas</h2><div class="rows">${wrow('Fallout: New Vegas', W.game && W.game.path ? esc(W.game.path) : f ? 'Looking in your Steam and GOG libraries…' : 'No folder yet', st + btn('Browse…', { k: 'ghost', icon: 'folder', act: 'pickFolder', arg: 'wfnv' }) + btn('Find', { k: 'ghost', icon: 'search', act: 'findFolder', arg: 'wfnv', busy: !!f }))}</div>
      ${W.game && !ok && !f ? `<div class="banner err" style="margin-top:12px" role="alert"><span class="bi">${ic('err', 22)}</span><div><div class="bt">${W.game.path ? 'That folder has no FalloutNV.exe' : 'Fallout: New Vegas was not found'}</div><div class="bs">Pick the folder that holds FalloutNV.exe, usually in your Steam or GOG library.</div></div><div class="ba">${btn('Browse…', { k: 'primary', icon: 'folder', act: 'pickFolder', arg: 'wfnv' })}</div></div>` : ''}
      ${ok ? `<p class="muted" style="margin:10px 4px">Version 1.4.0.525 · all DLC found</p>` : ''}`;
  },
  () => {
    const W = S.wiz, low = W.store && W.store.free < 20;
    wbar(btn('Next', { k: 'primary', act: 'wizNext', arg: '', dis: !W.store }));
    const free = W.store ? W.store.free : 0;
    return `<h2 style="margin:4px 0 12px">Toolbox storage</h2><div class="rows">${wrow('Storage folder', W.store ? esc(W.store.path) : 'Not set', (W.store ? (low ? stt('warn', `${free} GB free`) : stt('ok', `${free} GB free`)) : '') + btn('Browse…', { k: 'ghost', icon: 'folder', act: 'pickFolder', arg: 'wstore' }))}
      <div class="r" style="min-height:64px"><div style="flex:1"><div class="nm">Space</div><div style="margin-top:8px;max-width:460px">${pbar(Math.min(1, 30 / Math.max(free, 1)), low ? 'warn' : '')}</div></div><span class="end muted">About 30 GB for mods, LOD and TTW</span></div></div>
      ${low ? `<div class="banner warn" style="margin-top:12px"><span class="bi">${ic('warn', 22)}</span><div><div class="bt">Low on space</div><div class="bs">You can go on; TTW and a High LOD may not fit.</div></div><div class="ba">${btn('Pick another drive', { k: 'sec', act: 'pickFolder', arg: 'wstore' })}</div></div>` : ''}<p class="muted" style="margin:10px 4px">Mods live here, never in the game folder.</p>`;
  },
  () => {
    const W = S.wiz, j = J.wengine;
    wbar(btn('Next', { k: 'primary', act: 'wizNext', arg: '', dis: !W.engine || !!j }), { busy: !!j, extra: !W.engine && !j ? btn(U.wEngErr ? 'Try again' : 'Install engine', { k: 'sec', icon: 'dl', act: 'wizEngine', arg: '' }) : '' });
    return `<h2 style="margin:4px 0 12px">Install the engine</h2><div class="rows">${wrow('Engine 0.24.0', 'Runs the game · script extenders built in · 182 MB', j ? stt('run', `${Math.round(j.p * 100)} %`) : W.engine ? stt('ok', 'Installed') : U.wEngErr ? stt('fail', 'Download failed') : stt('wait', 'Not installed'))}</div>
      ${j ? `<div class="card sunk" style="margin-top:12px"><div style="margin-bottom:8px"><b>${['Downloading', 'Checking the download', 'Unpacking'][Math.min(2, Math.floor(j.p * 3))]}</b></div>${pbar(j.p)}</div>` : ''}
      ${U.wEngErr && !j ? `<div class="banner err" style="margin-top:12px" role="alert"><span class="bi">${ic('err', 22)}</span><div><div class="bt">The download stopped</div><div class="bs">Check your connection, then try again.</div></div></div>` : ''}`;
  },
  () => {
    const W = S.wiz, j = J.wttw, f = J['find:wfo3'], fo3 = W.fo3 && W.fo3.ok;
    const decided = W.ttw === 'added' || W.ttw === 'skipped';
    wbar(btn(W.ttw === 'skipped' ? 'Next' : 'Next', { k: 'primary', act: 'wizNext', arg: '', dis: !decided || !!j }), { busy: !!j, extra: !decided && !j ? btn('Skip for now', { k: 'ghost', act: 'wizTtw', arg: 'skip' }) + btn('Add TTW', { k: 'sec', icon: 'plus', act: 'wizTtw', arg: 'add', dis: !fo3 }) : W.ttw === 'skipped' ? btn('Add TTW instead', { k: 'ghost', act: 'wizTtw', arg: 'undo' }) : '' });
    return `<h2 style="margin:4px 0 4px">Tale of Two Wastelands <span class="tag">Optional</span></h2><p class="muted" style="margin:0 0 12px">Plays Fallout 3 inside New Vegas. You can add it later from Mods.</p>
      <div class="rows">${wrow('Fallout 3', W.fo3 && W.fo3.path ? esc(W.fo3.path) : f ? 'Looking…' : 'No folder yet', (f ? stt('run', 'Looking…') : fo3 ? stt('ok', 'Found') : W.fo3 ? stt('fail', 'Not found') : '') + btn('Browse…', { k: 'ghost', icon: 'folder', act: 'pickFolder', arg: 'wfo3' }) + btn('Find', { k: 'ghost', icon: 'search', act: 'findFolder', arg: 'wfo3', busy: !!f }))}
      ${wrow('TTW installer file', `Downloads › ${S.g.ttwFile}`, stt('ok', 'Found'))}</div>
      ${j ? `<div class="card sunk" style="margin-top:12px"><div style="margin-bottom:8px"><b>${['Checking the Fallout 3 files', 'Converting sound', 'Building archives', 'Patching plugins'][Math.min(3, Math.floor(j.p * 4))]}</b> <span class="muted">${Math.round(j.p * 100)} %</span></div>${pbar(j.p)}</div>` : ''}
      ${W.ttw === 'added' ? `<div class="banner ok" style="margin-top:12px"><span class="bi">${ic('ok', 22)}</span><div><div class="bt">TTW ${S.g.ttwVer} added</div><div class="bs">It shows in Mods, pinned at the top.</div></div></div>` : ''}
      ${W.ttw === 'skipped' ? `<div class="banner info" style="margin-top:12px"><span class="bi">${ic('info', 22)}</span><div><div class="bt">Skipped</div><div class="bs">Mods shows TTW as Not added, with an Add button.</div></div></div>` : ''}`;
  },
  () => {
    const W = S.wiz;
    wbar(btn('Next', { k: 'primary', act: 'wizNext', arg: '' }));
    const cards = [...SEED.map(sd => [sd.id, sd.name, sd.blurb, `${sd.on.length} mods on${sd.ttw ? ' · uses TTW' + (W.ttw !== 'added' ? ', add it later' : '') : ''}`, sd.stable]), ['empty', 'Empty', 'Every mod off. Build your own setup.', '0 mods on', false]];
    return `<h2 style="margin:4px 0 4px">Pick a starting profile</h2><p class="muted" style="margin:0 0 12px">A profile is a whole setup: mods, load order, settings and LOD. Switch any time from the top bar.</p>
      <div class="pcards" role="radiogroup" aria-label="Starting profile">${cards.map(c => `<button type="button" class="pcard" role="radio" aria-checked="${W.prof === c[0]}" data-act="wizProf" data-arg="${c[0]}" data-k="wizProf:${c[0]}"><span class="pt">${ic(c[0] === 'empty' ? 'plus' : 'user', 18)}${c[1]}${c[4] ? tag('Tested stable', 'ok', 'shield') : ''}</span><span class="pd">${c[2]}</span><span class="tag" style="align-self:flex-start">${c[3]}</span></button>`).join('')}</div>`;
  },
  () => {
    const W = S.wiz, j = J.wfar;
    const decided = W.far === 'built' || W.far === 'later';
    wbar(btn('Next', { k: 'primary', act: 'wizNext', arg: '', dis: !decided || !!j }), { busy: !!j, extra: !decided && !j ? btn('Later', { k: 'ghost', act: 'wizFar', arg: 'later' }) + btn('Build now', { k: 'sec', icon: 'play', act: 'wizFar', arg: 'build' }) : W.far === 'later' ? btn('Build now instead', { k: 'ghost', act: 'wizFar', arg: 'undo' }) : '' });
    const pseudo = { far: { q: W.q }, mods: [] };
    return `<h2 style="margin:4px 0 4px">First LOD</h2><p class="muted" style="margin:0 0 12px">LOD is what you see in the distance. Build it now, or later from the LOD page.</p>
      <div class="card farhero"><div>${farHero(pseudo, W.far === 'built' ? 'ok' : j ? 'building' : 'never')}</div><div>${j ? `<b>${['Reading your mods', 'Terrain', 'Objects', 'Trees', 'Packing'][Math.min(4, Math.floor(j.p * 5))]}</b> <span class="muted">${Math.round(j.p * 100)} %</span><div style="margin-top:8px">${pbar(j.p)}</div>` : W.far === 'built' ? stt('ok', 'Built') : W.far === 'later' ? stt('wait', 'Later · shows as Not built') : `<div class="muted">About ${QUAL[W.q][2]} min · ${QUAL[W.q][1]} GB</div>`}</div></div>
      <div class="sec">${sech('Quality')}<div class="bar" style="margin:0 4px 6px">${seg(Object.entries(QUAL).map(([k, v]) => [k, v[0]]), W.q, 'wizQ', 'LOD quality')}<span class="muted">${QUAL[W.q][3]}</span></div></div>`;
  },
  () => {
    const W = S.wiz;
    wbar(btn('Go to Home', { k: 'primary', act: 'wizFinish', arg: '' }));
    const pn = W.prof === 'empty' ? 'My profile' : SEED.find(x => x.id === W.prof).name;
    const rows = [['Fallout: New Vegas', stt('ok', 'Found')], ['Toolbox storage', W.store.free < 20 ? stt('warn', `${W.store.free} GB free`) : stt('ok', `${W.store.free} GB free`)], ['Engine', stt('ok', '0.24.0')], ['Tale of Two Wastelands', W.ttw === 'added' ? stt('ok', 'Added') : stt('wait', 'Not added')], ['Profile', `<span class="tag">${ic('user', 14)}${esc(pn)}</span>`], ['LOD', W.far === 'built' ? stt('ok', 'Built') : stt('wait', 'Not built')]];
    return `<div class="hero" style="margin-bottom:12px"><span class="okc">${ic('ok', 56)}</span><div><h1>You're set.</h1><p class="lead" style="margin:0">Press Play any time. Raul checks your setup first.</p></div></div><div class="rows">${rows.map(r => `<div class="r"><span class="nm">${r[0]}</span><span class="end">${r[1]}</span></div>`).join('')}</div>`;
  }
];
