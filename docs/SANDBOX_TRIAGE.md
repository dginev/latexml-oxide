# 10k sandbox triage workflow reference

> **Status (refreshed 2026-04-30):** current priority and corpus
> counts live in [`SYNC_STATUS.md`](SYNC_STATUS.md)'s dashboard.
> Sandbox work continues opportunistically after strict Perl
> format/package-loading parity checks.

The older per-cluster sandbox triage worksheet is archived at
[`archive/sandbox_failures_SYNC_STATUS.md`](archive/sandbox_failures_SYNC_STATUS.md).

That file tracked the focused 181-paper failure subset under
`~/data/sandbox_failures` (post-AR-flip, 2026-04-26 baseline) by
cluster, with a per-cluster fix-log. The original workflow used a
now-removed `tools/rerun_failures.sh` wrapper; the modern
equivalent is `tools/benchmark_canvas.sh --input-dir <focused>` or
piping ids to `tools/parity_check.sh -`.

This file ([`SANDBOX_TRIAGE.md`](SANDBOX_TRIAGE.md)) previously
held a session-by-session per-paper narrative through round 17.
Those narratives have been folded into commit messages and
`memory/project_session_history.md`. Keep this file as workflow
guidance; put current corpus status in [`SYNC_STATUS.md`](SYNC_STATUS.md)
or in fresh run artifacts.

## Two-phase workflow: canvas (`release`) → triage (`test`)

The 10k sandbox is run in two distinct phases, each with a
different cargo profile. The split is load-bearing — the two
profiles are *adversarial*, and trying to use a single
"compromise" profile gives a binary that is bad at both jobs.

### Phase 1: Canvas — `--release`

Goal: convert all 10k papers, classify into `ok` /
`conversion_error` / `timeout` / `oom_or_kill` / `abort` /
`segfault`, and produce `results.tsv`.

This is a **measurement** task, not a debugging one. What
matters:

* Wall-clock per conversion is low enough that 10k papers fit a
  coffee-break iteration loop.
* Per-worker RSS is low enough that 16 parallel workers fit in
  RAM (each guarded by `ulimit -v 8 GB`).
* Runtime numbers are *honest* — the categorizer (`timeout` vs
  `ok` vs `oom_or_kill`) only tells the truth at production
  optimization level.

Debug info is dead weight: at this scale you only ever read the
TSV summary, never a backtrace. So we use `--release`:

```
cargo build --release --bin cortex_worker --features cortex --jobs 20
tools/benchmark_canvas.sh --workers 8 --timeout 120
```

(Worker count default lowered from 20 → 8 on 2026-05-16: re-timing the
round22 slow tail showed graphics-bound papers were 5–10× slower at 20
workers because each gs/convert/inkscape fork-exec stack competes for
CPU+I/O. At 8 workers the per-paper overhead is ≤30% vs single-threaded
and **corpus throughput goes up**. Override with `--workers N` only when
the canvas is known to be compute-bound, not graphics-bound.)

The release profile gives `lto = "thin"` + `codegen-units = 20`
+ `strip = "symbols"` + `opt-level = 3`. For the local 32 GB /
20-thread laptop, that keeps strong runtime optimization while using
the available cores during a fresh build. For one-off maximum optimizer
scope, `cargo build --profile maxperf ...` keeps the old `fat` LTO /
single-CGU shape.

The output is a single `results.tsv` (one row per arxiv ID) plus
per-paper `.log` files in `$OUTPUT_DIR/`.

### Phase 2: Triage — `cargo test` / `cargo run` (default profile)

Once Phase 1 produces a list of failing arxiv IDs, the workflow
shifts completely. You're now debugging *one paper at a time*:
read its `.log`, attach a debugger, set breakpoints, mutate
state, edit code, rerun. Per-paper performance is irrelevant
(a single paper finishes in seconds even at `-O1`); diagnostic
richness is everything.

That's exactly what `[profile.test]` is tuned for:
`debug = "full"`, `debug-assertions = true`,
`overflow-checks = true`, `incremental = true`,
`panic = "unwind"`. And critically — incremental compilation
gives ~5-second turnaround on most edits, vs the 10+ min of a
cold release rebuild.

Workflow once `results.tsv` is in hand:

```
# Build a failure list from the canvas TSV
awk -F'\t' '$7 != "ok" {print $1}' \
  ~/data/10k_sandbox_html/results.tsv > /tmp/failures.txt

# Triage one paper (extracts the ZIP, finds the main .tex,
# runs `cargo run --bin latexml_oxide` under the test profile)
ID=$(head -1 /tmp/failures.txt)
tools/triage_failure.sh "$ID"

# Pass extra args through to latexml_oxide:
tools/triage_failure.sh 1407.5769 -- --timeout 600

# Keep the unzipped working copy around (e.g. for repeated edit
# loops or running additional tools against the source):
KEEP_TMP=1 tools/triage_failure.sh 0704.0192
```

`tools/triage_failure.sh` is the canonical Phase-2 entry point.
It is the companion to `tools/benchmark_canvas.sh` (Phase 1):
benchmark_canvas → release-grade canvas, triage_failure →
test-grade single-paper rerun.

For interactive debugging, use the same binary under gdb/lldb
— `debug = "full"` provides locals, line numbers, and full
scope info; `panic = "unwind"` lets the harness produce useful
panic messages and backtraces.

### Why one profile cannot do both

The two profiles' settings are *mutually exclusive optima*:

| Setting             | Canvas (`release`) | Triage (`test`) | Why exclusive |
|---------------------|-------------------:|----------------:|---|
| `opt-level`         | 3                  | 1               | `-O3` inlines aggressively → backtraces lose frames + parameters; `-O1` has tolerable test runtime + faithful frames |
| `lto`               | `"thin"`           | (off)           | Thin LTO keeps cross-crate optimization but parallelizes better on the local 20-thread machine; full fat LTO is reserved for `maxperf` |
| `codegen-units`     | 20                 | 256             | Moderate CGUs use local CPU during release builds; many CGUs remain best for incremental test rebuilds |
| `debug`             | `false`            | `"full"`        | DWARF doubles binary size and disk I/O — wasted under `release`, essential under `test` |
| `debug-assertions`  | off                | on              | Asserts cost runtime cycles — bad for wall-clock, great for invariant checking |
| `incremental`       | false              | true            | Incremental artifacts skew LTO decisions; required for fast iteration |
| `panic`             | `"unwind"` *       | `"unwind"`      | * Currently `unwind` because `pericortex` can't link with `panic="abort"`; otherwise `release` would prefer abort |

Reusing CI's profile here would not help either: CI is RAM-
bounded for a 16 GB GitHub runner, not runtime-bounded for a
10k-paper canvas. CI runs at `opt-level = 0` +
`codegen-units = 256` — the binary it produces is fine for
"do my tests pass" but its conversion times bear no relation
to the production-grade numbers we measure in Phase 1.

### Triage helper

`tools/triage_failure.sh <arxiv_id>` is the single-line entry
point for Phase-2 work:

* Looks up `${SANDBOX_DIR:-~/data/10k_sandbox}/<arxiv_id>.zip`.
* Unzips it under a temp dir (cleaned up on exit unless
  `KEEP_TMP=1`).
* Picks the main `.tex` (prefers `<arxiv_id>.tex`, `main.tex`,
  `paper.tex`, `ms.tex`; falls back to the first `.tex` found).
* Execs `cargo run --bin latexml_oxide -- <main.tex> "$@"` so
  the test profile, `RUST_BACKTRACE=1`, and incremental
  rebuilds all kick in by default.

Pass-through args use `--`:
`tools/triage_failure.sh 1407.5769 -- --timeout 600 --debug`.
