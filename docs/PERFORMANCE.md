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

> **Adding new principles:** Number sequentially. Include: principle statement, **Why** (rationale), **Examples** (good vs bad code), **When to apply** (scope/triggers).

---

## Critical Performance Evaluation

The current optimization work is directionally strong: the codebase already has
an interner-aware style guide, a standing corpus, release-mode profiling notes,
and specific evidence for math parsing and graphics/post-processing as the two
dominant outlier families. To make the project world class, the next step is to
treat performance as an engineering contract with budgets, attribution, and
regression ownership, not as a sequence of useful local sweeps.

### What "world class" should mean

1. **Strict service quality parity.** Optimizations must preserve LaTeXML
   semantics, diagnostics, output structure, resource handling, and failure
   behavior unless a deliberate compatibility decision is documented. Fast but
   non-equivalent conversions are regressions.

2. **Predictable tail latency.** Median speedups are not enough. Track p50, p90,
   p99, timeout rate, peak RSS, and output status across representative arXiv
   classes. The product goal is fewer slow and failed papers, not just a faster
   average paper.

3. **Phase-attributed budgets.** Every run should be explainable by phase:
   loading, digestion, math parsing, rewriting, graphics, XSLT, serialization,
   and external tools. A global wall-clock number is useful for release gating
   but too blunt for engineering decisions.

4. **Optimization acceptance requires proof.** Each substantial optimization
   needs a before/after benchmark on the corpus, one targeted stress case, and
   an output-quality check. If the win is workload-specific, document the
   workload boundary.

5. **Fast paths must be conservative.** Direct builders, caches, and skipped
   work should have narrow guards and fall back to the canonical path. The
   default should be correctness-first with telemetry proving when a shortcut
   was used.

### Critical Gaps

1. **No automated performance gate.** The corpus is documented but not yet a
   hard CI signal. Add a `perf-corpus` job that stores JSONL artifacts
   containing git SHA, command line, wall time, user/sys time, max RSS, phase
   timings, warning/error counts, and output hash. Fail only on clear regressions
   initially; trend everything.

2. **Insufficient phase telemetry.** The current notes rely on manual `perf`
   runs and selected phase splits. Add always-available low-overhead timers
   around major phases and expensive subphases: package loading, file lookup,
   gullet expansion, stomach digestion, Marpa parse, rewrite rules, graphics
   probes/conversions, XSLT, and archive I/O.

3. **Math parser remains the highest-risk hot path.** `1011.1955` shows that
   Marpa can dominate both wall time and RSS. The highest leverage work is not
   more generic allocation cleanup; it is ambiguity control and conservative
   pre-Marpa handling for repeated unambiguous shapes.

4. **External graphics work needs job-level accounting.** The post phase can be
   dominated by `gs`, ImageMagick, and Inkscape. Track per-asset command, source
   type, dimensions/page, output type, elapsed time, exit status, and cache hit.
   Without that, heuristics will look good on one fixture and regress another.

5. **Startup and package-loading costs matter for batch service.** Single-paper
   CLI runs hide opportunities from persistent workers, warmed package state,
   shared resources, and lookup caches. Measure cold vs warm worker conversion
   separately.

6. **Memory budgets are under-specified.** Peak RSS is recorded in selected
   notes but not budgeted. Add limits by workload class and track top allocation
   families with heap profiling for math-heavy and graphics-heavy cases.

7. **Output quality needs automated comparison.** Performance wins should be
   tied to output invariants: status severity, missing citation count, missing
   image count, MathML count, node count, link/resource count, and selected
   visual smoke tests. This protects service quality while allowing aggressive
   optimization.

### Priority Order

**P0: Build the measurement system.** Add structured phase telemetry, corpus
automation, and output-quality summaries. This is prerequisite work: without it,
every optimization remains partly anecdotal.

**P0: Stabilize the current outliers.** Keep `1011.1955` as the math/parser
sentinel and `1005.1610` as the graphics/post sentinel. Add at least one
bibliography-heavy, one macro-expansion-heavy, and one large-archive paper to
avoid overfitting to the current Tier A set.

**P1: Reduce Marpa work.** Use parse audits to identify repeated shapes, add
strictly guarded direct-XM builders for simple unambiguous math, then reduce
grammar ambiguity where the audit shows combinatorial parse families.

**P1: Make graphics conversion observable and reusable.** Cache conversions by
source identity, page, DPI, destination type, and relevant options. Coalesce
identical jobs within a document before spawning external tools. Keep per-asset
telemetry so cache behavior and slow tools are obvious.

**P1: Remove avoidable allocator pressure.** Continue interner and clone cleanup
only where profiles show it. Favor APIs that pass `SymStr`, slices, and borrowed
tokens through hot paths instead of allocating temporary `String`/`Vec` values.

**P2: Optimize service deployment.** For production workers, evaluate warmed
state, pooled resource discovery, persistent temp directories, bounded external
tool parallelism, CPU isolation, and timeout policy. These can beat code-level
micro-optimizations at service scale.

### Optimization Acceptance Checklist

Before merging a performance change:

1. Record release-mode before/after numbers for the standing corpus.
2. Include one targeted benchmark for the exact suspected bottleneck.
3. Compare output status and lightweight structural quality metrics.
4. Report wall time, user/sys time, max RSS, and phase timings.
5. State the expected workload boundary and any fallback path.
6. Keep the change easy to disable if it relies on a heuristic.

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

### Future directions

1. **Math fast paths before Marpa.** `1011.1955` spends about 2.3 s and 238 MB
   RSS in math parsing. Its parse audit is dominated by simple repeated
   shapes (`p(n)`, `eta(z)`, comma lists, simple subscripted function calls).
   Add conservative direct-XM builders for unambiguous common shapes before
   `parse_marpa`, then validate against the standing corpus and math tests.
   Release `perf` now confirms Marpa recognizer/precompute symbols as the
   largest XML-only hot band.

2. **Grammar ambiguity reduction.** Release `perf` and debug Callgrind both
   point at Marpa recognizer/precompute internals even when tree enumeration is capped.
   Prioritize grammar changes around function application, juxtaposition,
   comma lists, scripts, and `ATOM` boundaries. Use `LATEXML_PARSE_AUDIT=1`
   to select repeated patterns before changing rules.

3. **Graphics conversion cache/telemetry.** `1005.1610` is post/graphics
   bound, with release samples landing mostly in `gs` / `convert` PNG and zlib
   work. The small-vector SVG path is workload-specific: it is excellent for
   pathological vector PDFs but slower on this mixed/raster paper. Add per-asset
   timing and cache conversion results by source path, page, DPI, destination
   type, and graphics options before adding more heuristics. Coalesce identical
   conversion jobs within a document before spawning external tools, then make
   the hit/miss behavior visible in the phase report.

4. **Package/dump and libxml residual cost.** After `--nomathparse`, the
   remaining `1011.1955` samples are not one Rust loop; they are libxml wrapper
   access, allocator traffic, kpathsea package lookup, and dump/package loading
   call paths. Treat this as a startup/batch-throughput direction: measure
   whether preloaded state snapshots, package lookup caching, or lower-allocation
   dump parsing help before optimizing individual call sites.

5. **Allocation cleanup stays profile-driven.** The earlier `.to_string`
   sweep paid off, but further arena/state/Tokens cleanup should target a
   measured hot band. Candidate APIs remain `*_sym` state accessors,
   `Tokens` numeric/string conversions, and deep `Stored` / `Tokens` copies.

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
