# Perfect kernel — the 2026-10 close, the in-scope scoreboard s130-s138 and the off-the-path leads (archived 2026-11-13)

Moved verbatim from `docs/PERFECT_KERNEL.md` at the 2026-11 close (bundle int171a, DOCS1): the 2026-10-04 close status and its
"Still to close" list (every item since superseded: run 336 replaced run 329, the 2609 study and run 336's worse set folded
into `SYNC_STATUS.md` row R's G/G1 lines, the performance pass done as 62zu, sweep #149's four open bars fixed in 65a and
row 0 met at #152), the in-scope scoreboard table s130-s138, and the "Off the path" list (unverified since 2026-09-27).
The live close status (2026-11), the goal bar and the state table stay in `PERFECT_KERNEL.md`. Relative links re-pointed
to this directory; the text is otherwise as it stood.

## Close status (2026-10-04)

**Close status (2026-10-04).** Sweeps #146, #147 and #148 (61p6, 61q2, 61t2) meet G1-G5 on the goal set (0 / 0 / 0 /
0, the same 68 crash canaries), and every batch's arXiv A/B was neutral or explained. The ranked path is done: the bar
measured (sweep #135), philexmanual's anchor (59m), tabularray's outer keys (59n), TikZ speed (59q; G4 met since
s136), title pages and letters (59o, 59p), arabi (59r) and the rulings 7a-7e. The cortex reruns of sandboxes
2605/2606 (L6, runs 325-328, 2026-10-04) were followed by 61r's regression fixes. The user confirmed the goal bar
(2026-10-04); the ar5iv-css port is dginev/ar5iv-css#54 (branch `css/perfect-kernel-port`, 16335fe). Still to close:
- stream G: the full arXiv rerun, cortex run 329 (2,908,567 papers), stopped after 16,333 papers for 134
  regressions, fixed by 62a-62c and resumed 2026-10-04 17:46Z on container `cortex-worker-62c`; 61v's 1,780
  Error/Fatal results were re-queued for 62c. Compare it to run 306 only when complete. Later fixes found by mining
  the run (62d-62f) are on `perfect_kernel`, not deployed to the fleet. Open run-329 work: `SYNC_STATUS.md`
  ("Run-329 and sandbox open residuals"). Run 329 finishes on 62c (user 2026-10-04: no mid-run swap).
- the 2609 Error/Fatal study (run 329 paused 2026-10-05 02:29Z at 263,636 papers, user): batches 62s-62zt, each
  A/B-neutral or better with content checks (words, jing, PDF recall); the 849 Error papers' recheck went 8,176 errors
  / 86 clean (rc62za6) → 6,443 / 200 (rc62zt, c4dd3bdd9b). Rulings 2026-10-06: `\section` in a float lands as an
  inline sectional block with its float's id fixed first (62zs, after a demo), `\cprime` within the bibliography
  (62zr), `\foreach` over a locked macro keeps Perl's lock (already bounded), eqnarray `{##}` accepted. The residual
  tail is long and flat (largest cluster 23 papers; math-mode `_`/`^`/`$`, `\endgroup` interleavings — Perl-origin
  — and singles), recorded in `SYNC_STATUS.md`.
- the performance pass (user 2026-10-07): math parsing is 39 % of a random 2609 sample's instructions; byte classes
  as single Marpa terminals (62zu `c0e4393f39`, marpa-asf 0.4.0) cut 11.4 % over 440 random papers, 439
  byte-identical (`performance/PERFORMANCE.md` P3).
- stream G: run 336, the full arXiv rerun (2,947,191 papers), completed 2026-10-10: against run 306 error 12.86 % →
  6.32 %, fatal 0.87 % → 0.27 %; the 16,827 papers worse into error/fatal are clustered, root-caused and in fix batches
  64h-64p (`SYNC_STATUS.md` row G). The host's AppArmor `gs` profile had blocked the bare-host fleet's TMPDIR since
  2026-10-05 (every EPS/PS figure failed in runs 332-335); fixed 2026-10-07.
- sweep #149 (e57be39640, 2026-10-11, after 50+ batches incl. the M1/D1 kernel changes), compared with `ab_audit` to
  #148: errors 11,511 → 11,364, fatals 66 → 61, in-scope valid 1,580 → 1,585, missing words 13,739 → 13,715. Of 322
  goal-set regression rows, 318 are benign or improvements (UTC `\today`, frontmatter cleanup, TOC titles,
  bibliography numbering). The four open bars: G1 chessboard_and_beamer is an improvement (Fatal → status 2), its
  `accepted_residuals.tsv` row now `errors 4` (65a); G2 heria-proposal nests a captioned panel figure in a rotated
  float (62q);
  G4 pgf-interference-de/en timed out on SMT siblings of busy cores (`instructions:u` flat across the bisect, no
  regression). Real losses besides heria-proposal: recipebook's per-recipe `\author`/`\title` after the body and a
  bold font leak, and ascelike's "M.ASCE" split off as a creator. All are batch 65a. Triage:
  `~/data/pk_agents/main/agent_reports/2026-10-11_sweep149_triage.md`.
The sweep narratives #135-148, the ranked path steps 1-8 and the stream-G log are archived in
[`perfect_kernel/archive/PERFECT_KERNEL_PHASE59_62_CLOSE_2026-10-04.md`](PERFECT_KERNEL_PHASE59_62_CLOSE_2026-10-04.md).

## Scoreboard (in scope, s130-s138)

In scope (`tools/perfect_kernel/scoreboard.py 130`; clean = status 0-1; cpu_h = the sum of
per-document conversion seconds; recall from the inline-glue audit walker since s125; the corpus-wide rows s113-s130
are in the archive):

| sweep | head | docs | clean | fatal | timeout | errors | valid | scored | recall mean | median | %≥95 | missing | cpu_h | p90 s | p99 s | >60 s | >120 s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 130 | 57ak5 | 1602 | 1585 | 2 | 3 | 388 | 1569 | 1564 | 96.18 | 99.4 | 84.8 | 14929 | 0.92 | 2.5 | 26.7 | 11 | 4 |
| 131 | 57ck3 | 1602 | 1585 | 2 | 3 | 388 | 1569 | 1564 | 96.18 | 99.4 | 84.8 | 14928 | 0.99 | 2.8 | 27.1 | 11 | 4 |
| 132 | m58i | 1602 | 1585 | 2 | 3 | 350 | 1574 | 1564 | 96.16 | 99.4 | 84.8 | 14714 | 0.93 | 2.7 | 26.5 | 11 | 4 |
| 133 | 58q5 | 1602 | 1585 | 2 | 3 | 350 | 1576 | 1564 | 96.23 | 99.4 | 85.0 | 14435 | 0.94 | 2.7 | 26.9 | 11 | 4 |
| 134 | 59c1 | 1602 | 1586 | 2 | 1 | 351 | 1578 | 1565 | 96.23 | 99.4 | 85.0 | 14437 | 0.87 | 2.4 | 24.1 | 10 | 2 |
| 135 | 59l2 | 1602 | 1590 | 2 | 1 | 160 | 1578 | 1569 | 96.24 | 99.4 | 85.0 | 14667 | 0.86 | 2.5 | 24.1 | 10 | 2 |
| 136 | 60b4 | 1602 | 1588 | 2 | 0 | 202 | 1579 | 1567 | 96.58 | 99.4 | 86.0 | 14332 | 0.85 | 2.4 | 23.4 | 10 | 2 |
| 137 | 60f1 | 1602 | 1589 | 2 | 2 | 163 | 1577 | 1568 | 96.59 | 99.4 | 86.1 | 14320 | 0.92 | 2.7 | 24.7 | 10 | 2 |
| 138 | 60j4 | 1602 | 1591 | 2 | 0 | 163 | 1579 | 1570 | 96.64 | 99.4 | 86.4 | 14249 | 0.85 | 2.5 | 23.5 | 10 | 2 |

Crash canaries (772): 64 Fatal and 3 timeouts on every sweep since s130; cpu_h 1.18-1.32. Sweeps #139-148 are in the state table above and
`scoreboard.py`.

## Off the path (unverified since 2026-09-27)

**Off the path** (not re-verified since 2026-09-27 — probe each before taking it; the phase-56 leads and the healthy-subset projection are in
[`perfect_kernel/archive/PERFECT_KERNEL_PHASE56_NOTES_2026-09-27.md`](PERFECT_KERNEL_PHASE56_NOTES_2026-09-27.md)):
- **Kernel, arXiv 2605:** polytable needs array.sty's `\@mkpream`/`\@classz` builder (2605.08990); 2605.00058 ends
  `PushbackLimit` in 1 s and 2605.12601 at the 180 s timeout (resolved by 62w's 48M cap: 2605.27177, 621 raw-mhchem
  `\ce`, with 2 warnings, and 2605.04377 with 102); the cycle guard's false positive on a large repetitive table (2605.11798); the autoload hoist as a load at
  group level 0 (DIVERGENCES #282).
- **Engine:** pgffor's `\foreach \x/\tag` against the lock on amsmath's `\tag` (2605.31475, LEDGER 56hk) — RULED
  2026-10-06: keep Perl's lock; Rust now ends boundedly (the runaway cap's `Fatal:TooManyErrors` in 0.55 s, a
  partial document salvaged), and the simple shape `\foreach \x/\tag in {a/b}{[\x:\tag]}` gives Perl's output
  exactly ("[a:"); catoptions' residual option-stack-limit error.
- **Semantic coverage (axis 2b, `semantic_coverage.py`; s114: sections/lists/floats/refs 93-98 %, equations 93 %,
  graphics 88 %, `\part` 59.5 %):** the graphics family (36 documents short); ctex's localized part label reads
  "Part I"; beamer overlays are not acted on (Perl's `ltx_covered` wrapper, #270), and frames are not Perl's
  `ltx:slide` in `ltx:slidesequence` model; a `\caption` in a non-float box becomes its float, but the box's other
  content stays beside it.
- **Class census** (`~/data/pk_agents/w67/clscensus7/census7.tsv` on `vendor_dumps56id`: 496 / 500 usable classes
  clean): novel (12 errors: its images' `_` in text, SHARED; two layout `\the` errors; fontspec's main-font check),
  imsproc (1, NFSS `.fd` loading), udesoftec (1, `\abstract` constructor), upmethodology-document (2, SHARED); not
  usable: tikzposter (parked). Re-run with `clscensus7/run.sh <class> <binary>`, whose dump directory must match.
- **Performance beyond the ceiling:** lie-hasse and wheelchart exceed the budget in their own engines too (277 s,
  > 900 s); the TikZ manuals run at 2.5-3× lualatex (pgf-interference 166 s vs 68 s); the remaining lever class is
  native l3 int/tl primitives (PLANS 12) and the token-list representation.
- **SHARED with Perl, not pursued:** the arXiv 2605 Fatal remainder (els-mrw bibitems, class-body macros under
  OmniBus, enumitem `label=\theenumi` self-reference, `\theequation` in `\tag*`, the mode-frame family, missing
  packages); tufte's citation `\marginpar`s left empty (LEDGER 56ip); the silent-loss structural scan (empty `<p/>`,
  icon inline-blocks, struts) is settled.
