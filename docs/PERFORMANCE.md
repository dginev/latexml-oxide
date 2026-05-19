# Performance Optimization Principles

Repeatable checklist. Review before release milestones, after major
features, and during periodic optimisation passes.

---

## 1. Avoid string allocation on hot paths

Never use `.to_string()`, `String::from()`, or `format!()` when a string
is already in the interner arena.

- **String literals**: use `pin!("…")` — per-call-site
  `thread_local! OnceCell<SymStr>`, so subsequent calls are a branch +
  load. Saves the 30–50 ns intern-table probe.
- **Runtime strings**: use `arena::pin(s)`.
- **Comparisons and reads**: use `arena::with*` to access an existing
  `SymStr` without re-allocating.

```rust
// BAD
if token.text.to_string() == "endgroup" { … }
// GOOD
if token.text == pin!("endgroup") { … }
// ALSO GOOD
arena::with(token.text, |s| s == "endgroup")
```

## 2. Minimise `.clone()` — borrow or reorder instead

Borrow the value if you can. If lifetimes get in the way, reorder the
code so the borrow is short. Cloning a `Tokens` or `Vec<Token>` is
~40-80 ns of pointer-bumping per element.

```rust
// BAD: clone for inspection then use
let cloned = tokens.clone();
if cloned.first().is_some_and(|t| t.cc == CC_CS) { do_thing(tokens); }
// GOOD: borrow for inspection
if tokens.first().is_some_and(|t| t.cc == CC_CS) { do_thing(tokens); }
```

## 3. Run clippy and study lint neighborhoods

`cargo clippy --workspace -- -W clippy::perf -W clippy::redundant_clone`
catches the easy wins. When clippy fires on one site, scan adjacent code
— the same author usually wrote both.

## 4. Minimise math-parser ambiguity

The Marpa grammar produces all valid derivations. For ambiguous math
the parse-tree count is combinatorial; each surviving parse consumes
memory and CPU. Reducing parses from 50 to 3 is a 10–20× speedup on
math-heavy documents.

Three tools, in order of preference:

1. **Grammar rules** — kill ambiguity at recognition time.
2. **Semantic actions returning `Err`** — prune during tree
   construction. Reject nonsensical constructions
   (`f(x)(y)` as double-application when `f` isn't higher-order,
   mismatched fence pairs, empty operator sequences).
3. **`Pragma` rules** — select best parse from surviving candidates.
   Less useful for raw performance (all parses must complete first) but
   valuable for representation quality.

## 5. External-process discipline (fork-exec is not free)

Every `gs`/`convert`/`inkscape`/`kpsewhich`/`pdfcrop` costs 10–50 ms
ambient plus dynamic-linker and font-cache init for `gs`/`convert`.
**Coalesce, dedup, and cache before spawning — not after.**

Telemetry across 190k arxiv documents shows the `graphics` phase at
**36.5% of total wall** — the single largest band. Even a modest hit
rate on a content-keyed cache (~30%) translates to ~10% off corpus
wall.

```rust
// Persistent on-disk cache, keyed by content + render options
let key = blake3::hash(&[source_bytes, page.to_le_bytes(),
                          dpi.to_le_bytes(), dest_kind.as_bytes(),
                          opts.canonical()].concat());
if let Some(cached) = cache.get(&key) { return Ok(cached); }
let fresh = run_convert(...)?;
cache.put(&key, &fresh);
```

**In-document coalescing landed 2026-05-12** (`48fd96ac75`):
`Plan::Copy` and `Plan::Convert` key on `(SipHash(content),
graphicx_options)`. Witness arXiv:2402.01336 (LHCb 1067-author paper)
— 1083 `<ltx:graphics>` nodes → **17 output files**. Persistent
on-disk cache is the next concrete win.

Cache-key correctness checklist:
- Include: source-bytes hash, page index, target DPI, format, flags
  that influence rendering.
- Exclude: timestamps, tmpdir paths.
- Bump a `cache_namespace` constant when fixing a rendering bug; don't
  rely on hash invalidation.

---

## Phase distribution (190k aggregate, 2026-05-02..03)

10 stages × 10k arxiv documents (189,991 jobs) from
`telemetry.jsonl.gz`. Sum-of-phases / wall = **97.78%** (above the
92% gate in `TELEMETRY.md`).

| Phase | %wall | mean / job |
|---|---:|---:|
| **graphics** | **36.5%** | 1,047 ms |
| **digest** | **20.3%** | 582 ms |
| **math_parse** | **17.0%** | 488 ms |
| **build** | **11.5%** | 331 ms |
| xslt | 7.2% | 207 ms |
| mathml_pres | 1.8% | 51 ms |
| serialize / post_xml_parse / rewrite | <1% each | |
| crossref / post_scan / mathml_cont / bibliography | <0.5% each | |

Top four bands account for **85% of wall**. Everything else combined
is ~12%.

39.16 M formulae total (mean 206/job). 39.06 M parse attempts produced
45.84 M surviving parses — a **17% over-parse rate** (lever for
Principle 4). Max RSS: 1,692 MB.

---

## Active improvement plan

### P1 graphics phase (36.5% of wall, largest lever)

- **In-document dedup** — done (`latexml_post/src/graphics.rs` via
  `convert_job_ids` HashMap keyed `(source, page, options)`, mirroring
  Perl `$doc->cacheLookup`).
- **Persistent on-disk cache** — next concrete win. Key on
  `(blake3(source), page, dest_type, density, options_canonical)`
  under `$LATEXML_OXIDE_CACHE_DIR`
  (default `$XDG_CACHE_HOME/latexml-oxide/graphics/`). Bump
  `cache_namespace` when convert/inkscape version changes. Add
  `graphics_cache_hits`/`_misses` to telemetry before landing.
- **Output-size validation set**: `0809.3849`, `0908.3201`,
  `1003.0368`, `0803.4343`, `0907.4282`. Compare output bytes, image
  count, missing-image count, wall before/after.
- **Subprocess-only chain** decided 2026-05-12 — see SYNC_STATUS.md.

### P1 digest + build (31.8% of wall, pure-Rust hot path)

No external slack to recover; wins come from Principles 1–3.

- Profile a digest-heavy outlier under release `perf`: `0911.3024`
  (76% of paper wall in digest), `0909.4601` (66%). Look for `arena::*`,
  `Tokens::*`, HashMap rehashes.
- Audit `.clone()` sites on Token / Tokens / Vec<Token> per Principle 2.
- Don't over-index — large papers are inherently expensive in digest.

### P1 math (17.0% of wall)

The 17% over-parse rate is the lever.

- Run `LATEXML_PARSE_AUDIT=1` on telemetry top-5: `0912.4453` (60.6%),
  `1001.5072` (62.5%), `0912.5528` (35.6%), `0908.4292` (46.8%),
  `0909.3532` (62.0%). Rank by total parse time, attempt count,
  repeated token sequence.
- Add exact parsed-math caching where audit shows repeated identical
  normalised token streams in equivalent math context.
- Prefer grammar/semantic pruning for demonstrably-invalid ambiguity
  families: malformed operator chains, impossible double application,
  invalid script targets, empty/mismatched fences.

### P1 failure outliers

The 60s+ tail (5 timeouts + `0903.3465` Xy-pic/token-limit recovery)
needs bounded recovery and clear fatal results, not speed-ups. Treat
separately from the hot-path work.

### P2 allocation/startup cleanup

Profile-driven only. Candidates: `*_sym` state accessors, `Tokens`
conversions, `Stored`/`Tokens` deep copies, package lookup caching,
dump/package loading. Land only after a slow-tail or sentinel profile
shows them on the hot path.

---

## Optimisation acceptance checklist

Before merging a performance change:

1. Release-mode before/after for the standing corpus.
2. One targeted benchmark for the suspected bottleneck.
3. Compare output status and lightweight structural metrics.
4. Report wall, user/sys CPU, max RSS, phase timings.
5. State the expected workload boundary and any fallback path.
6. Keep the change easy to disable if it relies on a heuristic.

For math-parser changes additionally record: parse-count distribution,
total math-parse time, MathML/XMath count, any formulas using a cache
path. Structural math output must be reviewed on math-heavy fixtures
before treating the change as a win.

---

## Standing performance corpus

Run with idle-serial CLI (no `cortex_worker`):

```bash
target/release/latexml_oxide \
  --preload=ar5iv.sty \
  --path=$HOME/git/ar5iv-bindings/bindings \
  --dest=/tmp/out.html --timeout=60 <main.tex>
```

Papers live under `data/10k_sandbox/<id>.zip`; `complex/si.tex` is
in-tree. Helper: `tools/run_perf_corpus.sh`.

### Current baseline (2026-04-30, release)

| Paper | Wall | Note |
|---|---:|---|
| `0906.1883` | 0.76s | aa, birkmult |
| `1011.1955` | 3.88s | math-parser bound |
| `1009.1431` | 2.19s | — |
| `1008.4386` | 3.17s | near-threshold |
| `0909.2656` | 2.56s | — |
| `0911.4739` | 2.74s | JHEP |
| `1005.1610` | 4.37s | post/graphics bound |
| `0803.0466` | 2.30s | aa |
| `complex/si.tex` | 1.28s | siunitx-heavy |

Round-17 baselines saw 5–7s peaks; cumulative wins from `aa3c7c1bb`
(graphics parallelism, –66% on `0911.4739`), 61-site `arena::to_string`
→ `arena::with` sweep, and the 2026-05-12 perf pass closed those
outliers.

### perf profile signatures

- `1011.1955` XML (3.78s, 0.99 CPUs) — single-core math/body
  conversion. Top symbols: `marpa_r_earleme_complete` (7.45%),
  `postdot_items_create` (6.59%), `bv_scan` (2.48%), `marpa_b_new`
  (2.42%), `transitive_closure` (2.08%). With `--nomathparse` the
  Marpa band disappears; remaining samples are libxml wrapper/node
  access and allocator traffic.
- `1005.1610` HTML (2.83s, 3.92 CPUs) — parallel external graphics
  dominates. Top samples in child processes: `gs`, `convert`,
  `png_write_row`, zlib, libc string/alloc. Rust-side Marpa <1% flat.

### Regression trigger

Any corpus entry drifting **> +15%** wall vs last recorded baseline
between commits is a regression signal. Record a new row in a dated
sub-heading; do not overwrite history.

### Vector-SVG fast path (issue #902 validation)

The `--graphics-svg-threshold-kb N` opt-in (round 17) bypasses
ImageMagick `convert` for vector-authored PDFs that `convert`
rasterises absurdly slowly. Fixture: `fig8.pdf` from
[brucemiller/LaTeXML#902](https://github.com/brucemiller/LaTeXML/issues/902)
(41 KB, arxiv:1807.01606).

| Path | Graphics phase | Total wall |
|---|---:|---:|
| default (ImageMagick `convert`) | 32.4s | 32.4s |
| `--graphics-svg-threshold-kb 200` | 0.25s | 0.3s |
| **speedup** | **130×** | **111×** |

Regression coverage:
`latexml_post/tests/integration.rs::test_vector_svg_pathological_convert_case`
asserts <5s on this fixture (silently skipped without inkscape).

### Vector-PDF auto-detection (2026-05-16)

The cortex_worker `ar5iv` profile now passes
`graphics_svg_threshold_kb: 0` — the special value that **enables
auto-detect**. Auto-detect scans the PDF header (up to 256 KB) for
`/Subtype /Image` / `/Subtype/Image` markers; if absent AND the file
is at most 500 KB, the SVG path fires. If markers ARE found, the
gs/convert raster path runs unchanged.

Empirically (1904.01426 fixture, 33 vector pgfplots-style PDFs): with
auto-detect ON every figure renders to `<name>.svg`; with auto-detect
OFF every figure renders to `<name>.png`. Wall time is comparable
either way for this paper because pdftocairo and gs are both fast for
small PDFs — but the SVG output is vector (zoomable, no
rasterization), and on `fig8.pdf`-class pathological-convert papers
the speedup matches the original `--graphics-svg-threshold-kb 200`
opt-in (~130×).

Override:
* `LATEXML_GRAPHICS_VECTOR_AUTO_OFF=1` — disable auto-detect.
* `--graphics-svg-threshold-kb N` (N > 0) — force legacy size-only
  gate (used for canvases where the auto-detector misclassifies).

Regression coverage:
`latexml_post::graphics::tests::should_try_svg_path_auto_detect` —
positive on `cifar10_vector.pdf` and `pathological_vector.pdf`,
negative on `raster_with_image.pdf` (a 3-pixel ImageMagick raster
PDF with an explicit Image XObject).

### Graphics content cache (2026-05-16)

Content-keyed disk cache between `latexml_post::graphics` and the
`gs`/`convert`/`inkscape`/`mutool`/`pdftocairo` subprocess spawns.
Key = SHA-256 of `source bytes ‖ page ‖ density ‖ target-ext`;
storage at `$XDG_CACHE_HOME/latexml-oxide/graphics/<aa>/<hash>.<ext>`
with a `.dims` sidecar storing `width\nheight\n` (so cache hits skip
`read_image_dimensions` too — Perl-LaTeXML.cache parity).

Multi-process safe:

* Writes go through `<final>.tmp.<pid>.<nanos>` then atomic `rename(2)`
  — concurrent writers all converge to one final file.
* Reads hardlink the cache file into the destination; the hardlink
  survives any concurrent prune that unlinks the cache entry
  afterwards (POSIX `link(2)` semantics).
* LRU prune holds `flock(LOCK_EX | LOCK_NB)` on a `.prune.lock`
  sentinel — only one process prunes at a time, others skip.
* `ENOENT` mid-prune is tolerated (a writer raced this entry).

Measured impact on `1909.03909` (8 MB paper, 21 graphics jobs) single
threaded, release binary:

| State | Wall |
|---|---:|
| cold (cache empty) | 9.55s |
| warm (cache hot) | 5.07s |
| `LATEXML_GRAPHICS_CACHE_OFF=1` | 9.40s |
| 4× concurrent (cold, shared cache) | 11.45s total (0 leftover tmp files) |

Override:

* `LATEXML_GRAPHICS_CACHE_OFF=1` — bypass entirely.
* `LATEXML_GRAPHICS_CACHE_DIR=/path` — override cache root.
* `LATEXML_GRAPHICS_CACHE_MAX_MB=N` — size cap (default 2048).

The cache is robust to externally-deleted entries — `lookup` checks
`exists()` AND tolerates `link/copy` failures mid-operation, falling
through to a fresh conversion + `store()` silently. Regression
coverage: `graphics_cache::tests::missing_disk_file_triggers_quiet_regeneration`
+ `concurrent_writers_converge_to_one_cache_entry`.

### Sandbox worker default (2026-05-16)

`tools/benchmark_canvas.sh` default `WORKERS` lowered from 20 → 8.
Re-timing the round22 slow tail showed graphics-bound papers ran 5–10×
slower at 20 workers than single-threaded because each
gs/convert/inkscape fork-exec stack competes for CPU+I/O. At 8 workers
the per-paper overhead is ≤30% vs single-threaded; corpus throughput
goes up. Override with `--workers N` only when the canvas is known to
be compute-bound (math/digest-heavy) rather than graphics-bound.

---

## Mini-benchmark: beat 2× pdflatex on `1910.01256`

See SYNC_STATUS.md "Acceptance gates". Post-DEP-19 anti-bloat
batch (2026-05-19) wall time is **0.71s** (release, --dest=.html,
full post-processing) vs pdflatex idle ~1.11s — meeting the 2×
gate (2.22s) with a 3.13× margin. Was 1.18s on 2026-05-12 (and
0.73s pre-DEP-15); the .text shrink (~3 MiB) from DEP-15/17/18/19
helped icache locality slightly.

---

## Math-parser routing — current state (2026-05-18)

The math parser uses **HYBRID routing** by default
(`latexml_math_parser/src/parser.rs::parse_marpa`). One recognizer
pass produces one bocage; routing then branches on Marpa's
`Bocage::ambiguity_metric()`:

- `metric == 1` (unambiguous, 60–87 % of formulae in the
  corpora we've measured) → ordinary `Tree::next()` +
  `Actions::get_tree`. Skips ASF construction entirely.
- `metric >= 2`, bocage and-node count ≤ `HYBRID_AND_NODE_LIMIT`
  (default 500) → ASF traversal (`MathTraverser`). One post-order
  pass; subtree sharing amortizes work.
- `metric >= 2`, bocage exceeds the cap → libmarpa Tree iterator
  on the same already-built bocage with the same six legacy
  convergence caps (`max_unique=10`, `max_consecutive_dupes=16`,
  `max_time=30s`, etc.). Sidesteps the ASF allocation cliff.

Escape hatches: `LATEXML_MARPA_LEGACY=1` forces pure Tree
iteration; `LATEXML_MARPA_ASF_ONLY=1` forces pure ASF (no
hybrid routing, no large-bocage fallback). Both are intended
for divergence debugging only.

The 500-and-node cap exists because downstream consumers
(pragmatics selection, XMath builders) cannot usefully process
more than a handful of distinct parses per formula. Bigger
bocages are treated as a **pipeline-flaw signal**, not a
load-bearing case — candidates for grammar-level category
tightening or earlier action-time pruning. Override with
`LATEXML_MARPA_HYBRID_AND_NODE_LIMIT=N` (`0`/`none` disables).

### Measurements

**Article-2025.tex** (579 formulae, 87.3 % raw-unambiguous,
release+bench, single-thread):

| Mode | Wall (3-run avg) | vs LEGACY |
|---|---:|---:|
| **HYBRID default** | **12.45 s** | **1.01×** |
| `LATEXML_MARPA_LEGACY=1` | 12.32 s | 1.00× |
| `LATEXML_MARPA_ASF_ONLY=1` | 16.80 s | 1.36× |

**100-paper math-bound sample** (top-100 by `phase_math_parse_us`
in wp4 telemetry; release+native+cortex, 8 workers, 180 s timeout,
8 GB ulimit, quiet host, marpa master `0bf24111`):

| Mode | OK / 100 | OOM aborts | Wall (n=98) | Δ vs LEGACY |
|---|---:|---:|---:|---:|
| LEGACY | 98 | 0 | 2227.1 s | — |
| HYBRID (cap = 500) | 98 | 0 | 2238.6 s | **+0.5 %** |
| HYBRID, no cap (historical) | 79 | 19 | 2955.4 s on n=79 | — |

Per-paper distribution on the cap=500 / n=98 subset: median
0.0 %, mean +1.0 %, 76 of 98 within ±5 %. The cap+fallback fixed
the 19 OOMs the no-cap hybrid produced on this fixture.

### What we tried that didn't move the needle

Documented for triage when similar ideas resurface:

- `XM::Lexeme(String, _)` → `XM::Lexeme(Rc<str>, _)` and a
  thread-local ASCII byte cache — ~0 % on Article-2025. Kept
  because it removes deep-clone hazards from the marpa cache
  hit/insert paths.
- `MathTraverser::ParseTree = Rc<Vec<Option<XM>>>` — ~0 %. The
  marpa cache `.clone()` was already cheap relative to
  `compute_symches` work.
- marpa `HashMap<usize, _>` caches → `Vec<Option<_>>` indexed
  by sequential id + slice-children API — ~3 %.
- marpa `glades` + `nidset_by_id` → `Vec<Option<_>>` — another
  ~3 %.
- `SmallVec` for `Symch.factorings` — counter data
  (`max_factorings_per_symch=4`, 99.98 % singletons) shows this
  would *increase* memory by ~72 MB on a 3M-symch workload
  for ~zero runtime gain. **Closed without implementation.**

Total Rust-side micro-optimization: ~6 % cumulative. The
hybrid-routing decision delivered the ~37 % saving needed for
parity with LEGACY.

### Audit env vars

- `LATEXML_MATH_AMBIGUITY_AUDIT=1` — per-formula ambiguity-metric
  counter (raw-unambiguous fraction across the corpus).
- `LATEXML_MARPA_HYBRID_AUDIT_PARITY=1` — runs both ASF and
  Tree-iter on every raw-unambiguous formula; asserts they
  produce equivalent `ParseOutcome` (treats `Empty`/`Rejected`
  as "no parse survived" per `parity_outcomes_compatible`).
- `LATEXML_MARPA_ASF_AUDIT=1` — emits per-formula ASF traversal
  detail (peak alternative count, pruning counter, large-bocage
  fallback fire events with bocage stats).
- `MARPA_ASF_STATS=1` — opt-in marpa-side instrumentation
  (singleton fast path hit rate, glade count, factoring count,
  cache hit/miss). Prints one snapshot per converted document.

### Recorded principles

- **Massive bocage explosions are a pipeline flaw**, not a load
  to absorb. Documented as
  `feedback_ambiguity_explosion_is_a_flaw` in session memory.
  When the cap fires, fix the underlying grammar/action
  ambiguity; do not raise the cap.
- **The residual ASF_ONLY → LEGACY gap is structural.** ASF
  builds a Rust-side glade/Nidset representation that
  Step-iteration skips; the singleton fast path eliminates
  the factoring chain for 99.98 % of glades but the
  glade-bookkeeping fixed overhead persists. Further wins live
  in libmarpa C-side bocage walking (out of scope) or in
  restructuring ASF to skip Nidset/Glade for wholly-unambiguous
  forests (large refactor — HYBRID already achieves this from
  the user's perspective).

### Open items

- **`Pi^N(p,q,…)` ambiguity explosion** (witness 2310.16583):
  formulae produce 2762–3555 and-nodes each. UNKNOWN-as-function
  + paren-comma-list — grammar cannot distinguish `Pi^N(p,q)` as
  function-application vs `Pi^N · (p · q)`. Needs lexer-level
  recognition of capital-Greek-letter-as-function in math
  context, or a semantic pragma that prunes the multiplication
  reading when a comma-separated arg list is present.
- **4 GiB amsthm.sty load OOM**: several wp4 abort papers OOM
  with `memory allocation of 4294967296 bytes failed` (exactly
  2³² bytes) during `amsthm.sty` load. Smells like a signed
  underflow producing `usize::MAX + 1` in a Vec presize. Unrelated
  to the math-parser bocage path.
- **Standing regression watch**: HYBRID vs LEGACY wall on the
  100-paper math-bound sample. Quiet-host baseline: +0.5 %
  delta on n=98 both-OK subset. Re-run on every meaningful
  marpa or math-parser change; flag if HYBRID climbs back
  toward LEGACY parity.

### Reproducing the corpus measurements

```bash
# Build release+native+cortex once:
cargo build --release --bin cortex_worker --features cortex

# HYBRID (default):
tools/benchmark_canvas.sh \
  --input-dir <math-bound-100-zips>/in \
  --output-dir /tmp/out_hybrid \
  --workers 8 --timeout 180

# LEGACY control:
env LATEXML_MARPA_LEGACY=1 tools/benchmark_canvas.sh \
  --input-dir <same input> --output-dir /tmp/out_legacy \
  --workers 8 --timeout 180
```

Capture `results.tsv` (per-paper wall, status, category) and
`telemetry.jsonl` (phase breakdown). Compare on the both-OK
subset.
