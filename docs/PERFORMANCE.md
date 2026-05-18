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

See SYNC_STATUS.md "Acceptance gates" — tied within noise as of
2026-05-12 (1.18s latexml_oxide vs 1.11s pdflatex×2 idle).

---

## ASF traversal — perf opportunities (2026-05-17)

ASF became default in commit `312cb33bdd`. Measurement on
`1912.03329` (386 formulas, geometric topology, release build):

* **LEGACY**: 0.89s avg
* **ASF**: 1.16s avg → 1.13s after cartesian fast path

ASF carries ~20% overhead on unambiguous math. Hot spots
identified in `asf_traverser.rs` and `marpa/src/asf.rs`:

### Latexml_math_parser side

1. **Per-byte `String` allocation** (asf_traverser line 95):
   every byte-token glade allocates a 1-char `String` via
   `String::from_utf8(vec![byte])`. A typical paper has tens of
   thousands of byte glades — tens of thousands of 1-char
   allocations.

   Fix: change `XM::Lexeme(String, Meta)` to
   `XM::Lexeme(SymStr, Meta)` (interned via `latexml_core::arena`).
   SymStr is a `u32`; the 128 ASCII byte values get interned once
   and shared. Invasive — touches ~170 sites.

2. **`Meta::default()` allocation per byte**: every byte glade
   passes `Meta::default()` which constructs `Vec::new()` and
   `CurryConstraints::default()`. Even though empty, the struct
   moves are non-trivial. Sharing a single `Meta::EMPTY` reference
   via `Cow` would save the copies.

3. **Cartesian product cloning** (done): fast path for
   single-combo case landed in commit `7314e19599`. Saves N+1
   Vec allocations per rule reduction in the common case.

### Marpa-rs side (separate repo `~/git/marpa`)

1. **`HashMap<usize, Glade>` for glades** (`asf.rs:50`): glade
   IDs are sequential — a `Vec<Option<Glade>>` indexed by id
   would replace hash probes with array index loads.

2. **`HashMap<usize, ParseTree>` for traversal cache**
   (`asf.rs:138`): same as above, plus the cache hit `.clone()`
   on line 156 deep-clones the entire `ParseTree`. For ASF user
   types like `Vec<Option<XM>>`, this clones every `XM` tree —
   significant. Wrapping in `Rc<PT>` (single-threaded) or `Arc<PT>`
   would reduce clone to a refcount bump.

3. **`children: &HashMap<usize, ParseTree>`** passed to user
   callback (`asf.rs:207`): the user accesses children by
   `rh_glade_id(ix)` + `children.get(&id)` — a hash probe per
   position. A better API would pre-resolve children into a
   `&[&ParseTree]` indexed by RHS position.

4. **`compute_symches` allocations**: per-glade allocates
   `source_data`, `raw_factorings`, `factorings`, etc. Vecs.
   Reusable scratch buffers would reduce allocation traffic.

### Estimated impact

The byte-glade fix alone (item 1 of latexml_math_parser side) likely
closes the gap to LEGACY parity on unambiguous math. The marpa-rs
cache fixes would help all uses of `parse_and_traverse_forest`,
including in `marpa::tests::panda::*`.

Estimated effort: byte-glade arena fix ≈ 1-2 sessions; marpa-rs
cache fixes ≈ 1 session. Tracked as future perf work.

---

## ASF traversal — measured results (2026-05-17 follow-up)

Validation of the perf opportunities above on a math-heavy fixture
(`Article-2025.tex`, 579 `$`-delimited formulas, release build):

| Stage | ASF wall | LEGACY wall | Notes |
|---|---:|---:|---|
| Pre-optimization (initial measure) | 18.05s | 12.0s | baseline gap ≈ 50% |
| + `XM::Lexeme(Rc<str>, _)` + thread-local ASCII byte cache (`1a32531ce2`) | 18.0s | 12.0s | ~0% delta — byte-glade Lexeme String alloc was **not** the bottleneck |
| + `MathTraverser::ParseTree = Rc<Vec<Option<XM>>>` (same commit) | 18.0s | 12.0s | ~0% — the marpa cache `.clone()` was already cheap relative to compute_symches work |
| + marpa cache `HashMap<usize, PT>` → `Vec<Option<PT>>` + slice children API (marpa `7875bc8`) | 17.4s | 12.0s | ~3% — measurable but not load-bearing |
| + marpa `glades` and `nidset_by_id` → `Vec<Option<_>>` (marpa `325f615`) | 16.85s | 12.0s | another ~3% — cumulative ~6% reduction |

**Conclusion**: the residual ASF→Step-iteration overhead is
**structural**, not allocation-bound. ASF's `compute_symches` walks
every and-node in the bocage to enumerate factorings; for
unambiguous parses (the bulk case in real-world math), this is
purely a cost not amortized by any subtree sharing. The Step-
iteration path inside libmarpa skips that enumeration entirely.

The remaining ~40% gap on math-heavy unambiguous papers is not
recoverable via Rust-side micro-optimization. Two paths:

1. **Hybrid dispatch**: pre-flight `ambiguity_metric()`; route
   unambiguous parses through legacy Step iteration, ambiguous
   through ASF. Closes the gap on the common case but keeps two
   code paths alive.
2. **Continue ASF-only**: future wins live in libmarpa C-side
   bocage walking or a Rust-side fast-path inside `compute_symches`
   that detects single-and-node or-nodes and short-circuits the
   factoring enumeration. Larger refactor with diminishing returns.

The `Rc<str>` Lexeme + `Rc<Vec>` ParseTree + slice-API children
changes stayed in even though their direct impact was minimal —
they remove deep-clone hazards from the marpa cache hit/insert
paths (future-proofing) and are architecturally cleaner.

### Cargo.toml dev-time patch

The `[patch."https://github.com/dginev/marpa"]` in workspace
`Cargo.toml` routes the dep at the local marpa checkout. Once
the `asf-step3-generic-traverser` branch is pushed with the new
commits, update `latexml_math_parser/Cargo.toml`'s `marpa = { git
= "...", branch = "..." }` SHA and remove the `[patch]` block.

---

## HYBRID dispatch becomes default (2026-05-17, landed)

Path 1 from the previous section — hybrid dispatch — landed in
latexml-oxide commit `9318960974` and marpa commit `60b320b`. The
default `parse_marpa` now branches on `Marpa::Bocage::
ambiguity_metric()` after a single recognizer pass:

* metric == 1 → cheap `Tree::next()` + `Actions::get_tree` (the
  legacy code path's machinery)
* metric ≥ 2 → ASF traversal via `MathTraverser` (the ASF default
  from the previous round)

Final measurement on `Article-2025.tex` (579 math-heavy formulas,
bench profile, single-thread, 3-run avg):

| Mode | Wall | vs LEGACY |
|---|---:|---:|
| **HYBRID (default)** | **12.41s** | **1.018×** |
| `LATEXML_MARPA_LEGACY=1` | 12.21s | 1.00× |
| `LATEXML_MARPA_ASF_ONLY=1` | 16.67s | 1.37× |

The HYBRID:LEGACY ratio sits inside the 1.05× acceptance gate.
Hybrid recovers the ~4.5s ASF-only overhead on this fixture (87%
raw-unambiguous formulae per `LATEXML_MATH_AMBIGUITY_AUDIT=1`)
while preserving ASF's algorithmic advantage on the 13%
raw-ambiguous fraction.

### Why this works (and why ASF-only doesn't)

The ASF traverser visits every glade in the bocage and runs
`compute_symches` per glade — fixed per-glade overhead that
doesn't pay off for unambiguous input where subtree sharing
yields no amortization. Step-iteration via libmarpa's built-in
`Tree::next()` produces a single linear tree without that
overhead. The hybrid keeps the cheaper path for the common case
and routes only the truly ambiguous formulae (where ASF's
glade-once invariant amortizes work) through the heavier
machinery.

### Audit tooling

Two opt-in env vars verify the design holds at scale:

* `LATEXML_MATH_AMBIGUITY_AUDIT=1` — per-formula ambiguity-metric
  counter, used to confirm the per-corpus unambiguous fraction.
  Measured on:

  | Paper | Total parses | Unamb% |
  |---|---:|---:|
  | Article-2025 (algebraic topology) | 3902 | 87.3% |
  | TheDiskComplex (geometric topology) | 681 | 77.4% |
  | arxiv 2602.06085 (mixed STEM) | 130 | 60.0% |

* `LATEXML_MARPA_HYBRID_AUDIT_PARITY=1` — runs both paths on every
  raw-unambiguous formula and asserts they produce equivalent
  `ParseOutcome`. The assertion treats `Empty` and `Rejected(_)`
  as the same "no parse survived" outcome (the user-facing HTML
  is bit-identical in either case) and only fails on real
  `Accepted` vs anything-else mismatches. Runs clean on both
  Article-2025 (3902 calls) and TheDiskComplex (681 calls).

### What did NOT move the needle (cumulative ~6%)

Documented for future-session triage:

* `XM::Lexeme(String, _)` → `XM::Lexeme(Rc<str>, _)` and the ASCII
  byte-cache — ~0% delta. Action allocation isn't the bottleneck.
* `MathTraverser::ParseTree` → `Rc<Vec<Option<XM>>>` — ~0% delta.
  The marpa cache `.clone()` was cheap relative to `compute_symches`
  work.
* marpa `HashMap<usize, _>` caches → `Vec<Option<_>>` indexed by
  sequential id, plus `&[Option<PT>]` slice children API — ~3%.
* marpa `glades` + `nidset_by_id` → `Vec<Option<_>>` — another ~3%.

Total Rust-side allocation cleanup: ~6%. The hybrid routing
delivered the remaining ~37% reduction needed to reach LEGACY
parity. **Lesson**: structural algorithmic choices dominate
allocation micro-optimization for this workload.

---

## Codex senior-engineer optimization list — completed

Beyond the hybrid landing, the full optimization list from
`marpa/docs/ASF_PERFORMANCE_FINDINGS.md` was exhausted. Each
item, its outcome on `Article-2025.tex`, and the commit:

| # | Item | Commit | Impact |
|---|---|---|---|
| 1 | Hybrid routing | latexml `9318960974` / marpa `60b320b` | ASF→HYBRID: 17.0s → 12.4s |
| 2 | Singleton fast path in `compute_symches` | marpa (codex implementation, embedded in `60b320b`) | embedded in hybrid baseline |
| 3 | Bocage metadata caches (`AndNodeInfo`/`OrNodeInfo`) | marpa `a045778` | perf-flat (RefCell≈FFI), architectural encapsulation |
| 4 | Flatten factoring storage | DEFERRED per codex's "do not start here" caution | — |
| 5 | Clean traversal internals | marpa `a045778` (+ codex `60b320b` for generic recursion / VisitState) | quality + correctness |
| 6 | Odometer Cartesian product | latexml `109390fe92` | ~1.5% on HYBRID (only ambiguous-glade reduction) |

### Quality / correctness items added beyond the list

- **`collect_factorings` propagates `Result`** (marpa `96fd092`)
  — prior `.unwrap_or(AndNodeInfo{cause:-1,...})` silently
  mapped FFI errors to a bogus token-and-node default.
- **`parity_outcomes_compatible` helper + 8 unit tests**
  (latexml `0a8a171859`) — the audit's `Empty`-vs-`Rejected`
  false-positive on shallow pragma rejections is now guarded
  by regression tests.
- **`Glade.visited` field + `glade_is_visited` helper removed**
  — dead defensive code post-`VisitState`.
- **`Rc<str>` Lexeme + `Rc<Vec<Option<XM>>>` ParseTree** kept
  even though direct perf was 0% — removes deep-clone hazards
  from the marpa cache hit/insert paths.

### Final 3-way wall (closing state)

`Article-2025.tex`, bench profile, 3-run avg:

| Mode | Wall | vs LEGACY |
|---|---:|---:|
| **HYBRID default** | **12.45s** | **1.01×** |
| `LATEXML_MARPA_LEGACY=1` | 12.32s | 1.00× |
| `LATEXML_MARPA_ASF_ONLY=1` | 16.80s | 1.36× |

### Residual ASF_ONLY structural gap

ASF_ONLY remains ~37% slower than LEGACY. After all Rust-side
micro-optimization, the gap is **structural**: ASF builds a
Rust-side glade representation (Nidset + Glade allocations
plus the bocage walk in `compute_symches`) that Step-iteration
skips entirely. The singleton fast path eliminates the
factoring chain for 87% of glades, but the per-glade
bookkeeping fixed overhead persists.

Further wins require either libmarpa C-side surgery (out of
scope — we don't own libmarpa) or restructuring ASF to skip
Nidset/Glade allocation for wholly-unambiguous forests (large
refactor; hybrid already achieves this from the user
perspective by skipping ASF entirely for that case).

Both yield diminishing returns vs the hybrid escape hatch
that's now the default.

### Tests at session close

- marpa: **23/0** (including 2 new hybrid-routing tests:
  `hybrid_parse_returns_tree_for_unambiguous_input` with a
  `PanicTraverser` and `hybrid_parse_traverses_asf_for_ambiguous_input`)
- latexml-oxide: **1309/0** (1301 prior + 8 new
  `parity_outcomes_compatible_*` parity-helper unit tests)
- Parity audit (`LATEXML_MARPA_HYBRID_AUDIT_PARITY=1`) clean on:
  - Article-2025.tex (3902 parse calls)
  - TheDiskComplex.tex (681 parse calls)
- sin[XY] regression fixture (`physics.tex`): bit-identical HTML
  output across all three modes.
