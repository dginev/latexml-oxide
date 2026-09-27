# Perfect Kernel — progress ledger (living)

Protocol and status legend: [README.md](README.md). Phase 57 on; newest row last.
Per-document artifacts live under `~/data/perfect_kernel/<bundle>/<name>/`
(out of repo); the sweep tally file is `~/data/perfect_kernel/sweep_verdicts.tsv`.

## Oracle pass (same-TL ground truth)

2026-08-31, TL2025, `tools/perfect_kernel/oracle.sh` (pdflatex, lualatex
fallback/detection, 90s): **1548 / 2374 oracle-clean** (1248 pdflatex + 300
lualatex), 801 DOCUMENT-STALE (the shipped .tex no longer compiles on this
TL — e.g. a4wide.tex vs siunitx v3), 25 timeout. **The S1 bar applies to the
1548 oracle-clean docs only.**

## Phase 56 (closed 2026-09-27)

2026-08-31 → 2026-09-27: batches 4 → 56kx, sweeps #1 → #129. Clean documents (status 0-1) went from 947 / 2,374 at
sweep #5 to **1,933** at sweep #129 (`latexml_oxide.56kx5-rel`): errors 11,149, schema-valid 2,272, recall mean 95.76
(median 99.1, ≥95 % on 82.9 %), cpu_h 2.17. The per-sweep numbers since #113 are the scoreboard in
[`../PERFECT_KERNEL.md`](../PERFECT_KERNEL.md). Every phase-56 row — the fix log, the sweep history #1–#36, the S2/S3
baselines and the nicematrix exemplar (0 errors since 2026-09-04) — is in
[`archive/LEDGER_PHASE56_2026-09-27.md`](archive/LEDGER_PHASE56_2026-09-27.md), where a `LEDGER 56xx` reference resolves.

## Fix log

One row per landed kernel/engine fix attributable to this mission. Guard test
names are the durable part.

| Date | Fix | Cluster addressed | Guard test |
|---|---|---|---|
| 2026-09-27 | **batch 57a — phase 57 opens: the manual regression net (K17) lands; K13 designed** (user, 2026-09-27: phase 57 is the generalization pass, after sweep #129; ARCHITECTURE_THEMES 'Ordering recommendation'): `manual_net_select.py` picks a fixed set from a sweep by greedy coverage — first of what the corpus loads (bindings and raw files read as definitions), then of what it produces (XML element names and `class` values) — 1,000 manuals covering 4,078 of 4,230 load and 420 of 475 output features, 1,125 s serial (`manual_net.tsv`); `manual_net.sh` runs both binaries with their own dumps, validates both sides' XML and runs the repro catalog with both; `manual_net_compare.py [--recall]` fails toward flagging (missing manuals, incomplete repro runs, unscorable recall) and lists status, diagnostic, validity, byte, word, `tex=` and speed changes with a REGRESSIONS block. Review NO-SHIP → fixed: the comparer had passed with data missing, a reused outdir compared stale runs, relative dump paths fell back silently, and raw `.sty`/`.cls` loads were outside the selection. First run (56kw3 → 56kx5): tallies identical, byte diffs only timestamps, `\time`-seeded shuffles and heap addresses (now pinned or normalized); it caught `runaway_space_group_loop`'s count-less `% expect:` (fixed). Null run (A = B = 56kx5): 1,000 manuals and 968 repros, no change at all, 7 minutes at 24 jobs. The gate ladder's L2 names the net. K13 designed and recorded in KERNEL_CAPABILITIES (in-process before/after snapshots of a raw load against the binding, one chain walker, a `DeclaredMode` record as the one engine change; stages 0-3 with mutation tests). ARCHITECTURE_THEMES theme 6 records 56kv's class-options instance. Side finding: `\meaning` of a Rust closure macro prints a fat pointer's debug form (`CODE(Pointer { addr: …, metadata: DynMetadata(…) })`, definition.rs; Perl `CODE(0x…)`), for 57b. | — | `perfect_kernel_batch56::runaway_space_group_loop_is_still_a_loop` (header) |
| 2026-09-27 | **batch 57b — K13 stage 0: a definition keeps its declared mode; `\meaning` prints Perl's `CODE(0x…)`** (phase 57): (1) `DeclaredMode` (the mode after `text` is lowered, enter/leave horizontal, bounded, require/forbid math) is kept on `Primitive` and `Constructor` beside the digestion closures those options compile to, filled by `def_primitive`, `def_constructor`, `def_environment` and the DefMath constructors, which now build their prologue closures from it (so it cannot drift from digestion), read through `Definition::declared_mode()` — the one engine change the binding-conformance audit needs (KERNEL_CAPABILITIES K13); behavior-neutral. (2) `\meaning` of a macro coded in Rust printed the debug form of a `dyn` pointer (`CODE(Pointer { addr: …, metadata: DynMetadata(…) })`, into cnltx_en's text; side finding of 57a's review); the formatting sites (`\meaning`, the `Debug` and `Display` of `ExpansionBody`) cast to a thin pointer, giving Perl's `CODE(0x…)`. RESULTS: gates green (3487 tests, clippy, rustdoc, 0-error dumps); manual net 56kx5 → 57b byte-identical (1000 manuals, 970 repros, 0 regressions); witness cnltx_en re-converted: `CODE(Pointer { addr: …, metadata: DynMetadata(…) })` → `CODE(0x241d1bf6b70)`, errors 4 = 4. Review side finding (RED): a DefMath constructor in text mode converts silently where Perl warns (`requireMath` gated; SYNC side finding) | — | `perfect_kernel_batch56::{a_definition_keeps_its_declared_mode, meaning_of_a_closure_macro_is_perls_code_form}`; repros `expansion-primitives/meaning_of_a_closure_macro`, `math-parse/text_mode_defmath_constructor_is_reported` (RED) |
