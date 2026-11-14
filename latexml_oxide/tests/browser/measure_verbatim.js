// Render an HTML file at a phone width (375 px) and report the fancyvrb/fvextra
// verbatim geometry the issue #431/#702 guard asserts: the page's scroll width,
// each frame=single box's scroll vs client width, the one-row `short` line's
// height against its font size, and the client rects of each inline \verb.
//
// Usage:   node measure_verbatim.js <file-url>
// Output:  JSON { vw, docScrollWidth, frames: [{scrollWidth, clientWidth}],
//          shortLine: {height, fontSize}, codeRects: [n] }.  Exit 2 on error.
const { chromium } = require('playwright-core');

(async () => {
  const url = process.argv[2];
  if (!url) {
    console.error('usage: node measure_verbatim.js <file-url>');
    process.exit(2);
  }
  const browser = await chromium.launch({
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
        codeRects: Array.from(document.querySelectorAll('code.ltx_verbatim')).map((c) => c.getClientRects().length),
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
