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

**Landed**: in-doc coalescing (`48fd96ac75`, 2026-05-12) +
persistent on-disk cache (2026-05-16) — see "Graphics phase —
completed work" below. Cache-key contract: include source-bytes
hash + page + DPI + format + render-affecting flags; exclude
timestamps/tmpdir paths; bump a `cache_namespace` constant when
fixing a rendering bug rather than relying on hash invalidation.

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

### P1 graphics phase (36.5% of wall) — CLOSED

In-doc dedup (`Plan::Copy`/`Plan::Convert`), persistent on-disk cache,
vector-SVG fast path, vector-PDF auto-detect all landed. Detail moved
to "Graphics phase — completed work" below. Subprocess-only chain
decision recorded in SYNC_STATUS.md (2026-05-12). Output-size
validation set retained as regression fixtures: `0809.3849`,
`0908.3201`, `1003.0368`, `0803.4343`, `0907.4282`.

### P1 digest + build (31.8% of wall, pure-Rust hot path) — CLOSED 2026-05-19

Investigation closed: the residual digest cost is structural to the TeX
semantics, not a Rust-translation accident. **Do not reopen without
new evidence** (e.g. a paper whose digest-time profile diverges from
the pattern recorded below).

**Findings under `cargo build --profile bench` + `perf record
-F 999 -e cycles:u` on `2305.06773` (digest-heavy ACM-class fixture,
4.5s of 4.8s wall in digest, witness available under
`/home/deyan/data/430k_noproblem_sandbox/data/arxmliv/2305/2305.06773`).**

Top leaf-frame distribution (~4.7k samples):
* `__mm_movemask_epi8` 363 + `find_inner` 47 + `probe_seq` 47 +
  `get_offset_len_noubcheck` 111 — **all SwissTable hashbrown probe
  code, total ~10% of wall**. Inherent to `state.meaning.get(&token)`
  running once per CS/ACTIVE token in `read_x_token` AND once more in
  `invoke_token` / `lookup_digestable_definition`.
* `state::lookup_meaning` 121 + `with_meaning` 114 — the two state
  accessors. `with_meaning` is borrow-only; `lookup_meaning` clones.
* mimalloc traffic 79 + Token push/write/read 200 — per-token Vec
  growth in the digester output stream.

**Landed this round (all signatures unchanged — function-body wins
only):**
* `Catcode::name_sym` in `lookup_digestable_definition`
  (`f2e23d9570`) — replaces a per-call `arena::pin(cc.name())` (RefCell
  mut on the interner + hashmap probe) with the cached per-call-site
  `pin!()` SymStr. Fires on every non-active-or-cs token.
* 8-site migration `lookup_meaning(t).is_some()` / `.is_none()` →
  `has_meaning(t)` (`3f06ecebd6`). `has_meaning` already existed as a
  clone-free shadow ("keep this in sync with `lookup_meaning`, it is
  copied over for optimization purposes"). Affected `\csname`,
  `\ifdefined`, `\ifcsname`, KeyVal `qname` checks, etc. Wall
  4.81s → 4.68s median on quiet 2305.06773 (≈2.7%).
* `lookup_conditional` (`2b63a1a0a1`) — replaced
  `arena::pin(token.get_executable_name())` (String allocation +
  interner probe) with `Token::pin_cs_name()` (free SymStr access).
  Within noise on this paper, but removes one String + one probe per
  `\if…` dispatch.

**Why we stop here.** The remaining cost is the **TeX-shape
read-then-invoke double probe**: `gullet::read_x_token` looks up a
meaning to decide whether to expand; `stomach::invoke_token` looks it
up again to decide how to invoke. Combining them into one probe would
require restructuring `read_x_token`'s return type to surface the
already-resolved `Stored`. That is an API change driven by perf,
without an ergonomics win, on a gullet API that mirrors the TeX
original by design — **explicitly out of scope**
(user directive, 2026-05-19: "we will not change the API for perf
reasons unless it has a big ergonomics win. The gullet API is coming
from the TeX original to a degree and is mostly fixed, we live with
it"). Caching meanings with invalidation has the same blast radius
(every `assign_meaning` would need to dirty the cache) — also out of
scope.

**Companion lint sweeps this round** (function-body cleanups, also
closed):
* `clippy::redundant_clone` lib targets — 18 sites in core/engine/post
  (`2150d149f9`) + 156 sites in package/contrib (`a66b61e32e`).
* `clippy::or_fun_call` — 57 sites across non-math crates
  (`2544dbf47d`). Concentrated in `Font::get_size().unwrap_or(defsize())`
  and `Dimension::new(0)` / `T_CS!(\\relax)` fallbacks.
* `clippy::needless_collect` — 13 sites (`4b60784177`). Mostly
  `tks.extend(...collect::<Vec<_>>())` patterns in
  `base_parameter_types`.
* `clippy::stable_sort_primitive` — 5 sites (`6f8a412cc8`).
* `clippy::implicit_clone` — 38 sites (`9d181ad95f`).
* `clippy::manual_string_new` + `single_char_pattern` — 10 sites
  (`ae192d3c6c`).

After all of the above, `cargo clippy --workspace --all-targets` is
back to its 14-warning baseline (all in `latexml_math_parser` ASF
lane, collaborator's track). Tests 1328/0/0 throughout.

**Witnesses retained** for any future digest-perf re-examination:
`2305.06773` (ACM-class), `2103.00971` (tikz-heavy, 8.8s digest),
`2208.10851` (mystery digest-heavy 1.5s), `2307.10256` (4.5s digest),
all under `~/data/430k_noproblem_sandbox/data/arxmliv/<yymm>/<id>/`.

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

Profile-driven only. The `*_sym` accessor sweep landed in the
P1 close-out (`Catcode::name_sym` in `lookup_digestable_definition`,
`Token::pin_cs_name` in `lookup_conditional`, etc.). Remaining
candidates — `Stored`/`Tokens` deep copies, package lookup caching,
dump/package loading — land **only after** a slow-tail or sentinel
profile shows them on the hot path. The 2026-05-19 perf sweep
confirmed the current state.meaning HashMap traffic is the floor;
do not chase further internal allocations without evidence they're
above the SwissTable probe band.

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

### Graphics phase — completed work

One-line summaries of landed wins; detail lives in code + commit
messages. Keep here as breadcrumbs for regression triage.

- **In-doc coalescing** (2026-05-12, `48fd96ac75`) —
  `Plan::Copy`/`Plan::Convert` key on `(SipHash(content),
  graphicx_options)`. Witness: arXiv:2402.01336 1083 nodes → 17 files.
- **Persistent on-disk cache** (2026-05-16, content-keyed disk cache
  in `latexml_post::graphics`). SHA-256 of
  `source‖page‖density‖target-ext`; storage at
  `$XDG_CACHE_HOME/latexml-oxide/graphics/<aa>/<hash>.<ext>` plus
  `.dims` sidecar (skips `read_image_dimensions` on hit — Perl
  `LaTeXML.cache` parity). Multi-process safe via
  `<final>.tmp.<pid>.<nanos>` + atomic rename, hardlink-on-read,
  `flock(LOCK_EX|LOCK_NB)` on `.prune.lock` for LRU. Measured: warm
  9.55s → 5.07s on 1909.03909 (21 graphics jobs). Overrides:
  `LATEXML_GRAPHICS_CACHE_OFF=1`, `LATEXML_GRAPHICS_CACHE_DIR=...`,
  `LATEXML_GRAPHICS_CACHE_MAX_MB=N` (default 2048). Tests:
  `graphics_cache::tests::missing_disk_file_triggers_quiet_regeneration`,
  `concurrent_writers_converge_to_one_cache_entry`.
- **Vector-SVG fast path** (round 17, issue #902) — opt-in
  `--graphics-svg-threshold-kb N` bypasses ImageMagick for vector PDFs.
  Witness: `fig8.pdf` (41 KB) 32.4s → 0.3s (~130× / ~111×). Test:
  `latexml_post/tests/integration.rs::test_vector_svg_pathological_convert_case`
  (<5s assertion, skipped without inkscape).
- **Vector-PDF auto-detection** (2026-05-16) — `cortex_worker` `ar5iv`
  profile passes `graphics_svg_threshold_kb: 0`; auto-detect scans PDF
  header (≤256 KB) for `/Subtype /Image` markers and routes to SVG
  when absent and file is ≤500 KB. Overrides:
  `LATEXML_GRAPHICS_VECTOR_AUTO_OFF=1`, or
  `--graphics-svg-threshold-kb N>0` to force the legacy size-only
  gate. Test:
  `latexml_post::graphics::tests::should_try_svg_path_auto_detect`.
- **Sandbox worker default 20 → 8** (2026-05-16,
  `tools/benchmark_canvas.sh`) — graphics-bound papers ran 5–10×
  slower at 20 workers vs single-threaded due to gs/convert/inkscape
  fork-exec contention; at 8 workers per-paper overhead is ≤30% and
  corpus throughput is higher. Override `--workers N` only when the
  canvas is known compute-bound (math/digest-heavy) rather than
  graphics-bound.

---

## Mini-benchmark: beat 2× pdflatex on `1910.01256` — MET

2026-05-19: 0.71s release (full post-processing) vs pdflatex idle
~1.11s — 3.13× margin on the 2.22s gate. .text shrink (~3 MiB) from
DEP-15/17/18/19 helped icache locality. Re-measure under the
SYNC_STATUS.md "Acceptance gates" recipe after any large workspace
landing; flag a regression if margin shrinks below 1.5×.

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

Settled negative results. Re-litigate only on new evidence.

- `XM::Lexeme(String,_)` → `Rc<str>` + thread-local ASCII cache: ~0 %
  on Article-2025 (kept anyway, removes deep-clone hazards).
- `MathTraverser::ParseTree = Rc<Vec<Option<XM>>>`: ~0 %.
- marpa `HashMap<usize,_>` caches → `Vec<Option<_>>` + slice-children
  API: ~3 %.
- marpa `glades` + `nidset_by_id` → `Vec<Option<_>>`: ~3 %.
- `SmallVec` for `Symch.factorings`: counter data
  (`max_factorings_per_symch=4`, 99.98 % singletons) projected +72 MB
  RAM at 3M-symch workload for ~0 runtime gain — **closed**.

Total Rust-side micro-opt: ~6 % cumulative. HYBRID-routing decision
delivered the ~37 % needed for LEGACY parity.

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
