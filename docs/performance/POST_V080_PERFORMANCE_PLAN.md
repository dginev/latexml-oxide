# Post-v0.8.0 Performance Roadmap

> Written 2026-11-22 from first-principles analysis and `--profile bench`
> measurements (`cde8f526b17`) on 10 real arXiv papers (`~/Downloads/`) and
> Nasser-shaped large split ODE books (502–2,511 pages, `--split
> --splitat=subsection --navigationtoc=context`).
>
> Low-risk, non-refactor wins from the same review were staged directly for
> v0.8.0 (PR #973: readonly `ObjectDB` baseline skip, core-state release after
> `MakeBibliography`, `Scan` placeholder tracking, `CrossRef` memo bounding,
> `FileSink` fail-fast, zero-copy `finalize_html5`, `Rewrite` opt-in $O(1)$
> local `Replace` + root-retry skip, `XSLT` embedded `cache_key` +
> `new_transformed`, `Document::from_parts`, lazy rewrite hint formatting, and
> prepared/streamed `ObjectDB::save_as`).
>
> This document tracks the **larger architectural / moderate-refactor levers**
> intentionally deferred until after v0.8.0 ships, reconciled against
> `PERFORMANCE.md` (including P13, PERFE, and Closed Levers),
> `archive/STREAMING_POST_DESIGN_2026-07-06.md`, and `docs/SYNC_STATUS.md` row R
> (`RENDERFLEETBIG`, `SPILLKEEP`, `SERSTREAMUNSPLIT`, `MATHTAIL`, `POSTRESID`).
> All levers remain subject to the byte-identical output-neutrality contract
> unless explicitly authorized.
>
> **Toolchain caveat (`PERFORMANCE.md` P6):** local `--profile bench` builds
> link host system `libxml2`/`libxslt`, which runs ~15–20% slower on
> `phase_xslt` than the shipped static release binary (`libxml2 2.13.5 /
> libxslt 1.1.42`). Algorithmic multipliers (such as the 7.3×
> `--navigationtoc=context` blowup below) hold on both, but absolute XSLT A/Bs
> must be verified on the release toolchain.

---

## Measured Baselines (2026-11-22, `--profile bench`, `cde8f526b17`)

### 10 real arXiv papers (`--preload=ar5iv.sty --format=html5 --pmml`, 10/10 `rc=0`)

* **9 ordinary papers (22.81 s total, 233–506 MB RSS):** `math_parse` **27.1%**,
  `digest` **23.4%**, `build` **12.4%**, `xslt` **11.5%**, `rewrite` **4.3%**
  (with LHCb `2402.01336v5` spending 33% / 679 ms in `rewrite` before the
  pre-0.8.0 `Replace` fix), `graphics` **3.5%**, `bibliography` **2.9%**,
  `mathml_pres` **2.8%**, `crossref` **1.4%**.
* **Large math-dense paper (`arXiv-2511.03798v1`, 2.04 MB `.tex`, 91.5 MB XML,
  1.69M elements, 18,219 formulae):** **68.06 s**, **3,263 MB RSS**, **2.39
  parses/formula** (`43,543` total parses) — `math_parse` **32.8 s (48.1%)**,
  `build` **10.4 s (15.3%)**, `xslt` **9.1 s (13.3%)**, `mathml_pres` **3.0 s
  (4.3%)**, `digest` **2.9 s (4.2%)**.
* **Formula duplication rate across all 10 papers:** **48.8%** (`26,528` total
  vs `13,584` unique; `50.0%` on `2511.03798v1`, `53.0%` on `2402.10301v1`,
  `54.6%` on `2512.25064v1`).

### Nasser-shaped large split ODE books (`--split --splitat=subsection`, forced streaming)

* **2,511 pages, 8 workers (`--navigationtoc=context`, 15,000 formulae):**
  `55.93 s` wall, `165.96 s` user CPU (`130.3 s` in workers), `232 MB` peak RSS.
  - Parent serial wall: `math_parse` **23.15 s (41.4%)**, `digest` 6.68 s
    (11.9%), `build` 2.87 s (5.1%), `split` 1.38 s (2.5%), `post_scan` 0.82 s
    (1.5%).
  - Worker fleet wall: **20.04 s (35.8%)**.
* **502 pages, serial (`LATEXML_RENDER_JOBS=1`), WITH vs WITHOUT `--navigationtoc=context`**
  (50 sections/chapter with math in `\section` titles):
  - `xslt`: **1.69 s → 12.41 s (7.3×)**
  - `crossref`: **0.31 s → 3.03 s (9.8×)**
  - `mathml_pres`: **0.48 s → 2.91 s (6.1×)**
  - `math_parse`: **4.55 s (45.3% without navtoc)** — serial on 1 core in Pass 2.

---

## Ranked Post-v0.8.0 Levers

### P1. Lazy On-Demand Readonly SQLite `ObjectDB` for Parallel Render Workers (`RENDERFLEETBIG`)

* **Target:** Large split books (Nasser 115K pages / 1.82M DB objects, db ≈ 600 MB; `SYNC_STATUS.md` `RENDERFLEETBIG`: a worker's measured peak is `~42 MB + 4.85 × db`, estimated at `64 MiB + 6 × db` in `render_workers.rs::worker_estimate_bytes` — ~3.7 GB per worker on Nasser's whole book, so under `--max-memory=8192` the fleet affords 0 workers and renders serially). Note that releasing core state before post (`release_core_state_for_post`) recovers parent headroom, but unlocking multi-worker parallelism on a 600 MB DB still requires lazy worker attachment.
* **Historical alignment:** `docs/archive/STREAMING_POST_DESIGN_2026-07-06.md` §6 explicitly chose SQLite so N worker processes could share **one** mmap'd page cache instead of copying the whole DB into each child's `HashMap` — yet `ObjectDB::attach(..., readonly: true)` (`latexml_post/src/object_db.rs`) still eagerly executes `SELECT key, props FROM entries`, JSON-decodes every row, and parses every `Value::Xml` via libxml2 into RAM in **each** worker process. Completing lazy readonly attachment realizes the original §6 design.
* **Verified access pattern:** In Pass B workers, **only `CrossRef`** reads
  `ObjectDB`, and every hot access is a point `db.lookup(key)` query (the single
  `keys_iter()` fallback at `crossref.rs:1985` only triggers when
  `doc.idcache_len() > db.len()`, which is never true on split pages and can be
  unconditionally routed through `doc.idcache_iter()` for lazy DBs).
* **Design:**
  - Add a lazy readonly backend mode (`DbAttachOptions::lazy_readonly`) holding
    a prepared `SELECT props FROM entries WHERE key = ?1` statement,
    `PRAGMA mmap_size` (so all workers share one OS page cache), and a bounded
    in-memory cache (`HashMap<String, Option<Rc<Entry>>>`, ~16K–32K entries,
    clearing cache + `xml_holder` together on eviction so libxml memory stays
    bounded and drop order stays sound).
  - Return `Option<Rc<Entry>>` (or `&Entry` through a `&mut ObjectDB` lookup on
    `CrossRef`) and update `render_workers.rs`'s `worker_estimate_bytes`
    from `64 MiB + 6 × db` to a measured constant floor (~256 MB) once verified.
* **Expected impact:** Worker DB RAM **~3.7 GB → ~20–50 MB/worker** on Nasser's
  whole book, worker startup **~10–20 s → <1 ms**, unlocking full 8-worker
  concurrency on 16–32 GB hosts.
* **Related row-R split follow-ups:** `SPILLKEEP` (per-range serial fallback so
  finished workers can drop spilled XML pages, saving ~2.7 GB disk on Nasser),
  `RENDERPID` (pidfd-based worker signaling), and `GFXTEMPS` (sweeping dead
  worker staging files).
* **Validation:** `118_streaming_split_parity` + `120_midscale_streaming` +
  Nasser k2400/full byte-identical output and RSS comparison.

---

### P2. Content-Addressed Formula Memoization (BP-5 / P13)

* **Target:** Both ordinary arXiv papers (`math_parse` 27.1%), math-dense tail
  (`2511.03798v1` 48.1%), and large books (`41–45%` `math_parse` + `mathml_pres`).
* **Evidence & realistic ceiling (`PERFORMANCE.md` P13):** **48.8%** of all
  formulae across the 10 real `~/Downloads/` arXiv papers are exact duplicates
  (`SO(32)` 334×, `g_s` 288×, `E_8 \times E_8` 279×, `\frac{g_s}{H(y)H_o(x)}`
  76× on `2511.03798v1`), matching P13's 30-paper measurement of **46% by
  count**, **26–29% by TeX length** (shorter formulae repeat more often than
  long display equations; modeled realistic `math_parse` cut is **~25%** /
  **~5.5%** median paper wall, higher on repetitive catalogs).
* **Design:**
  - Memoize **MathParser** results inside a bounded per-conversion cache keyed
    on the serialized lexeme stream + every parse-affecting context input
    (active pragmas, `differential_evidence`, **`maybe_functions`** at
    `latexml_math_parser/src/parser.rs:605`, grammar/rule name, token
    attributes/roles/meanings/fonts that semantic actions inspect). Store a
    compact normalized parse tree or structural recipe and clone/rebind fresh
    IDs on cache hit.
  - Independently memoize **MathML[Presentation]** conversion (`latexml_post/src/mathml`)
    within a worker/conversion for identical `ltx:XMath` subtrees (crucial for
    both repeated equations and `--navigationtoc=context` title math — see P3).
* **Governance & correctness gate:** Math parser changes are currently **PARKED**
  (DG 2026-11-15), so the MathParser half needs explicit sign-off even when
  output-identical (the post-side `MathML[Presentation]` memo is outside the
  math parser). Gate with an ON/OFF differential env switch requiring 100%
  byte-identical core XML and final HTML across the full test suite and arXiv
  corpus.

---

### P2b. Median-Paper ArXiv Levers from `PERFORMANCE.md` P13 (Toward 1× Single-Pass `pdflatex`)

On median arXiv papers (`PERFORMANCE.md` P13, 32 papers: median `0.53 s`,
`1.22×` one `pdflatex` pass, `0.41×` full `pdflatex+bibtex` build), two
already-modeled levers outrank DOM/post micro-opts:
* **kpathsea `ls-R` persistent lookup memo (~−10.5% geomean):**
  `kpathsea_init_db` parses the 5.6 MB `ls-R` (251k entries) on the first lookup
  (~50 ms = 43% of the fixed floor). Persist the TeX Live tree lookup memo
  (`latexml_core/src/util/pathname.rs:1063`), stamped like the dumps, initializing
  kpathsea only on a miss (never persisting paper-local hits or negative
  answers). First step: log `pathname::kpsewhich` calls over ~200 papers to
  measure hit vs novel-miss rates.
* **One terminal per lexeme in `marpa-asf` (~−15–25% of `math_parse`, −3–9% total):**
  libmarpa accounts for ~48% of math-parse cycles while still scanning ~18-byte
  lexeme strings one earleme per byte. Requires a `marpa-asf` interface change
  while preserving `HYBRID_AND_NODE_LIMIT` routing byte-identically (and sign-off
  under math-parked).

---

### P3. Context Navigation TOC Caching ($O(k \cdot S^2)$ Elimination on Split Books)

* **Target:** Split books using `--navigationtoc=context` where section titles
  contain math (Nasser's ODE catalog, DLMF-style references).
* **Evidence:** On a 502-page book with 50 math-bearing sections per chapter,
  `--navigationtoc=context` multiplied `crossref` **9.8×** (0.31 → 3.03 s),
  `mathml_pres` **6.1×** (0.48 → 2.91 s), and `xslt` **7.3×** (1.69 → 12.41 s).
  Each chapter with $S$ sections and $k$ subsections per section scales as
  $O(k \cdot S^2)$.
* **Design (two output-neutral layers):**
  1. **Pre-convert / cache `ObjectDB` title `<ltx:Math>` to Presentation MathML**
     (or rely on P2's worker-level `MathML[Presentation]` memo): eliminates
     re-converting the same $S$ title formulae $k \cdot S$ times.
  2. **Cache sibling `<ltx:toclist>` assembly in `CrossRef`** keyed by
     `(parent_id, page_relative_dir)` so pages in the same chapter/directory
     reuse the pre-built sibling TOC entries (marking only the current page's
     active entry and uniquifying IDs) instead of re-cloning and re-resolving
     URLs for all $S$ siblings on every page.
  - Note: reducing the remaining XSLT cost on repeated TOC `<m:math>` trees
    requires either BP-2 Step 2 (native Rust rendering for MathML/TOC nodes) or
    an opt-in non-inline TOC format (out of scope for output-neutral work).

---

### P4. Parallel Pass-2 Segment Math Parsing (Streaming Large Books)

* **Target:** Large streamed books where `streaming_pass2` `math_parse` is
  **41–45% of total wall time** serial on 1 core (23.15 s / 55.92 s on our
  2,511-page 8-worker run).
* **Key insight vs BP-1:** General eager-mode parallel math parsing (BP-1) is
  blocked by the thread-local `SymStr` arena bridge. In contrast,
  `streaming_pass2` **already** has independent disk-staged XML fragments
  (`SegmentStore`), and `renumber_math_ids` is strictly per-`<ltx:Math>`.
* **Design:**
  - Parse/math-parse batches of independent segments across a bounded pool of
    worker threads or processes (each with its own thread-local `MathGrammar`
    and arena), then run the lightweight sequential ID-counter carry
    (`_ID_counter_*` + `anchors`) in segment order.
  - Preserve deterministic diagnostic ordering and `MathParser`'s per-conversion
    warning deduplication (`reset_conversion_notices` / `SEEN_WARNINGS` fold).

---

### P5. Complete Streaming Audit F3 (`SERSTREAMUNSPLIT`) & F4 Residuals

* **F3 (`SYNC_STATUS.md` `SERSTREAMUNSPLIT`):** `arXiv-2511.03798v1` (2.04 MB
  `.tex`, 91.5 MB XML) hit **3.26 GiB peak RSS** because unsplit conversions
  still hold the materialized core XML `String` alongside the post libxml DOM
  and libxslt output DOM. Route large unsplit CLI conversions through the same
  `StagedHandoff` file path already used for split runs.
* **F4 (Pass-2 shared ownership):** Beyond the pre-0.8.0 `Document::from_parts`
  and lazy `xpath_hint` fixes, wrap `node_fonts` in `Rc<HashMap<u64, Font>>`
  with a small per-fragment write overlay (eliminating `node_fonts.clone()` per
  segment) and split document-invariant `Rewrite` clause compilation from
  per-fragment execution (eliminating `rules.clone()` per segment).

---

### P6. Consolidate Per-Page `CrossRef` `RoNode` Walks (Low Priority — Already Off XPath)

* **Target:** Large split books (115K pages) and large single-page papers.
* **Reconciliation with PERFE (`PERFORMANCE.md`, 2026-11-15):** All of
  `CrossRef::process`'s standard descendant queries (`//ltx:bibref`, `//*[@idref]`,
  `//*[@labelref]`, `descendant::*[@decl_id or @meaning]`,
  `contains(concat(' ', @class, ' '), ...)`, etc.) are **already** answered by
  `latexml_core::common::xml_walk::WalkQuery` (read-only `RoNode` walks that
  wrap only matches — zero libxml XPath context or node-set allocation).
* **Remaining micro-lever:** Merging ~10 separate `RoNode` walks over the same
  page DOM into one shared pass (respecting mutations between `fill_in_tocs` and
  `fill_in_refs`) saves pointer-chasing only; rank well behind P1–P5.

---

### P7. Math Grammar Ambiguity Pruning (`MATHTAIL`), `POSTRESID`, & Longer-Horizon Levers

Carried forward from `PERFORMANCE.md` P3, `BEYOND_PERL_LEVERS.md`, and
`SYNC_STATUS.md` row R (confirmed still high-leverage by `arXiv-2511.03798v1`'s
2.39 parses/formula and 13.3% XSLT share):
* **Integrals (`DIFFOP_D` terminal):** pull `\int ... f(x)\,dx` off the
  ambiguous/legacy fallback path (high volume on both physics arXiv and Nasser's
  ODE book).
* **Bare `|x|` balanced-pair pre-lexer (`MATHTAIL`)** and **`f(x)` recognition-time hint**
  (note `MATHTAIL` 2106.09072 also shows a non-output-changing tree-enumeration
  reuse lever across bar-depth retries).
* **`POSTRESID` (`SYNC_STATUS.md`):** ~190 MB residual C-heap orphans from core
  into post on large books (remaining `insert_math_token` / `amsmath` /
  `rewrite.rs` 2-root residue).
* **BP-2 Step 2 (native Rust hot XSLT templates):** profile `LaTeXML-html5.xsl`
  apply-time templates (`xsltproc --profile`, ~27k cycles/MathML element per
  P13) on `2511.03798v1` and Nasser pages before transpiling the hottest
  math/inline templates (stylesheet compile is already refuted at <0.1%).
* **Token < 8 bytes / native expl3 hot primitives** for the 23.4% `digest` band
  on ordinary papers.
