/* SPDX-License-Identifier: GPL-3.0-only */
/* ===== vector drawings: LOD hero, location pictures, benchmark chart, welcome mark. No game art. ===== */
function farHero(p, st) {
  const q = p.far.q, dense = q === 'high' ? 3 : q === 'balanced' ? 2 : 1;
  const farCol = st === 'ok' ? 'var(--accent)' : st === 'never' ? 'var(--ctl)' : st === 'building' ? 'var(--info)' : 'var(--warn)';
  const dash = st === 'never' ? 'stroke-dasharray="4 5"' : '';
  let far = '';
  if (st !== 'never') {
    far += `<path d="M300 118 L330 84 L372 84 L396 118" ${dash}/>`;
    far += `<path d="M412 118 v-26 h12 v-10 h10 v36 M440 118 v-22 h12 v22" ${dash}/>`;
    far += `<path d="M462 118 q18 -22 36 0 q16 -16 30 0" ${dash}/>`;
    for (let i = 0; i < dense * 3; i++) { const x = 306 + i * (210 / (dense * 3)); far += `<path d="M${x} 118 v-${6 + (i % 3) * 2} M${x - 3} ${113 - (i % 3)} l3 -4 l3 4" opacity=".8"/>`; }
  } else far += `<path d="M300 118 L540 118" ${dash}/><text x="420" y="96" text-anchor="middle" fill="var(--muted)" stroke="none" font-size="12">No LOD yet</text>`;
  return `<svg viewBox="0 0 560 140" role="img" aria-label="Near objects at full detail on the left, the LOD on the right" fill="none" stroke-linecap="round" stroke-linejoin="round">
  <text x="22" y="22" fill="var(--muted)" font-size="12">Near · full detail</text><text x="300" y="22" fill="var(--text)" font-size="12" font-weight="600">LOD${st === 'ok' ? ' · matches your mods' : st === 'never' ? ' · not built' : st === 'building' ? ' · building' : ' · out of date'}</text>
  <g stroke="var(--text)" stroke-width="2">
    <circle cx="40" cy="80" r="6"/><path d="M40 86v18M40 104l-7 14M40 104l7 14M40 92l12-4M40 92l-9 6M54 86l10-3" />
    <path d="M86 118V80M78 92c0-8 16-8 16 0M78 92v-8M94 92v-8"/>
    <path d="M124 118V90L156 66 188 90V118M142 118v-18h14v18M166 92h12v10h-12z"/>
  </g>
  <path d="M276 32V128" stroke="var(--text)" stroke-dasharray="4 5" opacity=".7"/>
  <g stroke="${farCol}" stroke-width="2">${far}</g>
  <path d="M14 120H546" stroke="var(--line)" stroke-width="2"/></svg>`;
}

/* location picture: a flat vector scene shaped by place, profile mods and LOD quality */
function scene(place, o) {
  const r = rnd(hash(place + (o.seed || '')));
  const lush = o.trees, rich = o.look, q = o.q;
  const farN = q === 'high' ? 3 : q === 'balanced' ? 2 : 1;
  const dc = (PLACES.find(x => x[0] === place) || [])[2] === 'Capital Wasteland';
  const sky = dc ? (rich ? ['#3E5A4E', '#B7B98A'] : ['#56685E', '#A9A88A']) : rich ? ['#2E4A78', '#E79A5B'] : ['#4A6A8E', '#C9B48E'];
  const id = 'g' + hash(place + o.q + o.trees + o.look + (o.seed || ''));
  let s = `<svg viewBox="0 0 320 180" preserveAspectRatio="xMidYMid slice" role="img" aria-label="${esc(o.label)}"><defs><linearGradient id="${id}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${sky[0]}"/><stop offset="1" stop-color="${sky[1]}"/></linearGradient></defs><rect width="320" height="180" fill="url(#${id})"/>`;
  s += `<circle cx="${230 + r() * 50}" cy="${36 + r() * 16}" r="${rich ? 13 : 10}" fill="${rich ? '#FFD9A0' : '#F3E6C4'}" opacity=".95"/>`;
  if (rich) for (let i = 0; i < 4; i++) s += `<ellipse cx="${40 + r() * 240}" cy="${22 + r() * 30}" rx="${18 + r() * 22}" ry="4" fill="#fff" opacity=".18"/>`;
  for (let l = 0; l < farN; l++) {
    const y0 = 96 + l * 6, col = ['#7A6C8E', '#8E7A78', '#9C8468'][l];
    let d = `M0 ${y0}`; for (let x = 0; x <= 320; x += 20) d += ` L${x} ${y0 - 8 - r() * (22 - l * 5)}`; d += ' L320 180 L0 180Z';
    s += `<path d="${d}" fill="${col}" opacity="${0.55 + l * 0.15}"/>`;
  }
  s += `<path d="M0 128 Q80 118 160 126 T320 122 V180 H0Z" fill="${dc ? '#7E7A62' : rich ? '#B98A5A' : '#A89070'}"/>`;
  s += landmark((PLACES.find(x => x[0] === place) || [0, 0, 0, place])[3], r, rich);
  const nT = lush ? 9 : 3;
  for (let i = 0; i < nT; i++) {
    const x = 12 + r() * 296, y = 132 + r() * 30, h = 10 + r() * 12;
    s += lush ? `<path d="M${x} ${y} v-${h}" stroke="#4E3B2A" stroke-width="2"/><ellipse cx="${x}" cy="${y - h}" rx="${6 + r() * 4}" ry="${5 + r() * 3}" fill="#5F7A44"/>` : `<path d="M${x} ${y} l-4 -8 M${x} ${y} l3 -10 M${x} ${y} l6 -6" stroke="#6E6A44" stroke-width="1.6"/>`;
  }
  if (q === 'performance') s += `<rect width="320" height="110" fill="#C9B48E" opacity=".18"/>`;
  return s + '</svg>';
}

/* benchmark history: average line, 1% low dashed, change markers between runs */
function benchChart(p) {
  const runs = p.bench.slice(-8); if (runs.length < 2) return `<div class="empty">Run the benchmark twice to see a history.</div>`;
  const W = 680, H = 200, L = 34, R = 24, T = 18, B = 42;
  const vals = runs.flatMap(r => [r.avg, r.low]); const lo = Math.floor((Math.min(...vals) - 6) / 10) * 10, hi = Math.ceil((Math.max(...vals) + 6) / 10) * 10;
  const x = i => L + i * (W - L - R) / (runs.length - 1), y = v => T + (hi - v) * (H - T - B) / (hi - lo);
  let s = `<svg class="chart" viewBox="0 0 ${W} ${H}" role="img" aria-label="Benchmark history for ${esc(p.name)}: ${runs.map(r => r.avg + ' fps').join(', ')}">`;
  for (let v = lo; v <= hi; v += 10) s += `<line x1="${L}" x2="${W - R}" y1="${y(v)}" y2="${y(v)}" stroke="var(--line)"/><text x="${L - 8}" y="${y(v) + 4}" text-anchor="end">${v}</text>`;
  runs.forEach((r, i) => { if (i < runs.length - 1) { const ev = p.events.filter(e => e.t > r.t && e.t <= runs[i + 1].t); ev.forEach((e, k) => { const xm = (x(i) + x(i + 1)) / 2 + k * 8; s += `<line x1="${xm}" x2="${xm}" y1="${T - 6}" y2="${H - B}" stroke="var(--info)" stroke-dasharray="3 4"/><circle cx="${xm}" cy="${T - 6}" r="4" fill="var(--info)"><title>${esc(e.label)}</title></circle>`; }); } });
  s += `<polyline fill="none" stroke="var(--muted)" stroke-width="1.6" stroke-dasharray="5 5" points="${runs.map((r, i) => `${x(i)},${y(r.low)}`).join(' ')}"/>`;
  s += `<polyline fill="none" stroke="var(--accent)" stroke-width="2.4" points="${runs.map((r, i) => `${x(i)},${y(r.avg)}`).join(' ')}"/>`;
  const off = p.bench.length - runs.length;
  runs.forEach((r, i) => { const v = (p.versions || []).find(vv => vv.id === r.vid), lab = v ? vLabel(v.t) : fdate(r.t), cur = v && v.id === p.cur;
    s += `<g class="pt" role="button" tabindex="0" aria-label="${r.avg} fps, version ${esc(lab)}" aria-haspopup="menu" data-act="ptMenu" data-arg="${off + i}" data-k="pt:${off + i}"><circle cx="${x(i)}" cy="${y(r.avg)}" r="12" fill="transparent"/><circle cx="${x(i)}" cy="${y(r.avg)}" r="${cur ? 6 : 4.5}" fill="var(--accent)" ${cur ? 'stroke="var(--text)" stroke-width="2"' : ''}/></g><text x="${x(i)}" y="${H - 20}" text-anchor="middle">${esc(lab.split(' ').slice(0, -1).join(' '))}</text><text x="${x(i)}" y="${H - 6}" text-anchor="middle">${lab.split(' ').slice(-1)[0]}</text>`; });
  return s + '</svg>';
}

function welcomeMark() {
  return `<svg viewBox="0 0 200 200" width="180" height="180" role="img" aria-label="Raul's Toolbox" fill="none" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="100" cy="100" r="92" fill="var(--raise2)"/>
  <path d="M20 132 Q60 112 100 126 T180 120" stroke="var(--line)" stroke-width="3"/>
  <path d="M30 120 L58 90 L80 104 L106 74 L134 100 L156 86 L172 104" stroke="var(--muted)" stroke-width="2.4" opacity=".6"/>
  <g transform="translate(46 40) scale(4.4)" stroke="var(--brand)" stroke-width="1.5">${I.app}</g></svg>`;
}

/* drawn landmarks per kind of place (simple shapes, no game art) */
function landmark(kind, r, rich) {
  const B = '#5A4636', G = '#3D3550', S = '#8C877C', lit = c => rich ? c : '#6E6A62';
  const house = (x, w, h) => `<rect x="${x}" y="${126 - h}" width="${w}" height="${h}" fill="${B}"/><path d="M${x - 4} ${128 - h} L${x + w / 2} ${114 - h} L${x + w + 4} ${128 - h}Z" fill="${B}"/>`;
  const tower = (x, w, h) => `<rect x="${x}" y="${126 - h}" width="${w}" height="${h}" fill="${G}"/>${rich ? `<rect x="${x + 4}" y="${132 - h}" width="4" height="4" fill="#FFD27A"/>` : ''}`;
  const K = {
    town: house(60, 34, 22) + house(118, 26, 16) + house(170, 22, 14) + `<g fill="${B}"><rect x="226" y="74" width="4" height="50"/><rect x="242" y="74" width="4" height="50"/><ellipse cx="236" cy="72" rx="16" ry="10"/></g>`,
    city: [0, 1, 2, 3, 4, 5].map(i => tower(70 + i * 32, 22, 66 - (i % 3) * 14)).join('') + `<path d="M150 48 l6 -14 l6 14Z" fill="${G}"/>`,
    ruins: [0, 1, 2, 3].map(i => `<path d="M${60 + i * 52} 126 v-${30 + (i % 2) * 18} l12 -6 l6 10 l10 -4 v${36 + (i % 2) * 14}Z" fill="${G}"/>`).join(''),
    ruinsdc: [0, 1, 2, 3, 4].map(i => `<path d="M${40 + i * 52} 126 v-${26 + (i % 3) * 14} h10 l6 -8 l8 12 h8 v${22 + (i % 3) * 14}Z" fill="#4A4A44"/>`).join(''),
    coaster: house(70, 50, 26) + `<path d="M140 126 C160 60 200 60 214 96 S260 70 290 126" stroke="${S}" stroke-width="3" fill="none"/>`,
    dino: house(60, 40, 14) + `<path d="M180 126 v-20 h-10 l10 -30 c10 -8 26 -6 30 6 l-8 4 v40" fill="#4E7A4A"/>`,
    base: `<path d="M30 124 H290" stroke="${S}" stroke-width="2" stroke-dasharray="2 4"/>` + [0, 1, 2].map(i => `<path d="M${70 + i * 60} 124 l20 -18 l20 18Z" fill="#6A6450"/>`).join('') + `<rect x="250" y="84" width="6" height="40" fill="${S}"/><rect x="243" y="80" width="20" height="8" fill="${S}"/>`,
    quarry: `<path d="M20 126 L70 96 L130 104 L170 92 L240 110 L300 126Z" fill="#8E7E66"/><path d="M80 126 L110 108 L150 126Z" fill="#6E604C"/>`,
    dam: `<path d="M40 100 Q160 140 280 100 L280 132 Q160 160 40 132Z" fill="#C8BBA6"/><path d="M40 132 Q160 160 280 132 V180 H40Z" fill="#3E6F8E"/><rect x="120" y="84" width="10" height="26" fill="#9A8E7C"/><rect x="190" y="84" width="10" height="26" fill="#9A8E7C"/>`,
    water: `<path d="M0 132 Q80 124 160 132 T320 130 V180 H0Z" fill="#3E6F8E"/><path d="M40 140 h30 M150 150 h40 M230 142 h28" stroke="#9CC3D8" stroke-width="1.5" opacity=".6"/>` + house(250, 20, 12),
    rocks: `<path d="M30 130 L60 84 L92 92 L118 130Z M180 130 L214 70 L250 80 L282 130Z" fill="#A4553E"/><path d="M60 84 L92 92 L88 104 L64 98Z" fill="#C0705A"/>`,
    lodge: `<path d="M60 74 L120 100 L240 100 L300 74" fill="#E8EEF2" opacity=".5"/>` + house(130, 70, 30) + `<rect x="148" y="104" width="10" height="10" fill="${lit('#FFD27A')}"/>`,
    mountain: `<path d="M60 126 L160 46 L260 126Z" fill="#5E5868"/><path d="M160 46 v-22 M152 30 h16" stroke="${S}" stroke-width="2"/><circle cx="160" cy="22" r="3" fill="${lit('#FF7A7A')}"/>`,
    solar: [0, 1, 2, 3, 4, 5, 6].map(i => `<rect x="${30 + i * 36}" y="118" width="26" height="6" fill="#7EA6C0" transform="skewX(-20)"/>`).join('') + `<rect x="230" y="60" width="12" height="64" fill="${S}"/><circle cx="236" cy="58" r="8" fill="${lit('#FFE6A0')}"/>`,
    vault: `<path d="M80 126 Q160 70 240 126Z" fill="#7A6E5C"/><circle cx="160" cy="112" r="14" fill="#9A9A90" stroke="#6E6A62" stroke-width="3"/><text x="160" y="116" font-size="9" text-anchor="middle" fill="#3D3550">VAULT</text>`,
    farm: [0, 1, 2, 3, 4].map(i => `<path d="M20 ${130 + i * 6} H300" stroke="#5F7A44" stroke-width="2"/>`).join('') + house(220, 36, 20),
    dome: `<path d="M90 128 Q160 52 230 128Z" fill="#6E6A62"/><path d="M110 128 Q160 78 210 128" stroke="#8C877C" stroke-width="3" fill="none"/><rect x="150" y="66" width="20" height="8" fill="#8C877C"/>`,
    ship: `<path d="M30 134 L60 104 H250 L290 134Z" fill="#5A5A58"/><rect x="120" y="76" width="16" height="28" fill="#5A5A58"/><path d="M0 136 H320 V180 H0Z" fill="#3E5A58"/>`,
    tower: `<rect x="140" y="36" width="40" height="90" fill="#6E6A62"/><rect x="134" y="30" width="52" height="8" fill="#6E6A62"/>${rich ? [0, 1, 2, 3].map(i => `<rect x="148" y="${48 + i * 18}" width="6" height="6" fill="#FFD27A"/><rect x="166" y="${48 + i * 18}" width="6" height="6" fill="#FFD27A"/>`).join('') : ''}`,
    fortress: `<path d="M70 126 V92 L90 82 H230 L250 92 V126Z" fill="#5E5E5A"/><path d="M150 82 L160 60 L170 82Z" fill="#5E5E5A"/>`,
    monument: `<path d="M152 126 L156 30 L160 22 L164 30 L168 126Z" fill="#C8C2B4"/><rect x="100" y="124" width="120" height="4" fill="#8C877C"/>`,
    memorial: `<path d="M90 126 V100 H230 V126Z" fill="#C8C2B4"/><path d="M100 100 Q160 50 220 100Z" fill="#B8B2A4"/>` + [0, 1, 2, 3, 4].map(i => `<rect x="${104 + i * 26}" y="100" width="4" height="26" fill="#9A948A"/>`).join(''),
    bridge: `<path d="M0 104 H320" stroke="#6E6A62" stroke-width="8"/>` + [0, 1, 2, 3].map(i => `<rect x="${40 + i * 80}" y="104" width="8" height="40" fill="#6E6A62"/>`).join('') + house(150, 24, 12).replace(/126/g, '100')
  };
  return K[kind] || K.town;
}
