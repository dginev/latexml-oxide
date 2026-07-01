# Performance Optimization Principles

Repeatable checklist + current lever state. Review before release
milestones, after major features, and during periodic optimisation
passes.

This doc holds the **timeless principles** and the **current open/closed
lever state**. The per-paper empirical campaign log (slowest-100 testbed,
hotspot-by-hotspot deltas) lives in
[`ARXIV_PERFORMANCE.md`](ARXIV_PERFORMANCE.md); reliability witnesses
(timeout/OOM/hang) live in [`STABILITY_WITNESSES.md`](STABILITY_WITNESSES.md).
Detailed investigation narratives are in `git log` + commit messages —
this doc keeps outcomes, not sagas.

---

## Principles (the checklist)

### 1. Avoid string allocation on hot paths

Never `.to_string()`, `String::from()`, or `format!()` when the string is
already in the interner arena.

- **String literals**: `pin_static("…")` (per-call-site `OnceCell<SymStr>`,
  subsequent calls are branch+load; saves the 30–50 ns intern probe). The
  older `pin!("…")` macro is being phased out for literals.
- **Runtime strings**: `arena::pin(s)`.
- **Comparisons/reads**: `arena::with*` to read an existing `SymStr` without
  re-allocating.

```rust
// BAD                                   // GOOD
token.text.to_string() == "endgroup"     token.text == pin_static("endgroup")
                                          arena::with(token.text, |s| s == "endgroup")
```

### 2. Minimise `.clone()` — borrow or reorder

Borrow if you can; if lifetimes fight you, shorten the borrow. Cloning a
`Tokens`/`Vec<Token>` is ~40–80 ns/element of pointer-bumping. Inspect via
`.first()` / `.is_some_and(...)` on the borrow, then act on the original.

### 3. Run clippy and study lint neighborhoods

`cargo clippy --workspace -- -W clippy::perf -W clippy::redundant_clone`.
When clippy fires on one site, scan adjacent code — the same author usually
wrote both.

### 4. Minimise math-parser ambiguity

The Marpa grammar produces all valid derivations; for ambiguous math the
parse count is combinatorial, and each surviving parse costs memory+CPU.
Reducing 50 parses → 3 is a 10–20× speedup on math-heavy docs. Tools, in
order of preference:

1. **Grammar rules** — kill ambiguity at recognition time.
2. **Semantic actions returning `Err`** — prune during tree construction
   (reject impossible double-application, mismatched fences, empty operator
   sequences).
3. **`Pragma` rules** — select best parse from survivors (less useful for raw
   speed — all parses complete first — but key for representation quality).

**Massive bocage explosions are a pipeline flaw, not a load to absorb.** When
a convergence cap fires, fix the underlying grammar/action ambiguity; do not
raise the cap. (Memory: `feedback_ambiguity_explosion_is_a_flaw`.)

### 5. External-process discipline (fork-exec is not free)

Every `gs`/`convert`/`mutool`/`pdftocairo`/`kpsewhich`/`pdfcrop` costs 10–50 ms
ambient plus dynamic-linker + font-cache init for `gs`/`convert`. **Coalesce,
dedup, and cache before spawning — not after.** Graphics was the single
largest corpus band (36.5% of wall); in-doc coalescing + persistent on-disk
cache landed (see "Graphics — completed" below). Cache-key contract: include
source-bytes hash + page + DPI + format + render-affecting flags; exclude
timestamps/tmpdir paths; bump a `cache_namespace` constant when fixing a
rendering bug rather than relying on hash invalidation. Also memoize
extensionless kpathsea image lookups (hits AND misses) by
`(SOURCEDIRECTORY, path)` — repeated missing figures otherwise pay a fresh
fork-exec per `\includegraphics`.

### 6. No whole-tree `//` / `preceding::` scans inside per-node XSLT templates

**The recurring post-processing perf trap.** An XSLT `<xsl:value-of>` /
`<xsl:if>` whose XPath uses the descendant (`//`) or `preceding::` axis walks
the **entire document tree from the root**, yet runs **once per matched node**
→ O(nodes × tree-size) ≈ **O(n²)**. On a large book/thesis this pins XSLT at
60–150 s. The level/flag being computed is almost always a **document-global
constant** — hoist it into a single `<xsl:variable select="boolean(//…)"/>`
(evaluated once from the root) and reference the variable; or use the
Muenchian `<xsl:key>` method for distinct-by-value dedup. Output-neutral.

Three were found and fixed (all in `resources/XSLT/`, embedded at build time):
- `f:seclev-aux` heading-level (`LaTeXML-structure-xhtml.xsl`) — ARXIV_PERFORMANCE #2.
- `head-keywords` index dedup (`LaTeXML-webpage-xhtml.xsl`, `//…[not(.=preceding::…)]`
  → Muenchian key) — ARXIV_PERFORMANCE #3.
- `maketitle`'s per-title `//ltx:navigation` scan (`LaTeXML-structure-xhtml.xsl`)
  — ARXIV_PERFORMANCE #4.

**Audit conclusion (2026-06-29):** the html5 XSLT path now has **zero** per-node
whole-tree scans (full grep audit). Do not re-investigate XSLT O(n²) on large
docs unless a NEW per-node `//`/`preceding::` scan is added. These are shared
with upstream Perl LaTeXML (Perl keeps the O(n²)) — candidates to upstream.
libxml2 2.16 (Rust) is worse on these than Perl's 2.15.1, so the win is larger
for us. Pin any future XSLT hotspot with `xsltproc --profile` (the `libxslt`
crate's `transform()` doesn't expose profiling).

---

## Phase distribution (190k aggregate, 2026-05-02..03) — canonical

10 stages × 10k arXiv docs (189,991 jobs). Sum-of-phases / wall = 97.78%.

| Phase | %wall | mean/job |
|---|---:|---:|
| **graphics** | **36.5%** | 1,047 ms |
| **digest** | **20.3%** | 582 ms |
| **math_parse** | **17.0%** | 488 ms |
| **build** | **11.5%** | 331 ms |
| xslt | 7.2% | 207 ms |
| mathml_pres | 1.8% | 51 ms |
| serialize / post_xml_parse / rewrite | <1% each | |
| crossref / post_scan / mathml_cont / bibliography | <0.5% each | |

Top four bands = 85% of wall. 39.16 M formulae (mean 206/job); 17% over-parse
rate (the math lever). Max RSS 1,692 MB.

**Methodology traps (do not relearn):**
- **Profile with the ar5iv profile.** Production runtimes come from
  `cortex_worker`, which preloads `ar5iv.sty` (changes emulation decisions,
  defines PiCTeX etc.). A bare `latexml_oxide <main>.tex` gives a *false-fast*
  reading (e.g. `math0605199` 0.24 s bare-CLI-bailout vs 160 s real). Use the
  Standing-corpus recipe below.
- **Rank by single-paper telemetry, NOT the cortex `runtimes` report.** The
  fleet report is contention-inflated (RSS pressure, 72-worker scheduling):
  re-measured single-paper, the "90–160 s" witnesses are ~10 s. The phase
  *split* is the actionable signal, not the fleet absolute wall.
- `perf` is locked down on most hosts; profile via `LATEXML_TELEMETRY_OUT`
  phase walls + env-gated `Instant` probes, or `sudo sysctl
  kernel.perf_event_paranoid=-1` where allowed.

---

## Open levers

The phase bands above set priority. Current open work:

### P1 — math_parse (17% of wall, 17% over-parse) — the top remaining lever

Every math-heavy witness is now `math_parse`-bound (the `build` quadratic was
fixed — see Closed). The over-parse rate is the lever; see **Principle 4**,
[`MATH_OVERPARSE_DEEP_DIVE_2026-06-30.md`](MATH_OVERPARSE_DEEP_DIVE_2026-06-30.md)
(current measured and-node counts + ranked levers) and
[`MATH_PARSER_AND_ASF.md`](MATH_PARSER_AND_ASF.md).

**LANDED 2026-06-30 — differential-`d` lexer gating (output-neutral).** The lexer
emitted `XDIFFUNK`/`XDIFFID` (the diffop-competing branch) for *every* `d`
unconditionally; outside integrals the `diffop_apply` action always prunes it, so
Marpa built a ~71-and-node branch per `d<var>` only to reject it. `util.rs`
`node_to_grammar_lexemes_from` now downgrades `XDIFFUNK→UNKNOWN`/`XDIFFID→ID` when
the formula has no `INTOP` (same predicate `diffop_apply` uses, over the same node
list) — byte-identical output, removes the over-parse on every non-integral `d`
(`\frac{dx}{dt}`, `d`-as-variable, `d`-subscripts). High volume (differentials are
everywhere). Step 2 (a dedicated in-integral `DIFFOP_D` terminal so `∫(x·d·x)` is
never built, pulling `\int … f(x)\,dx` off the legacy fallback) is the follow-up.

Remaining open hot patterns (fresh 2026-06-30 measurements; the old
`MATH_AMBIGUITY_AUDIT` claims are **stale** — `\Pi^N(p,q,r)` and simple bare
`|x|≤|y|` are now *unambiguous*):
- **`f(x,y)` apply-vs-multiply** — the dominant residual; parens alone create the
  ambiguity (`f(x)` = 112 and-nodes vs `fx` unambiguous). NB an apparent latent
  regression: `speculative_prefix_apply` (semantics.rs) no longer checks the
  `MATHPARSER_SPECULATE` flag → speculative apply is always on; needs a same-host
  Perl parity check before touching (may change many outputs).
- **Bare `|x|` with ambiguous inner content** — e.g. `|v(x)| ≤ |v(x')|` (625
  and-nodes, legacy fallback): bar-pairing × inner apply-ambiguity. A balanced-pair
  **pre-lexer** pass (peer of the `STRETCHY_VERTBAR` hint) targets the pairing
  factor. Lower priority — the *simple* modulus inequality is already unambiguous.
- **Integrals** now the largest volume driver (`\int_0^1 f(x)\,dx` = 523 and-nodes,
  on the legacy fallback path); Step 2 of the differential lever above is the fix.

The architectural floor for a 2385–5000-formula paper is ~4 ms/formula
(marpa-C-bound: ~25% of self-time is the marpa-C engine, out of scope). 1–2 s
there needs the over-parse lever, not deeper marpa work. Audit with
`LATEXML_PARSE_AUDIT=1` / `LATEXML_MATH_AMBIGUITY_AUDIT=1` /
`LATEXML_MARPA_ASF_AUDIT=1`.

### P1 — graphics (36.5%) — largely CLOSED, two open traps

In-doc dedup, persistent disk cache, vector-SVG fast path, vector-PDF
auto-detect all landed (see "Graphics — completed"). Remaining: the
extensionless-kpathsea-lookup cache (Principle 5), and tikz-cd/xy/pgf
**native** rendering cost (NOT external `gs`/`convert` — these render in-Rust
and show up as digest+math+build on the formula count; see "tikz-cd cluster").

### FxHash libxml node-cache (measured ~28–30%, biggest single win) — SHIPS, pending upstream cleanup

The `rust-libxml` fork's `xmlNodePtr → Node` cache is probed on EVERY
`Node::wrap`; swapping its std SipHash `RandomState` for a dependency-free
FxHash pointer hasher cut wall on every node-heavy phase by **~28–30%**
(`1510.03361` 19.6→14.1 s; `1805.03265` tikz-cd 22.4→15.7 s). Output-identical
(map never iterated; pointer keys non-adversarial so HashDoS is moot).
**This already ships** in every build (dev/CI/release/maxperf) via a *committed*
`[patch.crates-io]` in the root `Cargo.toml` →
`KWARC/rust-libxml` branch `perf-improvements` (a public fork), so the corpus
run gets the win. The remaining task is supply-chain cleanliness only: land the
official libxml PR, publish 0.3.15, bump the `libxml` dep, drop the patch. (The
marpa FxHash already ships via the marpa git dep tracking `main`.)

### tikz-cd / pgf digest backlog

tikz-cd emits thousands of small math formulae (one per cell/arrow/label —
6825 from one doc); cost is the formula count × (digest + math_parse + build),
NOT external graphics. Levers (compounding): (1) reduce the per-cell formula
count in the binding; (2) the pgf digest hot path — lazy `Tokens::Debug`,
`Option<SymStr>` from `lookup_value*` to drop the `Cow::Borrowed`, a pgfplots
`\addplot table` Rust bypass; (3+4) fold into math_parse/build above.
SmallVec-backed `Tokens` was tried and **regressed** (struct bloat) — do not
retry without first shrinking `Token` below 8 bytes. The per-token cycle-guard
floor for graphics packages is raised to `CYCLE_GUARD_ACTIVATE_GRAPHICS = 150 M`
(pgf/tikz/xy bindings call `raise_cycle_guard_activate` at load) so healthy
100–155 M-token graphics streams don't pay the guard.

---

## Closed levers (do not reopen without new evidence)

One-line outcomes; detail in `git log` + commit messages.

- **`build` phase quadratic — FIXED (`335b6b83`, ~20×).** `math0605199`
  44.9 s → 2.1 s. Root cause: a single-`ltx:text`-child merge called
  `record_node_ids(node)` *inside* a per-grandchild move loop → O(G²) XPath
  `descendant-or-self::*[@xml:id]` re-scans; hoisted to one post-loop call.
  Runs in every conversion → broad latent win. (Earlier 7-site
  `get_child_nodes().is_empty()` → `get_first_child().is_some()` O(1)-emptiness
  sweep also landed.) Build is now linear (~0.8 ms/formula); the build-side
  sweep is exhausted.
- **P1 digest + build (pure-Rust hot path) — CLOSED 2026-05-19.** Residual
  digest cost is structural to TeX semantics, not a translation accident. perf
  floor is the `state.meaning` SwissTable double-probe (read_x_token decides
  whether to expand; invoke_token decides how to invoke — each probes once).
  Combining them = an API change on a gullet API that mirrors TeX by design —
  **out of scope** (user directive 2026-05-19: don't change the gullet API for
  perf without a big ergonomics win). Landed body-only wins: `Catcode::name_sym`
  / `has_meaning` (8 sites) / `Token::pin_cs_name`.
- **dhat allocation sweep — DONE (faithful, output byte-identical).** Cut
  multi-GB of *churn* (allocator pressure / RSS, matters for the fleet) but
  only ~1–2% single-process wall (digest/math_parse are CPU-bound, not
  alloc-bound). Landed: `serialize_aux` → single growing buffer; serialize attr
  loop reuses `get_attributes()`; `get_tag_action_list` borrows tag hashes;
  `fixedformat`/`get_node_qname` in-place writes; `read_until`/`read_tokens`/
  `List::revert` pre-sizing. Deferred architectural items (token-list COW, AST
  arena, `Font::relative_to` keys) need explicit sign-off (no-redesign-away-
  from-Perl constraint) — they ARE the Perl expansion/parse data model.
- **XSLT deep-DOM copy + max-depth — DONE.** `dup()` → `Rc clone()`
  (−120–130 MB/paper); `xsltMaxDepth = 1000` (faithful Perl port, graceful
  abort vs OOM). See STABILITY_WITNESSES Cluster A.
- **PGO / `target-cpu` (v3/native) — NO GAIN, closed.** maxperf is already at
  the fat-LTO + CGU1 ceiling; engine isn't SIMD-amenable (branchy
  catcode/macro dispatch). Keep portable `x86-64`. Tooling deleted. (Memory
  `pgo-isa-no-gain-2026-06-21`.)
- **Startup dump-parse lever (~50 ms of ~161 ms floor) — declined** as too
  small for release-critical risk; amortized to noise on long papers anyway.
  (`STARTUP_COST_ANALYSIS_2026-06-21.md`.)
- **`build-std` (panic_abort) — PARKED.** −0.11 MB (0.2%); `.eh_frame` is from
  the static C deps (mimalloc/libmarpa/zstd), which `-Z build-std` doesn't
  touch. Not worth the nightly fragility.

---

## Math-parser routing — current state

HYBRID routing by default (`latexml_math_parser/src/parser.rs::parse_marpa`).
One recognizer pass → one bocage; routing branches on
`Bocage::ambiguity_metric()`:

- `metric == 1` (unambiguous, 60–87% of corpus formulae) → ordinary
  `Tree::next()` + `Actions::get_tree`; skips ASF entirely.
- `metric ≥ 2`, and-node count ≤ `HYBRID_AND_NODE_LIMIT` (default 500) → ASF
  traversal (`MathTraverser`), one post-order pass with subtree sharing.
- `metric ≥ 2`, bocage exceeds the cap → libmarpa Tree iterator on the same
  bocage with the six legacy convergence caps. Sidesteps the ASF allocation
  cliff.

The 500-and-node cap exists because downstream consumers can't usefully process
more than a handful of parses; a bigger bocage is a **pipeline-flaw signal**
(tighten the grammar, don't raise the cap). Override:
`LATEXML_MARPA_HYBRID_AND_NODE_LIMIT=N` (`0`/`none` disables).

Escape hatches (divergence debugging only): `LATEXML_MARPA_LEGACY=1` (pure Tree
iteration), `LATEXML_MARPA_ASF_ONLY=1` (pure ASF). Audit knobs:
`LATEXML_MATH_AMBIGUITY_AUDIT=1`, `LATEXML_MARPA_HYBRID_AUDIT_PARITY=1`,
`LATEXML_MARPA_ASF_AUDIT=1`, `MARPA_ASF_STATS=1`.

**ASF gain** is asymptotic (cost ∝ glade count, not tree count): typical arXiv
formulae (5–50 trees) ~2–5×; pathological (hundreds–thousands of trees)
10–87×. HYBRID achieves LEGACY parity (+0.5% on a 100-paper math-bound sample,
n=98 both-OK, zero OOM; the cap fixed 19 OOMs the no-cap hybrid produced).

**Settled negative micro-opts (re-litigate only on new evidence):**
`XM::Lexeme → Rc<str>` ~0%; `MathTraverser::ParseTree = Rc<…>` ~0%; marpa
`HashMap → Vec<Option<_>>` ~3%; marpa glades→Vec ~3%; SmallVec for
`Symch.factorings` +72 MB RAM for ~0 gain (closed). Total Rust-side micro-opt
~6%; HYBRID-routing delivered the ~37% for LEGACY parity. The residual
ASF→LEGACY gap is structural (glade bookkeeping) — further wins are in
libmarpa C-side bocage walking (out of scope).

---

## Build-pipeline (binary perf + size)

The release deliverable is a maximally-performant, smallest `latexml_oxide`
(`maxperf`: opt-3, fat-LTO, CGU=1, panic=abort, stripped,
`--no-default-features --features runtime-bindings`). **Prerequisite for any
`-Z build-std`/codegen lever:** pin the nightly (`rust-toolchain.toml`) so
codegen is reproducible (nightly churn renamed
`panic_immediate_abort` → `-Cpanic=immediate-abort` mid-evaluation once).

**Size is structural, not waste (decision 2026-06-11: accept ~47 MB).** The
binary is ~60,000 small functions (one per `\def`/construct), NOT a few fat
generics: `package + engine + contrib + core ≈ 17 MiB` of attributable binding
code is the cost of porting LaTeXML's whole macro surface to native code.
`[59740 Others] = 26.4 MiB (79% of .text)`. There is no single fat generic to
de-monomorphize → no cheap size lever; the only knobs (drop package coverage;
data-table binding encoding) both fight the project's goals. Dumps gzip to
~870 KB (not the size driver). `runtime-bindings` (rhai) costs +2.23 MB (~4.8%)
— shipping it is the current decision (runtime opt-in, default conversions
unaffected); a lean + `+bindings` two-artifact split is the clean fallback if
size becomes a hard requirement.

Reproduce the size breakdown (symbol-preserving, no-LTO so code stays
attributed to its origin crate):
```
CARGO_PROFILE_RELEASE_STRIP=false CARGO_PROFILE_RELEASE_DEBUG=1 \
CARGO_PROFILE_RELEASE_LTO=off \
cargo bloat --release --no-default-features --features runtime-bindings \
  --bin latexml_oxide --crates        # drop --crates, add -n 30 for per-function
```

---

## Standing performance corpus

Idle-serial CLI (no `cortex_worker`), publish-grade binary:

```bash
target/release/latexml_oxide \
  --preload=ar5iv.sty \
  --path=$HOME/git/ar5iv-bindings/bindings \
  --dest=/tmp/out.html --timeout=60 <main.tex>
```

Papers under `data/10k_sandbox/<id>.zip`; `complex/si.tex` in-tree. Helper:
`tools/run_perf_corpus.sh`.

### Baseline (2026-04-30, release)

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

**Regression trigger:** any corpus entry drifting **> +15%** wall vs the last
recorded baseline is a regression signal. Record a new dated sub-heading; do
not overwrite history.

**perf signatures:** `1011.1955` (3.78 s, single-core) is math/body-bound — top
symbols `marpa_r_earleme_complete` (7.5%), `postdot_items_create` (6.6%),
`bv_scan`, `marpa_b_new`, `transitive_closure`; `--nomathparse` makes the Marpa
band vanish. `1005.1610` (2.83 s, 3.9 CPUs) is parallel external-graphics-bound
(`gs`/`convert`/zlib in children; Rust-side Marpa <1%).

### Math-bound corpus measurement (HYBRID regression watch)

```bash
cargo build --release --bin cortex_worker --features cortex
tools/benchmark_canvas.sh --input-dir <math-bound-100-zips>/in \
  --output-dir /tmp/out_hybrid --workers 8 --timeout 180
# LEGACY control: prefix with `env LATEXML_MARPA_LEGACY=1`
```
Quiet-host baseline: HYBRID +0.5% vs LEGACY on n=98 both-OK. Re-run on every
meaningful marpa/math-parser change; flag if HYBRID climbs toward LEGACY.

---

## Optimisation acceptance checklist

Before merging a performance change:

1. Release-mode before/after for the standing corpus.
2. One targeted benchmark for the suspected bottleneck.
3. Compare output status + lightweight structural metrics (output-neutrality
   is non-negotiable — a perf change that alters output is a bug; verify with a
   structural diff, not just error counts).
4. Report wall, user/sys CPU, max RSS, phase timings.
5. State the expected workload boundary and any fallback path.
6. Keep the change easy to disable if it relies on a heuristic.

For math-parser changes additionally record: parse-count distribution, total
math-parse time, MathML/XMath count, formulae using a cache path. Review
structural math output on math-heavy fixtures before treating it as a win.

---

## Graphics — completed work (breadcrumbs for regression triage)

- **In-doc coalescing** (`48fd96ac75`) — `Plan::Copy`/`Plan::Convert` key on
  `(SipHash(content), graphicx_options)`. arXiv:2402.01336 1083 nodes → 17 files.
- **Persistent on-disk cache** — SHA-256 of `source‖page‖density‖target-ext` at
  `$XDG_CACHE_HOME/latexml-oxide/graphics/<aa>/<hash>.<ext>` + `.dims` sidecar
  (Perl `LaTeXML.cache` parity). Multi-process safe (tmp+atomic rename,
  hardlink-on-read, `flock` LRU). Warm 9.55→5.07 s on 1909.03909. Overrides:
  `LATEXML_GRAPHICS_CACHE_OFF=1`, `LATEXML_GRAPHICS_CACHE_DIR`,
  `LATEXML_GRAPHICS_CACHE_MAX_MB` (default 2048).
- **Vector-SVG fast path** (#902) — `--graphics-svg-threshold-kb N` bypasses
  ImageMagick for vector PDFs. `fig8.pdf` 32.4→0.3 s (~130×).
- **Vector-PDF auto-detect** — `cortex_worker` ar5iv profile passes
  `graphics_svg_threshold_kb: 0`; scans PDF header for `/Subtype /Image`,
  routes to SVG when absent and ≤500 KB. Overrides:
  `LATEXML_GRAPHICS_VECTOR_AUTO_OFF=1` or `--graphics-svg-threshold-kb N>0`.
- **Sandbox worker default 20 → 8** — gs/convert fork-exec contention made
  graphics-bound papers 5–10× slower at 20 workers; raise `--workers` only when
  the canvas is known compute-bound.

Output-size regression fixtures: `0809.3849`, `0908.3201`, `1003.0368`,
`0803.4343`, `0907.4282`.

---

## Mini-benchmark: beat 2× pdflatex on `1910.01256` — MET

0.71 s release (full post-processing) vs pdflatex idle ~1.11 s — 3.13× margin
on the 2.22 s gate. Re-measure under the SYNC_STATUS "Acceptance gates" recipe
after any large landing; flag if margin < 1.5×.
