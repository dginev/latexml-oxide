# Performance Optimization Principles

> **Repeatable checklist.** Review before release milestones, after major features, and during periodic optimization passes. Each principle includes rationale and examples.

---

## 1. Avoid String Allocation on Hot Paths

**Principle:** Never use `.to_string()`, `String::from()`, or `format!()` when a string is already in the interner arena. For string *literals*, prefer the call-site-cached `pin!("…")` macro — it interns once per call site and resolves subsequent lookups with a `u32` load. For runtime strings, use `arena::pin(s)`. For comparisons and reads, use the `arena::with*` family.

**Why:** String allocation is one of the most frequent hidden costs. The arena interner exists precisely to avoid repeated heap allocations for the same string. Every `.to_string()` on an interned symbol defeats this purpose. `pin!("foo")` caches the interned `SymStr` in a per-site `thread_local! OnceCell`, avoiding the 30-50 ns intern-table probe on subsequent calls (just a branch + load).

**Examples:**
```rust
// BAD: allocates a new String just to compare
if arena::to_string(sym) == "foo" { ... }

// BETTER: resolve in-place without allocation
if arena::with(sym, |s| s == "foo") { ... }

// BEST (for literals): compare interned SymStrs directly
if sym == pin!("foo") { ... }

// BAD: allocate to store under a literal key
let name = "some_key".to_string();
state::assign_value(&name, ...);

// GOOD: use the sym-keyed state API with pin!
state::assign_value_sym(pin!("some_key"), ..., None);
```

**When to apply:** Any code that runs per-token, per-macro-expansion, per-digest, or per-node. State lookups, token comparisons, CS name checks. The sym-keyed state API (`lookup_bool_sym`/`assign_value_sym`/`with_value_sym`) takes `SymStr` *by value* — `SymStr` is a `u32` wrapper (Copy), so passing by value is cheaper than borrowing.

**`pin!` vs `arena::pin`:**
- `pin!("literal")` (macro) — per-site OnceCell cache, for string literals. Cheapest.
- `arena::pin(runtime_str)` (function) — single intern on a dynamic `&str` / `String`.
- Both return the same `SymStr` on equal input; the macro is just a fast path for known-at-compile-time strings.

---

## 2. Minimize `.clone()` — Borrow or Reorder Instead

**Principle:** Avoid `.clone()` for data that can be borrowed or is only needed for a short instruction scope. Rearrange code so that short flag-like methods run first and assign to local variables, preventing Rust ownership conflicts without cloning. For more complex cases, consider borrowing from first principles, using the string interner, or a `Cow<>` copy-on-write approach.

**Why:** Cloning complex structures (Tokens, Vec, HashMap) is expensive. Many clones exist only to satisfy the borrow checker, not because the data is truly needed in two places. Reordering operations or extracting small values first often eliminates the need entirely.

**Examples:**
```rust
// BAD: clone to avoid borrow conflict
let tokens = state.get_tokens().clone();
let flag = state.is_active();  // borrows state again
process(tokens, flag);

// GOOD: extract flag first (short borrow), then get tokens
let flag = state.is_active();
let tokens = state.get_tokens();  // no conflict now
process(tokens, flag);

// BAD: clone a large structure for a single read
let map = self.entries.clone();
if map.contains_key("foo") { ... }

// GOOD: borrow directly
if self.entries.contains_key("foo") { ... }

// ACCEPTABLE: Cow for conditional mutation
fn process(input: Cow<str>) -> Cow<str> {
    if needs_change(&input) { Cow::Owned(transform(&input)) }
    else { input }
}
```

**When to apply:** Every `.clone()` call is a candidate for review. Prioritize clones inside loops, hot paths, and clones of `Tokens`, `Vec<Token>`, `HashMap`, or `String`. Single-field scalar clones (bool, u32, Option<usize>) are fine.

---

## 3. Run Clippy and Study Lint Neighborhoods

**Principle:** Run `cargo clippy` and apply all fixes. Then study code in the vicinity of each lint — performance issues often cluster in clumps of poorly designed code. A clippy lint is a signal to review the surrounding 20-50 lines.

**Why:** Clippy catches redundant allocations, unnecessary conversions, suboptimal patterns (e.g., `map().flatten()` → `flat_map()`, `.iter().cloned().collect()` → `.to_vec()`). But more importantly, lint locations correlate with code that was written hastily. Fixing the lint often reveals adjacent inefficiencies that clippy doesn't flag.

**How to run:**
```bash
cargo clippy --workspace -- -W clippy::perf -W clippy::redundant_clone
```

**When to apply:** Before every release milestone. After major feature work. As a periodic sweep (monthly or quarterly). Focus on `clippy::perf` and `clippy::redundant_clone` warning groups.

---

## 4. Minimize Math Parse Ambiguity

**Principle:** Reduce the number of successful math parses while ensuring at least one survives. This is the highest-leverage optimization for documents with heavy math. Three complementary tools, ordered by effectiveness:

**Tool 1 — Grammar restructuring (highest impact):**
Restrict grammar category scopes, reduce the number of paths an expression can take toward the start category. Fewer ambiguous derivations = exponentially fewer parse trees.

**Tool 2 — Semantic pruning (high impact):**
Return `Err` from `semantics.rs` action functions for nonsensical constructions. This prunes during tree construction, before full parse completion. Examples: reject `f(x)(y)` as double-application when `f` isn't higher-order, reject mismatched fence pairs, reject empty operator sequences.

**Tool 3 — Pragmatic rules (representation quality):**
Add `Pragma` rules that check mathematical conventions (e.g., consistency of variable use, operator precedence expectations). Less useful for raw performance (all parses must complete first) but valuable for selecting the best parse from surviving candidates.

**Why:** The Marpa parser produces all valid derivations. For ambiguous math like `a+b*c/d`, the number of parse trees can be combinatorial. Each surviving parse is a full tree that consumes memory and CPU. Reducing parses from 50 to 3 can be a 10-20x speedup on math-heavy documents.

**When to apply:** When profiling shows math parsing as a bottleneck (common for documents with 100+ formulas). During grammar design reviews. When adding new math operators or constructs.

---

## 5. External-Process Discipline (fork-exec is not free)

**Principle:** Every external tool invocation (`gs`, `convert`, `inkscape`,
`kpsewhich`, `pdfcrop`, …) costs 10–50 ms ambient before any real work,
plus dynamic-linker and font-cache initialization for `gs` and
`convert`. Coalesce, dedup, and cache *before* spawning, not after.

**Why:** Telemetry across 190k arxiv documents shows the `graphics`
phase at **36.5 % of total wall** — the single largest band, larger
than `digest` (20.3 %) and `math_parse` (17.0 %). The bulk of that
budget is `gs`/`convert`/`inkscape` fork-exec + child runtime, not
work we can speed up by tuning Rust. Even a modest hit rate on a
content-keyed cache (e.g. 30 %) translates to ~10 % off total wall
across the corpus.

**Examples:**
```rust
// BAD: spawn one convert per asset, regardless of duplicates
for asset in assets {
    Command::new("convert").args(...).status()?;
}

// BETTER: coalesce by source identity within a document
let mut by_key: HashMap<CacheKey, Vec<&Asset>> = HashMap::new();
for asset in &assets { by_key.entry(asset.cache_key()).or_default().push(asset); }
for (key, group) in &by_key {
    let result = run_convert_once(group[0])?;
    for a in group { a.write_result(&result); }
}

// BEST: persistent on-disk cache keyed by source-hash + page + DPI + dest
let key = blake3::hash([source_bytes, page.to_le_bytes(), dpi.to_le_bytes(),
                        dest_kind.as_bytes(), opts.canonical()].concat());
if let Some(cached) = cache.get(&key) { return Ok(cached); }
let fresh = run_convert(...)?;
cache.put(&key, &fresh);
```

**When to apply:** Any time you would call `Command::new(...)` inside a
document conversion. The cost calculus is dominated by spawn-count
× per-spawn overhead, not by per-call CPU once the child is running —
so dedup/coalesce wins are larger than convert-flag tuning.

**Cache-key correctness checklist:**
- Include all inputs that change the output: source bytes (hash), page
  index, target DPI, target format, and any flags that influence
  rendering (background, antialiasing, density).
- Exclude environment-derived noise (timestamps, tmpdir paths).
- Exclude things that must invalidate: tool version (when fixing a
  rendering bug, bump a `cache_namespace` constant rather than relying
  on hash invalidation).

---

> **Adding new principles:** Number sequentially. Include: principle statement, **Why** (rationale), **Examples** (good vs bad code), **When to apply** (scope/triggers).

---

## Current Phase Distribution (190k aggregate)

> **Source of truth:** 10 stages × 10k arxiv documents (189,991 jobs total)
> from `/home/deyan/data/stage{01..10}_100k_html/telemetry.jsonl.gz`,
> collected 2026-05-02..03 with the per-job `cortex_worker` telemetry
> wired in §P0. Sum-of-phases / wall = **97.78 %** (above the 92 % gate
> from `docs/TELEMETRY.md`).

| metric | value |
|---|---:|
| jobs measured | 189,991 |
| total wall | 545,073 s (151.4 h) |
| sum of phases | 532,968 s (97.78 % of wall) |
| mean wall / job | 2.87 s |
| total formulae | 39.16 M (mean 206 / job) |
| total parse attempts | 39.06 M |
| total parses surviving | 45.84 M (1.17× attempts → 17 % over-parse rate) |
| max RSS observed | 1,692 MB |

Phase breakdown (sorted by share of wall):

| phase | sum_s | %wall | mean / job |
|---|---:|---:|---:|
| **graphics** | 199,020 | **36.51 %** | 1,047 ms |
| **digest** | 110,495 | **20.27 %** | 582 ms |
| **math_parse** | 92,613 | **16.99 %** | 488 ms |
| **build** | 62,836 | **11.53 %** | 331 ms |
| xslt | 39,264 | 7.20 % | 207 ms |
| mathml_pres | 9,636 | 1.77 % | 51 ms |
| post_xml_parse | 4,429 | 0.81 % | 23 ms |
| serialize | 4,274 | 0.78 % | 23 ms |
| rewrite | 3,682 | 0.68 % | 19 ms |
| crossref | 2,116 | 0.39 % | 11 ms |
| post_scan | 1,978 | 0.36 % | 10 ms |
| mathml_cont | 1,764 | 0.32 % | 9 ms |
| html5_fixups | 554 | 0.10 % | 3 ms |
| bibliography | 308 | 0.06 % | 2 ms |

**Top four bands account for 85 % of wall.** All other phases
combined (xslt + mathml + post_*) are 12 %.

**Outstanding telemetry gaps** (counters present in the schema but
emitting zero across stages 01–10 — wrappers not wired or counter not
incremented):
- `graphics_assets`, `graphics_subprocess_count` — **wired
  2026-05-03** (`latexml_post/src/graphics.rs` + `set_graphics_assets`
  / `add_graphics_subprocess` in `latexml_core::telemetry`). Worker
  threads tally a shared `AtomicU32` and merge into telemetry after
  `std::thread::scope` joins (per-thread `thread_local!` STATE is
  otherwise discarded on worker exit). Stage 11+ should show non-zero
  values; verify on next sweep.
- `external_tool_count`, `db_objects` — still zero across all stages.
- `math_images_us` and `split_us` are also zero across all stages and
  may simply not run on this corpus; verify before assuming they're
  bugs.

## Slow-Tail Snapshot (legacy framing — kept for history)

Current data source: `/home/deyan/data/*/*.tsv`, checked 2026-05-02. Only
`/home/deyan/data/100k_noproblem_sandbox_html/results.tsv` has a
`wall_time_s` column and rows over 10 seconds. The 600-row sample has no
`>10s` jobs. The `10k_failures_April30` TSVs track error counts only, so they
cannot be used for wall-clock slow-tail analysis.

Reproduce the full slow set:

```bash
awk -F '\t' 'NR==1{for(i=1;i<=NF;i++) ix[$i]=i; print; next}
  $(ix["wall_time_s"])+0 > 10 {print}' \
  /home/deyan/data/100k_noproblem_sandbox_html/results.tsv
```

Summary of the 100k HTML run:

| metric | value |
|--------|------:|
| total jobs | 100,000 |
| jobs over 10s | 1,862 |
| slow-tail share | 1.862% |
| slow-tail wall total | 27,698s |
| slow-tail average | 14.88s |
| slow-tail median | 12.5s |
| slow-tail p90 | 20.7s |
| slow-tail p95 | 26.9s |
| slow-tail p99 | 42.9s |
| max | 120.1s |

Runtime buckets:

| wall time | jobs |
|-----------|-----:|
| 10-15s | 1,347 |
| 15-20s | 310 |
| 20-30s | 139 |
| 30-60s | 56 |
| 60s+ | 10 |

Most slow jobs are successful conversions, not failure cases:

| category | jobs |
|----------|-----:|
| ok | 1,841 |
| conversion_error | 11 |
| timeout | 5 |
| abort | 3 |
| conversion_fatal | 2 |

Log-derived hints from the 1,862 slow rows:

| signal | jobs |
|--------|-----:|
| 20+ graphics jobs | 559 |
| 50+ graphics jobs | 123 |
| output size >= 10 MB | 122 |
| output size >= 25 MB | 16 |
| 1000+ math formulae | 348 |
| 2500+ math formulae | 49 |
| DBStatus >= 10,000 objects | 64 |
| Xy-pic log signal | 51 |
| token-limit recovery | 1 |

The slow tail has at least three distinct families:

1. **Graphics/output-heavy successful jobs.** 613 slow rows have either 20+
   graphics jobs or output >= 10 MB. Examples include `0809.3849` (34.0s,
   228 graphics, 35.8 MB), `0908.3201` (57.0s, 70 graphics, 29.5 MB),
   `1003.0368` (66.5s, 30.4 MB), and `0803.4343` (54.5s, 41.9 MB).

2. **Math/large-document successful jobs.** 373 slow rows have either 1000+
   formulae or 10,000+ DB objects. Examples include `astro-ph0204009` (114.8s,
   2795 formulae, 47,610 DB objects), `0911.0884` (39.0s, 12,446 formulae),
   `astro-ph0401354` (40.3s, 11,289 formulae), and `astro-ph0508017` (39.2s,
   98,153 DB objects).

3. **Failure/control-flow outliers.** There are only 10 rows at 60s+, and half
   are 120s timeouts. `0903.3465` is a 75.5s `conversion_fatal` with Xy-pic and
   token-limit recovery; this is not a normal hot path and should be fixed as a
   bounded recovery/timeout problem.

Top slow rows:

| paper | wall | category | output | formulae | graphics | DB objects |
|-------|-----:|----------|-------:|---------:|---------:|-----------:|
| `hep-ph0102035` | 120.1s | timeout | 0 | 0 | 0 | 0 |
| `math0608653` | 120.1s | timeout | 0 | 0 | 0 | 0 |
| `0705.3903` | 120.1s | timeout | 0 | 0 | 0 | 0 |
| `0907.3579` | 120.1s | timeout | 0 | 31 | 0 | 211 |
| `1001.3715` | 120.0s | timeout | 0 | 233 | 9 | 1,185 |
| `astro-ph0204009` | 114.8s | ok | 581 KB | 2,795 | 6 | 47,610 |
| `hep-th0109082` | 78.5s | ok | 137 KB | 387 | 0 | 14,805 |
| `0903.3465` | 75.5s | conversion_fatal | 834 B | 0 | 0 | 0 |
| `hep-ph0107113` | 73.6s | ok | 113 KB | 101 | 4 | 393 |
| `1003.0368` | 66.5s | ok | 30.4 MB | 141 | 4 | 264 |

## Improvement Plan

The plan is ordered by **share of total wall** as measured across the
189,991-job aggregate above. Optimization work that targets a sub-1 %
band must justify itself; the four headline bands (graphics, digest,
math_parse, build) carry 85 % of wall and are the only places where
single-digit-percent wins on aggregate are realistic.

### P0: Make every slow job phase-attributed — DONE 2026-05-03

Implemented per the contract in [`docs/TELEMETRY.md`](TELEMETRY.md).

Per-job phase wall + counts now emitted by `cortex_worker` as a
single-line `telemetry.json` member of each output ZIP, aggregated by
`tools/benchmark_canvas.sh` into `<output_dir>/telemetry.jsonl.gz`,
and consumed by `tools/perf_phase_summary.py` (per-phase share, top-N
papers, distribution) and `tools/perf_compare.py` (paired A/B Δwall,
per-phase Δ%, regression list).

Phases emitted (14 of 17 wrapped today; remaining 3 — Html5Fixups
plus per-formula `math_parse_buckets` and `MathImages` — deferred):
Bootstrap, Digest, Build, Rewrite, MathParse, PostXmlParse, PostScan,
Bibliography, Crossref, Graphics, Split, MathmlPres, MathmlCont,
Xslt, Serialize.

Acceptance check on `0704.0023`: sum-of-phase / wall = 95.6% (>=92%
gate). Confirms first telemetry-driven finding: `graphics` already
visible at 38% of wall on a single arxiv paper, motivating the P1
graphics conversion-cache work below.

### P1: Graphics phase (36.5 % of wall — top lever) — IN PROGRESS

This single phase outweighs every other Rust-internal phase combined.
Work items, in order:

1. **Wire per-asset graphics telemetry** — DONE 2026-05-03.
   `graphics_assets` (set on main thread) and
   `graphics_subprocess_count` (worker `AtomicU32` merged via
   `add_graphics_subprocess` after `std::thread::scope` joins). Need a
   single 10k sweep to confirm non-zero values, then we can speak
   quantitatively about subproc-per-paper distribution.
2. **Intra-document dedup** — already present in
   `latexml_post/src/graphics.rs:888-962` via `convert_job_ids`
   HashMap keyed on `(source, page, options)`. Mirrors Perl's
   `$doc->cacheLookup`. No work needed.
3. **Persistent on-disk cache** — *next concrete win*. Keyed on
   `(blake3(source_bytes), page, dest_type, density, options_canonical)`
   stored under `$LATEXML_OXIDE_CACHE_DIR`
   (default `$XDG_CACHE_HOME/latexml-oxide/graphics/`). Hash namespace
   bumps when convert / inkscape version changes (read once at startup
   via `--version`). Add `graphics_cache_hits` / `_misses` counters to
   the telemetry schema before landing the cache.
4. **Output-size validation set** — `0809.3849`, `0908.3201`,
   `1003.0368`, `0803.4343`, `0907.4282`; compare output bytes, image
   count, missing-image count, and wall time before/after.

Do not add more global ImageMagick/Inkscape heuristics until per-asset
telemetry shows which source types are actually slow and which path is
correct for each.

### P1: Digest + Build (31.8 % of wall — pure-Rust hot path)

`digest` (20.27 %) + `build` (11.53 %) is *almost as large* as the
graphics phase, but consists entirely of Rust code we control. There
is no external-process slack to recover; wins come from the Principle
1–3 hygiene already documented.

- Run `cargo clippy --workspace -- -W clippy::perf -W clippy::redundant_clone`
  and apply.
- Profile a digest-heavy outlier from telemetry top-5 (`0911.3024`
  76 % of paper wall, `0909.4601` 66 %) under release `perf`. Look for
  `arena::*`, `Tokens::*`, and HashMap rehashes.
- Audit hot `.clone()` sites on Token / Tokens / Vec<Token> per
  Principle 2.
- Don't over-index on micro-optimization — the digest phase reflects
  expansion volume, and large papers are inherently expensive.

### P1: Math and large-document jobs (17.0 % of wall)

The 190k aggregate shows **39.06 M parse attempts producing 45.84 M
surviving parses** (1.17×) — a 17 % over-parse rate. That is precisely
the lever Principle 4 targets. Math parsing is also the band most
likely to interact with output structure, so any change must clear the
acceptance checklist below.

- Run `LATEXML_PARSE_AUDIT=1` on telemetry top-5 by `math_parse`:
  `0912.4453` (60.6 % of paper wall), `1001.5072` (62.5 %),
  `0912.5528` (35.6 %), `0908.4292` (46.8 %), `0909.3532` (62.0 %).
  Rank by total parse time, attempt count, and repeated token sequence.
- Add exact parsed-math caching only where the audit shows repeated
  identical normalized token streams under equivalent math context.
- Prefer grammar/semantic pruning for demonstrably invalid ambiguity
  families: malformed operator chains, impossible double application,
  invalid script targets, empty/mismatched fences, and nonsensical
  differential/operator combinations.
- Track MathML count and structural output metrics before/after; a
  speedup that changes math structure without an explicit compatibility
  decision is a regression.

Do not start with broad direct-XM builders for many common shapes.
They are easy to make fast and hard to make LaTeXML-equivalent. Treat
them as a later, per-shape optimization only after audit data,
fixtures, and fallback behavior are clear.

### P1: Failure/control-flow outliers

The small 60s+ set should be handled separately from performance hot paths:

- Re-run the five 120s timeouts with phase telemetry and captured last log
  event. Timeouts with no formula/graphics counts likely die before normal
  reporting and need better watchdog attribution.
- Treat `0903.3465` as an Xy-pic/token-limit recovery bug. The goal is bounded
  recovery and a clear fatal result, not making that path fast.
- Add timeout rows to the regression corpus only after the failure mode is
  minimized; otherwise they will make performance signals noisy.

### P2: Allocation and startup cleanup

Keep interner, clone, and package-loading work profile-driven. The earlier
`.to_string` sweep was useful, but the current >10s data points first at
graphics, math/document scale, and bounded failure handling. Candidate cleanup
areas remain `*_sym` state accessors, `Tokens` conversions, deep `Stored` /
`Tokens` copies, package lookup caching, and dump/package loading, but only
after a slow-tail or sentinel profile shows them on the hot path.

### Optimization Acceptance Checklist

Before merging a performance change:

1. Record release-mode before/after numbers for the standing corpus.
2. Include one targeted benchmark for the exact suspected bottleneck.
3. Compare output status and lightweight structural quality metrics.
4. Report wall time, user/sys time, max RSS, and phase timings.
5. State the expected workload boundary and any fallback path.
6. Keep the change easy to disable if it relies on a heuristic.

For math-parser changes, also record parse count distribution, total math parse
time, MathML/XMath count, and any formulas that use a cache or another
nonstandard path. Structural math output must be reviewed on math-heavy fixtures
before the change is treated as a win.

---

## Standing Performance Corpus

The following papers form the regression corpus for engine / arena / gullet /
marpa changes. Run with a direct idle-serial invocation (no `cortex_worker`,
no parallel load):

```bash
/home/deyan/git/latexml-oxide/target/release/latexml_oxide \
  --preload=ar5iv.sty \
  --path=/home/deyan/git/ar5iv-bindings/bindings \
  --dest=/tmp/out.html --timeout=60 <main.tex>
```

Papers live as zipped sources under `/home/deyan/data/10k_sandbox/<id>.zip`;
`complex/si.tex` is in-tree at `latexml_oxide/tests/complex/si.tex`. The
helper script `tools/run_perf_corpus.sh` unzips each into a tmpdir and
records `exit` + wall-clock.

### Round-17 baseline (2026-04-21)

| paper          | main.tex                           | dt (s) | class / note                    |
|----------------|------------------------------------|-------:|----------------------------------|
| 0906.1883      | VanNeervenWeis_final_version.tex   |  0.67  | aa, birkmult (stub-guard fix)   |
| 1011.1955      | 1011.1955.tex                      |  3.49  | amsart `\DeclareMathSymbol`     |
| 1009.1431      | 1009.1431.tex                      |  2.11  | —                                |
| 1008.4386      | genealogy_final_CPAM.tex           |  2.59  | —                                |
| 0909.2656      | main.tex                           |  2.74  | —                                |
| 0911.4739      | lhc7.tex                           |  5.04  | JHEP — over 3s                  |
| 1005.1610      | OAM100507.tex                      |  7.38  | iopart — over 3s                |
| 0803.0466      | IIpaper15.tex                      |  2.31  | aa                               |
| complex/si.tex | si.tex                             |  2.06  | siunitx-heavy                    |

### Round-17 refresh (2026-04-21, after `aa3c7c1bb` graphics parallelism)

| paper          | main.tex                           | dt (s) | Δ vs baseline |
|----------------|------------------------------------|-------:|--------------:|
| 0906.1883      | VanNeervenWeis_final_version.tex   |  0.69  |   +3% (noise) |
| 1011.1955      | 1011.1955.tex                      |  3.49  |         flat  |
| 1009.1431      | 1009.1431.tex                      |  2.11  |         flat  |
| 1008.4386      | genealogy_final_CPAM.tex           |  2.69  |   +4% (noise) |
| 0909.2656      | main.tex                           |  1.95  |          −29% |
| 0911.4739      | lhc7.tex                           |  1.71  |          −66% |
| 1005.1610      | OAM100507.tex                      |  2.57  |          −65% |
| 0803.0466      | IIpaper15.tex                      |  1.35  |          −42% |
| complex/si.tex | si.tex                             |  2.22  |   +8% (noise) |

All Tier A papers now under 3.5 s — round-17 outliers resolved.
Commit `aa3c7c1bb` parallelises the Graphics phase's `convert` subprocess
fork-execs via `std::thread::scope` (no new dependency) with a worker
cap of `min(available_parallelism, 8)`.

### Round-17 second refresh (2026-04-21, after .to_string sweep)

Cumulative 61-site `arena::to_string` → `arena::with` / closure refactor
across 21 files (commits `741809e6e` through `7a5433cd4`). The sweep
targets wasteful `String` allocations whose resolved content was used
for a single comparison, prefix check, or passed as `&str` — replacing
each with a closure that resolves the interned SymStr in place.

| paper          | main.tex                           | dt (s) | Δ vs prev |
|----------------|------------------------------------|-------:|----------:|
| 0906.1883      | VanNeervenWeis_final_version.tex   |  0.70  |     flat  |
| 1011.1955      | 1011.1955.tex                      |  3.48  |     flat  |
| 1009.1431      | 1009.1431.tex                      |  2.09  |     flat  |
| 1008.4386      | genealogy_final_CPAM.tex           |  2.61  |      −3%  |
| 0909.2656      | main.tex                           |  1.96  |     flat  |
| 0911.4739      | lhc7.tex                           |  1.67  |      −2%  |
| 1005.1610      | OAM100507.tex                      |  2.42  |      −6%  |
| 0803.0466      | IIpaper15.tex                      |  1.25  |      −7%  |
| complex/si.tex | si.tex                             |  2.03  |      −9%  |

complex/si.tex is the gullet-bound workload where the arena churn
matters most — consistent with the session 116-117 finding that
arena-interner probes dominate. Tier A papers are math/figure-bound
and benefit less per site, but the accumulated saving is still
visible.

### 2026-04-30 refresh and profiling notes

Fresh release CLI build (`cargo build --release --bin latexml_oxide`) and
idle-serial corpus run:

| paper          | main.tex                           | dt (s) | current read                 |
|----------------|------------------------------------|-------:|------------------------------|
| 0906.1883      | VanNeervenWeis_final_version.tex   |  0.76  | small math-heavy control     |
| 1011.1955      | 1011.1955.tex                      |  3.88  | math-parser bound            |
| 1009.1431      | 1009.1431.tex                      |  2.19  | under 3 s                    |
| 1008.4386      | genealogy_final_CPAM.tex           |  3.17  | near-threshold outlier       |
| 0909.2656      | main.tex                           |  2.56  | under 3 s                    |
| 0911.4739      | lhc7.tex                           |  2.74  | under 3 s                    |
| 1005.1610      | OAM100507.tex                      |  4.37  | post/graphics bound          |
| 0803.0466      | IIpaper15.tex                      |  2.30  | under 3 s                    |
| complex/si.tex | si.tex                             |  1.28  | no longer current bottleneck |

Phase splits on representative outliers:

| paper     | mode                                    | wall | user CPU | max RSS |
|-----------|-----------------------------------------|-----:|---------:|--------:|
| 1005.1610 | XML only                                | 0.88s |   0.83s | 240 MB  |
| 1005.1610 | HTML                                    | 3.14s |  12.38s | 235 MB  |
| 1005.1610 | HTML, `--nomathparse`                  | 2.69s |  13.68s | 176 MB  |
| 1005.1610 | HTML, `--graphics-svg-threshold-kb 200` | 3.67s |  14.30s | 235 MB  |
| 1011.1955 | XML only                                | 3.60s |   3.42s | 533 MB  |
| 1011.1955 | XML, `--nomathparse`                   | 1.28s |   1.20s | 295 MB  |

Hardware-counter profiling was enabled after the first pass. Release `perf`
samples were collected from an unstripped release binary
(`CARGO_PROFILE_RELEASE_STRIP=none cargo build --release --bin latexml_oxide`);
normal release builds still follow the checked-in profile settings.

`perf stat -d` split the two current outlier families cleanly:

| paper     | mode     | elapsed | CPUs | read |
|-----------|----------|--------:|-----:|------|
| 1011.1955 | XML      | 3.78s   | 0.99 | single-core math/body conversion; backend pressure visible |
| 1005.1610 | HTML     | 2.83s   | 3.92 | parallel external graphics/post-processing dominates |

Flat release samples on `1011.1955` put the main XML cost back in Marpa:
`marpa_r_earleme_complete` (7.45%), `postdot_items_create` (6.59%),
`bv_scan` (2.48%), `marpa_b_new` (2.42%), `transitive_closure` (2.08%),
`marpa_g_precompute` (1.43%), and `_marpa_avl_probe` (1.14%). Alloc/free
and libxml/XPath work are the next visible bands. With `--nomathparse`, the
Marpa band disappears and the remaining samples are libxml wrapper/node
access, allocator traffic, and kpathsea package lookup/hash setup.

Flat release samples on `1005.1610` HTML mostly land in child processes:
Ghostscript (`gs`) and ImageMagick `convert` spend visible cycles in
`png_write_row`, zlib, libc allocation/string routines, and Ghostscript
internals. Rust-side Marpa functions are below 1% flat in that run. The earlier
debug Callgrind sample on `0906.1883` was consistent with the release Marpa
symbols, but should remain directional only.

### Regression trigger

Any corpus entry drifting wall-clock **> +15%** from its last recorded
baseline between commits is a regression signal. Record the new row in a
dated sub-heading here (don't overwrite); keep the old baseline so the drift
is visible in history.

### Validation: pathological-for-ImageMagick PDFs (issue #902)

The vector-SVG graphics path (opt-in via `--graphics-svg-threshold-kb N`,
round 17) is validated against `fig8.pdf` from
[brucemiller/LaTeXML#902](https://github.com/brucemiller/LaTeXML/issues/902)
(attached from arxiv:1807.01606), a 41 KB vector-authored PDF that
`convert` rasterises absurdly slowly.

End-to-end through `latexml_oxide --post` on a minimal 4-line document
containing only `\includegraphics{fig8.pdf}`:

| path                                | Graphics phase | total wall |
|-------------------------------------|---------------:|-----------:|
| default (ImageMagick `convert`)     |       32.4 s   |    32.4 s  |
| `--graphics-svg-threshold-kb 200`   |       0.25 s   |     0.3 s  |
| **speedup**                         |     **130×**   |   **111×** |

arxiv:1807.01606 has 15 such PDFs; serial convert would be ~8 minutes
(likely times out). Inkscape path: 15 × 0.25 s ≈ 4 s.

Regression coverage: `test_vector_svg_pathological_convert_case` in
`latexml_post/tests/integration.rs` asserts the inkscape path completes
in <5 s on this fixture (silently skipped when inkscape is absent from
PATH).
