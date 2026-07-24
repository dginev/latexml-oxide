# Issue #361 — very-large single document: memory + time profile (2026-07-24)

Analytical performance investigation of a **legitimately huge single document**
(reporter nasser1, issue #361): an "Archive of LaTeX StackExchange newsletters"
book — **232 806 lines / 7.9 MB**, 803 sections, 2255 subsections, **12 178
subsubsections**, tikz/tabular/array/verbatim-heavy, CRLF line endings. Command:
`latexml_oxide --splitat=subsection --format=html5 --dest=index.htm index.tex`.

**This is not a conversion bug.** Rust converts it **error-free (0 errors)** and
**beats Perl** on the same host (Perl throws many `\FancyVerbGetLine`
unbalanced-input errors at the CRLF EOF and had not finished at 2 min). The
user's reported fatals are the **default resource guards** being too tight for a
legit huge single doc (they are FLEET-tuned): peak RSS trips the 4.5 GB stomach
fuse (`LATEXML_RSS_CAP_BYTES`), and the ~20 s digestion trips the 60 s
`--timeout` on the reporter's slower VM. The reporter is unblocked via the
runtime knobs (`LATEXML_RSS_CAP_BYTES=<big> --max-memory <big> --timeout 0`);
this doc is the **performance follow-up** to shrink RAM + time faithfully.

## Baseline (fast dev box, release build, guards off)

`LATEXML_RSS_CAP_BYTES=60000000000 latexml_oxide --timeout 0 --max-memory 0 --splitat=subsection --format=html5 --dest=out/index.htm index.tex`

- **Peak RSS 9.05 GB**, **wall 38.8 s**, 0 errors, 3007 split pages / 79 MB out.
- Perl same-host: **7.36 GB and climbing, unfinished at 2 min, many errors.**

### RAM — a transient coexistence, not a leak

RSS-over-time correlated to phase markers (sampler: `/proc/PID/status VmRSS`):

```
Digesting  → 2 → 4 → 6 GB    digested boxes accumulate (whole doc held)
Building   → 7.6 → 8.6 GB    DOM built while ALL boxes still alive
Finalizing → 9.05 GB PEAK    boxes (~6.5 GB) + DOM (~2.5 GB) coexist
post-proc  → 1.1 GB          boxes freed; only the DOM remains
```

The whole document's digested boxes live until `Document::absorb` finishes
building the DOM, then drop. **Peak = boxes + DOM together.** During pure
digestion the boxes sit on the boxing stack in one context
(`localized_box_list_total ≈ 236 832` top-level entries at 6 GB, each a subtree
— recursive box count far higher). Diagnose with
`LATEXML_DEBUG_MEMBUDGET=1 LATEXML_RSS_CAP_BYTES=<N>` (dumps box_list /
localized_box_list sizes + a backtrace at the cap).

### TIME — flat, no silver bullet

`perf record -F 199 -g --call-graph dwarf` (needs `perf_event_paranoid<=1`),
`perf report --no-children`. Top self-time function is **2.4 %** — a flat
profile:

- **~10–12 % allocator churn** (`mi_free`, `_int_malloc`, `malloc`, `cfree`,
  realloc/free) — the largest bucket; scales with box/node volume.
- **~7–10 % libxml2 XPath + strings** (`xmlXPathNextDescendant` 2.4 %,
  `xmlStrEqual` 2.3 %, `xmlStrdup`/`xmlStrndup`) — Scan/CrossRef `descendant::`
  queries over the 3005-page / 63 K-object DOM.
- rest spread thin: gullet loop (`read_balanced`, `cycle_guard_checkpoint`,
  `read_x_token` ~3 %), `Node::_wrap` 1 %, `Rc<DigestedData>::drop_slow` 1 %,
  string interner 1.5 %, `from_utf8_lossy`/`Utf8Chunks` 1.7 %,
  `Document::get_node_font` 0.7 %.

Post-phase breakdown (`LATEXML_POST_AUDIT=1`): digestion **~20 s**, CrossRef
**5.1 s**, XSLT **5.8 s** (3005 pages × ~2 ms), Scan/Index/Bib/Graphics/parse
~1 s. Math parsing is unchanged at 1200 formulae (no re-parse).

**Takeaway:** the doc isn't pathological — it's just large. RAM peak is
boxes+DOM; time is allocation + DOM traversal, both proportional to box/node
count. So *reducing box/node volume/density helps both axes*.

## Landed — M1 (this PR)

**`List.font: Option<Font>` → `Option<Rc<Font>>`** (`latexml_core/src/list.rs`,
`latexml_engine/src/tex_box.rs`). `Tbox.font` was already `Rc<Font>`; `List`
(the variant that sized `DigestedData` at 424 B, inflated by the 328 B inline
`Font`) stored it by value. Fonts repeat massively; sharing them (set-once,
never mutated in place — verified: only `list.rs` + one `tex_box.rs` literal
touch `List.font`, no post-construction writes) dedups the data and shrinks every
box (`DigestedData` 424 → ~216 B).

- **Peak RSS 9.05 → 7.55 GB (−1.5 GB / −17 %)**, wall 38.8 → 37.6 s (no
  regression), 0 errors, output identical, full suite **1678/0**.

## TODO — M2 (contained density, safe, resume here)

After M1 the largest `DigestedData` variant is **`KeyVals` (208 B)**
(`digested.rs::DigestedData`); everything else is ≤152 B (`Whatsit`). Boxing it —
`KeyVals(KeyVals)` → `KeyVals(Box<KeyVals>)` — drops `DigestedData` to ~160 B,
shrinking *every* box by ~56 B (KeyVals is rare, so the added indirection only
touches rare accesses). Est. a few hundred MB; also trims the allocator bucket.
- Update the `DigestedData::KeyVals` construction + match sites (grep
  `DigestedData::KeyVals`). Measure for any time change; keep the full suite green.
- Smaller inner-heap follow-ups: audit `Whatsit.properties`/`List.properties`
  (`SymHashMap<Stored>`, 72 B per value) and `Whatsit.args` for boxes that carry
  empty/near-empty maps. `HashMap` doesn't allocate until first insert, so only
  populated maps cost — confirm before touching.
- A `#[test]` asserting `size_of::<DigestedData>()` stays ≤ a budget would guard
  against future variant bloat (there is a `// TODO` about this on the enum).

## TODO — M3 (streaming boxes→DOM — the big RAM lever, architectural)

The only path to a *large* further RAM cut: free each digested subtree **as it is
absorbed** into the DOM, instead of holding the whole box tree until Building
finishes. Target peak **~7.5 → ~3–4 GB** (DOM-dominated).

- **Where:** `latexml_oxide/src/core_interface.rs::convert_document` L424
  `document.absorb(&digested, None)` builds the whole DOM from one top-level
  `Digested`; `Document::absorb(&mut self, object: &Digested, …)`
  (`latexml_core/src/document.rs:650`) recurses **by reference**, so nothing is
  freed until `digested` drops after Building.
- **Change:** make the top-level absorb *consume* the box tree and drop each
  child subtree right after it emits its DOM nodes. `Digested` is
  `Rc<DigestedData>` — dropping frees only when the refcount is 1, so first
  confirm document-flow boxes are uniquely owned (saved/`\setbox` boxes are
  shared and won't free early — that's fine, they're rare).
- **Risk/scope:** `absorb` is the core builder; ~10 call sites take `&Digested`
  (`grep 'document.absorb(&'` across `latexml_*`). Likely only the **top-level**
  document absorb needs the consuming variant; nested constructor absorbs can
  stay by-reference. Add a `absorb_owned`/drain path rather than changing every
  signature. Correctness-critical: gate on the FULL suite + re-convert this
  witness (0 errors, 3007 pages, output identical) + a spread of normal papers.
- Overlaps but is DISTINCT from the deferred post-processing streaming split
  ([`STREAMING_POST_DESIGN_2026-07-06.md`](STREAMING_POST_DESIGN_2026-07-06.md),
  task #44): that streams the *post* DOM; this streams *digestion→build*.

## TODO — time (secondary; no silver bullet)

- Density work (M1/M2/M3) directly cuts the ~10–12 % allocator bucket.
- libxml2 XPath ~7–10 % is Scan/CrossRef `descendant::` traversal across 3005
  pages; look for redundant full-doc queries (e.g. batch per-page work, cache
  `get_node_font` walks) before touching libxml2.
- `from_utf8_lossy`/`Utf8Chunks` ~1.7 %: the CRLF/encoding path — check the mouth
  isn't re-decoding input repeatedly.

## Reproducer + measurement

- Source: issue #361 attachment `fatal_error_oxide_rc2_july_24_2026.zip`
  (`index.tex`; not committed — 7.9 MB). A synthetic proxy: a `book` with a few
  thousand math-titled subsections under `--splitat=subsection` exercises the
  same boxes+DOM peak shape.
- Peak RSS + wall: `/usr/bin/time -v`. Phase RSS: sample `/proc/PID/status
  VmRSS` every 0.3 s vs the `Digesting`/`Building`/`Finalizing`/`post` markers.
  Phase times: `LATEXML_POST_AUDIT=1`. CPU: `perf record -g --call-graph dwarf`
  on a `CARGO_PROFILE_RELEASE_DEBUG=1 CARGO_PROFILE_RELEASE_STRIP=false` build.
