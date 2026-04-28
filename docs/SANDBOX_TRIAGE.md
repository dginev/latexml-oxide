# 10k sandbox triage — see active worksheet

> **Active priority (2026-04-26):** strict-Perl dump parity. See
> [`SYNC_STATUS.md`](SYNC_STATUS.md) "Mission" and
> [`PERL_LOADFORMAT_AUDIT.md`](PERL_LOADFORMAT_AUDIT.md). Sandbox
> work continues opportunistically but is **not the gating front**;
> sandbox regressions during the dump-parity push are accepted per
> user directive.

The active sandbox triage worksheet is
[`sandbox_failures_SYNC_STATUS.md`](sandbox_failures_SYNC_STATUS.md).

That file tracks the focused 181-paper failure subset under
`~/data/sandbox_failures` (post-AR-flip, 2026-04-26 baseline) by
cluster, with a per-cluster fix-log. Workflow:

```
edit code → rebuild → ./tools/rerun_failures.sh → diff against
docs/sandbox_failure_181_triage.tsv → mark recovered papers [x]
```

This file ([`SANDBOX_TRIAGE.md`](SANDBOX_TRIAGE.md)) previously
held a session-by-session per-paper narrative through round 17.
Those narratives have been folded into commit messages and
`memory/project_session_history.md`. This file now exists only as
a redirect; do not write new triage notes here.

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
cargo build --release --bin cortex_worker --features cortex
tools/benchmark_10k.sh --workers 16 --timeout 120
```

The release profile gives `lto = "fat"` + `codegen-units = 1`
+ `strip = "symbols"` + `opt-level = 3`. The slow (~10 min cold)
build is amortized over 10k conversions.

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
It is the companion to `tools/benchmark_10k.sh` (Phase 1):
benchmark_10k → release-grade canvas, triage_failure →
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
| `lto`               | `"fat"`            | (off)           | LTO doubles link RAM and 5×s build time — fine once per 10k conversions, prohibitive per-edit |
| `codegen-units`     | 1                  | 256             | Few CGUs = best optimization; many CGUs = best incremental rebuild |
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
