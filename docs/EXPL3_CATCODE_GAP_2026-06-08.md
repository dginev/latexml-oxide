# expl3 catcode-clobber gap — spath3/xparse `unexpected:_` (2026-06-08)

**Status: OPEN, deep. The single biggest Rust-only error gap on the
sampled corpus (2112.11932: Rust 1003 vs Perl 5, +998). Four band-aid
fixes were tried and ALL regress something; reverted. The real fix is a
kernel-level `\ExplSyntaxOff` completeness fix, not a catcode-restore
patch. Documented here so a future attempt does not repeat the dead ends.**

> MEASUREMENT WARNING that gated this whole investigation: the default
> `latexml_oxide --timeout` is **60 s**, and the DEBUG binary is ~10-20×
> slower than release. Heavy papers (2112.11932, 2110.10227, 2203.05327,
> 2110.12034 …) hit the 60 s watchdog mid-conversion and abort BEFORE the
> error-producing content, so a debug sweep reports a falsely-LOW error
> count (2112.11932 looked like 0). **Always measure expl3 / heavy /
> timeout papers with `cargo build --release` and `--timeout 150`.** This
> is why an earlier note claimed the fix was "redundant" — it was reading
> truncated debug numbers. (See [[feedback_canvas_measurement_isolation]],
> [[feedback_timeout_release_only]].)

## Symptom

`\usetikzlibrary{knots}` → `spath3.sty` → (spath3 does
`\ProvidesExplPackage`, so `_`/`:` = catcode LETTER) →
`\RequirePackage{xparse}`. After xparse loads, the REST of spath3's body
(`\cs_new:Nn …`, hundreds of lines) is parsed with `_` = SUBSCRIPT, so
every `_` in an expl3 name lands in text mode → **975 `Error:unexpected:_
Script _ can only appear in math mode`** (+ 28 misc). Witness 2112.11932
(release serial: 1003; Perl: 5).

## Root cause

`latexml_package/src/package/xparse_sty.rs` raw-loads xparse.sty then
**unconditionally hardcodes** the document regime:
```rust
state::assign_catcode(':', Catcode::OTHER, Some(Scope::Global));
state::assign_catcode('_', Catcode::SUB,   Some(Scope::Global));
```
The comment explains why: Rust's expl3 kernel `\ExplSyntaxOff` is partial
and "doesn't fully restore" catcodes, so xparse hardcodes them back. But
it hardcodes the *document's* regime — WRONG when the caller is itself an
expl3 package (spath3), which needs `_`/`:` = LETTER for its continuation.
In real LaTeX, xparse's own `\ExplSyntaxOff` restores the caller's
pre-`\ExplSyntaxOn` regime (LETTER for spath3), so no hardcode is needed.

## Why the four band-aids all fail

Verified in RELEASE serial on 4 papers. `off` = current/committed.
Baselines: 2112.11932 off=1003(Perl 5); 2110.10227 off=102(Perl 47);
2203.05327 off=78(Perl 102); 2110.12034 off=45(Perl 34).

| fix | 11932 | 10227 | 05327 | 12034 | verdict |
|-----|------:|------:|------:|------:|---------|
| force `_`+`:`→LETTER global, gated grandparent_in_expl3 | 223 | 26 | **483** | **84** | regresses 05327/12034 |
| input_definitions save/restore `_`+`:` (unconditional) | **1** | 26 | **459** | 8 | best for 11932 but `:` restore breaks expl3-code |
| input_definitions save/restore `_` ONLY | 1003 | 33 | 46 | 12 | SAFE on corpus, helps 3 (all now < Perl), but **breaks glossary_test catastrophically: output 108 lines → 1** |
| xparse_sty save/restore caller `:`/`_` | 223 | 26 | **483** | **84** | same as force-LETTER (05327's xparse caller is expl3) |

Two irreconcilable constraints:
1. **spath3 needs `:` = LETTER restored after xparse** — without it the
   `unexpected:_` cascade stays (the `_`-only variant leaves 11932 at 1003).
2. **`:` must NOT be re-asserted at a package boundary** — `:` is part of
   `\group_end:`; `expl3-code.tex` opens a `\group_begin:` (≈ line 33075)
   that is closed only AFTER its loader returns (cross-boundary group), so
   restoring a stale `:` mid-load mis-tokenizes the eventual `\group_end:`
   → dangling group → "Attempt to close boxing group" cascade (2203.05327
   78→459). And restoring `_` at every boundary breaks glossaries
   (expl3-based) outright.

So no per-package catcode-restore can satisfy both. The `_`-only variant
is *corpus*-safe and Perl-validated-better on 3 papers, but the
glossary_test in-tree regression (108→1 lines) blocks it.

## The real fix (future work)

Make Rust's expl3 `\ExplSyntaxOff` (and the `\@pushfilename`/`\@popfilename`
expl-status stack it leans on) FULLY restore the saved catcode regime, so
that xparse's own `\ExplSyntaxOff` correctly returns `_`/`:` to the
caller's pre-`\ExplSyntaxOn` values. Then:
- delete the hardcoded `:`→OTHER/`_`→SUB reset in xparse_sty.rs (lines 22-23),
- spath3 keeps LETTER (its `\ExplSyntaxOn` group is still open), document
  callers get OTHER/SUB,
- cross-boundary `\group_begin:`/`\group_end:` are untouched (no per-boundary
  catcode poking),
- glossaries unaffected.

This is a kernel/gullet change (expl3-code group + catcode stack
fidelity), not a loader patch. Until then, 2112.11932-class papers
(tikz knots / spath3 / any expl3 pkg that `\RequirePackage`s xparse
mid-body) carry the `unexpected:_` cascade.

## SECOND, RELATED root: the codepoint-data block dangles a group

There is a sibling expl3 gap that produces a different but co-occurring
cascade — `Error:unexpected:} Attempt to close boxing group; current
frame is non-boxing group due to T_CS[\group_begin:] at expl3-code; line
33075`. Root: expl3-code.tex L33074-33245 reads `UnicodeData.txt` /
`CaseFolding.txt` / `SpecialCasing.txt` / `GraphemeBreakProperty.txt` via
`\ior_open` + `\ior_map_variable`, EACH inside a `\group_begin:`…
`\group_end:`, then calls `\__codepoint_finalise_blocks:` (L33084) which
iterates the per-block intarrays the map built. Our raw expl3 load can't
finish the ~35k-line per-line `\__codepoint_data_auxi:w` processing, so
the tables are (partly) empty and **`\__codepoint_finalise_blocks:`
errors → the matching `\group_end:` (L33085) is never reached → the
`\group_begin:` at L33075 dangles** → a downstream `}` (e.g. in babel)
hits the boxing-group mismatch and an undefined-CS cascade. Witness
2204.05282 (Rust 86 / Perl 0 — babel), contributes to 2203.05327.

Two dead ends tried here (2026-06-08):
- **Skip the data files in `\openin`** (exact-name match on the 4 files →
  no mouth → `\ifeof`=true → map no-ops → reach `\group_end:`). Result:
  big SPEEDUP (2204.05282 became 1s — these were a major slowness/timeout
  source), but **error count UNCHANGED (86)** — `\__codepoint_finalise_blocks:`
  still errors on the empty tables, so the group still dangles. AND it
  **breaks glossary_test** (108-line output → 1): glossaries depends on
  the *partially*-populated codepoint tables, so forcing them fully empty
  regresses it.
- (the catcode band-aids above also break glossary_test.)

**Every expl3 Unicode/catcode patch tried so far breaks glossary_test.**
The codepoint module is load-bearing (case-folding + glossary sorting),
so it cannot simply be stubbed empty. The correct fix is to make the raw
expl3-code.tex codepoint load actually COMPLETE (the per-line
`\__codepoint_data_auxi:w` / `\ior_map_variable` path + the
`\__codepoint_finalise_blocks:` intarray finalisation), which would both
populate the tables (glossaries happy) and balance the groups (cascade
gone) — and, by skipping nothing, keep case-folding correct. That is the
same "complete the expl3 kernel load" effort as the `\ExplSyntaxOff` fix
above; they should be tackled together as one focused expl3-kernel pass.

## Repro

```
\documentclass{article}\usepackage{tikz}\usetikzlibrary{knots}
\begin{document}$x$\end{document}
```
Build release; `--timeout 150 --preload=ar5iv.sty
--path=~/git/ar5iv-bindings/bindings`. Probe `\the\catcode\`\_` after the
`\usetikzlibrary{knots}` line — Rust shows 8 (SUB) where it should be 11
(LETTER) for the rest of spath3.
