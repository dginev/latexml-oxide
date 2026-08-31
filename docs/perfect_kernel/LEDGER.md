# Perfect Kernel — progress ledger (living)

Protocol and status legend: [README.md](README.md). Newest sweep first.
Per-document artifacts live under `~/data/perfect_kernel/<bundle>/<name>/`
(out of repo); the sweep tally file is `~/data/perfect_kernel/sweep_verdicts.tsv`.

## Oracle pass (same-TL ground truth)

2026-08-31, TL2025, `tools/perfect_kernel/oracle.sh` (pdflatex, lualatex
fallback/detection, 90s): **1548 / 2374 oracle-clean** (1248 pdflatex + 300
lualatex), 801 DOCUMENT-STALE (the shipped .tex no longer compiles on this
TL — e.g. a4wide.tex vs siunitx v3), 25 timeout. **The S1 bar applies to the
1548 oracle-clean docs only.**

## S2 schema-validation baseline (sweep-5 XMLs)

2026-08-31, `tools/perfect_kernel/validate.sh` (jing, urn-resolved schema):
**1,409 / 2,329 produced core XMLs are RelaxNG-valid (60.5%)**; 10,899
validation error lines across the remainder (`validate_verdicts.tsv`).

## Sweep history

| Date | Binary (commit, profile) | Corpus | Timeout | status 3/124/137 | status 2 | status 1 | status 0 | Notes |
|---|---|---|---|---|---|---|---|---|
| 2026-08-31 #1 | 1ef264a2bb release | 2374 | 120s | 40/5/0 | 1507 | 363 | 459 | **INVALIDATED** — run against the stale dual-TL dump (TL2023-vintage latex.ltx/expl3 labeled "2025"); kept for the before/after delta only |
| 2026-08-31 #2 | 9e347a3f15+161 release | 2374 | 120s | 34/10/0 | 1451 | 399 | 480 | fresh TL2025-pinned dump + OXIDIZED_DESIGN #161 DefPlain fix. Total error mass 145,375. **Oracle-clean slice (1548 docs): 424 clean + 310 warn-only (47%), 770 status-2, 8 fatal, 1 timeout; 58k error mass.** |
| 2026-08-31 #3 | +163/164/silence release | 2374 | 120s | 31/12/0 | 1449 | 400 | 482 | #163 makeindex, #164 raw-opt record, silence `\sl@StoreMessage` |
| 2026-08-31 #4 | 7ce3bb8005 release | 2374 | 120s | 28/16/0 | 1402 | 443 | 485 | + #165 `\@currsize`, beamer-hyperref parity, `\Hy@MakeCurrentHref`, KOMA minisec/labeling, memoir geometry, l3backend at begin-document. **Error-free 928 (39.1%), up from 822 baseline.** (Binary predates ltxdockit binding + `\@raw@classoptionslist`.) |
| 2026-08-31 #5 | b2ff296e4b release | 2374 | 120s | 38/7/0 | 1382 | 456 | 491 | + ltxdockit_cls.rs, `\@raw@classoptionslist`. **Error-free 947 (39.9%).** Fatal count noisy across sweeps (a concurrent test-suite run contended for CPU/RAM); timeouts likewise. Binary predates `\BreakableUnderscore` + `\glossary` schema guard (sweep 6 material). |

## Fix log

One row per landed kernel/engine fix attributable to this mission. Guard test
names are the durable part.

| Date | Fix | Cluster addressed | Guard test |
|---|---|---|---|
| 2026-08-31 | `tools/make_formats.sh` pins TEXMF* to the ambient kpsewhich's TL root (dual-TL schism poisoned the dump: "2025" dump carried expl3 2024-01-22) | expl3 props 124 docs, tagging sockets 74, l3sys 66+28, `\prop_new_linked` 36+14, `\IfPackageLoaded*`/`\IfFormatAtLeast*` — representatives collapsed 124→2, 74→1, 66→7, 36→6 | zero-`Error:` `--init=latex.ltx` gate; re-sweep delta |
| 2026-08-31 | OXIDIZED_DESIGN #161: `DefPlain` skips blanks before its required `{` (surpass-Perl, user-approved, branch-contained) | `Expected opening '{'` — `\lstnewenvironment` bodies on following lines (~148 docs incl. ltxdockit/cnltx families) | `cluster_package_guards::defplain_skips_blanks_before_brace` |
| 2026-08-31 | OXIDIZED_DESIGN #162: listings raw-line capture keeps the first body line when an argument probe crossed the newline (`gullet::pushback_is_empty`) | First body line silently lost in every `\lstnewenvironment[1][]`-style example env (content-loss, shared with Perl) | same guard (data-attr assertion) |
| 2026-08-31 | OXIDIZED_DESIGN #163: `\makeindex` allocates `\@indexfile` (kernel-contract subset) | 14 bundles (l3kernel's own manuals, robustindex, postnotes) — saveenv 2→1 err (Perl fatals) | `makeindex_allocates_indexfile` |
| 2026-08-31 | OXIDIZED_DESIGN #164: loader records `\@raw@opt@<name>.<ext>` + `\@raw@classoptionslist` (kernel L18521/L18718) | every `\ProcessKeyOptions` package dropped its options (10 bundles, codedescribe…); babel global language options loaded nil.ldf (4+ French bundles) — both fixed, `\og`→« » works | `process_key_options_sees_load_options`, `raw_classoptionslist_recorded` |
| 2026-08-31 | OXIDIZED_DESIGN #165: `\@currsize` defaults to `\normalsize` | linguistics family (5 bundles) — linguex-doc 6 err → **0 err 0 warn** | `currsize_default` |
| 2026-08-31 | beamer requires hyperref (Perl L1311 parity) + `\Hy@MakeCurrentHref` internal | beamertheme-* `\url` mass; l3doc anchor internal (8 bundles) | beamer golden (84_slides) |
| 2026-08-31 | l3backend loaded at begin-document (`\@expl@sys@load@backend@@`, latex.ltx L9472) | `\__color_backend_*` families undefined in every dump-mode conversion | prettytok witness; suite |
| 2026-08-31 | KOMA `\minisec`→`\paragraph*`, `{labeling}`→`{description}` (semantic); memoir geometry stubs (justified) | 17+11+4 bundles | scr*/memoir bindings |
| 2026-08-31 | silence.sty binding: `\sl@StoreMessage` internal | hep-* doc family (5+ bundles) | hep-acronym witness |
| 2026-08-31 | **ltxdockit_cls.rs** — NEW binding (user-directed lock-conflict resolution): semantic titlepage keyvals → frontmatter | 12-bundle biblatex/etoolbox manual family — abraces-doc 10 err → **0 err** | corpus witnesses |
| 2026-08-31 | rawclasses protocol guards (no code change — verified existing behavior) | binding precedence + bindingless-class raw load without OmniBus | `cluster_package_guards::rawclasses_binding_precedence_and_no_omnibus` (3 tests) |

## Named exemplar: nicematrix (the mission's reference bundle)

`nicematrix.tex` (6,954 lines, LuaLaTeX-authored manual; tikz, siunitx,
tcolorbox, enumitem, fancyvrb, titlesec, varwidth, adjustbox…).

| Date | Binary | status | errors | fatals | warnings | secs | Note |
|---|---|---|---|---|---|---|---|
| 2026-08-31 | 1ef264a2bb test | 2 | 102 (capped) | 0 | 79,166 | 8.5 | first baseline; dominant noise: `Unrecognized tabular template` from `{NiceTabular}`-family preambles hitting the alignment template reader (`alignment.rs:997`); error heads: `Extra alignment tab '&'` ×26, `\noalign cannot be used here` ×11, nested-sectioning schema errors |
| 2026-08-31 (end of session 1) | 42dee14566 test | 2 | **8** | 0 | **3** | 31.7 | after the dual-TL dump fix + session fixes the 79k-warning template noise and the `&`-cascades are GONE. Residual: nicematrix STUB binding limits (`\Block`, `{bNiceArray}` "no support … stub binding"), LuaTeX-only bits (`\automatichyphenmode`, luacode), own-dtx `\myfileversion`. Next lever = complete the nicematrix binding (bindings outrank raw by policy). |
