# Perfect Kernel — improvement-plans ledger (living)

Detailed, execution-ready plans distilled from root-cause investigations
(subagent fan-out 2026-09-01, user directive: up to 8 read-only agents,
plans recorded here, fixes executed from the main session as each plan
finalizes). A plan graduates DRAFT → FINAL when its root cause carries
file:line evidence and the fix shape has a named risk assessment; FINAL
plans are executed in batch order and the row moves to DONE with the
batch number. Keep the conclusion, not the play-by-play
(CLAUDE.md doc rules).

| # | Target (mass) | Status | Plan summary |
|---|---|---|---|
| P1 | aomart display-math break (aomsample ×2, ~80 errs) | INVESTIGATING | `\[` fails to enter display math only in the full doc (bare-class repro clean); bisect + class-source root-cause pending. |
| P2 | biblatex + droit-fr `expected:expandafter` cascade (197 errs, GENUINE-RUST-ONLY — Perl: 4 errs) | INVESTIGATING | Cascade roots in csquotes "No space factor codes for 'ASCII' encoding"; synthetic csquotes repros clean; doc-bisect pending. |
| P3 | xint `\XINT_div_start_c_<roman…>` infinite csname loop (tkz-grapheur ×4, xint-regression; survives depth 200k) | INVESTIGATING | Roman-numeral-accreting csname loop = a termination conditional we mis-evaluate; xint-source + tex.web comparison pending. |
| P4 | packdoc element rendering (algxpar-doc 315, numerica ~100) | INVESTIGATING | Per-`\OptionInd` mode error + orphaned `ltx:indexphrase`; verdict needed: parked mode-frame family vs independent fix. |
| P5 | Session-diff refactor pass (batches 24-32) | INVESTIGATING | DRY (duplicated VerbatimOut capture, beamer theme-load loops), idiom, perf (pack_parameters meaning-lookup, vsplit cloning), comment hygiene. |
| P6 | elsdoc regression 0→14 (sweep 22) | INVESTIGATING | Verbatim element never closes; suspects narrowed to batch-28 vsplit-voiding vs batch-29 verbatiminput changes (batch 30 + pagegoal A/B-cleared). |
| P7 | dijkstra-fr tabular-template macro expansion (7 errs) | INVESTIGATING | `\dijk_last_col_type` unexpanded in column spec; real `\@mkpream` fully expands preambles — our template reader doesn't; Perl-share check pending. |
| P8 | Post-undefined "window of 2 boxes" digestion loops (willowtreebook, fixdif, zx-calculus, knowledge, robust-externalize) | INVESTIGATING | One error-recovery mechanism suspected: error-stubbed CS breaks an `\ifx`-sentinel loop's termination; engine-level fix sought. |

## Standing execution queue (main session)

1. Execute FINAL plans in ascending risk order; batch 3-5 per suite run
   (feedback_batch_fixes_parallel_rootcause).
2. Every executed plan: guard test or witness re-run + LEDGER batch row.
3. Sweep after each 2-3 batches banks the corpus effect.

## DONE

(moves here with batch number)
