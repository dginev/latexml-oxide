// Render an HTML file at the given viewport widths and report the author block's mark geometry (FMRG, OD #1052):
// whether the block overflows horizontally, the shown boxes (names, marks, note bodies, contacts) drawn over one
// another, each shown mark's distance from its name and whether it sits on the name's line, and each shown note body
// printed with a creator: whether it sits under that creator's name and inside its column.
//
// Usage:   node measure_fmmarks.js <file-url> [width,...]
// Output:  JSON { "chrome/<width>": { overflow, overlaps: [[text, text]], marks: [{ name, mark, gap, onLine }],
//                                      bodies: [{ name, body, underName, inColumn }] } }. Exit 2 on error.
const pw = require('playwright-core');

(async () => {
  const url = process.argv[2];
  const widths = (process.argv[3] || '1280,375').split(',').map(Number);
  if (!url) {
    console.error('usage: node measure_fmmarks.js <file-url> [widths]');
    process.exit(2);
  }
  const browser = await pw.chromium.launch({ channel: 'chrome', headless: true, args: ['--no-sandbox', '--disable-gpu'] });
  const out = {};
  try {
    for (const w of widths) {
      const page = await browser.newPage({ viewport: { width: w, height: 900 } });
      await page.goto(url, { waitUntil: 'load' });
      out[`chrome/${w}`] = await page.evaluate(() => {
        const R = (e) => e.getBoundingClientRect();
        const shown = (e) => {
          if (!e) return false;
          for (let n = e; n; n = n.parentElement) {
            const s = getComputedStyle(n);
            if (s.display === 'none' || s.visibility !== 'visible' || Number(s.opacity) === 0) return false;
          }
          const r = R(e);
          return r.width > 0 && r.height > 0;
        };
        const block = document.querySelector('.ltx_authors');
        const text = (e) => e.innerText.replace(/\s+/g, ' ').trim();
        const overflow = block.scrollWidth > block.clientWidth + 1 || document.documentElement.scrollWidth > window.innerWidth + 1;
        const marks = [];
        const bodies = [];
        const boxes = [];
        for (const c of block.querySelectorAll('.ltx_creator')) {
          const nameEl = c.querySelector('.ltx_personname');
          const name = text(nameEl);
          // the name's last line box: its glyphs' rects, the last one
          const range = document.createRange();
          range.selectNodeContents(nameEl);
          const rects = Array.from(range.getClientRects()).filter((r) => r.width > 0);
          const last = rects[rects.length - 1] || R(nameEl);
          boxes.push([name, R(nameEl)]);
          let prevRight = last.right;
          for (const note of c.querySelectorAll('.ltx_note')) {
            if (note.parentElement.closest('.ltx_note')) continue;
            const mark = Array.from(note.children).find((k) => k.classList.contains('ltx_note_mark'));
            if (shown(mark)) {
              const m = R(mark);
              marks.push({ name, mark: text(mark), gap: Math.round((m.left - prevRight) * 10) / 10,
                onLine: m.top < last.bottom && m.bottom > last.top });
              prevRight = m.right;
            }
            const content = note.querySelector('.ltx_note_content');
            if (shown(content)) {
              const b = R(content);
              const col = R(c);
              boxes.push([text(content).slice(0, 40), b]);
              bodies.push({ name, body: text(content).slice(0, 40), underName: b.top >= last.bottom - 0.5,
                inColumn: b.left >= col.left - 1 && b.right <= col.right + 1 });
            }
          }
          for (const k of c.querySelectorAll('.ltx_contact')) if (shown(k)) boxes.push([text(k), R(k)]);
        }
        const overlaps = [];
        for (let a = 0; a < boxes.length; a++) {
          for (let b = a + 1; b < boxes.length; b++) {
            const [[ta, A], [tb, B]] = [boxes[a], boxes[b]];
            if (A.left < B.right - 1 && B.left < A.right - 1 && A.top < B.bottom - 1 && B.top < A.bottom - 1) {
              overlaps.push([ta, tb]);
            }
          }
        }
        return { overflow, overlaps, marks, bodies };
      });
      await page.close();
    }
  } finally {
    await browser.close();
  }
  console.log(JSON.stringify(out));
})().catch((e) => {
  console.error(String(e));
  process.exit(2);
});
