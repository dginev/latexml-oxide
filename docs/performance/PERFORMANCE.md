# Performance Optimization Principles

Repeatable checklist + current lever state. Review before release
milestones, after major features, and during periodic optimisation
passes.

This doc holds the **timeless principles**, the **current open/closed
lever state**, and a dated **Audit log** of periodic passes. The per-paper
empirical campaign log (slowest-100 testbed, hotspot-by-hotspot deltas) lives
in [`ARXIV_PERFORMANCE.md`](ARXIV_PERFORMANCE.md); reliability witnesses
(timeout/OOM/hang) live in [`STABILITY_WITNESSES.md`](STABILITY_WITNESSES.md).
Detailed investigation narratives are in `git log` + commit messages —
this doc keeps outcomes, not sagas.

---

## Principles (the checklist)

### 1. Avoid string allocation on hot paths

Never `.to_string()`, `String::from()`, or `format!()` when the string is
already in the interner arena.

- **String literals**: the `pin!("…")` macro — it is the per-call-site
  `OnceCell<SymStr>` cache (thread-local; first call interns via `pin_static`,
  every later call is a branch+load, no arena access). **Policy (user,
  clarified 2026-07-02): always the faster arena behavior, syntax is
  irrelevant** — so `pin!` for any literal on a path that executes more than
  once. *(The 2026-07-02 audit corrected this doc: an earlier revision
  attributed the OnceCell mechanism to `pin_static` and called `pin!`
  deprecated — backwards. The cached `pin!` landed 2026-04-20,
  `df720961d7`.)*
- **`arena::pin_static("…")`** (zero-copy static intern, per-call arena
  probe) remains for the two places `pin!` doesn't fit: non-literal
  `&'static str` *values* (`pin_static(var)` — a macro can't cache a varying
  input), and genuinely one-shot init (`Lazy` statics, model/state setup)
  where the two are equal-cost.
- **Runtime strings**: `arena::pin(s)`.
- **Comparisons/reads**: `arena::with*` to read an existing `SymStr` without
  re-allocating.

```rust
// BAD                                   // GOOD (hot path)
token.text.to_string() == "endgroup"     token.text == pin!("endgroup")
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
rendering bug rather than relying on hash invalidation.

`pathname::kpsewhich` lookups are **memoized** (hits AND misses, thread-local,
keyed by the candidate list — landed 2026-07-02): repeated probes of the same
missing asset were a fresh kpathsea probe each time. Mechanism note (audit
correction): this call is the **kpathsea crate**, in-process when libkpathsea
is statically linked (all release/production builds) — NOT a fork-exec; the
subprocess-`kpsewhich` fallback only applies to portable builds without the
linked library (where the memo saves a real 10–20 ms spawn per repeat). The
only true `kpsewhich` *subprocesses* in a conversion binary are one-shot
startup/dumper paths (`dump_paths.rs` year-detect, `ini_tex.rs`).

### 6. No whole-tree `//` / `preceding::` scans inside per-node XSLT templates

**The recurring post-processing perf trap.** An XSLT `<xsl:value-of>` /
`<xsl:if>` whose XPath uses the descendant (`//`) or `preceding::` axis walks
the **entire document tree from the root**, yet runs **once per matched node**
→ O(nodes × tree-size) ≈ **O(n²)**. On a large book/thesis this pins XSLT at
60–150 s. The level/flag being computed is almost always a **document-global
constant** — hoist it into a single `<xsl:variable select="boolean(//…)"/>`
(evaluated once from the root) and reference the variable; or use the
Muenchian `<xsl:key>` method for distinct-by-value dedup. Output-neutral.

Three were found and fixed (all in `latexml_post/resources/XSLT/`, embedded at build time):
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

## Phase distribution (190k aggregate, 2026-05-02..03) — SUPERSEDED, historical

> **The canonical budget is now the 60,469-doc 2026-07-10 measurement** in
> [`ARXIV_PERFORMANCE.md`](ARXIV_PERFORMANCE.md) "Corpus-wide phase budget":
> digest 19.7% · math_parse 19.2% · build 18.1% · **xslt 13.2%** · graphics
> **8.9%** · mathml_pres 4.5%. Graphics fell 36.5% → 8.9% after the
> graphics-cache work, so any lever ranked off the table below — notably
> "P1 — graphics (36.5%)" and Principle 5's "graphics was the single largest
> corpus band" — is **mis-ranked**. XSLT is now the most under-exploited band.

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

The canonical corpus phase bands (digest 19.7%, math_parse 19.2%, build 18.1%, xslt 13.2%, graphics 8.9%, mathml 4.5%) and recent raw-kernel sweeps set the active priorities:

> **2026-09-03 source reconciliation:** the ranked static findings and
> implementation handoff are in
> [`PERFORMANCE_AUDIT_2026-09-03.md`](PERFORMANCE_AUDIT_2026-09-03.md).
> The first isolated change is the discarded XMDual serialization; the audit
> also covers its document-wide `idref` scan, the core-to-post output buffer,
> pass-2 per-segment cloning, the whole-DOM split fallback, retained state
> capacity, alternate-format XSLT, and file lookup. Runtime impact is unmeasured
> unless the entry names an existing profile; keep static cost models separate
> from measured percentages.

### P1 — XMDual pruning: discarded serialization, then repeated global lookup

* **Current reality:** `Document::prune_xmduals` unconditionally calls
  `node_to_string(&dual)` and discards the returned `String`. Separately,
  `collapse_xmdual` evaluates `//*[@idref='<dualid>']` from the document for
  each collapsed dual.
* **Cost shape:** definite serialization/allocation waste, followed by a
  worst-case `O(collapsed_duals * document_nodes)` reference-repair loop. The
  delta has not yet been measured.
* **Order:** first remove only the discarded serialization and run a
  same-session byte-identical A/B. Then, as a separate change, build one
  `idref -> nodes` index and update its buckets when ids move. See audit F1/F2
  for edge cases and witnesses.

### P2 — Complete streaming across the core-to-post boundary

* **Current reality:** fragmented core conversion and the two-pass streaming
  split front-end are implemented. However, `ConversionResponse.result` still
  materializes complete core XML as a `String`; TeX-to-HTML passes that string
  to post, which may immediately spill it to disk. Pass 2 also clones the
  conversion-global font map and rewrite rules per segment and constructs an
  unused XML scaffold per fragment.
* **Fix:** add a writer/file-backed conversion API while retaining the current
  string API as a compatibility wrapper. Hand the CLI's file directly to post.
  Independently, share immutable pass-2 data, use fragment-local overlays, make
  rewrite diagnostics lazy, and construct fragment fields without `Self::new()`.
* **Boundary:** this attacks very-large-document RSS and segmented pass-2 CPU;
  it is not expected to move ordinary-paper medians. See audit F3/F4.

### P3 — math_parse (19.2% of wall, 17% over-parse)

Every math-heavy witness is now `math_parse`-bound. The over-parse rate is the primary lever; see **Principle 4**, [`MATH_OVERPARSE_DEEP_DIVE_2026-06-30.md`](../math/MATH_OVERPARSE_DEEP_DIVE_2026-06-30.md) and [`MATH_PARSER_AND_ASF.md`](../math/MATH_PARSER_AND_ASF.md).

* **Landed 2026-06-30 — differential-`d` lexer gating:** Downgrades `XDIFFUNK→UNKNOWN`/`XDIFFID→ID` when the formula has no `INTOP`, removing over-parse on every non-integral `d` (`\frac{dx}{dt}`, subscripts).
* **Settled intentional divergence:** `f(x,y)` apply-vs-multiply is intentional divergence #18 (`OXIDIZED_DESIGN_MATH.md` §18; do not re-attempt toward-Perl reverts without explicit user sign-off).
* **Open hot patterns:**
  - **Integrals (largest volume driver):** Step 2 of differential gating — a dedicated in-integral `DIFFOP_D` terminal so `∫(x·d·x)` is never built, pulling `\int … f(x)\,dx` off the legacy fallback path.
  - **Bare `|x|` with ambiguous inner content:** e.g. `|v(x)| ≤ |v(x')|` (625 and-nodes): balanced-pair pre-lexer pass targeting the pairing factor.
  - **Content-addressed formula memoization (BP-5):** Hash normalized formula token stream to reuse parse→XMDual→MathML across identical formulae in tables and matrices.

### P4 — Internal TeX counters in `State` (`if_count` / `if_limit`)

* **Current Reality:** `Conditional::invoke` calls `assign_value_sym::<i64>` with `Scope::Global`, walking every undo frame and performing per-frame hashbrown `remove_entry` (2.4% self-time on digest witnesses), plus the per-assignment `\globaldefs` probe (`state.rs:841`).
* **Fix:** Migrate internal TeX counters (`if_count`, `if_limit`, `tracingcommands`) to dedicated typed fields on `State`, eliminating the undo-frame walk entirely while preserving dump-filter compatibility.

### P5 — tikz-cd / pgf native digest volume & `Tokens` allocation

* **Current Reality:** tikz-cd and pgf emit thousands of small math formulae (one per cell/arrow/label — up to 6,800+ in a single document). Cost is formula count × (digest + math_parse + build), NOT external graphics.
* **Levers:** (1) reduce per-cell formula count in bindings; (2) lazy `Tokens::Debug`; (3) return `Option<SymStr>` from `lookup_value*` to drop `Cow::Borrowed`; (4) pgfplots `\addplot table` direct Rust bypass.
* *Note on SmallVec:* SmallVec-backed `Tokens` was tried and regressed (struct bloat); do not retry without shrinking `Token` below 8 bytes.

### P6 — Large-document fallback and retained-state memory

* **Whole-DOM split fallback:** `Split::process_pages` repeatedly removes and
  inserts at the front of `Vec`s and runs two ancestor XPath queries per page.
  The streaming front-end avoids this only when its input, destination, union,
  and size gate are eligible. Linearize the eager fallback with a deque/owned
  iterator and direct parent walks (audit F5).
* **State templates:** `STD_STATE` and `STY_STATE` retain the same 131,072-slot
  meaning-table reservation used to absorb the active state's dump. Give the
  templates a lean capacity profile and avoid constructing default maps that
  `State::new` immediately replaces (audit F6).
* **Hard constraint:** retain libxml2 for dynamic `DefRewrite` XPath. The
  fragmented architecture is the implemented solution to the measured DOM
  floor; wholesale DOM replacement remains a settled non-lever.


### P7 — Thermal & Concurrency Budget Limits (`docs/THERMALS.md`)

* **Current Reality:** On dev laptops (Intel hybrid P/E-core CPUs, e.g. i7-12800H), sustained multi-job execution pins temperatures at 95–96 °C with severe CPU clock throttling. Running sweeps alongside tests causes 100% swap exhaustion and hundreds of throttle events per second.
* **Operational Limits:**
  - Standalone sweeps/benchmarks: `JOBS=6..8` maximum (`JOBS=6` is quiet; `JOBS=8` throttles mildly).
  - Sweeps alongside other tasks: `JOBS=4` maximum.
  - Memory ceiling: Keep `JOBS × --max-memory <= 24 GB` to preserve headroom for rust-analyzer (~4 GB) and OS buffers.

### P8 — Lower-frequency global scans and lookup allocation

* **JATS/TEI:** both alternate stylesheets match paragraphs with
  `preceding::ltx:section`, a potential per-paragraph document scan. Establish
  intended scope, replace with a structural/keyed test, and require
  byte-identical output on scaling fixtures (audit F8).
* **File fallback:** the two fallback helpers compile the same regexes per
  invocation; lookup clones search paths and scans freshly materialized binding
  registries twice. Use `Lazy<Regex>`, a borrowed path view, and prebuilt exact
  plus lowercase indexes after collecting miss-count telemetry (audit F9).


---

## Audit log (periodic passes; newest first)

### 2026-09-03 — read-only algorithm and memory audit

Audited the current working tree against the performance, large-document,
streaming, startup, telemetry, and thermal documentation. No code was changed
and no fresh measurements were taken. The ranked findings, cost models,
implementation boundaries, validation matrix, stale-doc reconciliation, and
resume checklist are recorded in
[`PERFORMANCE_AUDIT_2026-09-03.md`](PERFORMANCE_AUDIT_2026-09-03.md).

The immediate next patch is deliberately narrow: remove the unused
`Document::prune_xmduals` subtree serialization and run a same-session,
byte-identical A/B. Keep the subsequent one-pass `idref` index separate. The
largest architectural residual is that fragmented core output is still
materialized as a document-sized `String` before the CLI hands it to post.

### 2026-09-03 — Wave 15 / Batch 54 & WebAssembly audit pass: interner hygiene, macro cycle fast-fail, and thermal budgeting

Investigation during the Wave 15 / Batch 54r sweep series (`perfect_kernel` branch) and the Stage 4 WebAssembly compatibility audit (see [`WASM_COMPATIBILITY_AUDIT.md`](../release/WASM_COMPATIBILITY_AUDIT.md) and [`HANDOFF_2026-09-03.md`](../perfect_kernel/HANDOFF_2026-09-03.md)):

1. **`SymHashMap` negative-probe interner pollution — already resolved:**
   The candidate was valid, but the current source already probes with
   `arena::get` before map lookup. Keep this as provenance, not open work.
2. **Macro-cycle fast-fail — already resolved:**
   Duty-cycled gullet/stomach guards are present, and the source-scaled token
   backstop now defaults to 4 billion. The older 1-billion/current-reality text
   was stale; do not implement a second independent ring.
3. **C-FFI decoupling of `marpa-asf` from `latexml_core`:**
   Audited `latexml_core/src/common/error.rs`: `marpa-asf` was pulled into core solely for `impl From<marpa::error::Error> for Error`. Relocating this to `latexml_math_parser` frees `latexml_core`, `latexml_engine`, and `latexml_package` from compiling C Marpa code.
4. **Codehigh LuaTeX O(n²) parser timeout (Batch 54k):**
   `codehigh` package documentation was spinning indefinitely in its Lua parser emulation; falling back to plain verbatim for this path brought the document from 180s timeout to <1s (`86e764fda4`).
5. **Host thermal throttling & memory budget (`docs/THERMALS.md`):**
   Documented host limits on Intel hybrid i7-12800H: running `sweep.sh` (xargs -P 10, up to 6 GB each) concurrently with `cargo nextest -j 8` causes 100% swap fill (8 GB) and severe thermal throttling (700+ throttle events/5s at 96 °C). Established hard operational rules: `JOBS=6..8` alone, `JOBS=4` alongside other tasks; `JOBS × --max-memory <= 24 GB`.

### 2026-08-23 — pre-0.7.6 diagnostic-only audit: eager-Debug! band + ranked backlog

Idle-box pass at `80999906da` (release build with symbols; 82-paper
`~/data/html_regressions/sandbox` corpus — serial sweeps, `perf --call-graph
lbr` on three phase-distinct witnesses, dhat allocation pass, clippy perf
sweep). **Diagnostic only — no code changed.** No regression since 2026-07-29:
`2405.14114` reproduces its post-guard wall (21.1 vs 21.45 s), and none of the
248 commits since added hot-path code (the default HTML5 XSLT still had zero
per-node `//` / `preceding::` scans; JATS/TEI were outside that audit).
Healthy-paper RSS p50 285 MB / max ~1 GB — normal-path
memory is fine. Ranked findings, all output-neutral by construction:

1. **Eager `Debug!` diagnostics — the headline (now Open levers P0).** On
   witness `2304.10050` (6.3 s, build-bound): `node_to_string` of the current
   subtree per text insert / element close ≈ **26%** of total conversion
   (children), plus `<str as Debug>::fmt` 7.1% self, fmt plumbing ~4%, and
   `generate_message!`'s eager `get_location()` → `pathname::split` ~2.6%.
   Cross-witness band: ~6–8% on a typical math paper (`2408.08292`), ~1–2% on
   the token-runaway digest witness (`2405.14114`). Shape is
   quadratic-flavored (each insert re-serializes the growing subtree). dhat
   confirms the memory side: the `open_text`/`open_text_internal` Debug! sites
   are ~440k blocks / ~84 MB churn on a 1.3 s paper (`2402.14207`).
   Token-frequency sites `constructor.rs:305` / `primitive.rs:101` pay one
   format+alloc per primitive invocation.
2. **Global `if_count` per conditional** (~4–5% on digest-heavy papers):
   `Conditional::invoke` → `assign_value_sym::<i64>` Global-assign walks every
   undo frame + per-frame hashbrown `remove_entry` (2.39% self on
   `2405.14114`), plus the per-assignment `\globaldefs` probe
   (`state.rs:841`). Faithful Perl semantics; the typed-`State`-field
   translation for LaTeXML-internal counters is the fix — still needs the
   dump-filter + `if_stack` review flagged 2026-07-29.
3. **`is_noexpand_family` string probe** — still 1.99% self on the digest
   witness; intern-time flag bit (SymStr-indexed bitvec) remains the fix.
4. **libxml2 on glibc malloc ≈ 11.3% self** on `2408.08292` (Rust side is on
   mimalloc). The closed `xmlMemSetup`→mimalloc experiment (2026-07-31) was
   confounded by the soft-yield RSS trigger on the 131 MB streaming witness;
   the ordinary-paper CPU case was never isolated. Re-open candidate with
   segmentation pinned; fork branch `feat-xml-mem-setup` has the wrapper.
5. **DOM/XPath mechanics**: `collect_walk_matches`
   (`latexml_post/document.rs:320`) — the traversal engine for every
   whole-document post query — allocates a `get_child_nodes()` Vec per
   recursion level (2.4% self + allocator share); its sibling
   `collect_split_pages` already uses `get_first_child`/`get_next_sibling`.
   `generate_id` (`document.rs:5931`) runs `ancestor::*[@xml:id][1]` through
   full XPath parse+eval per id-lacking node in `finalize_rec` — a direct
   parent-chain walk is equivalent. `XPath::findnodes` re-parses its
   expression string every call — a compiled-XPath cache would shave all
   repeated sites. `Node::_wrap` 4.3% self and per-FFI `CString` ~0.85% are
   rust-libxml API-shape costs (upstream candidates).
6. **Churn items**: `preload_font_map`/`load_font_map`
   (`content.rs:3507-3528`) re-`format!` the `"{encoding}_fontmap"` key on
   every per-character `decode_string` call — ~1M allocations on the 1.3 s
   dhat paper; memoize the key/Fontmap per encoding. `install_definition`'s
   `s!("{cs}:locked")` per `\def` (also interns a permanent `:locked` twin per
   cs; all writers are `Scope::Global`, so a side-set is the shape).
   `get_search_paths()` materializes `Vec<String>` per file probe.
   `Table = FxHashMap<SymStr, VecDeque<Stored>>` pays a heap VecDeque
   pointer-chase per meaning lookup where an inline-one-binding enum would do.
   Clippy perf sweep: 15 redundant-clone lints in package/engine/contrib/post
   (lib core is clean).

Tooling papercut: `LATEXML_TELEMETRY_OUT` truncates per job
(`File::create` in `write_telemetry_record`) — batch runs keep only the last
record though `perf_phase_summary.py` documents JSONL; switch to append.

Witness commands + full working notes: session scratchpad
`PERF_AUDIT_2026-08-23.md` (reproduce: symbols-kept release build,
`perf record --call-graph lbr`, decode with `perf report --no-inline` — fast,
vs ~15 min with inline resolution; read the `cpu_core` table).

### 2026-07-29 — per-token guard overhead: one-borrow gullet read + duty-cycled cycle guard + pinned hot keys

Fresh idle-box profiling pass (post-fleet), driven by an 82-paper
`~/data/html_regressions` telemetry sweep (release binary; the corpus is
regression-biased — digest 67% of its wall) and `perf` on its slowest
witness **2405.14114** (pgfplots, 23.9 s, 96% digest, 200M+ token stream).
Top self-time: `cycle_guard_checkpoint` **6.07%** + `read_resource_checkpoint`
**4.16%** — the engine's own per-token safety guards costing ~10% of the
conversion — plus per-call `arena::pin` interner traffic ~5% (repeated
keys/paths) and `is_noexpand_family` ~2%.

Three output-neutral changes landed (byte-identical HTML on both witnesses,
suite 1756/0):

1. **One-borrow combined read** (`gullet.rs::read_internal_token_checked`):
   the per-token trio `read_resource_checkpoint` → `read_internal_token` →
   `cycle_guard_checkpoint` (up to three thread-local `RefCell` borrows per
   token) merged into a single `GULLET` borrow shared by all four reader
   loops; limit breaches Fatal via `#[cold]` outlined helpers, messages and
   debug dumps preserved verbatim; a breach still consumes no token.
   `read_balanced` keeps comments in its result via a `CommentSink` param.
2. **Duty-cycled active cycle guard**: above the activation floor the guard
   fingerprints only 2048 of every 16384 tokens (ring reset at each ON-window
   start). A genuine infinite loop is *persistent*, so detection stays
   guaranteed within ~one duty period (~17k tokens — 4 orders of magnitude
   under the 400M token-limit backstop); verified live,
   `\def\y{}\def\x{\y\x}\x` still trips `Fatal:Timeout:Recursion` in 0.6 s.
   A legit huge stream (pgfplots data plots run 200M+ tokens, past even the
   150M graphics floor) stops paying a per-token fingerprint for the whole
   remainder of the run.
3. **Pinned hot keys** (mechanical `pin!`/`_sym` conversions): `Mouth` caches
   `source_sym` at construction so `get_locator`/`get_locator_from_start`
   build `Locator`s with zero interner probes (was: re-pin the source path
   per call — per conditional, per box); `Conditional::invoke` uses
   `pin!`-keyed `lookup_int_sym`/`assign_value_sym` for
   `if_count`/`if_limit`/`tracingcommands`; `after_assignment` →
   `remove_value_sym(pin!("afterAssignment"))`; `assign_internal`'s unscoped
   path probes `get_prefix_sym(pin!("global"))`. New `_sym` siblings:
   `lookup_int_sym`, `remove_value_sym`, `State::get_prefix_sym`,
   `Locator::from_sym`.

**Measured (same-session, interleaved best-of-3, release build):**
2405.14114 **23.85 → 21.45 s (−10.1%)** (borrow-merge+duty-cycle −6.7%,
pinned keys −3.4%), RSS flat ~690 MB, output byte-identical vs the pre-patch
binary; 1911.09517 (math-parse-bound, guard inactive) 5.95 → 5.88 s, output
identical. The 82-paper sweep sum went **192.2 → 185.6 s (−3.4%)** —
indicative only (the before-sweep ran with mild co-load; single run per
paper), with the digest tail carrying the win (2405.14114 −2.3 s, 2405.14573
−1.6 s, 2404.05509 −1.3 s). Post-patch profile confirms: the three former
functions (13.25% combined self-time) are one merged function at 9.01%, and
`get_or_intern` dropped off the >1.2% list.

**Profiling-method notes for this box** (Linux, hybrid P/E-core CPU):
`perf_event_paranoid` is 4 — set to 1 via sudo for the session, restored
after. perf emits TWO event tables (`cpu_atom`/`cpu_core`); the watchdog
thread's 100 ms poll wakeups dominate the tiny `cpu_atom` table (66% of its
samples) — a sampling artifact, not real CPU (`user+sys ≈ wall`); read the
`cpu_core` table. DWARF call-graph decode yields empty user stacks here; use
`--call-graph lbr`. callgrind records the main binary with an empty object
name (all-`???` annotation) on this box — use perf instead.

**Follow-up candidates (profiled, not yet done):**
- `is_noexpand_family` string probe — now the top discretionary residual
  (2.07% post-patch): record a "starts with `\special_relax`" bit per symbol
  at intern time (arena-side bitvec) → bit test.
- `install_definition` allocates + pins `"{cs}:locked"` per `\def`/`\let`
  (~0.7% + churn; Perl pays the same — a side-set of locked syms needs an
  assignment-hook design first).
- `assign_internal` Global-scope undo-frame walk (~4% incl. hashbrown
  `remove_entry` 1.85%) is faithful Perl State semantics (Perl also
  Global-assigns `if_count` per conditional) — a typed `State` field for
  LaTeXML-internal counters would be the "meaningful Rust types" translation,
  but needs dump-filter + `if_stack` review before attempting.

### 2026-07-06 — CrossRef O(n²)→O(n) on very-large split docs

Post-processing the 40 201-page `index.xml` witness (see
`docs/performance/STREAMING_POST_DESIGN_2026-07-06.md`) was dominated by **CrossRef at
40 min 47 s = 95 % of a 42 min 50 s run**. `CrossRef::process` runs once per
split page, and two per-page passes scanned *global* state — a latent
quadratic exposed only once split fires at 40 k-page scale (huge docs used to
collapse to one page):
- `fill_in_frags` iterated the **whole ObjectDB per page** (an inversion tuned
  for single math-heavy docs). Restored Perl's `//@xml:id` page-node walk,
  keeping the inverted loop only when a page has more id-nodes than the DB.
- `fill_in_relations`→`get_child_page_ids` **rebuilt+scanned a parent's full
  child-page list per sibling**. Memoized it (ObjectDB is read-only for the
  pass) with a position index, so `find_previous/next_page_id` are O(1).

Result (commit `4ec2587993`): CrossRef **40 min 47 s → 6.1 s**, whole run
**42 min 50 s → 2 min 18 s (18.6×)**, **byte-identical** output over all 40 201
pages (`diff -rq` clean) + synthetic 2K/4K docs (SHA-256 match); CrossRef now
scales linearly (67→145 ms for 2× pages). process_chain (per-page
XSLT+MathML+serialize+write, ~2 ms/page, linear) is now the long pole at ~85 s;
peak RSS (~21.6 GB) is unchanged — a memory concern tracked separately.

### 2026-07-02 — fleet-concurrent audit (idle re-baseline deferred)

Run **while the full-arXiv fleet occupied the box** (72 workers, load ~85), so
per the measurement discipline no absolute wall-clock numbers were taken —
scope was static/code checks, artifact checks, and live-fleet observation.

**Live-fleet observation** (corpus `arXiv` 2.82 M docs, `cortex_worker`
maxperf-cortex, one-conversion-per-process; numbers are contention-inflated
fleet context, NOT single-process baselines):
- Throughput **~44 k docs/hr** at 72 workers (normal band; ~44 h to finish).
- Per-doc wall (`runtime_ms`, n = 884,671 finalized): **avg 4.06 s, p50
  2.29 s, p90 9.02 s, p99 24.8 s, max 180 s** (the cortex timeout cap).
- Fatal rate 0.78% of completed; the perf-signal slice: `Timeout:
  PushbackLimit` 1,123, `TokenLimit` 718, `Recursion` 250, `IfLimit` 140
  (runaway guards, ~0.25% of done), `never_completed_with_retries` 1,069.

**Checks & outcomes:**
- **XSLT O(n²) re-audit — HOLDS.** Only XSLT change since the 2026-06-29
  zero-per-node-scan audit is the maketitle memoize fix itself; remaining `//`
  uses are document-global params/variables (verified `classPI`,
  `LaTeXML-common` date, jats/tei doc-level templates).
- **Spawn-site inventory — per design.** All runtime `Command::new` sites are
  the cached/coalesced graphics converters or one-shot startup/dumper
  `kpsewhich`; `line_fontmap`'s tftopl is `#[cfg(test)]`-only. Doc corrections:
  Principle 5's fork-exec claim for image lookups was mis-attributed (the call
  is the in-process kpathsea crate in production builds); the lookup memo
  landed anyway (subprocess-backend builds benefit fully).
- **Self-contained invariant — holds by design** (disk-first in the dev tree,
  embedded fallback for shipped binaries; strace showed the expected dev-tree
  reads of dumps/XSLT/CSS). The definitive rename-away re-verification is
  deferred — the running fleet reads `resources/dumps/` at every worker spawn;
  do not perturb mid-run.
- **Binary size — no drift.** `release/latexml_oxide` 47.1 MB (accepted ~47 MB
  decision, 2026-06-11); `maxperf-cortex/cortex_worker` 52.5 MB.
- **Clippy `-W clippy::perf -W clippy::redundant_clone`** — perf lints clean
  (deny-gated baseline); 7 lib-code redundant clones found: 3 in the
  `count_nested_same_fence` tie-break walk (**fixed** — walk now threads
  `Option<&str>`, killing a per-Apply `String::from` + per-node clones), 3
  cold ones fixed (`content.rs` load guard, `biblatex_sty` label,
  `latexml_sty` replace-tokens), 1 skipped as FP-suspect
  (`latex_constructs.rs:913` — `ctr` is used after the flagged clone; nursery
  lint caution).
- **pin!/pin_static doc correction** (Principle 1): the call-site-cached
  OnceCell mechanism belongs to the `pin!` macro (since 2026-04-20), not
  `pin_static`; an earlier doc revision had it backwards. Trade-off as
  measured from the code: `pin!` = fastest repeated call (branch+load) at the
  cost of a per-site thread-local static; `pin_static` = per-call arena probe,
  no per-site static. **Policy settled 2026-07-02 (user): always the faster
  behavior, irrespective of syntax** — `pin!` for repeated-path literals;
  `pin_static` only for non-literal `&'static str` values and one-shot init
  where the forms are equal-cost. The same-day follow-up sweep converted the
  ~101 literal `pin_static` sites in warm/hot files to `pin!` (per-element
  `Tag!("ltx:*")` compares in `base_xmath`, `get_node_qname`'s literal
  branches, constructor closures across engine/package/contrib); `token.rs`
  `Lazy` statics and state/model init keep `pin_static` (equal-cost
  one-shots). This retires the earlier "sweep pin! → pin_static" direction,
  which rested on the swapped doc text.
- **Commits since 2026-06-27 (81) reviewed for hot-path additions.** One watch
  item: the noexpand redesign (`6ac88769eb`+) put `is_noexpand_family()` — an
  arena `with_str` + short prefix memcmp — inside `meaning_key`, i.e. on the
  per-CS-token meaning-lookup path (×2 probes/token via
  read_x_token/invoke_token). Estimated a few ns/token; include in the
  post-fleet A/B (below). If it shows, the fix direction is a Token flag bit,
  not string checks. Logger inline notes and the ambiguous-math diagnostics
  are gated/cold — fine.

**Landed from this audit:** the `pathname::kpsewhich` thread-local memo
(hits+misses, 4096-entry epoch bound) and the clone/borrow fixes above — all
output-neutral (suite green).

**Deferred follow-ups (post-fleet, idle box):**
1. **Standing-corpus re-baseline** vs the (stale) 2026-04-30 table + paired
   `tools/perf_compare.py` on telemetry runs — the noexpand redesign and the
   June fix wave have never been idle-A/B'd.
2. Rename-away re-verification of the self-contained invariant.
3. The `speculative_prefix_apply` `MATHPARSER_SPECULATE` gate check (already
   under P1 math_parse) — parity first, then cost.

---

## Closed levers (do not reopen without new evidence)

One-line outcomes; detail in `git log` + commit messages.

- **`SymHashMap` negative string probes — FIXED in the 2026-09-03 source snapshot.**
  `get`/`get_mut`/`contains_key`/`remove` resolve with non-interning
  `arena::get`, so misses do not grow the thread-local arena. No isolated A/B
  was recorded; retain the invariant and its unit coverage.
- **Eager `Debug!` diagnostics on text-absorption path — FIXED (`80999906da`, 2026-08-23).**
  `Debug!` and `generate_message!` built debug strings and serialized XML subtrees via `node_to_string` before checking verbosity gates. Gating on `debug_record_enabled()` cut `2304.10050` from **6.21 → 2.71 s (−56%)**, and 82-paper sweep from **229.3 → 200.0 s (−12.8%)**, with byte-identical output.
- **FxHash libxml node-cache — FIXED & SHIPPED (2026-07-20).**
  Replaced std SipHash `RandomState` in `rust-libxml`'s `xmlNodePtr → Node` wrapper cache with a dependency-free FxHash pointer hasher. Wall time on node-heavy phases dropped by **~28–30%** (`1510.03361` 19.6→14.1 s; `1805.03265` tikz-cd 22.4→15.7 s). Published in **`libxml 0.3.16`** on crates.io.
- **DOM traversal mechanics — FIXED (2026-08-23).**
  `collect_walk_matches` sibling traversal (`get_first_child` / `get_next_sibling`) eliminated per-recursion-level `Vec<Node>` allocations (~2.4% self-time). `generate_id` parent-chain walk replaced per-call XPath `ancestor::*[@xml:id][1]` evaluation.
- **`is_noexpand_family` and fontmap memos — FIXED (2026-08-23).**
  Arena-indexed symbol vector memos eliminated string scanning on token meaning lookups and font mapping.
- **UTF-8 SIMD fastpath on `.cls`/`.sty` scan — FIXED (2026-08-18).**
  Replaced grapheme-aware lossy decode with byte-range scan, cutting ~3% CPU during package dependency scans.
- **Graphics pipeline (36.5% → 8.9% wall) — FIXED.**
  In-doc coalescing (`48fd96ac75`), persistent disk cache, vector-SVG fast path (`fig8.pdf` 32.4→0.3 s, ~130×), vector-PDF auto-detect, and worker count bounding (8 workers max).
- **Codehigh LuaTeX O(n²) parser timeout — FIXED (Batch 54k, `86e764fda4`, 2026-09-02).**
  Degraded unsupported codehigh LuaTeX tokenizer path to plain verbatim, eliminating multi-minute timeouts across documentation sweeps.
- **One-Borrow Gullet Checkpoint & Duty-Cycled Cycle Guard — FIXED (2026-07-29).**
  Merged 3 RefCell borrows into 1 in gullet token reading; duty-cycled active cycle guard (2,048 of every 16,384 tokens) cut 2405.14114 by −10.1%.
- **CrossRef O(n²) → O(n) on 40k split docs — FIXED (`4ec2587993`, 2026-07-06).**
  Restored page-node walk in `fill_in_frags` and memoized sibling index in `fill_in_relations`; dropped CrossRef on 40,201-page split doc from 40m47s to 6.1s (18.6× whole-run speedup).
- **`build` phase quadratic — FIXED (`335b6b83`, ~20×).** `math0605199`
  44.9 s → 2.1 s. Hoisted `record_node_ids` out of grandchild move loop; build is now linear (~0.8 ms/formula).
- **P1 digest + build (pure-Rust hot path) — CLOSED 2026-05-19.** Residual
  digest cost is structural to TeX semantics, not a translation accident. perf
  floor is the `state.meaning` SwissTable double-probe.
- **dhat allocation sweep — DONE (faithful, output byte-identical).** Cut
  multi-GB of *churn* (allocator pressure / RSS) via `serialize_aux` growing buffer,
  tag action list borrowing, and in-place `fixedformat`/`get_node_qname`.
- **XSLT deep-DOM copy + max-depth — DONE.** `dup()` → `Rc clone()`
  (−120–130 MB/paper); `xsltMaxDepth = 1000` graceful abort vs OOM.
- **PGO / `target-cpu` (v3/native) — NO GAIN, closed.** maxperf is already at
  the fat-LTO + CGU1 ceiling; engine isn't SIMD-amenable (branchy catcode/macro dispatch).
- **Startup dump-parse lever (~50 ms of ~161 ms floor) — declined** as too
  small for release-critical risk; amortized to noise on long papers.
- **`build-std` (panic_abort) — PARKED.** −0.11 MB (0.2%); `.eh_frame` is from
  static C deps (mimalloc/libmarpa/zstd).


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

### Baseline (2026-04-30, release) — STALE, re-baseline scheduled

**The 2026-07-02 audit flags this baseline as two months stale** (many engine
changes since, incl. the noexpand redesign — see the audit log). Re-run the
corpus on an idle box after the full-arXiv fleet completes (~2026-07-04) and
record a new dated sub-heading below.

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
# --no-default-features drops runtime-bindings (a default) so the untrusted-input
# worker has no Rhai/command-exec surface (SAFETY.md); also drops test-utils.
cargo build --release --bin cortex_worker --no-default-features --features cortex
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

## Closed investigations 2026-07-31 (131 MB witness campaign) — do not re-attempt without new conditions

Both measured on `flat_index.tex` at `--max-memory 48000`, maxperf, against the
campaign baseline (32:56 wall / 1942.9 s user, md5 `df589fcfd8…`; full series in
the STREAMING_CORE_DESIGN "PERF CONSOLIDATION" entry).

- **libxml2→mimalloc routing (`xmlMemSetup`): CLOSED — slower, reverted.**
  36:20 (+10%) vs a pre-registered >10%-faster keep bar. Not allocator
  overhead: the routing changed the RSS trajectory and the soft-RSS yield
  trigger feeds on RSS — 1,507 → 1,738,832 yields, 6,050 → 37,945 segments,
  and per-segment overhead ate the win. Peak RSS **−19%** (31.5 → 25.5 GB) is
  real: re-attempt ONLY for a memory-bound target, with segmentation pinned
  (fixed `LATEXML_SPILL_AT_MIB`/floor) so the trigger cannot confound. Fork
  branch `feat-xml-mem-setup` (rust-libxml, unmerged) has the wrapper ready.
- **MathParse ambiguity reduction: CLOSED — no lever exists on this workload.**
  The hybrid dispatch routes by RAW Marpa ambiguity (unambiguous → cheap
  tree-iter, no ASF). On the 19.9 MB witness slice the ASF never executed at
  all (`MARPA_ASF_STATS` snapshot `None` — that IS the measurement); on
  `si.tex` ASF engages but `max_factorings=1`. There is no
  discarded-enumeration pile to prune. MathParse's measured 41% ≈ 1.55 ms per
  formula of recognition + tree build + semantics + FFI: constant-factor
  levers only, no ≥40% single technique in the current architecture.

## Slow-call audit (perfect-kernel corpus) — user directive 2026-09-05

A conversion that takes more than about a minute must be **justified** by its
output: a large, highly structured, content-preserving XML/HTML. Anything else
is a performance root, and a long run that ends in a fatal with no output is
the worst case (time spent, nothing delivered).

Tool: `tools/perfect_kernel/slow_calls.sh <sweep_dir> [threshold_secs=60]` —
one row per slow document with the source size, the XML size, structure counts
(sections, `<Math>`, `<tabular>`, `<figure>`, `<picture>`/`<svg>`), the KB/s
rate and a verdict: `JUSTIFIED` (≥ 25 KB of XML per second, or ≥ 2 MB of
XML), `TIMEOUT` (killed at the cap), else `SUSPECT`. Run it after every sweep
and carry the SUSPECT/TIMEOUT rows into the LEDGER's sweep row as a perf
cluster (root-caused like any other: witness, mechanism, one lever per run).

Sweep #41 baseline (batch 56i release, 300 s cap): 65 calls over 60 s —
17 justified (source3 18 MB at 75 KB/s, circuitikzmanual 12.8 MB at 116 KB/s,
unicodefonttable 15.9 MB at 229 KB/s …), 14 timeouts (tzplot, tutodoc ×2 —
fixed in 56j — spreadtab ×2, pgf-periodictable, pgf-interference ×2,
latexsheet-esmx, kaytannollista-latexia, jpneduenumerate, chemobabel ×2,
bibleref-parse), 34 suspect, of which 21 ran 60–270 s and then died with a
0-byte XML (tikz-network 271 s, wtref-ja 242 s, pgf-spectraPreviewDataLSE
241 s, wheelchart 207 s, glossaries-extra-manual 155 s, datatool-user 144 s,
glossaries-user 144 s, tcolorbox 101 s, Explications_ScratchX 100 s, istgame
88 s, xebaposter, lie-hasse, handout, quran ×2, texnegar ×3, polyglossia,
tikz-among-us, latexbangla, expkv-bundle) and 13 delivered small outputs
slowly (platexcheat ×4 at ~265 s for 250 KB, tabularray 216 s for 1.9 MB,
rulercompass 103 s for 81 KB, tilings, graph35, functional, spath3,
tkz-grapheur-exemples). The 0-byte-fatal group is the first target: those
runs are pure waste.

