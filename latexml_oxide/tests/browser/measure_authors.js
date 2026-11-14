// Render an HTML file at the given viewport widths and report, per author, the issue #783 geometry: the starred
// pop-up marker of its contacts, each note's mark and content shown, and each contact's visibility and whether it
// sits below the name.
//
// Usage:   node measure_authors.js <file-url> [width,...]
//          (Chrome; with LATEXML_FIREFOX=<firefox executable> Firefox as well)
// Output:  JSON { "<engine>/<width>": { creators: [{ name, star, notes: [{ mark, content }], contacts: [{ text, visible, belowName }] }] } }.
//          Exit 2 on error.
const pw = require('playwright-core');

(async () => {
  const url = process.argv[2];
  const widths = (process.argv[3] || '1280,375').split(',').map(Number);
  if (!url) {
    console.error('usage: node measure_authors.js <file-url> [widths]');
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
        out[`${name}/${w}`] = await page.evaluate(() => {
          const R = (e) => e.getBoundingClientRect();
          const shown = (e) => {
            for (let n = e; n; n = n.parentElement) {
              const s = getComputedStyle(n);
              if (s.display === 'none' || s.visibility !== 'visible' || Number(s.opacity) === 0) return false;
            }
            return true;
          };
          const creators = Array.from(document.querySelectorAll('.ltx_creator')).map((c) => {
            const notes = c.querySelector('.ltx_author_notes');
            const nameBox = R(c.querySelector('.ltx_personname'));
            const star = notes ? getComputedStyle(notes, '::before').content : 'none';
            return {
              name: c.querySelector('.ltx_personname').innerText.trim(),
              star: star === 'none' || star === '""' || star === 'normal' ? 'none' : star,
              notes: Array.from(c.querySelectorAll('.ltx_note')).map((n) => ({
                mark: shown(n.querySelector('.ltx_note_mark')),
                content: shown(n.querySelector('.ltx_note_content')),
              })),
              contacts: Array.from(c.querySelectorAll('.ltx_contact')).map((k) => ({
                text: k.innerText.trim(),
                visible: shown(k),
                belowName: R(k).top >= nameBox.bottom - 0.5,
              })),
            };
          });
          return { creators };
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
