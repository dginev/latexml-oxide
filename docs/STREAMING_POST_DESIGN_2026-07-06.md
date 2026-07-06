# Streaming post-processing for very large split documents — design + staged plan

**Date:** 2026-07-06
**Status:** foundation landed; the two-pass streaming split is the pending half.
**Supersedes the resume half of** the original `HANDOFF.md` (large-index-database
hardening). Companion to `docs/reproducers/` witness `~/scratch/nasser/index.xml`
(614 MB, ~7M nodes, 40 000 one-equation sections, `--splitat=section`).

---

## 1. Problem

Post-processing a very large *split* document (the reporter's `index.xml`:
614 MB → a ~7 GB libxml2 DOM, split into 40 201 pages) peaks at **~15.6 GB**
resident and blows the default wall-clock timeout. Perl `latexmlpost` OOMs
outright (and hits libxml2's XPath nodeset ceiling first). The fundamental cost:
the whole DOM is built, then split into 40 k page-DOMs that are **all held
simultaneously** from Split through Scan/CrossRef/MathML/XSLT/write.

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

> **Note on peak RSS:** the floor makes the run *correct*, not *lean* — split
> succeeding means all 40 k page-DOMs are now resident (~15.6 GB, up from the
> old ~7 GB one-unsplit-DOM). Lean-RSS is the pending streaming split below.

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
