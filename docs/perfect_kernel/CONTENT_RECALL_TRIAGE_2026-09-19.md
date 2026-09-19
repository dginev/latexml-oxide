# Content-recall tail triage — 2026-09-19

Point-in-time triage of the S3 word-recall tail (sweep #104, `latexml_oxide.56dz`;
56ed/56ee added no content so it is current). Frozen snapshot — revalidate recall on
current `HEAD` before acting. Drives the **content-preservation axis** of the goal
([`../PERFECT_KERNEL.md`](../PERFECT_KERNEL.md) quality axes).

## Distribution (1850 docs, `perfect_kernel_s104_mono/s3_verdicts.tsv`)

| recall band | docs |
|---|---|
| ≥95% | 1388 |
| 90–95% | 170 |
| 80–90% | 124 |
| 60–80% | 84 |
| <60% | 54 |
| no golden (excluded, recall `-`) | 30 |

Content preservation is already strong corpus-wide (median 98.5). The `<60%` tail is
**mostly measurement artifacts, not real loss** — separating the two is the point of
this triage. All genuine losses here are **silent** (dropped body, never a truncating
`Fatal`).

## GENUINE content loss — fixable (ranked by absolute words missing)

Fix via raw interpretation, no new binding files (goal constraint).

| bundle/name | recall% | missing | mechanism | class |
|---|---|---|---|---|
| skeldoc/skeldoc | 13.2 | 249 | **AMBIGUOUS — deprioritized.** `\skelline{ +O{} +g }` body is `\skel_maybe:n { … \skel_line: }` (skeldoc.sty:297): it draws a skeleton LINE and the `+g` text arg is not typeset **by design** (skeldoc = "skeleton document"). But the golden renders real Lorem-Ipsum prose + verbatim command names, and our HTML keeps the lorem body while dropping other `\skel*`-arg prose. Genuine-loss vs faithful-placeholder needs word-level analysis on an obscure one-manual package — low leverage; verify pdflatex actually renders the missing arg text before treating as loss. | uncertain (verify oracle) |
| tuda-ci/DEMO-TUDaSciPoster | 8.1 | 147 | tcolorbox-poster (TUDaSciPoster) body blocks/columns dropped; 11 undefined-command warnings eat content | RUST-ONLY (raw class) |
| notebeamer/notebeamer-demo | 1.4 | 144 | beamer-derived frame bodies dropped, only the ruled-note grid survives | RUST-ONLY (raw class) |
| uantwerpendocs/uantwerpenexam-example1+2 | 31/35 | 88+140 | exam class cover-page/instructions frontmatter block not emitted (question bodies ARE present); one class fixes both | RUST-ONLY (raw class) |
| frankenstein/newclude | 1.1 | 1103 | `\DocInput{newclude.sty}` + frankenstein `\ProcessDTXFile` no-op; the whole ltxdoc-typeset `.sty` doc-comment body absent | likely SHARED — verify Perl first, deprioritize if Perl also empties |

## Semantic-markup finding (axis 2), FIXED this session

- **`{bibunit}` display math → text** (2605.02787): `$$…$$` inside `{bibunit}`
  (directly or via apxproof deferred appendix bodies) degraded to text mode →
  equation rendered as `<ltx:p>` text + `unexpected:_`. Fixed by
  `mode => "internal_vertical"` on the `{bibunit}` env (the `{center}`/`{figure}`
  precedent). Guard `cluster_package_guards::regress_2605_clusters::bibunit_keeps_display_math_semantic`.

## Measurement ARTIFACTS — do NOT re-chase (low recall ≠ loss)

- **Mismatched golden** (the `.tex` demo/template ≠ the `.pdf` package manual):
  geradwp (749), modular (146; subimport verified working), jacow A4/Letter (485×2;
  full sections+bib present, missing words not in the source).
- **Non-Latin encoding** (golden `pdftotext` garbles the script; XML has the text):
  montex/zanabazr (Mongolian translit), arabi/samplebook (Arabic, 4485 words in XML),
  greek-fontenc/test-tuenc-greek (7023 Greek chars), litetable zh-cn/zh-hk (566 CJK).
- **Visual / TikZ** (content is graphics, "missing" = figure/label text in images):
  bookcover, tkz-grapheur, chessboard-skakps, writeongrid, pgf-spectra, tikz-kalender.
- **Code-listing density** (listing identifiers counted as "missing" but present):
  dinbrief (84% source-word coverage), timeop, showexpl, pygmentex.
- **Embedded-PDF text** (`\includepdf`/`\includegraphics[page=]` external PDFs):
  newpax/doc-use-pax, doc-use-newpax.
- **Wrong language, not loss**: scrlttr2copy — `\blindtext` emits English under
  `ngerman` babel while the golden is German (a real blindtext/babel-language bug,
  content present).

## Method (reusable)

Content preservation is measured by S3 word-recall vs the golden PDF's distinct
words; `s3_sweep.sh` already excludes no-golden docs (`recall=-`). A low recall is
triaged genuine-loss vs artifact by: (a) is the missing word in the converted
source, (b) is the body text in the XML, (c) does the golden describe the SAME
document as the `.tex`. Error-count is a weak, partly-stale proxy — drive from this
tail and from semantic-markup gaps instead.
