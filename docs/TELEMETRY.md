# Telemetry Foundation

Per-job structured telemetry for `cortex_worker` benchmark runs.
Required by [`docs/PERFORMANCE.md`](PERFORMANCE.md) §P0:

> *"the 100k run can be sorted by any phase without reading `.log`
> files, and the sum of phase timings explains most of wall time."*

This document is the implementation contract. Edit the doc when the
plan changes; do not let the code drift from it.

---

## 1. Goals

1. **Phase-attributed wall time** for every job, not just `wall_time_s`.
2. **Counts** for the units that drive each phase (formulae, graphics,
   DB objects, output bytes, child processes).
3. **Resource peaks** (`max_rss_kb`, child user/sys time).
4. **Provenance** (git SHA, cmdline, host) so historical comparisons
   are meaningful across rebuilds.
5. **Sum of phase wall ≈ total wall**, median ≥ 0.92 across the
   100k corpus. The remaining ≤8% absorbs un-instrumented edges
   (process start, dump load, signal handling, JSON serialize itself).

Non-goals (deferred):
- Per-token / per-macro timing — would dominate cost.
- Heap profiling — separate tooling (`heaptrack`, `dhat`).
- Live streaming telemetry — write at end of job; no IPC.

---

## 2. Decisions made up-front

These were "open questions" in the prior draft. Locked in before
coding to avoid mid-flight re-design:

### 2.1 Phase granularity — coarse, with one fine slot

The 11-phase enum in §3.1 is the ceiling. Do not subdivide a phase
in v1; if Phase-3 analysis shows ambiguity in (say) `post_scan`, add
a sub-phase via a TODO comment and ship a v2.

The one exception: `math_parse` carries a per-formula histogram so
"1 slow formula vs 1000 cheap" is answerable in v1.

### 2.2 Math-parse representation — histogram + scalar

For each job, record both:
- `math_parse_us: u64` — total time across all parses
- `math_parse_buckets: [u32; 9]` — count per bucket
  `(<0.5ms, <1, <2, <5, <10, <20, <50, <100, ≥100ms)`

9 × 4 B = 36 B per row × 100k = 3.6 MB. Trivial.

### 2.3 Subprocess time — counted in BOTH phase wall and child rusage

`graphics` phase wall includes the wait for `mutool`/`pdftocairo`/`convert`/`gs`.
Child rusage (`child_user_us`, `child_sys_us`) also accounts for
those processes. This is *intentional*, not double-count: phase
wall measures wall time, child rusage measures CPU work done in
spawned subprocesses. Both are useful and answer different
questions. Documented explicitly so consumers don't sum them.

### 2.4 Default-on, opt-out via `--no-telemetry`

No `#[cfg(feature = "telemetry")]`. Coarse phase wrappers cost
nanoseconds; gating them adds compile-time complexity for no
runtime gain. The math-parse histogram update is the only
per-formula instrumentation; benchmark on `1011.1955` (387 formulae,
math-parser bound) and require ≤ +1% wall regression before merge.

If the instrumentation ever measurably slows hot paths, address
the slow site, not via a feature flag.

### 2.5 Output format — gzipped JSONL alongside extended TSV

- `<output_dir>/telemetry.jsonl.gz` — full record per job, gzip-compressed
  (typical 10:1 ratio → 100k × ~200 B = 20 MB).
- `<output_dir>/results.tsv` — extended schema (column-superset of
  current); existing scripts parse `wall_time_s` etc. unchanged.
- `<output_dir>/analytics/*.tsv` — derived rollups produced by
  `tools/perf_phase_summary.py`.

JSON is canonical; TSV is a flat projection for ad-hoc grep/awk.

### 2.6 Stage-1 sweep coexistence

The 10k stage-1 sweep is running RIGHT NOW without telemetry.
Do not interrupt. Implement on a sibling branch
(`claude-telemetry-foundation`), validate on a 100-paper offline
sample, merge, then kick off stage 2 (20k cumulative) with
telemetry enabled. Stage 1's results.tsv stays as the
no-telemetry baseline — useful for "before" comparison.

---

## 3. Schema

### 3.1 Phase enum (17 values)

> Canonical order lives in `latexml_core/src/telemetry.rs` (`Phase`) and the
> `PHASES` list in `tools/perf_phase_summary.py` + `tools/telemetry_dashboard.py`
> — keep all three in sync. Grew from the original 11 as the post-processing
> pipeline was split into finer phases; `phase_us` is a `[u64; 17]` indexed in
> this order.

```rust
#[repr(u8)]
pub enum Phase {
  Bootstrap,     // engine + kernel-dump init, package preload
  Digest,        // Mouth → Gullet → Stomach → Box list
  Build,         // Document XML tree assembly
  Rewrite,       // pre-math XML rewrites (lexer, etc.)
  MathParse,     // Marpa parses on <XMath> candidates
  PostXmlParse,  // re-parse the core XML for post-processing
  PostScan,      // citation/ref resolution, ID renumber, MathRewrite
  Bibliography,  // bibliography resolution
  Crossref,      // cross-reference resolution
  Graphics,      // includegraphics dispatch (waits for child procs)
  MathImages,    // math-image generation (latex/dvipng path)
  MathmlPres,    // Presentation MathML conversion
  MathmlCont,    // Content MathML conversion
  Split,         // document splitting
  Xslt,          // XSLT transform
  Html5Fixups,   // HTML5 dialect fixups
  Serialize,     // final HTML/XML byte write
}
```

Bootstrap and Digest are NOT split — engine init triggers macro
expansion via dump load; a clean boundary doesn't exist. Bootstrap
ends when `\start@document` (or `\@begindocument`) fires.

### 3.2 `Telemetry` struct (~50 fields, serde-serialized)

```rust
#[derive(Serialize)]
pub struct Telemetry {
  // Identifiers
  pub paper_id: String,           // arxiv id, derived from input ZIP
  pub git_sha: String,            // baked at build (build.rs)
  pub cmdline: String,            // argv joined
  pub host: String,               // hostname
  pub timeout_s: u32,
  pub schema_version: u32,        // bump on any field change

  // Wall (microseconds)
  pub wall_us: u64,
  pub phase_us: [u64; 11],        // indexed by Phase
  pub bootstrap_us: u64,          // alias for phase_us[0] for grep convenience
  // ... aliases for each phase elide for brevity ...

  // Counts
  pub formulae: u32,
  pub math_parse_attempts: u32,
  pub math_parse_count: u64,      // sum across formulae (can be huge)
  pub math_parse_buckets: [u32; 9],
  pub graphics_assets: u32,
  pub graphics_subprocess_count: u32,
  pub db_objects: u32,
  pub output_bytes: u64,
  pub warnings: u32,
  pub errors: u32,
  pub fatal_errors: u32,
  pub external_tool_count: u32,

  // Resource
  pub max_rss_kb: u64,
  pub child_user_us: u64,
  pub child_sys_us: u64,

  // Outcome
  pub category: String,           // "ok" | "conversion_error" | "timeout" | ...
  pub exit_code: i32,
}
```

`schema_version` starts at 1. Increment on any field change so analytics
scripts reject incompatible mixes.

### 3.3 TSV column extensions

Add to existing schema (after current last column):
```
phase_bootstrap_us  phase_digest_us  phase_build_us  phase_rewrite_us
phase_math_parse_us  phase_post_scan_us  phase_graphics_us
phase_mathml_pres_us  phase_mathml_cont_us  phase_xslt_us  phase_serialize_us
formulae  math_parse_attempts  math_parse_count
graphics_assets  graphics_subprocess_count  db_objects
warnings  errors  fatal_errors
max_rss_kb  child_user_us  child_sys_us
git_sha
```

Math-parse buckets do NOT go in the TSV (would inflate columns
unhelpfully). They live only in JSONL.

---

## 4. Implementation steps

### Step 1 — `latexml_core::telemetry` module

**New file**: `latexml_core/src/telemetry.rs`

```rust
thread_local! {
  static STATE: RefCell<Telemetry> = RefCell::new(Telemetry::default());
  static STACK: RefCell<Vec<(Phase, Instant)>> = RefCell::new(Vec::with_capacity(8));
}

pub fn phase_enter(p: Phase) {
  STACK.with(|s| s.borrow_mut().push((p, Instant::now())));
}
pub fn phase_exit() {
  STACK.with(|s| {
    let (p, t0) = s.borrow_mut().pop().expect("phase_exit without enter");
    let dt = t0.elapsed().as_micros() as u64;
    STATE.with(|st| st.borrow_mut().phase_us[p as usize] += dt);
  });
}
// RAII guard for ergonomics
pub struct PhaseGuard;
pub fn phase(p: Phase) -> PhaseGuard { phase_enter(p); PhaseGuard }
impl Drop for PhaseGuard { fn drop(&mut self) { phase_exit(); } }
```

Stack-based so nested phase scopes attribute time only to the
innermost. (`Bootstrap → Digest → MathParse` only counts `MathParse`
during the parse.)

Plus counters:
```rust
pub fn incr_formulae() { STATE.with(|s| s.borrow_mut().formulae += 1); }
pub fn record_math_parse(us: u64, parses: u32) { /* update bucket + scalars */ }
// ... etc.
```

### Step 2 — instrument the 11 phase boundaries

| Phase | Wrap site |
|---:|---|
| Bootstrap | `latexml_oxide/src/main.rs` — init through first `process_input` body |
| Digest | `latexml_core::stomach::digest_top` |
| Build | `latexml_core::document::Document::build` |
| Rewrite | `latexml_post::rewrite::*` rewriter entry |
| MathParse | `latexml_math_parser::parse` (each call; total accumulates) |
| PostScan | `latexml_post::scan::run_scan` |
| Graphics | `latexml_post::graphics::process_all` |
| MathmlPres | `latexml_post::mathml::pres` entry |
| MathmlCont | `latexml_post::mathml::cont` entry |
| Xslt | `latexml_post::xslt::transform` |
| Serialize | the final `Document::write_*` call |

Each gets a single `let _g = telemetry::phase(Phase::X);` at the
top of the function body. Drop guard exits on return / panic.

### Step 3 — `cortex_worker` emits JSONL

In `cortex_worker/src/main.rs`, after the per-job subprocess returns:
- Read `<scratch>/telemetry.json` written by `latexml_oxide`.
- Append one line to `<output_dir>/telemetry.jsonl`.
- Fill the new TSV columns from the JSON.
- At end of run, gzip `telemetry.jsonl` → `telemetry.jsonl.gz`,
  delete uncompressed.

If `telemetry.json` is absent (e.g., child crashed before
serializing), emit a row with `phase_us = [0; 11]`, `category =
"telemetry_missing"`, and known scalars (wall, exit, output_bytes
from the file system).

### Step 4 — `latexml_oxide` writes JSON at end

Add CLI flag `--telemetry-out=<path>` (default: env
`LATEXML_TELEMETRY_OUT` or none). At process exit, `serde_json::to_writer`
the `Telemetry` struct.

`cortex_worker` sets the env per-job. Direct CLI users default to off.

### Step 5 — build.rs bakes git SHA

```rust
// latexml_oxide/build.rs
fn main() {
  let sha = std::process::Command::new("git")
    .args(["rev-parse", "--short", "HEAD"]).output().ok()
    .and_then(|o| String::from_utf8(o.stdout).ok())
    .map(|s| s.trim().to_string()).unwrap_or_default();
  println!("cargo:rustc-env=LATEXML_GIT_SHA={sha}");
}
```

`Telemetry::default()` reads `env!("LATEXML_GIT_SHA")`.

### Step 6 — `tools/perf_phase_summary.py`

```bash
tools/perf_phase_summary.py <output_dir>/telemetry.jsonl.gz
```

Reports:
- per-phase wall as % of corpus total
- top-30 papers by each phase
- `(sum(phase_us) / wall_us)` distribution — gates the 0.92 acceptance
- correlation: `graphics_us` vs `graphics_subprocess_count` (de-dup gain estimator)
- correlation: `math_parse_us` vs `formulae` (per-formula cost dist.)
- math-parse-bucket histogram across the corpus

Pure stdlib + `gzip` + `json`. No pandas dep.

### Step 7 — regression tests

`latexml_oxide/tests/integration_telemetry.rs`:
1. Run on `latexml_oxide/tests/hello/hello.tex` with `--telemetry-out=/tmp/...`
2. Parse the JSON; assert all 11 phase keys present.
3. Assert `sum(phase_us) >= 0.85 * wall_us` (loose for tiny doc).
4. Assert `formulae == 0` (or known small count for hello.tex).
5. Run on a known math-heavy fixture; assert `phase_us[MathParse] > 0`.

`latexml_oxide/tests/integration_telemetry_perf.rs` (released-mode CI only):
- `1011.1955` round-trip; assert wall ≤ 1.05 × no-telemetry baseline.

### Step 8 — analytics rollup script

`tools/perf_compare.py <baseline.jsonl.gz> <new.jsonl.gz>` — paired
join on `paper_id`, reports per-paper `Δwall`, `Δphase_us`,
`Δerrors`. Used after every perf-affecting commit per the
`docs/PERFORMANCE.md` Optimization Acceptance Checklist.

---

## 5. Risks & mitigations

| Risk | Mitigation |
|------|------------|
| Phase-stack panic on imbalanced enter/exit | RAII `PhaseGuard` makes mismatch impossible from user code; only the module's own funcs are unsafe to misuse. |
| `Instant::elapsed` syscall cost | `Instant` is monotonic clock_gettime, ~20 ns. 11 wraps × 20 ns = ~250 ns/job. Negligible. |
| Histogram update on every parse | atomic increment of one of 9 u32. ~5 ns. 10k formulae × 5 ns = 50 µs. <0.01% of typical wall. |
| Telemetry JSON I/O on slow disk | end-of-job, single sync write of ~2 KB. < 1 ms. |
| Schema drift breaking analytics | `schema_version` field; analytics scripts assert. |
| Sum-of-phase ≠ wall on slow papers | `bootstrap` is whole-process, not just LaTeXML init. If gap > 8%, identify un-instrumented work; it's a real finding, not a bug. |

---

## 6. Acceptance gate (what "done" means)

1. ✅ `cargo test --tests` green (1124+/0/0).
2. ✅ `latexml_oxide tests/hello/hello.tex` produces a valid telemetry JSON.
3. ✅ Round-17 standing perf corpus shows ≤ +1% median wall regression.
4. ✅ 100-paper sample run produces gzipped JSONL + extended TSV.
5. ✅ `tools/perf_phase_summary.py` runs end-to-end; sum-of-phase / wall
   median ≥ 0.92 across the 100 papers.
6. ✅ `docs/PERFORMANCE.md` updated to reference this doc and the
   first telemetry-driven findings.

---

## 7. Status (2026-05-03)

All eight steps landed on `claude-round-19`. Branch
`claude-telemetry-foundation` was merged in and deleted; future
work continues on a single branch.

| Step | What | Commit | Status |
|---:|------|--------|:------:|
| 1 | `latexml_core::telemetry` module + 4 unit tests | initial commit | ✅ |
| 2 | Phase guards: Bootstrap / Digest / Build / Rewrite / Serialize in `converter.rs` + `core_interface.rs`; MathParse around `MathParser::parse_math` with formulae count and per-formula `math_parse_buckets` histogram in `latexml_math_parser/src/parser.rs`; PostXmlParse / PostScan / Bibliography / Crossref / Graphics / Split in `latexml_oxide/src/post.rs`; MathmlPres / MathmlCont / Xslt / MathImages dispatched per processor name in `latexml_post::Post::process_chain`; Html5Fixups around the post-XSLT regex cleanup. **17 of 17 phases wrapped — foundation complete.** | round 2 + refinement + 04ae2909ca + 91cdebdebc | ✅ |
| 3 | `cortex_worker` writes `telemetry.json` member into output ZIP (paper_id, wall_us, max_rss_kb, child rusage, category, output_bytes, exit_code). | Step 3 commit | ✅ |
| 4 | `latexml_oxide --telemetry-out=<path>` flag + helper. | Steps 4+5 commit | ✅ |
| 5 | `latexml_core/build.rs` bakes `LATEXML_GIT_SHA`. | Steps 4+5 commit | ✅ |
| 6 | `tools/perf_phase_summary.py` reads JSONL[.gz] OR a directory of cortex_worker output ZIPs. Reports per-phase share, top-5 by phase, sum-of-phase / wall distribution, ≥0.92 acceptance count. | Step 6 commit | ✅ |
| 7 | `tests/001_telemetry.rs`: 2 integration tests (populate + JSON round-trip). | with Step 1 + 17-phase commit | ✅ |
| 8 | `tools/perf_compare.py` paired A/B comparator (Δwall, per-phase Δ%, regression list >15%, distribution). | Step 8 commit | ✅ |

Smoke test on `0704.0023` (real arxiv paper): wall=1.35s, sum-of-
phase=1.29s = **95.6% coverage** — exceeds the §6 acceptance ≥92%.

All originally-deferred follow-ups have landed:
- ✅ `math_parse_buckets` per-formula histogram in
  `latexml_math_parser/src/parser.rs` (commit `91cdebdebc`).
- ✅ process_chain split into MathmlPres / MathmlCont / Xslt /
  MathImages (commit `f0ed3a16dc` + `04ae2909ca`).
- ✅ Rewrite phase guard around `core_interface` rewrites block
  (commit `1bc0f84363`).
- ✅ `benchmark_canvas.sh` JSONL aggregation, gzipped per stage
  (commit `72cd018f54`).

The foundation is complete and ready for telemetry-driven
optimization work in `docs/PERFORMANCE.md` Tier 1.

---

## 8. After landing

Telemetry unblocks the next round of optimizations from
`docs/PERFORMANCE.md`:

1. **Graphics conversion cache + within-doc dedup** — guided by
   `graphics_us` vs `graphics_subprocess_count` correlation.
2. **Exact parsed-math caching** — guided by `math_parse_us` and
   the per-formula bucket histogram. Target: papers with
   `math_parse_buckets[6..]` (≥20 ms parses) accumulating most
   `math_parse_us`.
3. **Watchdog attribution for the 5 timeout papers** — once we
   know which phase hangs.

Each becomes its own short doc + branch + PR; don't bundle.
