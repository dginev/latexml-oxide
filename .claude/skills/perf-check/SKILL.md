---
name: perf-check
description: >
  Measure latexml-oxide conversion performance correctly, pick the right build
  profile, and avoid re-litigating settled optimization questions. Use when
  benchmarking, comparing against Perl, choosing a profile, diagnosing a slow/OOM
  conversion, or when tempted to optimize startup/codegen. Invoke for "is this
  slower?", "benchmark X", "which profile", "optimize startup", "profile this
  paper", "/perf-check".
---

## Build profiles — use the right one (Cargo.toml)

| Profile | Command | Use |
|---|---|---|
| `test` (default) | `cargo build` / `cargo test` / `cargo run` | **All day-to-day dev + triage.** Full debug info, debug-assertions, overflow-checks, fast incremental. Best diagnosability. |
| `ci` | `cargo test --profile ci` | **GitHub runner only** (16 GB, fast compile). NOT what local dev should mimic. |
| `release` | `cargo build --release` | **Sandbox sweeps + Perl-parity measurement.** Strong-optimized, thin-LTO. |
| `maxperf` | `cargo build --profile maxperf` | **Distribution artifact.** Fat-LTO, CGU=1, `panic=abort`. Slowest build; smallest+fastest binary. |

Distribution recipe (what `tools/make_release.sh` uses): `cargo build
--no-default-features --features runtime-bindings --profile maxperf --bin
latexml_oxide`.

## Settled questions — DO NOT re-open (measured, closed, tooling deleted)

- **PGO / target-cpu (`x86-64-v3`, `native`): NO measurable gain.** Rigorous
  measurement (2026-06-21, disjoint train/measure split, best-of-3 interleaved):
  total speedup 0.999×, identical medians. `maxperf` is *already* at the
  compiler ceiling (fat-LTO + CGU=1 captures what PGO would unlock on thin-LTO),
  and the hot path (branchy catcode/macro dispatch) is neither SIMD-amenable nor
  statically branch-skewed. Ship the portable `x86-64` baseline. Do not
  re-attempt without a major engine-architecture change.
  (`pgo-isa-no-gain-2026-06-21` memory; `docs/PERFORMANCE.md`.)
- **Startup floor (~161 ms): the ~50 ms dump-parse lever was declined.**
  Decomposed: proc init ~6 ms, bootstrap ~9 ms, **latex dump load ~85 ms**
  (60% text-parse), `_constructs`+digest+build+serialize ~80 ms. The clean lever
  (binary dump format / parallel-parse) is ~50 ms ≈ 10% of the 507 ms median —
  declined as too small to justify the dump-pipeline parity re-validation risk.
  Compiling dumps to Rust code (vs gzip-blob embed) cost +10 min compile for zero
  win, abandoned. **Do not optimize the dump load.**
  (`docs/archive/STARTUP_COST_ANALYSIS_2026-06-21.md`.)

## Measurement discipline (this is where perf claims go wrong)

- **Same-conditions A/B only.** A one-off wall-clock reading on an idle vs busy
  machine is noise — we once read 0.65 s and retracted a "5× faster" claim once a
  same-conditions before/after showed ~3.35 s → ~3.1 s (no real change). Always
  measure base and candidate back-to-back under the same load; prefer best-of-3
  interleaved on a quiesced box.
- Tools: `tools/perf_compare.py` (paired A/B of two telemetry corpus runs:
  Δwall, Δphase_us), `tools/perf_phase_summary.py` (per-phase rollups),
  `tools/run_perf_corpus.sh` (Tier-A serial regression baseline). Full
  methodology + Perl-parity baselines: `docs/PERFORMANCE.md`. Witness papers for
  timeout/OOM/peak-RSS/hang regressions: `docs/STABILITY_WITNESSES.md`.

## Test-suite gotcha: `MemoryBudget` cascade ≠ code bug

A cascade of failures on *basic* documents (article/book/itemize) that **pass at
`--test-threads=1`** is the **process-wide RSS fuse**
(`stomach.rs::check_timeout`, `Fatal:Timeout:MemoryBudget RSS … > cap`), not a
regression. libtest runs one process with one thread per test, so on this
64-core/128-thread box `-j128` runs ~128 conversions whose *aggregate* RSS trips
the fuse. The default cap **stays 4.5 GB** (a parallel one-paper-per-process
fleet would OOM at 9 GB); the test harness raises it to 9 GB via
`init_test_rss_cap()`. **Diagnosis:** re-run with `-- --test-threads=1`; a
non-`<Math>` fixture failing is another tell a MathML change is innocent. Always
use `cargo test --workspace --tests --no-fail-fast` for the true count (default
fail-fast stops at the first failing *binary*). (`cargo-test-rss-fuse-parallelism`
memory.)

## Self-contained-binary invariant (a correctness-of-distribution check)

A conversion must not *read* latexml-oxide's *own* resources from disk at
runtime — dumps, RelaxNG schema, XSLT/CSS/JS are embedded and served from memory.
New code that adds a runtime read of an owned resource must `include_bytes!`/
`include_str!` it instead. (Reading the host texmf tree via kpathsea is allowed.)
Verify by renaming `resources/dumps/` away and converting, or `strace` for the
XSLT. Rationale: `docs/OXIDIZED_DESIGN.md` → Guiding Principles.
