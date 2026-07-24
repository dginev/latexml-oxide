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
- After the landed M1+M2 density work this is **6.43 GB / 37.1 s** (same 0
  errors / 3007 pages / 79 MB). The analysis below describes the *original*
  baseline; the phase *shape* is unchanged, only the magnitudes shrink.

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

## Landed — M1 + M2 (this PR)

### M1 — share `List` fonts

**`List.font: Option<Font>` → `Option<Rc<Font>>`** (`latexml_core/src/list.rs`,
`latexml_engine/src/tex_box.rs`). `Tbox.font` was already `Rc<Font>`; `List`
(the variant that sized `DigestedData` at 424 B, inflated by the 328 B inline
`Font`) stored it by value. Fonts repeat massively; sharing them (set-once,
never mutated in place — verified: only `list.rs` + one `tex_box.rs` literal
touch `List.font`, no post-construction writes) dedups the data and shrinks every
box (`DigestedData` 424 → ~216 B).

- **Peak RSS 9.05 → 7.55 GB (−1.5 GB / −17 %)**, wall 38.8 → 37.6 s (no
  regression), 0 errors, output identical, full suite **1678/0**.

### M2 — box the `KeyVals` variant

**`DigestedData::KeyVals(KeyVals)` → `KeyVals(Box<KeyVals>)`**
(`latexml_core/src/digested.rs`). After M1 the enum's size ceiling was `KeyVals`
(208 B: two `String`s, three `Vec`s, two `HashMap`s) — a **rare** variant setting
the price of *every* box. Boxing it drops `DigestedData` **208 → 168 B** (the
ceiling is now `Whatsit`, `RefCell<Whatsit>` = 160 B + discriminant), and the
indirection is only paid on rare KeyVals accesses.

- **Same-session A/B** (same host, same binary flags, M1 build vs M2 build):
  peak RSS **7.20 → 6.43 GB (−0.77 GB / −11 %)**, wall 37.8 → 37.1 s (no
  regression), 0 errors, 3007 pages both sides, full suite **1679/0**.
- **Output verified byte-identical**: `diff -rq` over both 3007-file trees
  reports zero differences. This is a pure representation change.
- **Cumulative M1+M2 vs the original baseline: 9.05 → 6.43 GB (≈ −29 %)** at
  unchanged wall time. Note the *earlier* M1 reading was 7.55 GB where this
  session measured 7.20 GB for the same code — **peak-RSS runs carry ~5 %
  variance**, so trust same-session A/B deltas over cross-session subtraction.
- The win exceeds a naive "40 B × top-level boxes": every box in the *recursive*
  tree pays the enum size, and each is a separate `Rc` allocation whose
  `16 B + payload` rounds up into an allocator size class, so the shrink also
  drops some boxes into a smaller class.
- Only **one** downstream site needed a change — `enumitem_sty.rs::extract_keyvals`
  clones the value out (`(**kv).clone()`); the ~40 other `DigestedData::KeyVals(..)`
  match sites auto-deref through the `Box` unchanged.
- Guarded by **`digested_data_size_budget`** (`digested.rs`, ≤168 B) so a future
  fat variant can't silently re-inflate the per-box footprint. The
  `#[allow(clippy::large_enum_variant)]` on the enum is no longer needed and was
  removed (along with the stale size TODO it carried).
- Checked and **not** worth pursuing: `Whatsit.properties`/`List.properties`
  (`SymHashMap<T>` is a newtype over `HashMap`, which does not allocate until the
  first insert), so empty property maps already cost zero heap.

### The next density ceiling (open, needs data before acting)

`Whatsit` (160 B) now sets the 168 B budget. Boxing it too would drop
`DigestedData` to ~104 B (`TBox`/`List` `RefCell` = 96 B) — but unlike `KeyVals`,
`Whatsit` is a **hot, common** variant, so the extra allocation + indirection per
whatsit could cost more time than the RAM is worth. Measure the live box-type
distribution first; do not box it on size reasoning alone.

## TODO — M3 (streaming boxes→DOM — the big RAM lever, architectural)

The only path to a *large* further RAM cut: free each digested subtree **as it is
absorbed** into the DOM, instead of holding the whole box tree until Building
finishes.

**What it can and cannot buy — measured.** Post-M1+M2 phase profile (0.3 s
`VmRSS` sampling against the phase markers):

```
 0.0s  0.02 GB
 1.9s  0.84 GB  Digesting     ← boxes accumulate, strikingly LINEAR
 5.6s  2.34 GB  Digesting        (~+0.40 GB/s, no inflection)
 9.3s  3.83 GB  Digesting
10.8s  4.46 GB  Digesting     ← END of digestion: the box plateau B
11.1s  4.59 GB  Building      ← DOM starts, boxes ALL still alive
14.8s  6.10 GB  Building
20.4s  6.43 GB  Finalizing    ← PEAK  = B + D
22.2s  1.43 GB  Finalizing    ← boxes dropped; DOM + engine only
37.0s  2.11 GB  post-processing
```

Today peak = `B + D` with **B ≈ 4.5 GB** (end of digestion) and **D ≈ 1.9 GB**
(what Building adds on top). Streaming makes boxes fall as the DOM rises, and
since the DOM total (~1.9 GB) is far smaller than the box mass being released,
the sum decreases monotonically from the moment Building starts. So the best
achievable peak is **B itself ≈ 4.5 GB** — i.e. **6.43 → ~4.5 GB, −1.9 GB
(−30 %)**.

It is `max(B, D)`, **not** `D`: digestion has already materialised the whole box
tree before Building starts, and that plateau is a hard floor for any
Building-phase change. Going below it would require interleaving digestion with
building — a far deeper change, and one that departs from Perl's
`digest-then-absorb` shape.

- **Where:** `latexml_oxide/src/core_interface.rs::convert_document` L424
  `document.absorb(&digested, None)` builds the whole DOM from one top-level
  `Digested` (`convert_document` already takes it **by value**, so the top-level
  handle is ours to consume). `Document::absorb`
  (`latexml_core/src/document.rs:650`) is **not** recursive over Lists — it runs
  an explicit worklist `Vec<Cow<Digested>>`, and for a `List` it pushes
  `list.borrow().unlist()`, i.e. a **clone** of the children `Vec<Digested>`.
  That clone is exactly why nothing frees: the parent `List` keeps its own strong
  refs, so the tree only dies when the root `digested` drops after Building.
  (`TBox`/`Whatsit` subtrees re-enter `absorb` from their `be_absorbed`, which
  can stay by-reference — they are freed when their worklist entry drops.)
- **Change:** add a consuming entry point (`absorb_owned(Digested)`), used *only*
  at the top-level call site, whose worklist holds owned `Digested`. For the
  `List` arm, when `Rc::strong_count(&entry) == 1` — we are the sole owner, so
  emptying it is unobservable — `mem::take` the children out of the `List`
  instead of cloning them; otherwise fall back to today's `unlist()` clone. Each
  worklist entry then drops right after it is absorbed, freeing its subtree.
- **Why the refcount test is sound:** there are **no `Weak<DigestedData>`** and no
  `Rc::downgrade` anywhere in the workspace (verified), so `strong_count == 1`
  really does mean nobody else can observe the mutation. Shared boxes
  (`\setbox`/`\usebox`, State-held) simply take the clone path and are not freed
  early — correct, just not optimised.
- **Why it will actually free (the feasibility question):** the optimisation is
  worthless if the stomach still holds the same handles. It does not —
  `stomach::expire_local_box_list` (`stomach.rs:965`) hands the body's boxes back
  via `std::mem::swap` on `box_list`, so the stomach **gives up ownership**;
  `digest_internal` (`core_interface.rs:608`) then accumulates them into a plain
  local `Vec` and wraps it in one `List`. The document spine is therefore
  uniquely owned, and `strong_count == 1` holds along it.
- **Risk/scope:** `absorb` is the core builder; ~10 call sites take `&Digested`
  (`grep 'document.absorb(&'` across `latexml_*`) and should stay as they are.
  Correctness-critical: gate on the FULL suite + re-convert this witness and
  `diff -rq` the 3007-file tree against the pre-change output (this is how M2 was
  validated) + a spread of normal papers.
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
- **Always A/B in one session.** Peak RSS carries ~5 % run-to-run variance (the
  same M1 build measured 7.55 GB once and 7.20 GB later), so a cross-session
  subtraction can invent or erase a few hundred MB. Build the "before" binary by
  `git checkout <prev-commit> -- <the changed files>`, measure, restore with
  `git checkout HEAD -- <same files>`, rebuild, measure again.
- **Output-identity gate for any memory change**: convert to two separate dest
  dirs and `diff -rq out_before out_after` — must report zero differences across
  all 3007 files. (The emitted "created on <Month D, YYYY>" stamp is
  day-granular, so same-day runs are byte-comparable.) M1+M2 both pass this.
