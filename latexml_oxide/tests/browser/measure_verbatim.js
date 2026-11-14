// Render an HTML file at a phone width (375 px) and report the fancyvrb/fvextra
// verbatim geometry the issue #431/#702 guard asserts: the page's scroll width
// with its tables hidden (a table of unwrapped cells may widen the page, as in
// TeX), each frame=single box's scroll vs client width, the one-row `short` line's
// height against its font size, the client rects of each inline \verb outside and
// inside a table cell, the rendered text of the verbatim line holding `return`, and
// (issue 853) the `plain line` of a plain Verbatim beside the `margin line` of an
// xleftmargin one: each line's row height and left edge, the width of the margin's
// em-space text before the margin line.
//
// Usage:   node measure_verbatim.js <file-url>
//          LATEXML_BROWSER_ENGINE=firefox LATEXML_FIREFOX_PATH=<playwright firefox> renders in Firefox.
// Output:  JSON { vw, docScrollWidth, frames: [{scrollWidth, clientWidth}],
//          shortLine: {height, fontSize}, codeRects: [n], cellCodeRects: [n],
//          returnLine, plainLine: {height, left}, marginLine: {height, left, marginWidth} }.
//          Exit 2 on error.
const { chromium, firefox } = require('playwright-core');

(async () => {
  const url = process.argv[2];
  if (!url) {
    console.error('usage: node measure_verbatim.js <file-url>');
    process.exit(2);
  }
  const browser =
    process.env.LATEXML_BROWSER_ENGINE === 'firefox'
      ? await firefox.launch({ executablePath: process.env.LATEXML_FIREFOX_PATH, headless: true })
      : await chromium.launch({
          channel: 'chrome',
          headless: true,
          args: ['--no-sandbox', '--disable-gpu'],
        });
  try {
    const page = await browser.newPage({ viewport: { width: 375, height: 800 } });
    await page.goto(url, { waitUntil: 'networkidle' });
    const out = await page.evaluate(() => {
      const lines = Array.from(document.querySelectorAll('.ltx_text.ltx_verbatim.ltx_inline-block'));
      const short = lines.find((e) => e.textContent.trim() === 'short');
      const ret = lines.find((e) => e.textContent.includes('return'));
      const box = (text) => {
        const e = lines.find((l) => l.textContent.trim() === text);
        if (!e) return null;
        // A line's row: the xleftmargin wrapper around it, if any (its em spaces may take a row of their own).
        const row = e.parentNode.classList.contains('ltx_inline-block') ? e.parentNode : e;
        return { e, height: row.getBoundingClientRect().height, left: e.getBoundingClientRect().left };
      };
      const plain = box('plain line');
      const margin = box('margin line');
      let marginWidth = null;
      if (margin) {
        // The margin is the text before the line inside its wrapper (U+2003 em spaces).
        const range = document.createRange();
        range.setStart(margin.e.parentNode, 0);
        range.setEndBefore(margin.e);
        marginWidth = range.getBoundingClientRect().width;
      }
      const rects = (sel) => Array.from(document.querySelectorAll(sel)).map((c) => c.getClientRects().length);
      const cellCodeRects = rects('td code.ltx_verbatim');
      const codeRects = rects(':not(td) > code.ltx_verbatim');
      document.querySelectorAll('.ltx_tabular').forEach((t) => { t.style.display = 'none'; });
      return {
        vw: window.innerWidth,
        docScrollWidth: document.documentElement.scrollWidth,
        frames: Array.from(document.querySelectorAll('.ltx_framed_verbatim')).map((f) => ({
          scrollWidth: f.scrollWidth,
          clientWidth: f.clientWidth,
        })),
        shortLine: short && {
          height: short.getBoundingClientRect().height,
          fontSize: parseFloat(getComputedStyle(short).fontSize),
        },
        codeRects,
        cellCodeRects,
        returnLine: ret && ret.innerText,
        plainLine: plain && { height: plain.height, left: plain.left },
        marginLine: margin && { height: margin.height, left: margin.left, marginWidth },
      };
    });
    console.log(JSON.stringify(out));
  } finally {
    await browser.close();
  }
})().catch((e) => {
  console.error(String(e));
  process.exit(2);
});
