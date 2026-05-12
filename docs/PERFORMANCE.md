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

---

## Mini-benchmark: beat 2× pdflatex on `1910.01256`

See SYNC_STATUS.md "Acceptance gates" — tied within noise as of
2026-05-12 (1.18s latexml_oxide vs 1.11s pdflatex×2 idle).
