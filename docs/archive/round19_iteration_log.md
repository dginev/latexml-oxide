# Round-19 iteration log (archived)

Verbose per-stage narrative captured during the round-19 100k canvas
mission (2026-05-02 / 2026-05-03). Stages 1–10 each cleared the
zero-REAL_REGRESSION gate; cumulative result was 99,774 / 100,000
papers OK (99.77%). Final summary lives in `docs/SYNC_STATUS.md`;
this file preserves the per-stage triage detail for reference.

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
[`docs/performance/TELEMETRY.md`](TELEMETRY.md). Per-job phase wall + counts
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

**2026-05-03 Stage 9 (10k slice [80000, 90000), cumulative=90k)
cleared the gate** — `~/data/stage09_100k_html/`. **9985 [ok] /
13 [conversion_error] / 1 [error] / 1 [timeout] = 99.85% raw OK**.
15 errors → 5 PERL_REGRESSION (0808.3583, 0809.4243, 0810.4392,
0811.1175, 0812.3908) + 1 PERL_CAPPED (0901.0054, P>=101 vs R=15) +
2 BOTH-CLEAN-on-retry (0903.3183, 0903.3465 — transient
cortex_worker SIGABRT/timeout under 16-worker contention) +
7 OUT-OF-SCOPE + **0 REAL_REGRESSION**.

Cumulative through Stage 9: **89,797 OK / 90,000 = 99.78%** raw,
**0 unfixed REAL_REGRESSION across all 90k papers**.

Concurrent CI fix: `xcolors_test` (65_graphics) had been failing
since commit 39c7ad8b70 (xcolor: dvipsnames sRGB override) because
that override silently changed many dvipsnames colors away from
xcolor's naive `R=(1-c)(1-k)` math. After auditing pink and other
near-tristimulus colors, the dvipsnames sRGB table was reverted in
commit 66d61be6b7 (the c!p extrapolation fix is kept). xcolor's
internal model is naive cmyk→rgb, and that's what most modern PDF
viewers do; the Acrobat-SWOP override traded one set of "looks
wrong" surprises for another. Reverting restores library-internal
consistency.

**2026-05-03 Stage 10 (10k slice [90000, 100000), cumulative=100k)
cleared the final gate** — `~/data/stage10_100k_html/`. **9977 [ok] /
21 [conversion_error] / 1 [error] / 1 [timeout] = 99.77% raw OK**.
23 errors → 5 PERL_REGRESSION + 1 invalid (0907.2492 — PDF
mis-named as `.tex`; landed `345ace6fb1` to emit
`Fatal:invalid:not_tex_source`) + 17 OUT-OF-SCOPE/transient + **0
REAL_REGRESSION**. Stage 10 saw one Perl-timeout false-positive
(0912.2378 R=18 P=16-partial → R=18=P=18 on 5min retry).

🎯 **MISSION ACCOMPLISHED — 100k canvas REAL-regression-free.**
Cumulative across all 10 stages: **99,774 OK / 100,000 = 99.77%**
raw, **0 unfixed REAL_REGRESSION across all 100,000 papers**.
Round-19 closes with the strongest sandbox guarantee yet.

