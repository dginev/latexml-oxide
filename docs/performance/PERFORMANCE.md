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

### P4 — Internal TeX counters in `State` (`if_count` / `if_limit`) — LANDED (batch 56db, 2026-09-18)

* **Was:** `Conditional::invoke` called `assign_value_sym::<i64>` with `Scope::Global`, walking every undo frame and performing per-frame hashbrown `remove_entry` (2.4% self-time on digest witnesses), plus the per-assignment `\globaldefs` probe.
* **Fix:** `if_count`/`if_limit` and `absorb_count`/`absorb_limit` are typed `State` fields (`next_if_id()`, `if_limit()`, `next_absorb_id()`, `absorb_limit()`); the general mechanism is *a Perl plain global, or a Value only ever assigned `'global'` and never dumped, becomes a typed State scalar*. `tracingcommands` is excluded: it is a group-scoped count register (TeX_Debugging.pool.ltxml:213-225), so it must keep the Value table. picC −4.4 % instructions.

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

Investigation during the Wave 15 / Batch 54r sweep series (`perfect_kernel` branch) and the Stage 4 WebAssembly compatibility audit (see [`WASM_COMPATIBILITY_AUDIT_2026-09-03.md`](../release/WASM_COMPATIBILITY_AUDIT_2026-09-03.md)):

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
`docs/archive/STREAMING_POST_DESIGN_2026-07-06.md`) was dominated by **CrossRef at
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

## OPEN PROGRAM 2026-09-18 — pgf/TikZ throughput: reach pdflatex speed (user directive; the first profile below was UNSYMBOLIZED and proves nothing about flatness)

`tikz-network.tex` (170 TikZ pictures, 0 errors) is the perfect-kernel corpus's
largest sanctioned slow call at ~222 s under the sweep; pdflatex needs 34 s for
the same manual (the intrinsic-cost oracle), so latexml-oxide runs at ~6× native
on real pgf expansion (every `\Vertex` fires ~10 `\tikzset` + `\ifthenelse` +
pgfkeys + a pgf scope, tikz-network.sty:407-520). `perf record` over the full run
(root-causer `~/data/pk_agents/w23/regr85/tikz-network-perf/`, `perf_top40_*`):
Rust engine 84 % self time with **no symbol above 0.7 %** (token inner loop —
`read_cs_name_inner`, `Mouth::open`, `read_dimension`), libxml2 0.08 %, no math
(pictures emit `svg:g`). Not quadratic, not clone churn, not the parser — a diffuse
constant factor; the only engine-wide gains are the sub-1 % token-throughput
items above. The one measurable band, kpsewhich subprocesses at 8–15 %, appears
ONLY when `TEXMF*` is left unpinned (the dual-TL guard, pathname.rs:184, then
forks `kpsewhich` per candidate list — 108 of them always-missing `*.rhai`
binding probes); the sweep pins it (`run_doc.sh:42-46`), so the 222 s is already
the fast path. zx-calculus (191 s) and tkz-grapheur (187 s) are the same family.
**Reopened the same day (user: "We can be as fast").** The "flat profile" was taken on
the stripped `--release` binary — every Rust frame was a hex address, so the
0.7 % ceiling per symbol is an artifact of missing symbols, not evidence of a
diffuse cost. Program: (1) `cargo build --profile bench` (symbols) in its own
`CARGO_TARGET_DIR`; (2) `perf record --call-graph lbr` on one heavy picture
(`~/data/pk_agents/w23/regr85/tikz-network-perf/picC.tex`) and on the manual,
`perf report --no-inline` ranked by self time; (3) one lever per run with
pre-registered bars: tikz-network wall (222 s → target 34 s), picC wall, a
non-TikZ control (a 200-page text manual) unchanged, `perf stat` instructions.
Suspects to rank: token interning/arena pins per `\csname`, `Tokens` clones in
`\pgfmath`/`\pgfkeys` expansion, the pushback/mouth structure, per-token state
lookups (catcode, meaning), `check_timeout` cadence, definition dispatch, the
SVG driver's path building. Tracked in `docs/perfect_kernel/PLANS.md` item 7.

**Symbolized profile (2026-09-18, `~/data/pk_agents/w23/perf_pgf/NOTES.md`).**
picC (one heavy picture): 887 M token reads, 35.8 ns and 632 instructions per
token, IPC 3.17; pdflatex runs 6.07× fewer instructions on the same input — a
document-model constant factor, linear in pictures. Top self time on the manual:
`read_internal_token_checked` 9.9 %, `read_x_token` 5.5 %, `read_balanced_with_close`
4.8 %, `read_arguments` 3.1 %, `substitute_parameters` 3.1 %, `Parameter::read`
2.6 %, `assign_internal` 2.4 % — and a **14 % inclusive `getenv` band**:
`Parameters::read_arguments`/`read_arguments_and_digest` called
`std::env::var("LXML_TRACE_ARGS")` on EVERY macro/primitive/constructor
invocation (the environment lock plus a byte scan of `environ`, longer under the
`TEXMF*` pin). The stripped profile had shown it only as ambient libc.

**Lever 1 landed (batch 56da): the trace switch read once** (`Lazy<Option<String>>`,
parameter.rs — gullet.rs's `TRACE_GROUP_END` idiom). Bars, picC under a loaded
host (`perf stat` instructions are load-independent): 559.5 G → 497.8 G
instructions (−11.0 %), wall 37.9 s → 33.3 s (−12.2 %), XML byte-identical;
tikz-network's manual to be read off sweep 89.

**Lever A landed (batch 56db): `if_count`/`if_limit` are typed `State` fields**
(P4; `State::if_count`, `next_if_id()`/`if_limit()`/`set_if_limit()`). Every
`\if…` paid a Value lookup, an `assign_internal(Global)` — the `\globaldefs`
probe plus the undo-frame walk with a hashbrown `remove_entry` per open group —
and a second, always-missing Value lookup for the limit. Perl assigns `if_count`
`'global'` (Conditional.pm:62) and keeps `$LaTeXML::IF_LIMIT` as a plain global,
so the typed scalars are the same counter; both keys were already dump-skipped.
Bar ≤ 487 G, measured picC 497.8 G → 475.7 G instructions (−4.4 %, `perf stat`,
XML byte-identical, 0 errors).

**Lever B measured and reverted (2026-09-18)**: `noexpand_shadowed` early-out
through the memoized `is_noexpand_family` plus settling the memo at the
producer — picC 475.7 G → 477.4 G instructions (+0.35 %): for the ~all
non-family CS tokens the thread-local memo probe costs more than the
`with_str` prefix scan it replaced; the producer-time memo has no measurable
effect. Correct (reviewed) but not a win; only the B3 shape (a family bit in
`Token`'s padding byte, set at construction) can remove the probe — deferred
with P5.

**Lever C landed (batch 56de): one borrow per scanned brace.** Every `{`/`}`
(tex.web §358 `incr`/`decr(align_state)`) paid a `locals!()` borrow for the
trace's `before` value, a `locals_mut!()` for the bump, and a call into
`trace_align_state` that dereferenced the off-by-default switch — now one
`shift_align_group_count` borrow, the trace read behind the switch. Bar ≤ 468 G,
measured picC 475.7 G → 467.4 G (−1.7 %), XML byte-identical. Not taken: gating
the mutation on `has_reading_alignment()` — the count stack is also pushed
without a reading alignment (`gullet.rs:2524`, the unit test at
`local_assignments.rs:333`), so the symmetric-skip argument does not hold as
stated; revisit only with a tex.web §774-shaped `push_alignment` model.

**Lever D landed (batch 56df): `substitute_parameters` sized to template +
arguments.** The result `Vec` was sized to the template alone, so every
argument-bearing expansion regrew; now the template length plus the sum of the
present arguments' lengths (exact for the single-use body, one regrow for a
reused argument), and a body naming no parameter (`\@gobble`, `\use_none:n`)
keeps the template-only size so an empty result stays allocation-free (the
reviewer's catch: the unguarded bound gave every gobble a malloc/free pair).
Bar ≤ 472 G, measured picC 467.4 G → 465.2 G (−0.5 %, the bar missed by 0.3
points; landed as a byte-identical net positive).

**Lever E landed (batch 56dg): no String per number read.** `read_normal_integer`
stringified every token it read (`Token::to_string()` = arena `with_str` +
`fmt::write` + a heap String) to test for a decimal digit, then dropped the
String on the common internal-quantity path (`\pgf@x`, a count register); now a
non-allocating `with_str` digit peek, the String built only in the decimal arm
(tex.web §440-448 `scan_int` inspects codes, never text). Bar ≤ 458 G, measured
picC 465.2 G → 459.6 G (−1.2 %, bar missed by 0.35 points; byte-identical).

**Round 2 profile (2026-09-18, `~/data/pk_agents/w23/perf_pgf/round2/NOTES.md`,
on the lever-D binary):** 887,136,902 token reads per picC — unchanged by every
lever (they cut cost per token, never the count); document-to-package token ratio
≈ 1:3,000,000, so the cost is the pgf interpreter's macro bodies; pgfmath is
already a native f64 binding. Self-time bands: token I/O ≈ 32 %
(`read_internal_token_checked` 12.5 %, `read_x_token` 6.4 %,
`read_balanced_with_close` 5.7 %), argument read + substitution ≈ 14 %, meaning
resolution ≈ 8-9 % (every CS resolves twice: `read_x_token` then
`stomach::invoke_token`, tex.web §340-341 resolves once), allocator ≈ 7 %,
dispatch ≈ 6 %. Ranked next: (G) resolve each token's meaning once — a
one-slot resolved-definition cache from `read_x_token` to `invoke_token`,
invalidated on meaning assignment; HIGH value, HIGH risk; bar ≤ 452 G. (F) the
no-expand probe as an unchecked per-symbol bitset (`#[thread_local]
UnsafeCell<Vec<u8>>`) — the naive memo route is lever B's dead end, the residual
is the `RefCell` borrow; the per-Token pad-byte bit is REJECTED (Token is Copy,
rebuilt by `T_CS!` and dump replay, so a family-named token off the producer
would misroute in `meaning_key`); MED risk, ≈1.5-2 %. (I) `unread_expansion`'s
manual reverse-push loop as one `extend` — verify LLVM has not fused it; < 0.5 %.

**Lever G measured and reverted (2026-09-18, design
`~/data/pk_agents/w23/perf_pgf/leverG/NOTES.md`)**: the second meaning
resolution costs only 0.72 % of cycles (`lookup_digestable_definition` under
`invoke_token`; the round-2 2-3 % estimate was wrong), and carrying the
resolved `Stored` from `read_x_token` to `invoke_token` on the stack — the
low-risk shape, no cache, no invalidation — measured picC 459.6 G → 466.9 G
(+1.6 %): the `Stored` clone on the read side costs more than the hashbrown
probe it saves (`invoke_token` cloned only `entry.front()`; the read-side
clone runs inside `with_meaning`'s borrow). Dead end; the thread-local-slot
shape was ruled out in the design for a larger invalidation surface with
the same clone.

Cumulative on picC since the program opened: 559.5 G → 459.6 G instructions
(−17.9 %); 4.97× pdflatex's 92.4 G (was 6.07×).

**Direction (user, 2026-09-18):** gains must be algorithmic and strategic;
the gullet, stomach, mouth and document stay ergonomic and idiomatic — no
caches of resolved state, carried meanings, scratch buffers or unsafe cells
(levers B and G, both reverted, were that shape and both measured worse).
The allocator study (`~/data/pk_agents/w23/perf_pgf/alloc/NOTES.md`) confirms
the residual there is ≈ 8 % of cycles, ~5 % of it values that escape (Perl
returns them too) and ≤ 2 % scratch-buffer micro-levers — not pursued.

**Where picC's tokens actually go (`~/data/pk_agents/w23/perf_pgf/token_budget/NOTES.md`,
pdflatex `\tracingmacros` as the oracle, 43.3 M expansions on the one picture):
97.65 % expl3 — datatool v3's CSV parser, which tikz-network's `\Vertices`/`\Edges`
re-run per row (`\DTLloaddb` re-parses the file on every call, `\DTLforeach` re-walks
the db per row, datatool re-types every field each pass); pgf's own layers are
< 2 % (pgfmath 0.6 %, already native; tikz frontend 0.03 %). Rust and pdflatex
expand the same raw bodies the same number of times (SHARED cost; 20.5 token
reads per expansion). So "the pgf case" is the expl3/datatool case, and the
strategic lever is a native datatool database layer — `\DTLloaddb` (CSV → rows,
cached by file), `\DTLforeach`/`\DTLforeachkeyinrow`, `\DTLifeq`, the row getters —
scoped to the used slice, the rest forwarded raw; bar picC ≤ ~30 M token reads.
General expl3 levers that help every expl3-heavy manual: native `\tl_map_function`
/ `\__tl_range_*` (33.8 % of the expl3 expansions) and `\int_eval:n` /
`\int_compare:nNnTF` (27 %). l3regex native is a settled dead end (removed
2026-06-20).

**Corpus weight (`~/data/pk_agents/w23/perf_pgf/token_budget2/NOTES.md`, the
same tally on the other slow manuals and over all 2,374 logs):** tikz-network's
datatool shape is a narrow outlier (21 manuals, 10 % of the pgf wall). The
dominant pattern is the style-heavy tikz diagram, and there the budget is
**pgfkeys dispatch** — zx-calculus 74-79 % of expansions (`\pgfkeyscurrentkey`,
`\pgfkeys@splitter`, `\pgfkeys@spdef`, `\pgfkeys@parse@main`, `\pgfkeys@ifcsname`);
pgfplots/pgf-interference are l3fp (64 %); tabularray is pure l3 tl/prop. 827
manuals load `pgfkeys.code.tex` = 6,057 s = **69 % of the corpus wall** (tcolorbox's
option system is pgfkeys too); estimate 1,500-2,500 s (17-29 %) removable.
pgfmath is already native on both sides; pgfkeys runs 100 % raw in Rust
(`pgfkeys_sty.rs`, a 7-line shim) and in Perl, whose own native engine
(`pgfkeys.code.tex.ltxml:40-544`) is disabled as "not quite right or complete".
**The strategic lever is a native pgfkeys dispatch, Perl-anchored, with the key
tree kept in the raw `\csname` storage so every package that pokes it keeps
working and a raw fallback for the long tail** — design in
`~/data/pk_agents/w23/perf_pgf/pgfkeys_native/NOTES.md`, MEASURED on the Rust
side (bench binary, body-present vs body-absent): the style-heavy ZX circuit is
57.9 % body instructions and a 40-box tcolorbox document 61.6 %, of which the
per-node option dispatch alone is 35 % of a conversion (300 styled nodes with
vs without a 9-key style list: 15.18 G vs 9.85 G). Shape: the
`pgfmath_code_tex.rs` pattern — load the whole raw `pgfkeys.code.tex`, then
override only the hot entry points natively on the SAME `\pgfk@<key>` csname
storage (`\pgfkeys@ifcsname`, `\pgfkeysifdefined`, `\pgfkeysgetvalue`,
`\pgfkeysvalueof`, `\pgfkeyssetvalue`, `\pgfkeyslet` first: slice 0, XML cannot
change; then the `\pgfkeys{}`/`\pgfkeysalso{}`/`\pgfqkeys{}{}` parse+dispatch loop
over `.code`/`.style`/`.default`/`.initial`/`.cd`/store keys with a three-probe raw
fallback — `\ifpgfkeysfilteringisactive`, `\ifpgfkeys@syntax@handlers`,
`\pgfkeys@case@three` rebound — and per-key unknown → raw `\pgfkeys@unknown`:
slice 1; `\pgfkeysdef` family: slice 2). Handlers stay raw (they are keys under
`/handlers/`; the native path invokes their `.@cmd`), so this is strictly more
complete than Perl's abandoned engine (its stubbed filtering/family/syntax
handlers, `pgfkeys.code.tex.ltxml:170/463/522`, run real TeX here). TDD: an
ON/OFF switch and ten ≤15-line fixtures requiring byte-identical core XML; bars
zx_full 25.42 G, tcb_full 18.23 G, keys_heavy 15.18 G, picC unchanged.

**Slice 0 landed (batch 56dk): the eight leaf accessors native**
(`pgfkeys_code_tex.rs`: `\pgfkeys@ifcsname`, `\pgfkeysifdefined`,
`\pgfkeysifassignable`, `\pgfkeysgetvalue`, `\pgfkeyslet`, `\pgfkeyssetvalue`,
`\pgfkeysaddvalue`, `\pgfkeysvalueof`; the raw file loads whole first; values
stored verbatim, parameter-free and unpacked; `\pgfkeysvalueof` `\relax`-defines
an undefined name as `\csname` does — the differential harness caught that one).
Measured, byte-identical XML: zx_full 25.51 G → 23.28 G (−8.7 %), tcb_full
18.31 G → 17.59 G (−3.9 %), keys_heavy 15.23 G → 14.74 G (−3.2 %), picC unchanged
(datatool-bound tripwire). Five fixtures under
`latexml_oxide/tests/cluster_regressions/pgfkeys/` run ON and OFF: the
`LATEXML_PGFKEYS_NATIVE=0` switch (read once at binding load, a development
differential like `LATEXML_POST_STREAM_SPLIT`) keeps the raw accessors, and
the guard requires byte-identical core XML both ways. The review of slice 0
found two more divergences the first fixtures did not reach — `\pgfkeysvalueof`
on an undefined key is the raw file's `\pgfkeys@relax`, never a `\csname`
definition of the key (:193-194), and `\pgfkeysaddvalue` assigns locally
(:125-135) — both fixed and pinned by the extended accessors fixture.

**Slice 1 landed (batch 56dl, corrected in 56dn): the parse-and-dispatch loop
native.** `\pgfkeys{}`, `\pgfkeysalso{}` and `\pgfqkeys{}{}` are macros (as in
the raw file, :320/:589/:607 — a `}` where their list should be is refused like
any macro argument) expanding to internal primitives that put the list FLAT into
the stream as `<items>,\pgfkeys@mainstop` and run one step; the step is a
locked primitive under the raw macro's own name, `\pgfkeys@parse`: it reads one
item up to its top-level `,` (`\pgfkeys@@normal#1,`), resolves it —
`\pgfkeys@unpack`'s key/value split and `\pgfkeys@spdef` space stripping, the
default-path prefix, `\pgfkeyscurrentkey`/`RAW`/`name` and `\pgfkeys@pathtoks`,
the `.@def` default and `\pgfkeysvaluerequired`, cases one/two/three and
`\pgfkeys@unknown`, first-char syntax handlers — and puts back
`\iftrue\iftrue <handler tokens> \fi\fi \pgfkeys@parse` (one `\fi` after an empty key). Every token a handler can see is
raw's: the handler body runs in the main loop, where it may open a box or a
group that later stream tokens close (zx-calculus's `/tikz/on layer/.code=
{\pgfonlayer{#1}\begingroup\aftergroup\endpgfonlayer\aftergroup\endgroup}`,
tikzlibraryzx-calculus.code.tex:2415 — a nested `digest` of the body hit the
end of its mouth inside the `\hbox`, zx_full 0 → 29 errors); a handler that
scans forward sees the real remaining keys (robust-externalize's placeholder
re-scan swept a Rust-side continuation token into a key name, sweep #95
14 → 204 errors); a handler that over-grabs one token takes the inert `\fi`
`\pgfkeys@unpack` leaves (:382-388), not the continuation (the same manual's
`\robExtArgumentList` m-grab, sty:4230, made `/robExt/\lx@pgfkeys@set{@}parse`
and a recursion Fatal). Two earlier shapes were measured and dropped: a braced
`{rest}` argument re-read per item (tcb_full +7 %) and a thread-local slot table
(faithful to nothing a handler can scan). `run_handler` lets `\pgfkeys@code` to
`\relax` when the handler is absent (`\pgfkeysgetvalue`, :175 — a rawstyles pgf
load reaches `\pgfkeys{/pgf/.is family}` before the handlers exist,
pgfsys.code.tex:19; scsnowman-sample 318 → 1,001 errors + Fatal, chuushaku
423 → 994 without it); `\pgfkeys@spdef` strips EVERY leading space as the raw
chain does in this engine (parameter-text-initial space matched against the
run; Perl identical; neoschool's `\newtcolorbox` `, #1` + indented `[`);
filtering (`\ifpgfkeysfilteringisactive`) and a reconfigured case-three
dispatch hand the rest of the list to the raw step body
(`\futurelet\pgfkeys@possiblerelax\pgfkeys@parse@main`, :326), re-checked at
every step. Slice 0's `\pgfkeysvalueof` also learned to keep the raw
`\csname…\endcsname` shape (:194): the stored macro is reached in TWO expansion
steps, which circuitikz's `\unexpandedvalueof` (circuitikz-1.7.2-body.tex:987-998)
counts with a triple `\expandafter` — the one-step native handed its checker
the value's first token and the circuitikz manual a stray `\fi` (sweep #94).
Measured, byte-identical XML ON/OFF (56dn, the raw-stream shape; the slot-table
cut of 56dl read 10.62/13.25/11.90 G — the `\iftrue\iftrue…\fi\fi` and the macro
entry points cost 2-6 %, the price of a stream a handler can scan): zx_full 23.28 G →
**11.29 G (−51.5 %)**, tcb_full 17.59 G → **13.57 G (−22.9 %)**, keys_heavy
14.74 G → **12.10 G (−17.9 %)**, picC 459.6 → 457.6 G (datatool-bound); the
zx-calculus manual 150 → 73 s, the circuitikz manual 74 → 59 s. Cumulative for
the two slices: zx_full −56 %, tcb_full −26 %, keys_heavy −21 %. Fourteen fixtures now run
ON and OFF (`LATEXML_PGFKEYS_TRACE=1` prints one line per dispatched key, the
bisection aid). **Slice 2 landed (batch 56dp): the definition handlers native.** `.code`,
`.style`, `.initial`, `.default` and `.cd` (pgfkeys.code.tex:772, :826, :842,
:852, :994) and `\pgfkeysdef` under them (:648-652) run in Rust and store
exactly what the raw handlers store — `\pgfk@<key>/.@cmd` a `\long` macro with
the parameter text `#1\pgfeov` and a parameter-packed body, `/.@body` the code
verbatim (what `.append style`/`.add code`/`.show code` read back), `.style` as
`.code=\pgfkeysalso{…}` with the nested item's `\pgfkeyscurrentkey`/`name`/
`value` left behind as the raw handler leaves them, the handler's `#1\pgfeov`
argument as `\pgfkeyscurrentvalue` expanded once with one outer group stripped.
A native handler runs only while the handler key still holds the raw file's
definition (a `\let` snapshot `\lx@pgfkeys@raw@<h>` taken at load), so a
package that redefines `/handlers/.code` gets its own. Byte-identical ON/OFF:
zx_full 11.29 → **9.98 G (−11.6 %)**, tcb_full 13.57 → 13.39 G (−1.3 %),
keys_heavy and picC unchanged — the definition-phase lever the histogram
predicted (72 % of zx's dispatches, 23 % of tcb_full's, 4 % of the tcolorbox
manual's); the tcolorbox manual converts identically both ways. Cumulative for
the three slices: zx_full 25.51 → 9.98 G (−61 %), tcb_full 18.31 → 13.39 G
(−27 %), keys_heavy 15.23 → 12.09 G (−21 %). Fifteen fixtures run ON and OFF.

**The non-pgfkeys floor, profiled (2026-09-18, `~/data/pk_agents/w23/perf_pgf/tikzcore/`).**
keys_light (300 tikz nodes, no keys) runs at 8.47 G / 628 ms against pdflatex's
394 ms — about 1.1× at steady state once the one-time kpathsea directory scan
(~30 % of that small run) is set aside: pure TikZ is at parity, because
pdflatex's heaviest per-node layer, pgfmath (46 % of its expansions), is native
here (`pgfmath_code_tex.rs`), pgfsys is native, and the raw residual (pgfcore
soft-path 11 %, tikz@ frontend 3 %) is a fraction of an already-fast
conversion — there is no tikz-core lever. picC (457 G, 5.55× pdflatex) and the
tikz-network manual (2,784 G, 161 s vs 29 s, 5.48×) have self-time profiles
indistinguishable from each other and from a pure token interpreter (42 % token
I/O, 15 % argument reading, 13 % meaning resolution, native pgf 0.0 %, document
0.05 %): they are datatool v3, which splits every CSV line with l3regex
(`datatool.sty:10761-10817`, `\__regex_build_new_state:` 3,630× per two-row
`\Vertices`; 327,113 expansions per call, 60 % expl3 primitives, the public
`\DTL*` 0.1 %) and then walks rows with `\DTLforeach`/`\DTLifeq` cascades
(tikz-network.sty:793-817, :969-997). Perl ships no datatool binding and is as
slow. NEXT levers, ranked: (1) a native datatool CSV load that bypasses the
l3regex split and populates the identical DB store the getters read (`\DTLread`
:12227/:12799 → `\__datatool_load_csv:` :10817; field order, `#`/catcodes in
fields, numeric detection `\@dtl@checknumerical`; HIGH risk, on/off harness on
picC with `count(svg:g)==83`), (2) native `\DTLforeach`/`\DTLforeachkeyinrow`/
`\DTLifeq` over that store (MED); together they are the whole picC/tnman gap.
Porting l3regex natively is the settled dead end above. **Lever (1) landed as batch
56dt** (`datatool_sty.rs`, the design below, byte-identical registers to pdflatex on
17 probe CSVs): picC 458.2 → 423.4 G (−7.6 %), the tikz-network manual 180.6 →
165.5 s (−8.3 %) — the load is ~8 % of the document. **Lever (2) landed as batch
56du**: the walk itself is cheap; 95 % of a `\Vertices` call was `\DTLifeq`
(~20 per row, ~216 M instructions each: `\DTLifnumerical` ×2 through l3fp/l3regex
and two `\text_purify` passes). Native `\DTLifeq`/`\DTLifstringeq` (fp equality
when both operands parse as numbers, else the expanded string forms, folded
under `*`) with the loop left raw: picC 458.1 → **16.3 G (−96 %)**, 28.2 → 1.1 s;
the tikz-network manual 190.7 → **25.0 s** (on/off in one run), pdflatex 34.5 s on
the same host — the directive's headline document is past parity. The raw numeric branch in this engine was also
wrong (`\DTLifeq{5}{5.0}` false); the native follows pdflatex.

**The remaining slow calls, profiled (2026-09-19, `~/data/pk_agents/w23/perf_pgf/slow2/`).**
With tikz-network at pdflatex speed, the eight SUSPECT calls over 60 s were
profiled against pdflatex on the same host: pgf-spectra (90 s) runs at 0.77×
pdflatex's 118 s, tabularray (109 s) against a pdflatex that never finishes
(stuck at page 48 after 21 minutes), circularglyphs 1.4×, tilings 3.4× (62 s vs
18 s, inline), rulercompass and l3kernel source3 without a fair baseline here.
Every profile is the generic token interpreter (`read_internal_token_checked`
8-14 % on top everywhere, token I/O 21-64 %, argument reading 3-12 %, meaning
resolution 7-13 %); pgfmath, pgfsys and pgfkeys natives are cold, and no native
leaf remains — the hot code is the raw pgf-core drawing pipeline itself
(tilings: 63,100 key dispatches, 60 % `.try`, are under 0.5 % of its 1.7 G token
reads). A native `.try` (pgfkeys.code.tex:1024) is faithful and general but
worth under 0.5 % of wall — declined as a perf lever. The two real problems are
blowups, not per-token cost: pgf-PeriodicTable 344 s at 24.8 GB RSS building
282,289 `svg:g` (53 redraws × 118 cells; document/libxml self-time under 0.5 %,
the memory is the retained DOM) and wheelchart timing out at 400 s and 10 GB.
Wheelchart is settled (2026-09-19, DIFFICULT_CASES §D11): multiline `arc data`
under `arc around text` (wheelchart.sty:2938-3105, a per-line `text along path`
decoration walk whose arc-split step degenerates) spins every engine — pdflatex
produces no PDF in 120 s on a 10-line repro and sticks at page 20/56 on the
manual, Perl aborts on 100 l3fp errors in 9 s — and the 10 GB is transient
interpreter working set, not DOM or a leak; the regex at :2347-2351 is ~5 % of
self-time. The periodic table is settled by three batches (LEDGER 56dv-56dx): the
picture-end streaming seam plus the resident-wrapper spill bound the box tree,
a fused eager run restarts under `--streaming` from the CLI, and `remove_node`
now frees what it removes (the libxml fork left every removed subtree as an
orphan — ~5-6 MB per tikz picture, eager and streaming alike, a corpus-wide
memory lever). The manual streams to completion at the 6 GB ceiling (0 fatals,
3,326 pictures, 379 s); under the eager sweep's 420 s cap the fuse-then-restart
sequence still runs long, which is the residual for that one witness. Beyond them the next
general lever is the token itself (`Token` under 8 bytes and the allocator, P5),
which reaches every TikZ, pgf and expl3 document and is HARD.

**Native datatool load — the design (2026-09-18, `~/data/pk_agents/w23/perf_pgf/datatool/`).**
A loaded database is four global registers plus per-key indices, and every
reader is a delimited-macro consumer of them (datatool.sty): `\dtldb@<name>`
(toks, one row body per row: `\db@row@elt@w \db@row@id@w<id>\db@row@id@end@
[\db@col@id@w<i>\db@col@id@end@ \db@col@elt@w<val>\db@col@elt@end@ …]*
\db@row@id@w<id>… \db@row@elt@end@`, :3493/:3516, written :12473, read by
`\@dtl@foreachrow` :6843), `\dtlkeys@<name>` (toks, one column body per column:
`\db@plist@elt@w …col id, key, type, header… \db@plist@elt@end@`, :3465,
written :12336, read by `\dtlforeachkey` :6932), `\dtlrows@<name>` and
`\dtlcols@<name>` (ints, :3543-3544), and `\dtl@ci@<name>@<key>` (the column
index of a key, :12347). So a native load that writes byte-identical register
contents is transparent to every getter, and datatool's own DBTEX-v3 reload
(`\@dtl@reconstruct@data` :12073-12093) is the template: one `gset` of each
toks register, two int sets, one `\csgdef` per key. Measured on picC's 8-row
CSV ×200: the raw `\DTLloaddb` 0.807 s per load, the same database built by
`\DTLnewrow`/`\DTLnewdbentry` 0.637 s — the per-entry store's concat-middle
(:4791) is O(cols²) per row, so a native split that feeds `\DTLnewdbentry`
would win only 21 %; the lever must write the store directly. Parse rules to
reproduce byte-exact: separator `,`/delimiter `"` (:12267/:175), split keeping
spaces then rejoining a separator inside a quoted field (:12730/:12740),
doubled-delimiter unescape (:10800/:10811), surrounding-space trim (:181/:12759),
`csv-content=tex` (tikz-network's default: fields keep document catcodes) vs
`literal` (the six-case `\regex_replace_case_all` + `\tl_set_rescan`,
:12359-12378) vs `no-parse`, `\__datatool_parse:`/`\DTLdatumtype` column typing
(:12541, even under `convert-numbers=false`), blank lines ignored (:12401),
`omitlines`, `noheader`/`headers=`/`keys=` (:12292-12350). Harness:
`LATEXML_DATATOOL_NATIVE=0`, byte-identical XML on picC (`count(//svg:g)==83`)
and a datatool-user sample, plus `\DTLdbLog` (:12822) register dumps identical
on/off; fixtures `fix3.csv` (quoted embedded comma, numeric column, `#` in a
field, a blank line, `omitlines=1`). Perl has no datatool binding (parity-neutral).

The profile of the slice-1 binary (`~/data/pk_agents/w23/perf_pgf/slice2/`)
shows the residual is generic gullet macro machinery serving the RAW handler
bodies plus allocator churn; the dispatch histograms
(`LATEXML_PGFKEYS_TRACE=1`) put the definition handlers at 72 % of zx_full's
16,556 dispatches (`.code` 6,008, `.style` 4,891 — each `.style` a nested
`\pgfkeys{…/.code=\pgfkeysalso{#1}}`), ~50 % of a pgfplots example, 23 % of
tcb_full, and only 4 % of the tcolorbox manual, whose 2.39 M dispatches are 79 %
case one (USE) and 13 % `.try`. Ranked: (1) native `\pgfkeysdef` (:648) with
native `.code` (:772) and `.style` (:826) — the `/.@cmd` macro is `\long` with
parameter text `#1\pgfeov` and a PARAMETER-PACKED body, `/.@body` holds the
value verbatim (:651) and later `.append style`/`.add code`/`.show code`
(:783/:806/:837) read it, `.style` skips the nested `\pgfkeys`; keep
`\pgfkeysedef` (:653) raw first (the `\edef` timing seam); (2) `\pgfkeysdefargs`
(:675) / `\pgfkeysdefnargs` (:727) for the `n args` forms (three slots,
`/.@args` = `{pattern}\pgfeov`, the `/.@@body` indirection :753); (3) `.initial`/
`.default`/`.cd` as thin natives over `\pgfkeyssetvalue`/`\pgfkeysdefaultpath`;
(4) `.append style`/`.add code` deferred. For USE-heavy documents `.try` (:1024, a
full re-dispatch under `\ifpgfkeyssuccess`) is a natural slice 3, and the tikz
path-construction/`\pgfmath`/node-box floor (keys_heavy: 8.6 of 12.1 G is not
pgfkeys at all) outranks every remaining pgfkeys slice. Not lever inputs: chemobabel
(parked, LuaTeX-ja), lie-hasse (runaway TokenLimit after a mode-frame error —
separate bug), wheelchart (MemoryBudget runaway), l3kernel/source3 (memory). Settled dead ends: SmallVec-backed `Tokens` (blocked by
`Token == 8 B`, P5), pooled `Tokens` allocator and a reused `read_balanced`
scratch (both a public `Tokens` API change), lowering `read_balanced`'s cap 16
(net-neutral), LBR call graphs (unsupported on this PMU; use `--call-graph fp`),
and a full `perf report` call graph on the 610 MB bench binary (> 5 min symbol
load; use `-g none` flat self). Beyond these, closing the 6× needs fewer tokens
per picture (pgf binding emitting less) — the harder program.
