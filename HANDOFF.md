# HANDOFF — Hardening post-processing for very large index databases

**Date:** 2026-07-06
**Goal:** Make `latexml-oxide` post-processing survive very large "index database"
inputs (speed + lean memory), where **Perl LaTeXML OOMs and hits libxml2's XPath
nodeset limit**. Reported real example downloaded to `~/scratch/nasser`.

> **STATUS 2026-07-06 (updated):** The correctness + foundation floor is
> **DONE and verified** — `index.xml` now converts successfully
> (`Split into 40201 pages`, no XSLT nodeset death, parse streamed from file,
> engine-init skipped) on branch `harden-post-large-index`. rust-libxml gained a
> checked-XPath API and a streaming `reader::TextReader` (tested, pushed,
> CHANGELOG'd on `perf-improvements`). The remaining half — the **two-pass
> streaming split** to cut peak RSS from ~15.6 GB to <1 GB — is **staged as a
> dedicated, parity-validated follow-up**, designed in
> [`docs/STREAMING_POST_DESIGN_2026-07-06.md`](docs/STREAMING_POST_DESIGN_2026-07-06.md)
> (the new resume point for that work). The original steps 1–7 below are kept for
> reference; steps 1–5 correspond to the landed floor, step 6 to the deferred
> streaming split.

---

## 1. The reproduction case (`~/scratch/nasser`)

| File | Size | Notes |
|---|---|---|
| `index.tex` | 19 MB | LaTeX "index database" source |
| `index.xml` | **614 MB** | latexml core output (the post-processing input) |
| `index.latexml.log` | — | core conversion succeeded (52m 7s in Perl) |
| `index.latexmlpost.log` | — | **Perl `latexmlpost` FAILED** |

Perl's failure (the user's report):
```
XPath error : Memory allocation failed : growing nodeset hit limit
  In Post::Document[index.htm] .../index.xml
LaTeXML died! at .../LaTeXML/Common/XML/XPath.pm line 36.
```

**Document scale** (~7M element nodes):
`xml:id`=640,200 · `XMTok`=4,400,000 · `XMApp`=1,480,000 · `Math`=40,000 ·
`equation`=40,000 · `section`=40,000 · `chapter`=200.
It's 40,000 tiny one-equation sections — the reporter ran `--splitat=section`.

Perl command that failed:
```
latexmlpost --format=html5 --splitat=section --dest=index.htm index.xml
```

---

## 2. The analogous latexml-oxide run — IT ALSO FAILS (silently)

Run (post-only; a `.xml` input auto-routes to the post pipeline):
```
target/release/latexml_oxide ~/scratch/nasser/index.xml \
  --format html5 --splitat section --dest OUT/index.html
```

Findings:

1. **Default run OOM-killed by its own watchdog.** True peak RSS = **7.09 GB**
   (614 MB source → ~11× as a libxml2 DOM). The `--max-memory` default is
   **6144 MiB** (`latexml_oxide/bin/latexml_oxide.rs:178`), so the watchdog
   (`latexml_core/src/watchdog.rs`) kills it at ~5 s with
   `Fatal:oom:rss … exceeded the 6144MB ceiling`.

2. **With the ceiling raised (`--max-memory 240000`) it gets further but still
   fails — and the real failures are HIDDEN.** Phase timings (release,
   `LATEXML_POST_AUDIT=1`):
   ```
   PostDocument::new_from_string  4876 ms
   Scan                            601 ms   (Info:split:result [not split])  ← !!
   MakeIndex                       453 ms
   MakeBibliography                880 ms
   CrossRef                       8618 ms
   Graphics                        437 ms
   process_chain (MathML+XSLT)   18270 ms
   → runtime error: resources/XSLT/LaTeXML-block-xhtml.xsl line 203 value-of
   → Error:post:convert Post-processing failed: XSLT transformation failed
   ```
   No HTML produced.

### Root cause (same as Perl, but double-swallowed)

libxml2 evaluates any `//X[predicate]` by first materializing
`descendant-or-self::node()` (>10M nodes here) and hits its **hardcoded 10M
`XPATH_MAX_NODESET_LENGTH` "growing nodeset hit limit"**, returning **NULL**.

Confirmed pattern map (via `xmlllint --shell`, mirrors libxml2 exactly):

| Query | Result | Used by |
|---|---|---|
| `//ltx:section` (typed, no predicate) | ✓ 40000 | libxml2 optimizes typed descendant scan |
| `//*[@xml:id]` | ✗ **NULL** | `set_document_internal` idcache — **silently empty** |
| `//ltx:Math[not(ancestor::ltx:Math)]` | ✗ **NULL** | `math_processor` |
| `//ltx:index[preceding-sibling::ltx:section]` | ✗ **NULL** | a `Split` union arm |
| `count(//node())` | ✗ **NULL** | proves the `descendant-or-self::node()` blow-up |

Both layers then **swallow** the NULL:
- **rust-libxml** `Context::evaluate` → `Err(())` (no detail) —
  `~/git/rust-libxml/src/xpath.rs`.
- **latexml_post** `PostDocument::findnodes_at` → `vec![]` on `Err(_)` —
  `latexml_post/src/document.rs:505-508`.

**Consequence chain (all silent):**
1. `//*[@xml:id]` NULL → **idcache built empty** at parse time
   (`document.rs:200`, in `set_document_internal`).
2. The `Split` union (built by `make_splitpaths`, `latexml_oxide.rs:1066`)
   contains predicated `//ltx:index[preceding-sibling::…]` arms → the whole
   union evaluates NULL → `Split::process` sees `pages.is_empty()` →
   `Info:split:result [not split]` (`latexml_post/src/split.rs:450,457`).
3. **Nothing splits** → the full 614 MB doc stays as ONE document.
4. XSLT then runs on the giant doc and its own internal XPath
   (`classPI = //processing-instruction()…`, `value-of` at
   `LaTeXML-block-xhtml.xsl:203`) hits the same limit → **hard XSLT failure**.

**The linchpin:** if split actually triggered, every page would be one tiny
section (~110 `XMTok`) and MathML/XSLT/memory would all be trivial. Split is
blocked ONLY by the swallowed nodeset-limit error.

Extra waste observed:
- **XML-input (post-only) mode still runs a full TeX engine init**
  (`converter.prepare_session`, `latexml_oxide.rs:604`) — loads `TeX.pool` +
  dump it never uses.
- **The 614 MB file is `read_to_string` into a Rust `String`**
  (`latexml_oxide.rs:673`) and then `parse_string`'d — a whole extra 614 MB
  copy on top of the DOM. User directive: *use libxml2 file methods, don't
  vivify a 500 MB string.*

---

## 3. Agreed plan + scope decisions

User picked (via question): **Full correctness fix (steps 1–4)**, with
**query rewrites living in `latexml_post`** (keep libxml2 as-is; do NOT raise
the nodeset limit — that only trades a crash for the OOM the reporter already
hit). Plus later directive: **stream / use file parsing for massive inputs;
don't vivify the 500 MB string; strategic chunking where native streaming is
unavailable.**

Layered plan (maps to the task list, `TaskList` IDs in brackets):

1. **[#1] rust-libxml — surface XPath errors.** ✅ DONE (compiles).
2. **[#2] latexml_post — stop swallowing.** `findnodes_at` logs a `Warn!`/
   `Error!` on XPath failure instead of silent `vec![]`.
3. **[#3] latexml_post — limit-safe full-doc queries.** Rewrite the
   `//X[predicate]` full-doc queries as typed-search + Rust-side predicate /
   manual DOM traversal so **split triggers**; then all downstream phases run
   on the 40k small pages. Targets: `set_document_internal` (`//*[@xml:id]` +
   `.//processing-instruction('latexml')`) and `Split::get_pages`
   (the `make_splitpaths` union). Consider `math_processor`'s
   `//ltx:Math[not(ancestor::ltx:Math)]` too (per-page it's small once split
   works, but the query is still predicated).
4. **[#4] Avoid the 614 MB String + [#5] skip engine init.** Route XML-input
   post-processing through `PostDocument::new_from_file` (`document.rs:246`,
   uses `parse_file` → `xmlReadIO`, streaming chunked read — no giant String),
   and skip `prepare_session` when input is already-converted XML.
   `run_post_processing(xml: &str, …)` (`latexml_oxide/src/post.rs:112`) will
   need to stop requiring the whole doc as `&str` (it currently also uses `xml`
   for `extract_svg_fragments`, a `xml.contains("package=\"ar5iv")` sniff, and
   the empty-input check — re-derive these from the parsed Document or a cheap
   bounded read).
5. **[#6] Streaming split (deep, lean-RSS).** Even with split working, the
   CURRENT pipeline parses the full 614 MB → 7 GB DOM first, then holds ALL
   40k page DOMs at once (`docs: Vec<PostDocument>` in `post.rs`), so peak RSS
   stays ~7 GB (just redistributed) — it COMPLETES but isn't lean. To actually
   cut peak: stream. **rust-libxml has NO safe `xmlTextReader` wrapper yet**
   (only raw bindings in `src/default_bindings.rs`) — native streaming needs a
   new reader API (`xmlReaderForFile` + `xmlTextReaderExpand`/`Preserve` to get
   one section subtree at a time). Blocker: **CrossRef needs a global ObjectDB
   built across ALL pages before it resolves cross-page refs** → a streaming
   design is inherently two-pass (pass 1: Scan → ObjectDB; pass 2:
   CrossRef+MathML+XSLT+write+free each page), re-materializing pages so no
   more than one full page DOM is resident. A cheaper interim win: after split,
   **free the parent DOM immediately** and **write+drop each page as soon as
   its XSLT finishes** instead of collecting all of them.

6. **[#7] Verify end-to-end** on `index.xml`: split triggers, HTML pages
   produced, peak RSS much lower; run `cargo test` in affected crates.

---

## 4. What's DONE (this session)

### rust-libxml (`~/git/rust-libxml`, on commit `783ff068`)
`src/xpath.rs` — **landed, compiles** (`cargo build` clean; 5 pre-existing
`transmute` warnings unrelated):
- `use crate::error::StructuredError;`
- New `pub struct XPathError { message: Option<String>, code, domain }` with
  `from_last_error()` (snapshots `xmlGetLastError()`), `is_nodeset_limit()`
  (matches "nodeset" / "Memory allocation failed"), `Display`, `Error`.
- New `evaluate_checked` / `node_evaluate_checked` /
  `node_evaluate_readonly_checked` → `Result<Object, XPathError>`.
- Existing `evaluate` / `node_evaluate` / `node_evaluate_readonly` reimplemented
  as `*_checked(...).map_err(|_| ())` — **zero API break** for existing callers.
- Not yet empirically verified that `xmlGetLastError()` returns the
  "growing nodeset hit limit" text after a NULL eval (verify in Step 7 with a
  unit test / the real file).

**IMPORTANT — to consume the local rust-libxml change:** the workspace patches
`libxml` to the KWARC git repo (`Cargo.toml` around line 50):
```toml
[patch.crates-io]
libxml = { git = "https://github.com/KWARC/rust-libxml", branch = "perf-improvements" }
```
Local `~/git/rust-libxml` HEAD == that pinned commit (`783ff068`). To test the
edit either (a) temporarily point the patch at the path:
```toml
libxml = { path = "/home/deyan/git/rust-libxml" }
```
or (b) commit + push to the `perf-improvements` branch and `cargo update -p libxml`.
Prefer (a) for local iteration; revert before landing (or push upstream since we
maintain the crate).

### latexml-oxide
- Task list created (`TaskList`): #1 done, #2–#7 pending.
- No latexml-oxide source changed yet.

---

## 5. Remaining work — concrete pointers

- **[#2] `latexml_post/src/document.rs:488-509` `findnodes_at`:** switch to
  `ctx.evaluate_checked` / `ctx.node_evaluate_checked`; on `Err(e)` emit
  `Warn!`/`Error!` (latexml_core error macros) including `e` and the xpath, then
  return `vec![]`. This alone converts silent corruption into a loud diagnostic
  (CLAUDE.md "fail toward flagging errors"). `Err` only fires on real evaluation
  errors (empty results are `Ok` with 0 nodes), so no log spam.
- **[#3] `document.rs:198-238` `set_document_internal`:** replace
  `findnodes("//*[@xml:id]")` and `findnodes(".//processing-instruction('latexml')")`
  with a manual recursive DOM walk (rust-libxml `Node` child/sibling iteration)
  collecting elements carrying `xml:id` and the latexml PIs. No XPath → no
  materialization.
- **[#3] `latexml_post/src/split.rs:67,450` `get_pages`:** evaluate each union
  arm as a **typed** search (`//ltx:section`, `//ltx:part`, `//ltx:chapter`,
  `//ltx:bibliography`, `//ltx:appendix`, `//ltx:index` — all confirmed
  limit-safe), then apply the `preceding-sibling::`/`parent::` predicates in
  Rust. Keep document order. (Union string source: `make_splitpaths`,
  `latexml_oxide/bin/latexml_oxide.rs:1066`.)
- **[#4/#5] `latexml_oxide/bin/latexml_oxide.rs`:** for `is_xml_input(&source)`
  (line ~672): (a) skip `converter.prepare_session` (line 604) — gate engine
  init on "will actually convert TeX"; (b) don't `read_to_string` (line 673) —
  pass the path so post uses `new_from_file`. Requires refactoring
  `run_post_processing(xml: &str, …)` (`latexml_oxide/src/post.rs:112`) to take
  a path/Document; re-derive the 3 `&str` uses noted in Step 4 above.
- **[#6]** see Step 5 of the plan; likely needs a `TextReader` wrapper PR to
  rust-libxml. Scope/land the interim "write+free each page" win first if the
  full streaming rewrite is too big for the pass.

---

## 6. How to reproduce & verify

```bash
# Build (release for real perf/RSS numbers; debug OOMs faster due to the 6GB guard)
cd ~/git/latexml-oxide
cargo build --release --bin latexml_oxide

# RSS-monitored analogous run (script written this session):
SP=/tmp/claude-1000/-home-deyan-scratch-nasser/<session>/scratchpad   # see below
bash $SP/run_monitored.sh ./target/release/latexml_oxide \
     ~/scratch/nasser/index.xml $SP/out release_full

# Quick manual run with phase timings + high ceiling:
LATEXML_POST_AUDIT=1 ./target/release/latexml_oxide ~/scratch/nasser/index.xml \
  --format html5 --splitat section --max-memory 240000 --dest /tmp/out/index.html

# Probe the libxml2 limit directly (no rebuild needed):
{ echo "setns ltx=http://dlmf.nist.gov/LaTeXML"; \
  echo 'xpath //ltx:index[preceding-sibling::ltx:section]'; } \
  | xmllint --shell ~/scratch/nasser/index.xml    # → "Object is empty (NULL)"
```
The RSS-monitor script (`run_monitored.sh`) and prior run outputs are in this
session's scratchpad:
`/tmp/claude-1000/-home-deyan-scratch-nasser/a28e33f6-472b-437b-85bb-4b88ee916572/scratchpad/`
(re-create the script easily — it just backgrounds the binary and samples
`/proc/PID/status:VmRSS` every 0.5 s).

**Success criteria (Step 7):** `Info:split:result [Split into N pages]` with
N≈40000, HTML pages written, XSLT no longer errors, peak RSS materially below
7 GB (target: well under, once streaming/incremental-write lands), and
`cargo test` green in `latexml_post` + `latexml_oxide`.

---

## 7. Machine / environment notes
- Dev box: 246 GB RAM, 128 cores — it does NOT OOM here (the reporter's WSL box
  does); we gate on the **watchdog ceiling + measured peak RSS**, not a hard OOM.
- libxml2 on system: 2.15.x (`xmllint` reports `libxml version 21502`);
  `XPATH_MAX_NODESET_LENGTH` = 10,000,000 (hardcoded in libxml2 `xpath.c`).
- Build profiles: default `test` for dev; `--release` for perf/RSS numbers
  (CLAUDE.md §Build & Test). A `target/debug/latexml_oxide` and
  `target/release/latexml_oxide` were built this session.
- `latexml_post` deps: `libxml = "0.3.14"` + `libxslt = "0.1.4"` (both libxml2).
