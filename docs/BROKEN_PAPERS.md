# Broken Sandbox Papers — Round-18 Real Regressions

Snapshot: **2026-05-01**, post-roster (`050a32b1b`) and post-Perl-cap-fix (`f5e8314ff`).

These are the **only confirmed real Rust regressions** remaining in the
sandbox. All other papers in the original failure set are either fixed,
out-of-scope (Perl also fails), or undetermined (Perl-capped). Random
canvas sampling (1029 papers) confirms zero new real regressions.

## In-scope worksheet (Rust > Perl, Perl NOT capped)

### ~~1. Cluster H — `\@personname`/`\@add@frontmatter@now` `}` follow-on~~ — FIXED
**Papers:** `physics0002038` was R=5 vs P=4, `cond-mat0011517` was R=7 vs P=6.
**FIXED** in commit `7319e3fbc` (`base_utilities.rs`): dropped the
spurious `before_digest=>bgroup, after_digest=>egroup` wrapping
around `\@add@frontmatter@now`. Perl's `bounded => 1` flag on Primitives
is unused by the dispatcher (Primitive.pm doesn't push/pop a frame for
it). Rust's group-wrap was the extra `}` that Perl never emitted.
Both now match Perl exactly: OUT-OF-SCOPE (P=R, both have content errors).
[Original cluster description below for reference.]
**Delta:** +1 each (cosmetic).
**Trigger:** revtex/aas papers where `\@personname` end-mode fails
(Constructor `mode=>'restricted_horizontal'` in `\author{...}` body
that's actually in `internal_vertical`). After the end-mode error,
Rust emits an extra `Error:unexpected:} Attempt to close a group
that switched to mode internal_vertical`. Perl absorbs this `}`
silently between the end-mode error and the next CS-trigger error.
**Fix locus:** `latexml_core/src/stomach.rs` `egroup`/`endgroup` —
the Perl Stomach.pm code is functionally identical at this level
(verified 2026-05-01). The divergence must be in either:
(a) what frame structure `\@personname` constructor leaves behind
when end_mode_opt fails (Rust may push frame differently than Perl),
or (b) the `}` reaches Rust's egroup but Perl's egroup short-circuits
through a different code path. Deferred — +1 cosmetic only.

### ~~2. Cluster A — `\@@numbered@section args[0] = "section\par"`~~ — FIXED
**Paper:** `math0010095` was R=11 vs P=0. **FIXED** in commit
`4d445b71c` (`latex_constructs.rs`): `\@@numbered@section` and
`\@@unnumbered@section` now strip a trailing `\par` /
`\@startsection@hook` / `\relax` token from `args[0]` (the section-type
identifier) before propagating it to `ref_step_counter`,
schema-tag selection, and the `\lx@format@title@@` invocation.
Reverted token list is also rebuilt from the sanitized string.
The trigger is a `{}` parameter reader picking up a trailing
`\par` when an upstream BoxedEPS-style figure-block precedes the
section. Now Rust=Perl=0.

### ~~3. emulateapj `\fig{...}` opens figure inside footnote~~ — FIXED
**Paper:** `astro-ph0503342` was R=33 vs P=0. **FIXED** by porting
Perl's smart `\fig Semiverbatim Token` peek dispatch in
`aas_support_sty.rs:149-168` (commit pending). Now Rust=Perl=0.
Perl peeks the token after the first arg: if `{` follows it's
the 3-arg figure-with-caption form; otherwise treat as a single-arg
`\ref` shorthand. Rust's prior thin `\fig Semiverbatim → \aas@fig{#1}`
always opened a figure, materializing `<ltx:figure>` inside
`<ltx:note>` (schema violation) when papers used `\fig{label}`
inside `\footnote{...}` or `\caption{...}`.

### 4. mn2e `\hbox` mode-mismatch tabular cascade — NEW 2026-05-01
**Paper:** `0903.4199` (R=10001 cap vs P=0).
**Trigger:** `\documentclass{mn2e}`. First errors are 2x
`\hbox Attempt to end mode 'restricted_horizontal' in 'restricted_horizontal'`
followed by `\lx@begin@alignment` mode-switch error and a tabular
cascade of 9989 `Error:unexpected:&` (alignment marks fired in
broken state). The `&`-cascade hits the 10001 default cap.
**Fix locus:** mn2e's `\hbox` usage in tabular machinery — likely
similar family to 1112.6246 (FIXED via mn2e_support drop). Need
to investigate which `\hbox{...}` in mn2e cls is hitting end_mode
mismatch in restricted_horizontal context.
**Witness:** discovered in random 2000-paper sample 2026-05-01.

### 5. \thefootnote\par / \ext@footnote\par — Cluster A reprise
**Paper:** `hep-ph0204075` (R=2 vs P=0).
**Trigger:** Same `{}` parameter reader trailing-`\par` contamination
as math0010095, but in footnote machinery. Errors:
`\thefootnote\par` undefined + `\ext@footnote\par` undefined.
The Cluster A symptom-fix in `\@@numbered@section` (commit
`4d445b71c`) doesn't cover this path — `\refstepcounter{footnote}`
or similar code goes through a different csname-build chain.
**Fix locus:** Either (a) extend `strip_trailing_cs` to all
`ref_step_counter` ctype inputs (in `latexml_core/src/binding/counter/dialect.rs`),
or (b) fix the underlying parameter-reader bug.

### 3. Math state cumulative — `\hbox` runaway
**Paper:** `hep-th0005268` (R=10001 cap vs P=26 Perl uncapped).
**Trigger:** display math `\be ... \ee` block at line 737-739
(after preceding `\be ... \ee` at 733-735). First localized error
is `Error:unexpected:\lx@end@inline@math Attempt to end mode 'math'
in 'math'` at line 732 col 55, then `'math' in 'display_math'` at
line 734 col 12. After line 738, cap-hitting cascade of
9971 × `Error:unexpected:\hbox`.
**Fix locus:** unidentified. Math-mode tracking accumulation —
likely the same cluster as the math0205073-family that the roster
fix masked (different unrelated trigger).
**Bisection witness:** lines 1-735 = 4 errors; lines 1-738 = 10001 cap.

## Re-running these

```
tools/parity_check.sh physics0002038 cond-mat0011517 math0010095 hep-th0005268
```

Should always show:
- `physics0002038`: REAL REGRESSION (P=4 vs R=5)
- `cond-mat0011517`: REAL REGRESSION (P=6 vs R=7)
- `math0010095`: BOTH CLEAN (FIXED `4d445b71c`)
- `hep-th0005268`: REAL REGRESSION (P=26 vs R=10001)

If any drop to OUT-OF-SCOPE or BOTH CLEAN, the corresponding fix has
landed; remove from this list and credit the commit.

## Out-of-scope and Perl-capped (not regressions)

See `docs/out-of-scope/` for papers where Perl also fails. See
SYNC_STATUS.md round-18 sweep summary for the cap-uncertain
classifications (hep-th0010165, hep-ph0007044, astro-ph0204393,
hep-ph0001306).
