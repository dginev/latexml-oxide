// Render an (X)HTML file in a real browser and report the geometry of every
// display-equation content cell (`td.ltx_eqn_cell.ltx_align_center`).
//
// Used by the Rust guard for issue #527: a display equation whose content is
// wrappable (`\text{}`-only ⇒ `ltx_markedasmath`, or mixed math+text ⇒ a
// `<math>` with `<mtext>`) must render on ONE line and must not clip. Without
// the `.ltx_eqn_cell` nowrap rule the pure-text cell STACKS (tall cellHeight)
// and the mixed cell CLIPS (scrollWidth > clientWidth) — this measures both.
//
// Usage:   node measure.js <file-url>
// Output:  JSON array, one object per cell: { cellHeight, clientWidth,
//          scrollWidth, overflow }.  Exit 2 on error (no browser, etc.).
const { chromium } = require('playwright-core');

(async () => {
  const url = process.argv[2];
  if (!url) {
    console.error('usage: node measure.js <file-url>');
    process.exit(2);
  }
  const browser = await chromium.launch({
    channel: 'chrome',
    headless: true,
    args: ['--no-sandbox', '--disable-gpu'],
  });
  try {
    const page = await browser.newPage({ viewport: { width: 800, height: 400 } });
    await page.goto(url, { waitUntil: 'networkidle' });
    const cells = await page.evaluate(() =>
      Array.from(document.querySelectorAll('td.ltx_eqn_cell.ltx_align_center')).map((cell) => {
        const r = cell.getBoundingClientRect();
        return {
          cellHeight: Math.round(r.height),
          clientWidth: cell.clientWidth,
          scrollWidth: cell.scrollWidth,
          overflow: cell.scrollWidth - cell.clientWidth,
        };
      }),
    );
    console.log(JSON.stringify(cells));
  } finally {
    await browser.close();
  }
})().catch((e) => {
  console.error(String(e));
  process.exit(2);
});
