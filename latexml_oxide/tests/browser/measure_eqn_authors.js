// Render an HTML file at the given viewport widths and report the geometry the
// issue #810/#475/#783 guards assert: per display-equation table, whether a tag
// sits beside its row's formula or wraps into it, the table's own scroll width
// and the page's; the vertical gap between consecutive display tables; and, per
// author, the visibility and position of its contacts and the starred-note marker.
//
// Usage:   node measure_eqn_authors.js <file-url> [width,...] [screenshot-prefix]
// Output:  JSON { <width>: { docScrollWidth, vw, eqns: [...], gaps: [...],
//          creators: [...] } }.  Exit 2 on error.
const { chromium } = require('playwright-core');

(async () => {
  const url = process.argv[2];
  const widths = (process.argv[3] || '1280,768,375').split(',').map(Number);
  const shot = process.argv[4];
  if (!url) {
    console.error('usage: node measure_eqn_authors.js <file-url> [widths] [shot-prefix]');
    process.exit(2);
  }
  const browser = await chromium.launch({
    channel: 'chrome',
    headless: true,
    args: ['--no-sandbox', '--disable-gpu'],
  });
  const out = {};
  try {
    for (const w of widths) {
      const page = await browser.newPage({ viewport: { width: w, height: 900 } });
      await page.goto(url, { waitUntil: 'networkidle' });
      out[w] = await page.evaluate(() => {
        const R = (e) => e.getBoundingClientRect();
        const eqns = Array.from(document.querySelectorAll('table.ltx_eqn_table')).map((t) => {
          const rows = Array.from(t.querySelectorAll('tr.ltx_eqn_row')).map((tr) => {
            const tag = tr.querySelector('.ltx_eqn_eqno .ltx_tag');
            const math = Array.from(tr.querySelectorAll('td.ltx_eqn_cell:not(.ltx_eqn_eqno) math'));
            if (!tag || !math.length) return null;
            const tr_ = R(tag);
            const left = Math.min(...math.map((m) => R(m).left));
            const right = Math.max(...math.map((m) => R(m).right));
            const top = Math.min(...math.map((m) => R(m).top));
            const bottom = Math.max(...math.map((m) => R(m).bottom));
            const beside = tr_.left >= right - 0.5 || tr_.right <= left + 0.5;
            const below = tr_.top >= bottom - 0.5;
            return {
              tag: tag.textContent.trim(),
              tagLeft: Math.round(tr_.left), tagRight: Math.round(tr_.right),
              mathLeft: Math.round(left), mathRight: Math.round(right),
              overlaps: !beside && !below, below,
            };
          }).filter(Boolean);
          return { id: t.id, scrollWidth: t.scrollWidth, clientWidth: t.clientWidth,
                   right: Math.round(R(t).right), rows };
        });
        const tables = Array.from(document.querySelectorAll('table.ltx_eqn_table'));
        const gaps = [];
        for (let i = 1; i < tables.length; i++) {
          gaps.push(Math.round(R(tables[i]).top - R(tables[i - 1]).bottom));
        }
        const creators = Array.from(document.querySelectorAll('.ltx_creator')).map((c) => {
          const notes = c.querySelector('.ltx_author_notes');
          const name = c.querySelector('.ltx_personname');
          const nr = name && R(name);
          return {
            star: notes ? getComputedStyle(notes, '::before').content : null,
            contacts: Array.from(c.querySelectorAll('.ltx_contact')).map((k) => {
              const kr = R(k);
              const vis = getComputedStyle(k).visibility === 'visible'
                && getComputedStyle(k.closest('.ltx_author_notes_content') || k).visibility === 'visible';
              return { role: k.className.replace(/.*ltx_role_/, ''), visible: vis,
                       text: k.innerText.trim(), belowName: nr ? kr.top >= nr.bottom - 0.5 : null,
                       top: Math.round(kr.top) };
            }),
          };
        });
        return { vw: window.innerWidth, docScrollWidth: document.documentElement.scrollWidth,
                 eqns, gaps, creators };
      });
      if (shot) await page.screenshot({ path: `${shot}-${w}.png`, fullPage: true });
      await page.close();
    }
    console.log(JSON.stringify(out));
  } finally {
    await browser.close();
  }
})().catch((e) => {
  console.error(String(e));
  process.exit(2);
});
