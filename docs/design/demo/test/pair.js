// SPDX-License-Identifier: GPL-3.0-only
// usage: node test/pair.js <before.png> <after.png> <out.png> — labelled side-by-side picture
const { chromium } = require('playwright');
const fs = require('fs'); const path = require('path');
const [, , a, b, out] = process.argv;
const d = f => 'data:image/png;base64,' + fs.readFileSync(f).toString('base64');
(async () => {
  const br = await chromium.launch(); const p = await br.newPage({ viewport: { width: 2600, height: 900 } });
  await p.setContent(`<body style="margin:0;background:#0A0B0D;font:600 22px system-ui;color:#E6E8EB"><div id="w" style="display:flex;gap:24px;padding:20px;align-items:flex-start;width:max-content">${[['BEFORE · design page v9', a], ['AFTER · demo', b]].map(([t, f]) => `<div><div style="margin-bottom:10px">${t}</div><img src="${d(f)}" style="display:block;width:1280px"></div>`).join('')}</div></body>`);
  await p.waitForTimeout(200); await (await p.$('#w')).screenshot({ path: out }); await br.close();
})();
