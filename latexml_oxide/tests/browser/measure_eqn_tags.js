// Render an HTML file at the given viewport widths and report the geometry the
// issue #810 guards assert: per display-equation row, whether its tag sits beside
// the formula or runs into it, whether the tag is visible (not behind the table's
// horizontal scroll or off the page), and the formula's offset from the line's
// centre; per table, its scroll width when it scrolls, and (issue #972) each row's top, height and
// formulas' vertical extent, and the lefts of its content cells (not the pads or the tag), which an aligned display
// shares across its rows; its computed overflow-x, a vertical scrollbar on it or a box above it, and whether its
// formulas reach into the next table cell. Notes are opened, as on hover.
//
// Usage:   node measure_eqn_tags.js <file-url> [width,...]
//          (Chrome; with LATEXML_FIREFOX=<firefox executable> Firefox as well)
// Output:  JSON { "<engine>/<width>": { eqns: [{ id, scrollWidth, clientWidth, rows, lines, overflowX, vbar, spill }] } }.
//          Exit 2 on error.
const pw = require('playwright-core');

(async () => {
  const url = process.argv[2];
  const widths = (process.argv[3] || '1280,768,375').split(',').map(Number);
  if (!url) {
    console.error('usage: node measure_eqn_tags.js <file-url> [widths]');
    process.exit(2);
  }
  const engines = [['chrome', () => pw.chromium.launch({
    channel: 'chrome', headless: true, args: ['--no-sandbox', '--disable-gpu'] })]];
  if (process.env.LATEXML_FIREFOX) {
    engines.push(['firefox', () => pw.firefox.launch({
      executablePath: process.env.LATEXML_FIREFOX, headless: true })]);
  }
  const out = {};
  for (const [name, launch] of engines) {
    const browser = await launch();
    try {
      for (const w of widths) {
        const page = await browser.newPage({ viewport: { width: w, height: 900 } });
        await page.goto(url, { waitUntil: 'load' });
        // A note's content shows on hover only; open it in place so a display in a note is measured.
        await page.addStyleTag({ content: '.ltx_note_content{display:block !important;position:absolute}' });
        out[`${name}/${w}`] = await page.evaluate(() => {
          const R = (e) => e.getBoundingClientRect();
          const eqns = Array.from(document.querySelectorAll('.ltx_eqn_table')).map((t) => {
            const lineL = R(t).left + t.clientLeft;
            const lineR = lineL + t.clientWidth;
            const rows = Array.from(t.querySelectorAll('.ltx_eqn_row')).map((tr) => {
              const tag = tr.querySelector('.ltx_eqn_eqno .ltx_tag');
              const math = Array.from(tr.querySelectorAll('.ltx_eqn_cell:not(.ltx_eqn_eqno) math'));
              if (!tag || !math.length) return null;
              const g = R(tag);
              const left = Math.min(...math.map((m) => R(m).left));
              const right = Math.max(...math.map((m) => R(m).right));
              const beside = g.left >= right - 0.5 || g.right <= left + 0.5;
              return {
                tag: tag.textContent.trim(),
                overlaps: !beside,
                visible: g.left >= lineL - 0.5 && g.right <= lineR + 0.5 && g.right <= window.innerWidth + 0.5,
                offset: Math.round((left + right) / 2 - (lineL + lineR) / 2),
              };
            }).filter(Boolean);
            const lines = Array.from(t.querySelectorAll('.ltx_eqn_row')).map((tr) => {
              const ms = Array.from(tr.querySelectorAll('math')).map(R);
              return {
                top: Math.round(R(tr).top),
                height: Math.round(R(tr).height),
                lefts: Array.from(tr.querySelectorAll(':scope > .ltx_eqn_cell'))
                  .filter((c) => !/ltx_eqn_(center|left|right)_pad|ltx_eqn_eqno/.test(c.className))
                  .map((c) => Math.round(R(c).left - R(t).left)),
                // A row of one spanning cell (\intertext): CSS tables ignore its colspan.
                spanning: !!tr.querySelector(':scope > .ltx_eqn_cell[colspan]:only-child'),
                // The row's formulas' vertical extent (unrounded): a row's must not reach into the next's.
                math: ms.length ? [Math.min(...ms.map((m) => m.top)), Math.max(...ms.map((m) => m.bottom))] : null,
              };
            });
            // Issue #972: a scroll container on the way up with a vertical scrollbar (a display's, or a note's), and
            // the formulas' visible right edge (clipped where an ancestor clips) past their cell into the next one.
            let vbar = false;
            let right = Math.max(...Array.from(t.querySelectorAll('math')).map((m) => R(m).right));
            const cell = t.closest('td, .ltx_td');
            for (let e = t; e && e !== document.body; e = e.parentElement) {
              const c = getComputedStyle(e);
              if (/auto|scroll/.test(c.overflowY) && e.scrollHeight > e.clientHeight) vbar = true;
              if (c.overflowX !== 'visible' && (!cell || cell.contains(e))) right = Math.min(right, R(e).right);
            }
            const next = cell && cell.nextElementSibling;
            const spill = !!next && right > R(cell).right + 0.5 && right > R(next).left + 0.5;
            return { id: t.id, scrollWidth: t.scrollWidth, clientWidth: t.clientWidth, rows, lines,
              overflowX: getComputedStyle(t).overflowX, vbar, spill };
          });
          return { eqns };
        });
        await page.close();
      }
    } finally {
      await browser.close();
    }
  }
  console.log(JSON.stringify(out));
})().catch((e) => {
  console.error(String(e));
  process.exit(2);
});
