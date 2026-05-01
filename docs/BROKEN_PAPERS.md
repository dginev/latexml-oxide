# Broken Sandbox Papers — Round-18 Real Regressions

Snapshot: **2026-05-01**, post-roster (`050a32b1b`) and post-Perl-cap-fix (`f5e8314ff`).

These are the **only confirmed real Rust regressions** remaining in the
sandbox. All other papers in the original failure set are either fixed,
out-of-scope (Perl also fails), or undetermined (Perl-capped). Random
canvas sampling (1029 papers) confirms zero new real regressions.

## In-scope worksheet (Rust > Perl, Perl NOT capped)

### 1. Cluster H — `\@personname`/`\@add@frontmatter@now` `}` follow-on
**Papers:** `physics0002038` (R=5 vs P=4), `cond-mat0011517` (R=7 vs P=6).
**Delta:** +1 each (cosmetic).
**Trigger:** revtex/aas papers where `\@personname` end-mode fails
(Constructor `mode=>'restricted_horizontal'` in `\author{...}` body
that's actually in `internal_vertical`). After the end-mode error,
Rust emits an extra `Error:unexpected:} Attempt to close a group
that switched to mode internal_vertical`. Perl absorbs this `}`
silently between the end-mode error and the next CS-trigger error.
**Fix locus:** `latexml_core/src/stomach.rs` `egroup`/`endgroup` —
either suppress duplicate emit when same frame just errored on
`end_mode_opt`, or audit Perl Stomach.pm for the divergence.
**Memory:** unrecorded specific (just SYNC_STATUS).

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
