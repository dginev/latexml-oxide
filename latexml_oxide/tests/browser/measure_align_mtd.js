// Report the horizontal padding + rendered inter-column gap of native-MathML
// alignment cells — the `<mtd>` of an `mtable[columnspacing="0pt"]`, i.e. an
// `aligned`/`align`/`gather`/`split` nested inside another math environment,
// which LaTeXML renders as ONE `<math>` with a tight `<mtable>`.
//
// Issue #755: browsers add a default 0.4em horizontal padding to every `<mtd>`,
// which — on top of the deliberate `columnspacing="0pt"` — double-spaces the
// relation (`y(x) = …` rendered with ~3x the intended space). The LaTeXML.css
// reset must bring that padding to 0, leaving only the relation operator's own
// lspace/rspace as the gap. Matrix/array cells (bare `<mtd>`, non-zero
// `columnspacing`) are deliberately NOT matched by the reset and are not measured
// here.
//
// Usage:   node measure_align_mtd.js <file-url>
// Output:  JSON { cells: [{paddingLeft, paddingRight}, …], firstRelGap }, where
//          firstRelGap is the px distance from the right edge of the first row's
//          left column to the left edge of its relation operator (the visible
//          `y(x)`→`=` gap). Exit 2 on error (no browser, etc.).
const { chromium } = require('playwright-core');

(async () => {
  const url = process.argv[2];
  if (!url) {
    console.error('usage: node measure_align_mtd.js <file-url>');
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
    const result = await page.evaluate(() => {
      const cells = Array.from(
        document.querySelectorAll('math mtable[columnspacing="0pt"] mtd'),
      ).map((cell) => {
        const cs = getComputedStyle(cell);
        return { paddingLeft: cs.paddingLeft, paddingRight: cs.paddingRight };
      });
      // Behavioural cross-check: the first aligned row's left-column→relation gap.
      let firstRelGap = null;
      const table = document.querySelector('math mtable[columnspacing="0pt"]');
      if (table) {
        const row = table.querySelector('mtr');
        const tds = row ? row.querySelectorAll('mtd') : [];
        if (tds.length >= 2) {
          const rel = Array.from(tds[1].querySelectorAll('mo')).find((o) =>
            ['=', '≠', '≤', '≥', '<', '>'].includes(o.textContent),
          );
          if (rel) {
            firstRelGap =
              rel.getBoundingClientRect().left - tds[0].getBoundingClientRect().right;
          }
        }
      }
      return { cells, firstRelGap };
    });
    console.log(JSON.stringify(result));
  } finally {
    await browser.close();
  }
})().catch((e) => {
  console.error(String(e));
  process.exit(2);
});
