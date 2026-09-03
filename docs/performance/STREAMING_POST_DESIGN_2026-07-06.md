# Streaming post-processing for very large split documents — design + staged plan

**Date:** 2026-07-06 (§3 status updated 2026-07-31)
**Status:** **the two-pass streaming split (§3) is IMPLEMENTED** —
`latexml_post/src/stream_split.rs` + the front-end selection in
`latexml_oxide/src/post.rs` (branch `feat-streaming-post-split`). The
deferral's revisit trigger ("a <64 GB target appears") fired on 2026-07-31:
the 131 MB witness's 2.68 GB core XML OOM'd a 31 GB laptop *during the
whole-DOM parse* (>22 GB and >26.6 GB under two caps, exit 137, zero pages) —
no configuration reaches success there. The implementation deviates from the
§3.2 sketch in one structural way: instead of expanding page subtrees and
running Scan inline (post-order), the stream **assembles each page's XML as
text and spills it at page close**, then a separate **pre-order Scan sweep**
re-parses one spilled page at a time — preserving Scan's order-sensitive
semantics (SITE_ROOT from the first page, ancestor-before-descendant parent
inference, `children` list order) byte-for-byte. Wrapper subtrees (non-page
elements containing pages, e.g. back-matter) take a mini-DOM descent via
`TextReader::expand_to_document`. Parity gate (§3.3):
`latexml_oxide/tests/118_streaming_split_parity.rs` — byte-identical rendered
pages vs the DOM split on a fixture exercising run adjacency, TOC
suppression, `inlist="toc"` lookahead, wrapper descent, unnamed pages,
template copies and inherited attributes. Gate: auto for file input ≥ 1 GiB
(`LATEXML_POST_STREAM_SPLIT=1/0` forces; `LATEXML_POST_STREAM_THRESHOLD`
tunes). The gate also exposed and fixed a latent DOM-split defect (inherited
`xml:lang` copy silently skipped — namespaced-attribute read) and a
rust-libxml one (`expand_to_document` minted a `default:` prefix onto
default-namespace content; fixed in libxml 0.3.18 with three new reader
APIs: `attributes_qname`, `value`/`is_empty_element`/`event`, `outer_xml`).
**Witness proof (2026-07-31, this laptop, maxperf):** `flat_index.xml`
(2.68 GB core XML) `--splitat=subsubsection --max-memory=26000` → exit 0,
**115,519 pages, 11 GB HTML, 37:31 wall, 17.4 GB peak RSS** (split ~1 min at
~0.6 GB; Scan sweep to ~3 GB; the render loop dominates and retains
~150 KB/page — follow-up below). Baseline: the whole-DOM parse alone
exceeded 26.6 GB with zero pages written.

**Ceilings that remain (pre-existing, verified identical on the rc4 binary,
2026-08-01):** a page (or an unsplit whole document) too large for libxslt
fails its transform ("XSLT transformation failed: Unknown error applying
stylesheet") — measured at a ~260 MB chapter page (`--splitat=chapter`) and
at a 300 MB unsplit document; both split paths fail IDENTICALLY (parity
holds in the failure case: same 5 of 6 pages written, same Error). The
unsplit-giant case additionally stacks the whole-DOM RAM cost and the
2 GiB `xmlBuffer` output ceiling. Splitting at a granularity that keeps
pages libxslt-sized (section and below for this witness) is the supported
mode at book scale. Note also the driver's standing signal-integrity wart:
a failed post run writes an EMPTY destination with process exit 0 (the
Error is in the log and the status code) — improving the CLI exit policy is
a separate decision.

Follow-ups (perf only, not correctness): (a) the render loop's ~150 KB/page
retention (ObjectDB is ~3 GB of it; the per-page `DocOwnedNode` drip on
id-dense math pages is the suspect — a `set_linked()` relink API in
rust-libxml would remove it); (b) serialize the core→post handoff flat
(`spill_flat`-style) to halve the streamed bytes — measured 50.9 % of the
witness's core XML is decorative indentation.
The 2026-09-03 source audit adds two adjacent residuals: avoid materializing
that handoff as a Rust `String` at all, and linearize `Split::process_pages`
for whole-DOM fallback/under-threshold runs. See
[`PERFORMANCE_AUDIT_2026-09-03.md`](PERFORMANCE_AUDIT_2026-09-03.md) F3/F5.
**Supersedes the resume half of** the original `HANDOFF.md` (large-index-database
hardening). Companion to `docs/reproducers/` witness `~/scratch/nasser/index.xml`
(614 MB, ~7M nodes, 40 000 one-equation sections, `--splitat=section`).

---

## 1. Problem

Post-processing a very large *split* document (the reporter's `index.xml`:
614 MB → a ~7 GB libxml2 DOM, split into 40 201 pages) peaks at **~21.6 GB**
resident (measured, uncapped) and blows the default wall-clock timeout. Perl
`latexmlpost` OOMs outright (and hits libxml2's XPath nodeset ceiling first).
The fundamental cost: the whole DOM is built, then split into 40 k page-DOMs
that are **all held simultaneously** from Split through
Scan/CrossRef/MathML/XSLT/write.

## 2. What landed (the correctness + foundation floor — DONE, verified)

The conversion now **succeeds** where it silently failed. On `index.xml`:
`Split into 40201 pages` (was silent `[not split]`), no XSLT nodeset death,
parse streamed from file. Landed on branch `harden-post-large-index`:

1. **Limit-safe queries** (`latexml_post`, commit `90d60d968c`).
   `//X[predicate]` full-document XPath overflows libxml2's 10M
   `XPATH_MAX_NODESET_LENGTH` (materializes `descendant-or-self::node()`,
   returns NULL) — silently swallowed → empty idcache + split matching nothing.
   Replaced `//*[@xml:id]`, `.//processing-instruction('latexml')` and the
   `make_splitpaths` union with limit-safe pre-order **DOM walks**
   (`scan_ids_and_pis`, `find_split_pages`/`collect_split_pages`), predicates
   applied in Rust. `findnodes_at` now `Warn!`s on a NULL evaluation instead of
   returning `vec![]` (fail-toward-flagging).
2. **Stream the file, skip engine init** (`latexml_oxide`, commit `c82fe29fd6`).
   `.xml` input routes through `run_post_processing_from_file*` →
   `PostDocument::new_from_file` (libxml2 `xmlReadIO`, no 614 MB `String`); the
   in-memory path (cortex fleet / LSP / tests) is byte-identical via a shared
   `PostInput`-parameterized impl. `prepare_session` (TeX.pool + dump) is skipped
   for XML input.
3. **rust-libxml streaming foundation** (branch `perf-improvements`,
   `20736684` + `62fc10a5`, CHANGELOG'd `52dbc523`):
   * `xpath::XPathError` + `evaluate_checked` / `node_evaluate_checked` /
     `node_evaluate_readonly_checked` — surface the nodeset-limit cause.
   * `reader::TextReader` — safe `xmlTextReader` pull parser: `from_file`,
     `read`/`read_next`, `node_type`/`is_element`/`depth`/`local_name`/
     `namespace_uri`, `read_to_next(pred)` (streamable downward-name XPath
     subset), `expand()` (borrowed `RoNode`) and **`expand_to_document()`** (an
     owned, namespace-reconciled `Document` copy — the unit XSLT/serialize
     consume). Unit-tested for namespace reconciliation + lifetime safety.
4. **CrossRef O(n²)→O(n)** (`latexml_post`, commit `4ec2587993`). Making split
   fire exposed a latent quadratic: `CrossRef::process` runs once per page, and
   two per-page passes scanned *global* state — `fill_in_frags` iterated the
   whole ObjectDB per page (an inversion tuned for single math-heavy docs), and
   `fill_in_relations`→`get_child_page_ids` rebuilt+scanned a parent's full
   child-page list per sibling. On `index.xml` this was **40 min 47 s (95 % of
   wall time)**. Fixed faithfully to Perl semantics: restore Perl's `//@xml:id`
   page-node walk in `fill_in_frags` (inverted loop kept only when a page has
   more id-nodes than the DB has entries), and memoize `get_child_page_ids` +
   record sibling positions so `find_previous/next_page_id` are O(1). Result:
   CrossRef **40 min 47 s → 6.1 s**, whole run **42 min 50 s → 2 min 18 s**,
   byte-identical output over all 40 201 pages. The eager path is now time-
   viable, so the streaming split below is a **memory-only** concern.

> **Note on peak RSS:** the floor makes the run *correct* and (after item 4)
> *fast*, but not *lean* — split succeeding means all 40 k page-DOMs are
> resident (~21.6 GB, up from the old ~7 GB one-unsplit-DOM). Lean-RSS is the
> pending streaming split below; peak RSS was untouched by the CrossRef fix.

## 2a. LANDED 2026-07-30: page-major rendering (PR #451)

The memory problem turned out to be the *driver's loop nesting*, not the
absence of file streaming. The driver was phase-major — `run_phase` over a
`Vec<PostDocument>` for Scan, MakeIndex, MakeBibliography, CrossRef, Graphics,
then MathML/XSLT, then a write loop — so every page stayed alive at every
boundary, at ~1.6 MB of per-document overhead each (own `xmlDoc`, dictionary,
id table, lazily-built caches). But only **Scan** needs global knowledge, and
it emits *strings* into the ObjectDB; everything after it is page-local.

So Scan runs globally (spilling each scanned page and freeing it), then pages
are re-read one at a time through CrossRef → Graphics → MathML → XSLT → write
→ drop. Peak became the one-time split DOM plus ONE page.

Measured on `index.xml` (614 MB core XML, `--splitat=subsubsection`):

| | before | after |
|---|---|---|
| outcome | exit 137, memory ceiling | **exit 0** |
| peak RSS | 80 GB, linear in pages | **15.98 GB, flat** |
| pages | **0** | **40,201** |
| HTML | none | 2.25 GB |
| errors | 6 silently-failed queries | **0** |

Also fixed in the same PR: whole-document `//X[pred]` queries are answered by
traversal (a closed predicate grammar) instead of a materialized node-set, and
an unanswerable evaluation is an `Error` rather than an empty result — six such
queries had been failing on this document, so post generated **no MathML, no
crossrefs, no images** and wrote a 0-byte HTML while exiting 0.

Markup verified against **Perl LaTeXML 0.8.8** on a ~10-page fragment of the
witness at the same `--splitat`: identical page count, page names, and
document-relation markup link-for-link. (That also proved the extra
`<link rel="chapter">`/`"section"` relations on the big document are
Perl-faithful content recovered by the query fix — the pre-change binary emits
them too — and surfaced a pre-existing defect where navigation link text
rendered `TEMPORARY_DOCUMENT_ID`, filed separately.)

**What remains** for the streaming split below: peak still includes the
one-time split DOM (~16 GB for this 614 MB input, so ~70 GB for a 2.66 GB
core XML), because Split must parse the whole document to find page
boundaries. Streaming that parse is the remaining lever.

## 3. Pending half — two-pass streaming split

**Goal:** never build the whole DOM. Stream the file, materialize **one page
subtree at a time** (`TextReader::expand_to_document`), so peak ≈ *one page DOM
+ the ObjectDB* (tens of MB) instead of 15.6 GB.

CrossRef needs a **global ObjectDB built across all pages** before any page
resolves → the pipeline is inherently **two passes** over the file:

```
pass 1  ── stream file ──►  expand each page ► Scan ► ObjectDB + page-tree metadata ► free
pass 2  ── stream file ──►  expand each page ► CrossRef + Graphics + MathML + XSLT ► write HTML ► free
```

Streaming the 614 MB file twice costs ~2×10 s of parse — cheap vs the memory win.
Passes 1 and 2 **reuse the existing per-`PostDocument` processors** (Scan,
CrossRef, MathML, XSLT) unchanged.

### 3.1 The hard part — hierarchical page extraction + navigation

Not a simple "yield each `<section>`". `Split` (`latexml_post/src/split.rs`,
port of Perl `Post::Split`) does whole-tree surgery that a forward stream cannot
do naively:

* **Hierarchy.** `--splitat=section` over `book > chapter > section` makes
  *chapters* pages too. A chapter's DOM interleaves its **own** content
  (intro paragraphs) with its nested **section-pages**. `process_pages` extracts
  a page **plus its following siblings** and builds the page's TOC from the
  extracted set. Streaming can't expand a `<chapter>` whole (that's 200 sections
  = not lean) — it must expand at the **leaf** (`<section>`) granularity and
  **reconstruct** each container page (chapter/book) from metadata + that
  container's *own* (pre-first-child) content.
* **Navigation.** prev/next/up links and the per-page nav-TOC need the **global**
  page list. Cheap to precompute in pass 1 as *metadata* (id, parent-id,
  localname, title, destination for each of 40 k pages — a few MB), then wire in
  pass 2.
* **Inherited attributes.** `process_pages` copies `xml:lang` /
  `backgroundcolor` from ancestors — must be threaded through pass-1 metadata.
* **`inlist="toc"` propagation**, unnamed-page naming, id-cache removal, etc.

### 3.2 Proposed structure

1. **Pass 1 (`stream_scan`).** `TextReader` over the file. Maintain an ancestor
   stack (depth/localname/id). On each **page-boundary** element start
   (localname ∈ split units, in `ltx:` ns): `expand_to_document()` the *leaf*
   subtree, run `Scan` into the shared `ObjectDB`, record a `PageMeta { id,
   parent_id, localname, destination, title, inherited attrs, in_toc }`, then
   `read_next()` to skip the subtree. For **container** pages (chapter/book):
   capture their **own** leading content separately (the nodes before the first
   nested page) — either by expanding only up to the first child page, or by a
   dedicated shallow copy. Build the page-tree from `PageMeta.parent_id`.
2. **Between passes.** Finalize destinations/names (port of `prenamePages`) and
   the nav graph from the page-tree; run `MakeIndex`/`MakeBibliography` off the
   `ObjectDB` (they already only need the DB + the placeholder nodes).
3. **Pass 2 (`stream_emit`).** `TextReader` again. For each page: rebuild the
   page `PostDocument` (leaf subtree via `expand_to_document`; container pages
   assembled from their own content + a generated child-TOC), splice navigation
   from the page-tree, run CrossRef + Graphics + MathML + XSLT, serialize, write,
   **free**. Peak = one page + ObjectDB + page-tree metadata.

### 3.3 Parity gate (non-negotiable — canvas-triage golden rules)

The streaming split MUST produce byte-identical pages to the DOM split, else it
is a silent divergence. Plan:

* Keep the **DOM `Split` path as the default/fallback**; gate the streaming path
  behind an opt-in (e.g. `--stream-split`, or auto for XML file input above a
  size threshold **only once parity-proven**).
* Add tests that run BOTH paths on small multi-level fixtures (book > chapter >
  section, with an index/bibliography/appendix, `inlist="toc"`, nested labels)
  and assert **byte-equal** per-page output + identical navigation.
* Only widen the auto-threshold after parity holds across the fixtures + a
  sampled diff on `index.xml`'s first/last N pages vs a DOM run on a big box.

## 4. Concrete pointers

* Reader API: `libxml::reader::TextReader` (rust-libxml `src/reader.rs`,
  `perf-improvements`). `expand_to_document()` is the owned-page unit.
* `Split` to mirror: `latexml_post/src/split.rs` — `presort_pages`,
  `prename_pages`/`get_page_name`, `process_pages` (the sibling-extraction +
  per-page TOC), `add_navigation`.
* Post driver: `latexml_oxide/src/post.rs` `run_post_processing_impl` — the
  `PostInput::File(path)` arm is where a streaming split would branch in
  (currently it parses whole-DOM via `new_from_file` then runs the normal
  pipeline). `Scan`/`CrossRef`/`MathML`/`XSLT` processors are per-`PostDocument`
  and reused as-is.
* Metadata source: pass-1 `Scan` already populates `ObjectDB`
  (`latexml_post/src/object_db.rs`); page-tree metadata is the new structure.
* Witness: `~/scratch/nasser/index.xml`; RSS monitor
  `scratchpad/run_monitored.sh`; success = `Split into ~40000 pages`, HTML
  written, **peak RSS well under 7 GB** (target: one-page + ObjectDB).

## 5. Cheaper interim (if the full streaming split is deferred further)

Disk-spill: build the DOM once (peak ~7 GB during parse+split), spill each page's
intermediate XML to a temp file after Split, **free the DOM**, then re-read +
process + write + free one page at a time. Halves peak (~15.6 GB → ~7–8 GB) with
no navigation reimplementation, but still builds the full DOM once (not <7 GB).
Not "streaming" — a fallback only.

## 6. Next: parallel page rendering (roadmap gap 3, designed 2026-08-02)

Pass B renders ~115k witness pages serially in ~37 min; XSLT is ~60% of that
wall. Audit conclusions (file:line verified 2026-08-02) that fix the design:

* **In-process page threads are blocked twice.** (1) `ObjectDB` is `!Send`:
  `Value::Xml(libxml::tree::Node)` is `Rc<RefCell<_>>`, plus the `xml_holder`
  document (`latexml_post/src/object_db.rs:274-292`). (2) `XSLT::process`
  serializes the whole transform behind a process-wide `XSLT_LOCK`
  (`latexml_post/src/xslt.rs:643-662`) because libxslt/libxml2 keep
  non-thread-safe process-global state (input-callback + EXSLT registries,
  error context, dictionaries) — concurrent transforms were WITNESSED
  deadlocking (`52_source_map` under `cargo test`). With XSLT pinned serial,
  Amdahl caps any thread pool at ~2.5×. Do not re-attempt threads without
  first making ObjectDB Send AND settling libxslt global state — both.
* **Chosen shape: process-level page-range workers.** Child processes get
  fresh libxslt globals (no lock contention, no Send constraints) and per-page
  isolation for free. Plan:
  1. ObjectDB file persistence (currently none — `object_db.rs:272` "no
     external DB persistence"): **SQLite via rusqlite `bundled`** (user +
     assistant design session 2026-08-02). Why SQLite over the field: the
     worker handoff needs cross-process page-cache sharing (mmap'd reads make
     N children share ONE physical db copy — kills the N×db RAM term), and
     the Perl `--dbfile` parity case needs transactional per-entry updates —
     Perl's ObjectDB is a Berkeley-DB tied hash with Storable-frozen entries,
     i.e. the precedent IS an embedded keyed store, not a snapshot. Bundled
     sqlite adds no new toolchain requirement (we already cc-build libxml2/
     libxslt) and the artifact is `sqlite3`-CLI-inspectable. Ranked out:
     redb (weak cross-process story, buys only purity), LMDB/heed (map-size
     preallocation misfits an unknown-size artifact), rkyv+mmap (no
     incremental update — wrong for the parity case), sled (unmaintained),
     serde-into-HashMap (full copy per child — the exact memory problem).
     Schema: `entries(key TEXT PRIMARY KEY, props BLOB)` (postcard-encoded
     property map, `Value::Xml` as serialized XML re-adopted via `adopt_xml`)
     + a metadata table. Staleness: the handoff db is a per-run temp artifact
     (no staleness by construction); the cross-run dbfile carries
     `PRAGMA user_version` (schema), the writer's binary version, and
     per-entry source-document + scan-timestamp provenance so re-scanning a
     document first deletes its stale entries (mirror Perl ObjectDB.pm
     timestamp semantics — read it first, record divergences in
     OXIDIZED_DESIGN). Lookup cost is a non-issue: ~µs/row against 19 ms/page
     XSLT; benchmark child open + end-to-end render vs in-memory baseline on
     the witness before landing (`perf-check`).
  2. A hidden worker mode in the `latexml` binary (env-gated, not a public
     flag): read db + a page-range manifest of spilled page paths, run the
     per-page pipeline (CrossRef → Graphics → MathML → XSLT → write), emit
     `Status:conversion:N` last — the parent max-folds child statuses exactly
     as core/post statuses already fold (`conversion_status_line` contract).
  3. Parent: after the index/bib baton sweeps, spill db, partition pages,
     spawn N children, stream/append their stderr+logs in child order
     (deterministic fold, mirroring `graphics.rs:2444-2447`), keep
     `check_timeout`/telemetry on the parent only
     (`stomach.rs:22-37` deadline and `telemetry.rs:168-209` stacks are
     thread- and process-local; workers time out via their own watchdog).
  4. N from the headroom rule: each child holds one page + one db copy, so
     N ≈ clamp(available_ram / db_rss, 1, cores); respect the cortex nesting
     caution (`graphics.rs:2189-2204`) — inside a cortex fleet default N=1.
* **Measured phase shares** (witness `--telemetry-out`, 2026-08-03, maxperf
  at `69ec59620f`, wall 5065 s): core ≈ 2549 s — math_parse **1289 s (50.6%
  of core)**, build 872 s, digest 296 s, rewrite 92 s; post ≈ 2288 s — XSLT
  **944 s**, CrossRef **738 s** (far larger than earlier estimates), split
  277 s, MathML-Pres 139 s, scan 113 s, serialize 48 s. 523,676 formulae /
  509,958 parse attempts. Consequences: (a) the worker-process pool covers
  crossref+mathml+xslt+serialize ≈ 1868 s → ~234 s at 8 workers; (b)
  parallel MATH PARSING is confirmed as the top core lever (1289 → ~160 s
  at 8 threads, needs a Send grammar/State snapshot — own design);
  (c) CrossRef's 738 s also invites algorithmic attention independent of
  parallelism (child-page memo hit rates on a 115k-page db).
  Telemetry-record wiring gap noted: its own warnings/errors/db_objects/
  output_bytes fields read 0 despite the 12,094-warning verdict — the JSON
  record snapshots before the fold; small follow-up.
* **MEASURED (PR #490, 2026-08-03): 8 render workers take the witness post
  pass 37:31 → 12:17 wall (3.05×)** — all 115,519 pages, exit 0, clean
  verdict/status fold, parent peak RSS 3.0 GB (rendering left the parent
  process; the whole fleet ran inside a 28 GB cgroup). The remaining 12:17
  is dominated by the still-serial prologue (split scan + index/bib sweeps
  + db save + spawn) plus ~6 min of parallel render — matching the ~5-8 min
  prediction from the phase shares. Byte-parity: sampled pages identical to
  the serial baseline; the mid-scale harness pins it systematically.
* **Rejected:** making ObjectDB Send + thread pool with serial XSLT (cap
  ~2.5×, large refactor for a small ceiling); hoisting XSLT to the main
  thread with worker-produced DOM strings (still serializes the 60% share).
