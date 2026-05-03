# Engine Sync Status — Task List

**Mission (active 2026-04-30):** 100k "no-problem" sandbox parity.
Phase 2 canvas at `/home/deyan/data/100k_noproblem_sandbox/`. Every
paper there is a Perl LaTeXML "no problem" conversion (zero errors,
zero warnings on TL2025 + ar5iv preset). Mission completes when
`latexml_oxide` matches that 100% clean rate paper-for-paper.

A sandbox paper is **in scope** iff Perl LaTeXML on TL2025 with
`--preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings` produces
0 errors on it. Mission completes when every in-scope paper also
produces 0 errors on Rust.

**Phase 1 (DONE):** 7898-paper canvas; 7731 OK = 97.89% (PR #220,
commit `71b0a3e82`).

Earlier per-iteration narrative: `docs/archive/`. Tactical insights:
`docs/WISDOM.md`. Upstream Perl bugs: `docs/KNOWN_PERL_ERRORS.md`.
Intentional divergences: `docs/OXIDIZED_DESIGN.md`. Branch fix list:
`git log master..claude-round-19`.

---

## Current state (2026-05-02 evening)

**Round-19 sandbox parity sweep**: 305 canvas-failed papers triaged.
Final classification after branch `claude-round-19`:

| Verdict | Count | Notes |
|--------|------:|-------|
| BOTH CLEAN | many | both Rust and Perl now produce 0 errors |
| OUT-OF-SCOPE | many | Perl=Rust both >0; not Rust-only regressions |
| PERL_REGRESSION | many | Rust beats Perl (40+ across full sweep) |
| **REAL_REGRESSION** | **0** | All three regressions fixed (REG-1: 24b430885c, REG-2: 86b5e9a764, REG-3: 21811fe31d) |

**Latest random-sample validation (150 papers, 100 ok + 50 failed)**:
108 BOTH CLEAN, 0 REAL_REGRESSION, 12 Rust-beats-Perl wins, no
regressions on previously-clean papers.

**2026-05-02 100-paper random canvas sample**: 99 BOTH CLEAN, 1
PERL_REGRESSION (Rust beats Perl), 0 REAL_REGRESSION. Confirms the
canvas is essentially clean post-fixes; remaining REG-1 + REG-2 are
isolated edge cases.

**2026-05-02 late-evening 500-paper random canvas sample
(`/tmp/sample500.tsv`)**: 497 BOTH CLEAN, 3 PERL_REGRESSION
(`cond-mat0004132` P=14 vs R=0; `astro-ph0407266` P=1 vs R=0;
`0901.0054` P>=101 vs R=15 capped), **0 REAL_REGRESSION**. The
all-clean trajectory holds at 500-paper scale.

**2026-05-03 Telemetry foundation landed** — 8/8 steps from
[`docs/TELEMETRY.md`](TELEMETRY.md). Per-job phase wall + counts
flow from `latexml_core::telemetry` through `cortex_worker` into
`telemetry.json` ZIP members; `tools/benchmark_canvas.sh` aggregates
to `telemetry.jsonl.gz`; `tools/perf_phase_summary.py` and
`tools/perf_compare.py` consume. **17/17 phases wrapped — foundation
complete** (Bootstrap, Digest, Build, Rewrite, MathParse,
PostXmlParse, PostScan, Bibliography, Crossref, Graphics,
MathImages, MathmlPres, MathmlCont, Split, Xslt, Html5Fixups,
Serialize). Per-formula `math_parse_buckets` histogram landed in
commit `91cdebdebc`; smoke-test on math/array.tex (2 formulae)
populates `[0,1,1,0,0,0,0,0,0]`. 95.6% sum-of-phase coverage on
0704.0023 vs the ≥92% acceptance gate.

**Test suite post-telemetry**: `cargo test --tests --no-fail-fast --
--skip xcolors_test` → **1129/0/0 across 48 binaries** (xcolors_test
skipped — pre-existing upstream regression from master `39c7ad8b70`,
fixture not regenerated; tracked separately). Telemetry foundation
introduced zero regressions on the full integration suite.

**2026-05-03 Stage 1 (10k benchmark_canvas) cleared the gate**
(`~/data/stage01_100k_html/`, release cortex_worker, 16 workers).
**9965 [ok] / 34 [conversion_error] / 1 [error] = 99.65% raw OK.**
Parity-triage of all 35 non-OK papers via `parity_check.sh` against
current Perl LaTeXML: 1 BOTH CLEAN, 29 OUT-OF-SCOPE (Perl=Rust both
>0), 4 OUT-OF-SCOPE? (Perl-capped at MAX_ERRORS=101), 1
PERL_REGRESSION (`hep-th0005268` R=21 vs P=26),
**0 REAL_REGRESSION**. Per
[staged 100k protocol](feedback_staged_100k_protocol.md), the zero-
regression gate is cleared. Triage TSV at
`~/data/stage01_non_ok_parity.tsv`.

**2026-05-03 Stage 2 (10k slice [10000, 20000), cumulative=20k)
cleared the gate** — `~/data/stage02_100k_html/`, release
cortex_worker (16 workers, 120s timeout). **9974 [ok] / 26
[conversion_error] / 0 [error] = 99.74% raw OK** (better than
stage 1's 99.65%). Triage of all 26 non-OK papers via
`parity_check.sh`:

| Verdict | Count | Notes |
|--------|------:|-------|
| BOTH CLEAN | 2 | math0107222 (fixed by `2cbc6274fc`), physics0207082 (Perl-timeout/now-OK) |
| OUT-OF-SCOPE | 20 | Perl=Rust both >0; not Rust regressions |
| PERL_REGRESSION | 4 | Rust beats Perl: hep-ph0112138 (R=6 vs P=12), hep-ex0204024 (R=2 vs P=4), hep-ph0110283 (R=96 vs P=101 capped), astro-ph0204393 (R=91 vs P=101 capped) |
| **REAL_REGRESSION** | **0** | Stage 2 cleared the zero-regression gate |

Rust supersedes Perl on 4 more papers (now 17+ total in
[project_rust_supersedes_perl.md](feedback_no_speculative_bindings.md)).
Triage TSV at `~/data/stage02_non_ok_parity.tsv`.

**math0107222 fix (commit `2cbc6274fc`)**: PiCTeX `\setdots` was
undefined; `\setdashes` required mandatory `<#1>` arg. Both stubs
now use `\@ifnextchar<` dispatch supporting both `\setdots` and
`\setdots <0.05cm>` forms (Perl-faithful no-op).

**2026-05-03 Stage 3 (10k slice [20000, 30000), cumulative=30k)
cleared the gate** — `~/data/stage03_100k_html/`. **9975 [ok] /
25 [conversion_error] / 0 [error] = 99.75% raw OK** (best stage
yet — vs stage 1's 99.65% and stage 2's 99.74%). Triage of all
25 non-OK papers via `parity_check.sh`:

| Verdict | Count | Notes |
|--------|------:|-------|
| OUT-OF-SCOPE | 23 | Perl=Rust both >0; not Rust regressions |
| PERL_REGRESSION | 2 | Rust beats Perl: hep-ph0211300 (R=24 vs P=48), hep-th0308103 (R=38 vs P=101 capped) |
| **REAL_REGRESSION** | **0** | Stage 3 cleared the zero-regression gate |

Triage TSV at `~/data/stage03_non_ok_parity.tsv`. Cumulative
through Stage 3: **29,914 OK / 30,000 papers = 99.71%**, with
**0 REAL_REGRESSION across all 30k papers**.

**2026-05-03 Stage 4 (10k slice [30000, 40000), cumulative=40k)
cleared the gate** — `~/data/stage04_100k_html/`. **9976 [ok] /
24 [conversion_error] / 0 [error] = 99.76% raw OK** (best yet).
Triage of all 24 non-OK papers:

| Verdict | Count | Notes |
|--------|------:|-------|
| BOTH CLEAN | 1 | math0409286 |
| OUT-OF-SCOPE | 18 | Perl=Rust both >0 |
| OUT-OF-SCOPE? | 1 | math0406156 (Perl-capped) |
| PERL_REGRESSION | 3 | math0403005, hep-ph0407026, hep-ph0410354 |
| REAL_REGRESSION (FIXED) | 1 | **quant-ph0406132 — fixed by `3e71dc3f7e`**: PiCTeX `\putrectangle` stub was missing; verified R=1→0 on patched binary |
| **POST-FIX REAL_REGRESSION** | **0** | Stage 4 cleared the zero-regression gate |

Triage TSV at `~/data/stage04_non_ok_parity.tsv`. Cumulative
through Stage 4: **39,890 OK / 40,000 = 99.73%** raw, **0
REAL_REGRESSION post-fix**.

**`\putrectangle` fix (commit `3e71dc3f7e`)**: PiCTeX
`\putrectangle corners at <x1> <y1> and <x2> <y2>` was undefined
(Plain TeX path via `\input pictex`); added a 4-numeric-arg
gobble stub matching pictex.tex's no-render policy.

**2026-05-03 Stage 5 (10k slice [40000, 50000), cumulative=50k)
cleared the gate** — `~/data/stage05_100k_html/`. **9973 [ok] /
27 [conversion_error] / 0 [error] = 99.73% raw OK**. Triage of
all 27 non-OK papers:

| Verdict | Count | Notes |
|--------|------:|-------|
| OUT-OF-SCOPE | 17 | Perl=Rust both >0 (incl 1 OOS-cap cs0502050) |
| PERL_REGRESSION | 8 | hep-ph0411166, hep-ph0501037, astro-ph0506245, cs0508085, hep-ph0508175, hep-ph0509359, hep-ph0510024, cond-mat0511296 |
| **REAL_REGRESSION** | **0** | Two papers (cs0412098, astro-ph0502153) initially flagged REAL by 90s-Perl-timeout false positive; reclassified OUT-OF-SCOPE on TIMEOUT_SECS=300 retry (R=Perl in both cases) |

Triage TSV at `~/data/stage05_non_ok_parity.tsv`. Cumulative
through Stage 5: **49,863 OK / 50,000 = 99.73%** raw, **0
REAL_REGRESSION across all 50k papers**.

**Diagnostic insight**: At 90s Perl timeout, `parity_check.sh`
can yield false-positive REAL_REGRESSIONs when Perl's first
errors lag Rust's by more than the cutoff. Re-running with
TIMEOUT_SECS=300 confirmed both candidates were OOS — Perl
hits the same errors on full run, just slower. Future stages
should use 5min retry on any REAL hits as a sanity check.

**2026-05-03 Stage 6 (10k slice [50000, 60000), cumulative=60k)
cleared the gate** — `~/data/stage06_100k_html/`. **9981 [ok] /
19 [conversion_error] / 0 [error] = 99.81% raw OK** (best stage
yet). Triage of all 19 non-OK papers:

| Verdict | Count | Notes |
|--------|------:|-------|
| OUT-OF-SCOPE | 13 | Perl=Rust both >0 |
| OUT-OF-SCOPE? (Perl-cap) | 1 | astro-ph0603369 (R=310 vs P=101 capped) |
| PERL_REGRESSION | 5 | gr-qc0601055, physics0602049, hep-ph0604191, physics0608148, plus 1 capped |
| **REAL_REGRESSION** | **0** | Stage 6 cleared the zero-regression gate |

Triage TSV at `~/data/stage06_non_ok_parity.tsv`. Cumulative
through Stage 6: **59,844 OK / 60,000 = 99.74%** raw, **0
REAL_REGRESSION across all 60k papers**.

**2026-05-03 Stage 7 (10k slice [60000, 70000), cumulative=70k)
cleared the gate** — `~/data/stage07_100k_html/`. **9981 [ok] /
19 [conversion_error] / 0 [error] = 99.81% raw OK** (tied with
Stage 6 best). Triage of all 19 non-OK papers:

| Verdict | Count | Notes |
|--------|------:|-------|
| BOTH CLEAN | 1 | 0708.2784 (already-fixed in newer binary) |
| OUT-OF-SCOPE | 13 | Perl=Rust both >0 |
| PERL_REGRESSION | 4 | hep-ph0411166...0706.2862 — Rust beats Perl on 4 papers |
| **REAL_REGRESSION (FIXED, self-introduced)** | **1** | **0705.3903 — `\setdashes <2mm>` in Plain TeX context** broke because my earlier `\setdashes`/`\setdots` rewrite (commit `2cbc6274fc`) used `\@ifnextchar<` which is a LaTeX2e kernel macro absent in Plain TeX. Replaced with `\futurelet` dispatch (commit `0f8475b8a2`). Verified R=11→0. Lesson: stay Plain-TeX-compatible in `latexml_contrib/src/pictex_tex.rs` since `\input pictex` runs in Plain-TeX preamble. |
| **POST-FIX REAL_REGRESSION** | **0** | Stage 7 cleared the zero-regression gate |

Triage TSV at `~/data/stage07_non_ok_parity.tsv`. Cumulative
through Stage 7: **69,825 OK / 70,000 = 99.75%** raw, **0
unfixed REAL_REGRESSION across all 70k papers**.

**2026-05-03 Stage 8 (10k slice [70000, 80000), cumulative=80k)
cleared the gate** — `~/data/stage08_100k_html/`. **9987 [ok] /
13 [conversion_error] / 0 [error] = 99.87% raw OK** (NEW best
stage). 13 errors → 12 OUT-OF-SCOPE + 1 PERL_REGRESSION
(0804.4000 R=12 vs P=24) + **0 REAL_REGRESSION**.

Triage TSV at `~/data/stage08_non_ok_parity.tsv`. Cumulative
through Stage 8: **79,812 OK / 80,000 = 99.77%** raw, **0
unfixed REAL_REGRESSION across all 80k papers**.

Stage 9 (90k cumulative) is unblocked.

While Stage 2 ran, also:
- Verified the lipsum cluster
  (`project_lipsum_clist_map_73.md`) is GREEN:
  `cargo test --test 83_expl3 str_lowercase` → 4/0/0.
- Memory cleanup: marked `\vspace`, `\psfig`, lipsum, aa.cls,
  `\setdots` clusters as fixed in `MEMORY.md` index.
- Two stale build warnings cleaned up:
  - `tex_tables.rs:728` — `last_token` dead-init after REG-3 fix
    (commit `a76724e361`).
  - `telemetry.rs:358` — `field!`-macro trailing dead-write
    (commit `8711c8a66e`).

**Round-19 verification**: re-tested all 29 papers from the original
round-19 REAL_REGRESSION list; ALL now BOTH CLEAN or OUT-OF-SCOPE
(no longer regressions).

`cargo test --tests` 1124/0/0.

**REG-3 root cause (fixed)**: `digest_alignment_column`'s OUTER loop
in `latexml_engine/src/tex_tables.rs` did NOT reset `last_token`
across iterations. Perl's `digestAlignmentColumn`
(`LaTeXML/blib/lib/LaTeXML/Engine/TeX_Tables.pool.ltxml:367-396`)
sets `$token` per `readXToken(0)` call, so when the gullet returns
undef (mouth exhausted, e.g. mid-cell `\input` finishes) the
`if (!$token)` check terminates the column. In Rust the
`while let Some(xtoken) = read()` body only updates `last_token`
on `Some`; on the OUTER iter after INNER 1 / INNER 2 exhausted
the mouth, `last_token` still pointed at the previous content
token, the `last_token.is_none() || last_is_end` check skipped,
and the column re-fed `(column_before, marker, last_token)` into
an empty gullet — re-invoking `\begin{picture}` infinitely.
Fix: reset `last_token = None` at the start of each OUTER iter.
One-line addition + 9-line comment, see
`tex_tables.rs:711` (commit on top of round-19).

---

## Open work — the 2 remaining REAL_REGRESSIONs

Each requires dedicated multi-iteration architectural work. Bisections
and root causes are recorded; the steps below are the proposed line
of attack.

### REG-1: math0403005 — FIXED `24b430885c` (2026-05-02 evening)

R=29 → R=3 (now PERL_REGRESSION; Rust beats Perl by 24).
Root cause: `\noalign` primitive's `bgroup()` leaked a frame because
the body's `{...}` was processed independently — the `{` pushed
ANOTHER frame, the `}` popped only one, leaving the primitive's
bgroup leaked. The leaked frame cascaded into `\vtop`/`\hbox`
mode-end mismatches in p{}-column arrays. Fix: read+discard the
`{...}` body and `egroup()` to balance the primitive's bgroup.

**Witness paper**: `/home/deyan/data/100k_noproblem_sandbox/arxmliv/0403/math0403005/math0403005.zip`
(file `scs8-for-arxiv.tex`, around lines 570-600).

**Symptom**: Both Rust and Perl error on the same `\noalign` /
`\end{center}` / `\end{table}` cascade caused by user-malformed
TeX inside `\vtop{...}`. But Rust emits **3 extra `\vtop`-labeled
frame errors** that Perl doesn't:

```
Error:unexpected:\@end@array Attempt to close … due to T_CS[\vtop]
Error:unexpected:\endgroup Attempt to close … due to T_CS[\vtop]
Error:unexpected:\lx@begin@alignment Attempt to close … due to T_CS[\vtop]
```

These are **recovery-cascade noise** — same root cause as Perl's 1
visible error, just multiplied by Rust's stricter mode-frame
discipline.

**Plan of attack:**

1. **Reproduce in min repro** — bisect scs8-for-arxiv.tex around the
   error site (lines 570-580 in the source) to find the smallest
   `\vtop{ … malformed … }` snippet that fires the same cascade.
2. **Instrument `\vtop`'s digestion** — temporarily add `eprintln!`
   in `latexml_engine/src/tex_box.rs:747` (`\vtop` DefConstructor
   `before_digest`/`after_digest`) and at `push_stack_frame`/
   `pop_stack_frame` to count frames opened/popped per `\vtop`
   invocation.
3. **Compare with Perl** — run min repro through `latexml --debug=stomach`
   (or equivalent verbose flag) and diff the frame-pop sequence.
4. **Identify the redundant Error firings** — likely `egroup` is
   raising `Error!("unexpected", ...)` 3× when the same broken state
   should be raised once and then suppressed/recovered until next
   well-formed token.
5. **Fix candidates (ordered by safety)**:
   * a. **Once-per-cascade gate**: in `latexml_core/src/stomach.rs`'s
        `egroup` (line 273) and `\@end@array` / `\lx@begin@alignment`
        primitives, debounce repeated mode-frame errors with the same
        `groupInitiator` token (only emit the first within a single
        digest call).
   * b. **Frame stripping on first error**: when egroup hits a
        mode-switch frame and errors, also pop the frame so subsequent
        egroups see a fresh state.
   * c. **Match Perl's swallow-and-recover semantics**: Perl emits 1
        error here and continues; Rust should match. The Perl path
        likely lives in `Stomach.pm` `egroup`/`endMode` — port the
        recovery branch verbatim.
6. **Verify**: math0403005 R=29 → R=27 (== Perl) reclassifies to
   OUT-OF-SCOPE. Sweep round-19 to confirm no regressions on the
   other 38 papers.

**Difficulty**: Medium — recovery-cascade noise rather than a hard
correctness bug. Low-risk because we're matching Perl, not changing
the digestion semantics.

---

### REG-2: math-ph0501074 — FIXED `86b5e9a764` (2026-05-02 evening)

R=15→0 BOTH CLEAN. Root cause: Rust's `read_next_conditional`
(gullet.rs:1191) was calling `read_token`, which includes the
"alignment-template trigger" (`align_group_count==0` + `&` →
`handle_template`). Perl's `skipConditionalBody` reads at lower
level, bypassing this trigger during `\else`-skip. When `\ifx.#1.\else`
skipped past `\begin{pmatrix} 1 & 2 \end{pmatrix}` looking for
`\else`, the `&` inside pmatrix was mistakenly handled as the OUTER
align's column-end, mutating alignment state and cascading into 10
mode-frame errors. Fix: switch `read_next_conditional` to use
`read_internal_token` directly with manual BEGIN/END tracking,
matching Perl byte-for-byte. See wisdom memory
`wisdom_lefteqn_pmatrix_align_leak.md` for the deep bisect path.

_REG-2 historical investigation log moved to commit history (86b5e9a764)
and `wisdom_lefteqn_pmatrix_align_leak.md`. Root cause was NOT in the
pmatrix/inline-math interaction (the original hypothesis); it was in
`read_next_conditional` calling `read_token` and thus firing the
alignment-template trigger during `\else`-skip. See
`wisdom_skip_no_align_trigger.md` for the generic pattern._

---

### REG-3: 0909.5169 — pstex_t `\put(0,0)` Pair-param parse failure (R=10001, P=0)

**Witness paper**: `/home/deyan/data/100k_noproblem_sandbox/arxmliv/0909/0909.5169/0909.5169.zip`
(file `v-Dims.tex`, line 28: `\def\pstex#1{\begin{array}{c}\input
figs/#1.pstex_t \end{array}}`).

**Symptom**: A pstex_t file (`\input figs/RMoves.pstex_t` inside
math-mode `\begin{array}`) starts with `\begin{picture}(0,0)%` —
standard PiCTeX/pstex output. Rust's `{picture}` env signature is
`{picture} Pair OptionalPair` (Pair = `(x,y)`). Inside math-mode
array context, the Pair reader fails to consume `(0,0)`, fires
`Error:expected:Pair Missing argument Pair`, then mode-frame
cascade explodes to 10001 errors (Rust's MAX_ERRORS cap).

**Bisected min repro** (verified on this branch, 2026-05-02):

```latex
\documentclass{article}\begin{document}
\begin{array}{c}\input test.pstex_t\end{array}
\end{document}
```
with `test.pstex_t` containing just `\begin{picture}(0,0)\end{picture}\n`.

* Perl: **0 errors** (clean).
* Rust: **501 errors** cascade starting with `Error:unexpected:&
  Extra alignment tab '&'` followed by `Error:expected:Pair Missing
  argument Pair for Constructor[\begin{picture}…]`.

Same content INLINE (no `\input`) is clean in Rust too. Same
content in `\input` but OUTSIDE `\begin{array}` is clean. So the
trigger is **`\begin{array}` + `\input` together**.

**Refined diagnosis** (further deep-dive 2026-05-02):

Backtrace from instrumented `picture.before_digest`:
```
0: closure at latex_constructs.rs:8443
1: Constructor::execute_before_digest
2: Constructor::invoke_primitive
3: stomach::invoke_token
4: tex_tables::digest_alignment_column at tex_tables.rs:809
5: tex_tables::digest_alignment_body at tex_tables.rs:631
6: closure for \@end@array's after_digest at tex_tables.rs:41
```

**The bug is in Rust's `\halign`/array cell re-digestion.** When
`\@end@array` fires its `after_digest`, it calls
`digest_alignment_body` which loops calling `digest_alignment_column`.
That re-digests cell tokens. Each re-digestion of `\begin{picture}`
creates a fresh picture invocation. The first iteration consumes
`(0,0)` correctly via Pair, but subsequent iterations (which
shouldn't be happening for a single-cell single-row array) keep
re-invoking picture, each time finding `\end{picture}` next and
firing "Missing argument Pair", cascading to 10001 errors.

**Why `\input` matters**: When the picture content is INLINE
(`\begin{array}{c}\begin{picture}(0,0)\end{picture}\end{array}`),
the env-machinery captures picture's body BEFORE array
sees the cell. Picture's body is closed before array starts cell
walking. Single, clean invocation.

When the content comes via `\input`, the file's tokens are pushed
to the gullet and read DURING array's cell digestion. Picture's
env-handler runs from inside `digest_alignment_column` (instead of
outside). Body capture inside this nested context doesn't bound
the alignment loop's iteration — picture's body tokens stay in the
input stream and get re-walked.

(An earlier candidate — make Pair use expanding peek
[`read_x_token` + unread] — was rejected: it's a divergence from
Perl's `ifNext`, and only reduced the cascade to 501 errors without
fixing the root cause.)

**Plan of attack:**

1. **Inspect `\input` token-streaming** —
   `latexml_engine/src/tex_file_io.rs:191`. When `\input` reads a
   file, does it preserve the surrounding alignment context? In
   particular, are the token-stream's pushback / catcode / align-state
   markers correctly maintained across the file-IO boundary? Compare
   with Perl's `\input` (TeX_FileIO.pool.ltxml).
2. **Inspect `\begin{array}` cell-template setup** — find where
   the `c`-column expands to `\hfil $\displaystyle ## $ \hfil &`
   (or equivalent) and ensure the trailing `&` is the `\halign`-machinery
   separator, not a user-visible gullet token.
3. **Trace the actual token sequence** — temporarily add `eprintln!`
   before `Pair`'s `ifNext` call to print the next 5 tokens. This
   confirms whether `&` is sitting between `\begin{picture}` and
   `(` (root cause is upstream) or whether `(` is consumed by Pair
   itself (root cause is in Pair / read_token).
4. **Diff with Perl trace** — `latexml --debug=tokens` on the
   same min repro and compare the token sequence at the picture-env
   boundary.
5. **Fix candidates (in order of root-cause-fidelity)**:
   * a. **Fix the `&` leak** — most likely a `\halign` /
        `\begin{array}` template setup divergence. The fix should
        match Perl's column-template emission so cell-separator
        tokens stay inside the alignment machinery, not visible to
        the gullet's `read_token`.
   * b. **Cap error cascade at first `Pair` failure** (pragmatic
        bound): when picture's required `Pair` reader returns
        `ArgWrap::None`, instead of erroring + cascading, gobble
        balanced text up to `\end{picture}` via XUntil and
        silently emit an empty `<ltx:picture>`. This bounds the
        damage at 1 error (or 0 if we Warn instead of Error).
        Acceptable as a defense-in-depth even after fix (a) lands.
6. **Verify**: 0909.5169 R=10001 → R=0 (== Perl) BOTH CLEAN under
   fix (a). Sweep round-19 + 100-paper random ok-status sample to
   confirm no regressions in any Pair-using construct (picture,
   pstricks, qbezier, etc.).

**Difficulty**: Hard — root cause is in `\halign` / `\input`
machinery, not the easy "Pair reader" surface. Fix (b) is a
low-risk fallback if (a) requires more iteration than available.

---

## Engine file open gaps

| File | Status | Open Gap |
|------|--------|----------|
| `base_parameter_types.rs` | MINOR | Parameterized `CommaList:Type` form unported (no Perl users). |
| `tex_box.rs` | MINOR | Box dimension edge cases. |
| `tex_fonts.rs` | MINOR | `\fontdimen` array semantics; per-font `\hyphenchar`. |
| `tex_tables.rs` | MINOR | Padding CSS classes (XSLT concern). |
| `plain_base.rs` | OPEN | Some closure-backed defs need conversion to Token bodies for dump round-trip. |
| `latex_base.rs` | OPEN | Closure-backed defs need conversion or relocation to `latex_constructs.rs`. |

---

## Tikz known diffs vs Perl

1. foreignObject transform Y / width/height
2. Arrow tip shape (different path data)
3. SVG viewBox / total width differs slightly
4. tikz matrix uses `<svg:g class="ltx_tikzmatrix">` (Rust) vs
   inline-blocks (Perl)

---

## Permanent ignores

* **Sandbox out-of-scope:** ns1–ns5 (52_namespace, no DTD); 2402.03300,
  2410.10068, 2511.03798 (Perl also fails).
* **Rust supersedes Perl** (both still in scope, but Rust passes
  where Perl errors): `1207.6068`, `0909.3444`, plus 40+ papers
  identified in round-19 sweep (memory:
  `project_rust_supersedes_perl.md`).
* **Unported pools:** `BibTeX.pool.ltxml` (skipped via `--nobibtex`).

---

## Acceptance gates

| Gate | Current | Target |
|---|---|---|
| `cargo test --tests` | 1124/0/0 | unchanged across all task work |
| `latexml_oxide --init=plain.tex` | 0 errors | 0 errors |
| `latexml_oxide --init=latex.ltx` | 0 errors | 0 errors |
| 10k canvas (Phase 1, complete) | 7731 / 7898 = 97.89% | n/a (canvas retired) |
| 100k "no-problem" canvas (Phase 2, active) | downloaded — sweep pending | 100% match Perl |
| Round-19 305-paper triage | 3 REAL_REGRESSION | 0 REAL_REGRESSION |

---

## Distribution follow-up

Once TL2025 dumps stay robust through a CI cycle: `include_bytes!`
`{plain,latex}.dump.txt` for TL2022 … TL2026 and select at runtime
by `kpsewhich --version`. Currently dumps load from
`resources/dumps/` on disk.

---

## Earlier work (archived)

Pre-round-19 fix history (Round-17 squashed master, plus rounds 18
and 19 in branch `claude-round-19`) is preserved in `git log` and
`docs/archive/`. Major commits in `claude-round-19` (30 total):

* `d44f1cb38` Token/XToken `\relax` sentinel on EOF (cleared 3 papers
  via cascade-removal: 0910.2125, hep-th0302065, gr-qc0304029)
* `817d91624` XUntil re-Invoke `\def`-family primitives (cleared
  0805.1712, cs0502037)
* `6ac613b48` xy.sty pre-loads amstext for in-math `\text`
  (math0211451)
* `799abc9e2` `\@trivlist → \relax` (Perl L1732, 0802.2207)
* `1e34cdd27` multirow brace-presence gate (3 papers)
* `a6b4cb5161` Pre-flight `\documentstyle` checks not eagerly load
  (12-paper psfig cluster)
* `5c1ec07da` IEEEtran `\if@twocolumn=true` journal default
* `0155b56c1` IEEEtran private `\if@technote`/`\if@confmode`
* `342b237199` ntheorem [standard] option triggers std raw load
* …plus 21 smaller package-binding bridges and stubs.

See `git log --oneline master..claude-round-19` for the full list.
