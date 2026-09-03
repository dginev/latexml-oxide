# Issue #361 — very-large single document: memory + time profile (2026-07-24)

> **Status update 2026-09-03:** this document remains the representation-density
> record and dead-end ledger. Fragmented core conversion and streaming split
> subsequently landed as the structural large-document solution; do not resume
> the original whole-document architecture from this baseline. Current writer,
> pass-2, split-fallback, and retained-state residuals are ranked in
> [`PERFORMANCE_AUDIT_2026-09-03.md`](PERFORMANCE_AUDIT_2026-09-03.md).

Analytical performance investigation of a **legitimately huge single document**
(reporter nasser1, issue #361): an "Archive of LaTeX StackExchange newsletters"
book — **232 806 lines / 7.9 MB**, 803 sections, 2255 subsections, **12 178
subsubsections**, tikz/tabular/array/verbatim-heavy, CRLF line endings. Command:
`latexml_oxide --splitat=subsection --format=html5 --dest=index.htm index.tex`.

**This is not a conversion bug.** Rust converts it **error-free (0 errors)** and
**beats Perl** on the same host (Perl throws many `\FancyVerbGetLine`
unbalanced-input errors at the CRLF EOF and had not finished at 2 min). The
user's reported fatals are the **default resource guards** being too tight for a
legit huge single doc (they are FLEET-tuned): peak RSS trips the cooperative
stomach fuse (~4.5 GB at the default ceiling), and the ~20 s digestion trips the
60 s `--timeout` on the reporter's slower VM. The reporter is unblocked via the
runtime knobs — **`--max-memory 0 --timeout 0`**; this doc is the **performance
follow-up** to shrink RAM + time faithfully.

> **Knob note (post-PR #363).** `--max-memory` is now the *single* memory knob:
> the soft stomach fuse is derived from it (`soft_cap_from_ceiling`, 75 % of the
> ceiling — which reproduces the historical ~4.5 GB fuse at the 6144 MiB
> default), and `--max-memory=0` disables **both** the soft fuse and the hard
> watchdog. It is also the *winner*: it overrides `LATEXML_RSS_CAP_BYTES`, so no
> env var can quietly countermand what you typed. That env still governs
> embedders which never parse CLI flags — the library test harness and the
> `cortex_worker` fleet, which pins each child to its `--max-rss-mb` — but in the
> binary it is no longer the thing to reach for. Note `LATEXML_RSS_CAP_BYTES=0`
> is NOT equivalent to `--max-memory=0`: it only silences the soft fuse where it
> still applies, and never touches the watchdog. The measurements below were
> taken with both guards off and are unaffected; only the invocation is simpler.

## Baseline (fast dev box, release build, guards off)

`latexml_oxide --max-memory 0 --timeout 0 --splitat=subsection --format=html5 --dest=out/index.htm index.tex`

- **Peak RSS 9.05 GB**, **wall 38.8 s**, 0 errors, 3007 split pages / 79 MB out.
- Perl same-host: **7.36 GB and climbing, unfinished at 2 min, many errors.**
- After the landed M1+M2+M4 density work this is **5.99 GB / 36.9 s** (same 0
  errors / 3007 pages / 79 MB) — a **−34 %** peak-RSS cut at unchanged wall time.
  The analysis below describes the *original* baseline; the phase *shape* is
  unchanged, only the magnitudes shrink.

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
`LATEXML_DEBUG_MEMBUDGET=1 latexml_oxide --max-memory <MiB>` (dumps box_list /
localized_box_list sizes + a backtrace when the derived soft fuse trips, i.e. at
75 % of `<MiB>` — so pick a ceiling whose 75 % is where you want the dump).

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

## Landed — M1, M2 (PR #362) + M4 (this PR)

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
- Guarded by **`digested_data_size_budget`** (`digested.rs`; tightened to ≤128 B
  by M4) so a future fat variant can't silently re-inflate the footprint. The
  `#[allow(clippy::large_enum_variant)]` on the enum is no longer needed and was
  removed (along with the stale size TODO it carried).
- Checked and **not** worth pursuing: `Whatsit.properties`/`List.properties`
  (`SymHashMap<T>` is a newtype over `HashMap`, which does not allocate until the
  first insert), so empty property maps already cost zero heap.

### M4 — box `Whatsit`'s two reversion-cache slots

`Whatsit` (160 B) then set the 168 B budget. The tempting move — boxing the
**variant** — is the wrong one, and the census says why (below). What actually
pays is boxing two *fields*: **`reversion: Option<Tokens>` → `Option<Box<Tokens>>`**
and **`dual_reversion: Option<HashMap<Tokens>>` → `Option<Box<…>>`**
(`latexml_core/src/whatsit.rs`). Those are the memo slots of a reversion cache
that **Rust cannot fill**: `revert(&self)` has no mutability, so the write-back is
commented out (`whatsit.rs` ~L379). Perl *does* cache there (`Whatsit.pm`
L134-138), so these are an unimplemented port, **not** dead code — box, don't
delete. `Option<Box<_>>` is 8 B via the null-pointer niche and allocates nothing
while `None`, so the slots cost ~nothing until the cache is wired up.

`DigestedData` **168 → 128 B**; `RefCell<Whatsit>` 160 → 120.

- **Same-session A/B** on the witness: peak RSS **6.41 → 5.99 GB (−0.41 GB /
  −6.5 %)**, wall **36.87 → 36.86 s (flat)**, 0 errors, 3007 pages, output
  **byte-identical** (`diff -rq`, zero differences). Full suite **1682/0**.
- Two call sites changed (`self.reversion.clone()` → `.as_deref().cloned()`); the
  `dual_reversion` lookup auto-derefs through the `Box` unchanged.

### Why NOT to box the `Whatsit` variant (census, 5 payloads)

Live box-type distribution at peak — the payload dependence is real, so this was
measured across deliberately contrasting documents:

| document | TBox | **Whatsit** | List | live boxes |
|---|---|---|---|---|
| #361 book (232 K lines) | 87.4 % | **4.5 %** | 6.1 % | 11.5 M |
| equality_big (math bench) | 82.7 % | **6.2 %** | 10.3 % | 131 K |
| tikz unit tests | 81.3 % | **3.5 %** | 13.1 % | 170 K |
| si (siunitx) | 57.4 % | **16.4 %** | 23.5 % | 77 K |
| mathtools (AMS) | 58.7 % | **16.7 %** | 22.1 % | 9 K |

Boxing a variant trades −64 B on every *other* box against **+96 B** on each box
of that variant (the `Rc` payload shrinks into a smaller mimalloc bin, but a
second allocation appears) — break-even at **~40 %**. Whatsit never approaches
that, so boxing the variant *would* save memory — but it costs one malloc/free
per Whatsit plus a pointer hop on the hot absorb path, for a further 24 B/box
beyond M4. Not worth it; **stop here**. Note the share *falls* as documents grow
(the 16 % readings are 14–34 KB files), so the margin is widest exactly where
memory matters.

Also measured, and the reason M4 targets fields rather than the whole struct:
across ~601 K whatsits on all five documents, `dual_reversion` and `reversion`
were `Some` **zero** times, while `args` (4.8 %) and `properties` (9.1 % on #361
but 46 % on the tikz doc) are genuinely used — so those two are **not** boxing
candidates, and their occupancy is strongly payload-dependent.

### The floor for this line of attack

`TBox`/`List` (`RefCell` = 96 B) now bound the enum at **104 B**, i.e. at most
24 B/box remains available from variant boxing. Density work is essentially
exhausted; further RAM cuts need a different mechanism (and M3 below shows the
obvious one does not pay).

## SETTLED DEAD-END — M3 (streaming boxes→DOM): implemented, measured, reverted

Freeing each digested subtree **as it is absorbed** looked like the one remaining
*large* RAM lever. It was implemented in full, passed every correctness gate, and
**did not pay** — so it was reverted. Do not re-attempt it in this shape.

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

Peak = `B + D` with **B ≈ 4.5 GB** (end of digestion) and **D ≈ 1.9 GB** (what
Building adds on top). *In theory* streaming makes boxes fall as the DOM rises,
and since the DOM total is far smaller than the box mass being released, the sum
would decrease from the moment Building starts — capping peak at **B ≈ 4.5 GB**,
i.e. −1.9 GB (−30 %). That is the number this experiment chased; the
implementation reached ~4 % of it, for the structural reason below.

Note the ceiling is `max(B, D)`, **not** `D`: digestion has already materialised
the whole box tree before Building starts, and that plateau is a hard floor for
*any* Building-phase change. Going below it would require interleaving digestion
with building — a far deeper change, and one that departs from Perl's
`digest-then-absorb` shape.

### What was built

`Document::absorb` (`latexml_core/src/document.rs:650`) is **not** recursive over
Lists — it runs an explicit worklist `Vec<Cow<Digested>>`, and for a `List` it
pushes `list.borrow().unlist()`, i.e. a **clone** of the children
`Vec<Digested>`. That clone is why nothing frees today: the parent keeps its own
strong refs, so the tree only dies when the root `digested` drops after Building.

The experiment added a consuming entry point `Document::absorb_owned(Digested)`
used *only* at the top-level call site (`core_interface.rs::convert_document`,
which already takes `digested` by value and never reads it again). Its worklist
holds **owned** `Digested`; in the `List` arm, when the entry is `Cow::Owned`
**and** `Rc::strong_count == 1`, it `mem::take`s the children out of the parent
instead of cloning them, so each worklist entry's subtree dies the moment it is
absorbed. Everything else falls back to the `unlist()` clone.

Two preconditions were verified, and both hold — the design was *not* the
problem:
- **The refcount test is sound.** No `Weak<DigestedData>` / `Rc::downgrade`
  exists anywhere in the workspace, so `strong_count == 1` really is sole
  ownership. (`Cow::Owned` must be checked *as well*: a borrowed entry can also
  have count 1 — the caller's only handle — and stealing from it would silently
  empty a List its owner still means to use.)
- **The spine is genuinely ours.** `stomach::expire_local_box_list`
  (`stomach.rs:965`) hands each body's boxes back via `mem::swap` on `box_list`,
  so the stomach gives up ownership; `digest_internal` accumulates them into a
  plain local `Vec` wrapped in one `List`.

### Why it does not pay

Correctness gates all passed — full suite **1679/0**, witness 0 errors / 3007
pages, and output **byte-identical** (`diff -rq`, zero differences). But:

| | peak RSS | wall |
|---|---|---|
| M2 (no streaming) | 6.43 GB | 37.1 s |
| M3 streaming, run 1 | 6.15 GB | 37.5 s |
| M3 streaming, run 2 | 6.23 GB | 37.7 s |

−3–4 %, i.e. **inside the ~5 % run-to-run variance** — not a defensible win. The
RSS curve shows why: the late cliff where the boxes are released *en masse*
(6.13 → 0.88 GB at t ≈ 23 s) is **still there, unmoved**. The boxes are not dying
during Building.

Instrumenting the `List` arm (steal vs clone counters) gave the reason:

```
steals = 54 526 (10.5%)   clones = 463 530 (89.5%)   children traversed = 8 282 516
```

The steal engages, but only along the **top-level worklist spine**. The box mass
hangs off `TBox`/`Whatsit` children, and their contents are absorbed by the
*constructor* API — `Whatsit::be_absorbed` → `Definition::do_absorption(&self,
document, whatsit)` → `document.absorb(&arg, …)` — each a fresh worklist seeded
`Cow::Borrowed`, which by construction may not steal. Those nested calls are the
463 K clone-path traversals. A subtree therefore still dies only when its
top-level ancestor is dropped, which is far too coarse to move the peak.

**To actually pay**, `be_absorbed`/`do_absorption` would have to *consume* their
arguments — a deep change to the binding/constructor API that every constructor
implements, and one that would fight the reversion machinery (`args` are re-read
to revert whatsits). That is a large parity risk for ~2 GB on pathological
documents. Not worth it; the ~4.5 GB end-of-digestion plateau stands as the
floor, and going below *that* needs digest↔build interleaving, which departs from
Perl's `digest-then-absorb` shape.

Distinct from the deferred post-processing streaming split
([`STREAMING_POST_DESIGN_2026-07-06.md`](STREAMING_POST_DESIGN_2026-07-06.md),
task #44), which streams the *post* DOM and is unaffected by this result.

## TODO — time (secondary; no silver bullet)

- Density work (M1/M2) directly cuts the ~10–12 % allocator bucket.
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
