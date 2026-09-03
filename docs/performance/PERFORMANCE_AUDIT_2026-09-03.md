# Read-only performance and memory audit (2026-09-03)

**Status:** diagnostic handoff; no code changed and no fresh benchmark run.
**Source snapshot:** `c6779fdc128f` plus the working tree as it existed on
2026-09-03. The tree was not clean, so re-check source locations before acting.

This audit reconciles the current source with the existing performance record.
It focuses on wasted work, non-linear algorithms, and retained or repeatedly
copied state. Runtime impact is called **measured** only where an existing
profile supports it; otherwise the cost model is a static finding that still
needs a same-session A/B.

Read first:

- [`PERFORMANCE.md`](PERFORMANCE.md) for phase budgets, methodology, and the
  acceptance checklist.
- [`ARXIV_PERFORMANCE.md`](ARXIV_PERFORMANCE.md) for the empirical campaign and
  the output-neutrality contract.
- [`ISSUE_361_MEMORY_TIME_PROFILE_2026-07-24.md`](ISSUE_361_MEMORY_TIME_PROFILE_2026-07-24.md)
  for the large-document allocation profile and settled dead ends.
- [`STREAMING_CORE_DESIGN_2026-07-29.md`](STREAMING_CORE_DESIGN_2026-07-29.md)
  and [`STREAMING_POST_DESIGN_2026-07-06.md`](STREAMING_POST_DESIGN_2026-07-06.md)
  for the implemented fragmented core and streaming split architectures.
- [`BEYOND_PERL_LEVERS.md`](BEYOND_PERL_LEVERS.md) for the longer-horizon math
  and XSLT work.
- [`../THERMALS.md`](../THERMALS.md) before any sweep or full-suite run.

## Recommended execution order

| Order | Work | Workload boundary | Evidence |
|---:|---|---|---|
| 1 | Remove discarded XMDual serialization | all math-bearing documents | definite wasted work; unmeasured delta |
| 2 | Index XMDual `idref` users once | math-dense / large DOM | removes `O(duals * nodes)` scans; unmeasured delta |
| 3 | Move `if_count` / `if_limit` to typed state | digest-heavy corpus | already measured at about 4-5% on witnesses |
| 4 | Stream core output directly to a file/writer | very large TeX-to-HTML/XML | removes a document-sized Rust `String` |
| 5 | Remove pass-2 per-segment cloning/construction | highly segmented streaming documents | removes `segments * global_state` traffic |
| 6 | Linearize whole-DOM split fallback | many-page split documents | definite quadratic front-removal pattern |
| 7 | Use lean standard/style state templates | startup and retained per-conversion RSS | definite excess reserved capacity; bytes unmeasured |
| 8 | Fix JATS/TEI preceding-axis matches | large JATS/TEI output | potential per-node document scan |
| 9 | Cache file-fallback data | package/file lookup-heavy documents | repeated regex compilation and cloning |

Orders 1 and 2 should be separate changes. The first is the smallest reliable
next step and supplies an uncontaminated measurement before changing lookup
structure.

## F1: discarded XMDual subtree serialization

**Location:** `latexml_core/src/document.rs`,
`Document::prune_xmduals`, currently near line 4602.

```rust
self.document.node_to_string(&dual);
```

The returned `String` is not read. The call serializes each selected XMDual
subtree and allocates its text only to drop it. Unlike the eager `Debug!`
serialization fixed in August, this call is unconditional and remains in the
current source.

**First patch:** delete only this call. Do not combine it with F2.

**Expected effect:** lower CPU and allocation traffic in finalization, scaling
with the number and size of XMDual subtrees. The magnitude is unknown. The
August debug fix demonstrated that subtree serialization can be expensive, but
its measured percentages must not be attributed to this different call.

**Validation:** compare release-mode wall, user CPU, max RSS, core XML, final
HTML, status, and error counts. Start with the existing build/math witness
`2304.10050`; add one formula-dense witness and a small XMDual structural test.
Use the production ar5iv profile, not a bare CLI fast-fail. If the ordinary
phase telemetry cannot isolate pruning, add a temporary env-gated `Instant`
probe and remove it or formalize it before merge.

## F2: XMDual reference repair scans the whole document per dual

**Location:** `latexml_core/src/document.rs`,
`Document::collapse_xmdual`, currently near line 4750.

```rust
self.findnodes(&s!("//*[@idref='{}']", dualid), None)
```

For each collapsed dual, XPath starts from the document and searches all
elements for a matching `idref`. With `D` collapsed duals and `N` document
nodes, the upper-bound shape is `O(D*N)`, in the same family as earlier
internal XPath loops already replaced by direct traversal.

**Implementation direction:** before the pruning/collapse loop, scan
`//*[@idref]` once and build `FxHashMap<String, Vec<Node>>`. When an id moves
from `dualid` to `contentid`, update only the nodes in that bucket. Move the
bucket to `contentid` as well so chained remaps remain correct. Preserve the
current reverse processing order and missing-id behavior.

**Validation:** add a generated test with increasing independent XMDual/ref
pairs and report time at 1x, 2x, and 4x. Doubling should approach 2x, not 4x.
Also cover multiple refs to one id, no refs, and a remap chain. Require
byte-identical core XML and HTML.

## F3: the core-to-post handoff is not end-to-end memory bounded

**Locations:**

- `latexml_oxide/src/converter.rs:139`: `ConversionResponse.result` is an
  `Option<String>`.
- `latexml_oxide/src/converter.rs:348`: the streaming DOM is serialized into a
  `String`.
- `latexml_oxide/bin/latexml_oxide.rs:1512`: TeX-to-post passes that in-memory
  XML string to `run_post_processing_logged`.
- `latexml_oxide/src/post.rs:482`: a large split handoff may then write the
  already-materialized string to a temporary file.
- `latexml_core/src/document.rs:1738`: the recursive serializer is specialized
  to `&mut String` rather than `std::io::Write`.

The fragmented core bounds its live DOM, but the public result and CLI handoff
still require output-sized Rust memory. For a large split job, the string may
be retained while the post front-end spills and reads page fragments. For a
non-split or unsupported split, post still builds a whole libxml2 DOM. The CLI
help claim that peak memory is bounded by fragment size should be scoped to the
core stage until this handoff is removed.

**Implementation direction:**

1. Generalize core serialization and recursive segment splicing to an
   `io::Write` sink with propagated I/O errors.
2. Add a writer/file-backed conversion API while retaining `convert() ->
   ConversionResponse` as the compatibility wrapper that collects a `String`.
3. In the CLI, write streamed core XML directly to a managed handoff file when
   post-processing follows; call `run_post_processing_from_file_logged` on it.
4. For XML-only output, write directly to the destination or buffered stdout.
5. Use RAII cleanup and preserve partial-output/error semantics. Do not make a
   second output-sized copy during UTF-8 conversion.

**Validation:** force streaming on existing `114_streaming_*` fixtures, exercise
the in-memory compatibility API, and run `118_streaming_split_parity`. Compare
output bytes across eager, old streaming, and writer-backed streaming paths.
On a large witness, sample RSS over time and show that the core XML handoff no
longer appears as a live Rust allocation. Report disk bytes and I/O time as well
as RSS; this change intentionally trades memory for sequential disk I/O.

## F4: pass 2 repeats conversion-global work for every segment

**Locations:**

- `latexml_oxide/src/core_interface.rs:685` clones the entire `node_fonts` map
  into every fragment.
- `latexml_core/src/document.rs:329` implements `new_empty_fields()` as
  `Self::new()`, creating and then discarding an unused XML document scaffold.
- `latexml_oxide/src/core_interface.rs:742` clones the rewrite-rule deque per
  fragment. `Stored::Rewrite` owns a `Box<Rewrite>`, so this is not a shallow
  container-only clone.
- `latexml_oxide/src/core_interface.rs:548` builds XPath diagnostic strings
  even when rewrite timing is disabled and the rule is not slow.

For `S` segments, `F` fonts, and `R` rewrite data, the cumulative allocation
and copy traffic includes `O(S*F + S*R)`. Only one fragment is live at a time,
so this is principally a CPU/allocator-throughput defect rather than an
`S`-multiplied peak-RSS defect. The streaming design records a real witness at
roughly 459,000 segments, making small per-segment constants material.

**Implementation direction:**

- Replace `new_empty_fields() -> Self::new()` with a constructor that initializes
  fields directly and does not allocate an `XmlDoc` that will be overwritten.
- Move rewrite hint/preview formatting inside the enabled timing or slow-rule
  branch.
- Store the conversion-global font map behind `Rc`; give each fragment a small
  local overlay because `set_node_font` can insert. Lookups check the overlay
  before the shared base. Avoid `Cow<HashMap>` if most fragments mutate once,
  since that would clone the full base on first write.
- Audit `Rewrite` compilation to split document-invariant clause compilation
  from fragment-dependent label/scope resolution. Share only the immutable
  compiled portion; preserve rule order and mutable invocation semantics.

Land the direct-constructor and lazy-diagnostic changes separately from the
ownership redesign. Measure pass-2 time, allocations, and segment count on the
same run; a normal small document is not an adequate witness.

## F5: whole-DOM split fallback uses quadratic front operations

**Location:** `latexml_post/src/split.rs`, `Split::process_pages`, currently
lines 174-280.

The function repeatedly performs `entries.remove(0)`, `removed.insert(0, ...)`,
and `removed.remove(0)`. Each operation shifts the remaining vector. A long
sibling run therefore has quadratic movement. Before surgery it also formats
and evaluates two `ancestor-or-self::*[...]` XPath expressions per page to
inherit `xml:lang` and `backgroundcolor`.

The streaming split front-end avoids this path only when its file-size gate,
destination requirement, and supported-union checks engage. Smaller inputs,
forced-off runs, unsupported split expressions, and fallback cases still use
the whole-DOM implementation.

**Implementation direction:** consume page entries through `VecDeque` or an
owned iterator. While walking previous siblings, append to a temporary vector
and reverse once. Match removed nodes without front deletion. Resolve the two
inherited attributes with a single direct parent-chain walk.

**Validation:** preserve `118_streaming_split_parity` as the cross-front-end
oracle. Add an eager-path scaling fixture with long adjacent sibling runs and
force `LATEXML_POST_STREAM_SPLIT=0`; report 1x/2x/4x time and output bytes.

## F6: catcode templates retain dump-sized state tables

**Locations:**

- `latexml_core/src/state.rs:341-342` reserves 8,192 `value` buckets and
  131,072 `meaning` buckets in `State::default`.
- `latexml_core/src/state.rs:383-395` constructs `STD_STATE` and `STY_STATE`
  through the same default.
- `latexml_core/src/state.rs:413-415` eagerly initializes both templates.
- `latexml_core/src/state.rs:561-573` overrides `value` and `catcode` through a
  struct update from `State::default`, so some default allocations are also
  constructed only to be discarded.

The active state benefits from dump-sized capacity. The standard/style states
are rotation templates used primarily for catcode regimes, yet each retains a
131,072-capacity meaning table and other pre-sized maps while idle.

**Implementation direction:** add an explicit capacity/profile choice such as
`ActiveDump` versus `CatcodeTemplate`, or extract a lean catcode regime object.
State rotation currently swaps whole `State` values, so a capacity profile is
the lower-risk first design. Build the final struct directly rather than using
a default whose overridden fields allocate.

**Validation:** record allocated bytes immediately after `Core::new`, after dump
load, and after one standard/style rotation. Confirm dump-load time does not
regress for the active state and macOS eager-initialization ordering remains
unchanged. Do not quote a byte saving until it is measured.

## F7: `if_count` remains on the generic global assignment path

**Locations:** `latexml_core/src/definition/conditional.rs:188` and
`latexml_core/src/state.rs:868-890`.

Every conditional updates `if_count` with `Scope::Global`. The generic path
probes assignment state, walks undo frames, and removes the key from each
relevant frame. This is already documented in `PERFORMANCE.md` P4 and measured
at about 4-5% on digest-heavy papers, with `remove_entry` at 2.39% self on the
named witness.

**Implementation direction:** use typed scalar fields for `if_count` and
`if_limit`; provide lookup/assignment adapters where dump filtering or external
bindings require their names. Review `if_stack`, reset behavior, state rotation,
dump serialization, and `\globaldefs` semantics before bypassing the table.

This is the highest-confidence corpus-wide percentage in this audit, but F1 is
still the recommended first patch because it is smaller and obviously unused.

## F8: JATS and TEI contain a preceding-axis match

**Locations:**

- `latexml_post/resources/XSLT/LaTeXML-jats.xsl:114`
- `latexml_post/resources/XSLT/LaTeXML-tei.xsl:76`

Both match paragraphs using `preceding::ltx:section`. Evaluating a preceding
axis for many paragraphs can scan an increasing fraction of the document and
become quadratic. This is outside the default HTML5 stylesheet, so it does not
invalidate the earlier HTML5-specific XPath cleanup, but it matters for large
JATS/TEI conversions.

Establish the intended structural scope before replacing it. Prefer a keyed or
structural test that does not search the document for every candidate. Benchmark
both formats on generated documents with increasing section/paragraph counts;
require byte-identical transformed output.

## F9: file fallback repeatedly compiles and clones lookup data

**Locations:**

- `latexml_core/src/binding/content.rs:3141-3154` and `3220-3230` compile the
  same three fallback regexes on each call.
- `latexml_core/src/binding/content.rs:3338` obtains an owned `Vec<String>` of
  all search paths for every lookup.
- `latexml_core/src/binding/content.rs:3381-3390` obtains and scans the binding
  name registry twice for exact and case-insensitive matches.
- `latexml_core/src/state.rs:3560-3577` shows that the nominal search-path view
  clones strings.

Use shared `Lazy<Regex>` values, expose a borrowed callback/view for search
paths, and build exact plus lowercase binding indexes when dispatchers register.
Preserve path order, case behavior, recursive `//` semantics, and filesystem
fallback precedence. This is low-risk cleanup, but rank it only after collecting
lookup count/miss telemetry on a package-heavy witness.

## F10: parallel math parsing needs a portable symbol boundary

This is a roadmap correctness constraint, not a current runtime defect.

`BEYOND_PERL_LEVERS.md` BP-1 proposes sending formula token/box IR to Rayon
workers with independent thread-local arenas. `Token.text` is a `SymStr`
(`latexml_core/src/token.rs:290`), and the backing arena is thread-local
(`latexml_core/src/common/arena.rs:32`). A numeric symbol handle produced on one
thread cannot be interpreted against another thread's independently populated
arena. State and libxml nodes also carry explicit thread-affinity constraints.

Before BP-1, define a portable formula snapshot using owned or shared strings,
then re-intern on the worker and return a portable parse result for sequential
DOM grafting. Measure the O(tokens) boundary conversion. Content-addressed
formula memoization is the lower-risk math lever to attempt first, provided its
key includes every parser context input and uses a bounded cache.

## Documentation reconciliation

Several apparent open items in the current documents are already implemented
in the source snapshot and must not be picked up without re-verification:

- `SymHashMap` string lookups already use non-interning `arena::get` probes.
- Gullet/stomach cycle guards exist; `DEFAULT_TOKEN_LIMIT` is currently four
  billion, not the one-billion state described by the older open-lever text.
- Debug formatting is lazily gated.
- `generate_id` and `collect_walk_matches` use direct parent/sibling walks.
- Font mapping is memoized, telemetry append is fixed, and the streaming split
  front-end has landed.

The same-day documentation reconciliation replaced those stale P1/P2 open
entries with the XMDual and streaming residuals, marked the older audit claims
as historical, and recorded the implemented fixes in closed history. Keep this
list as a source-verification warning when importing older agent memories.

## Handoff checklist

1. Re-read `git status`; preserve unrelated working-tree changes.
2. Start with F1 only. Capture a release baseline and post-change run in the
   same idle session, using the production ar5iv profile.
3. Record wall, user/sys CPU, max RSS, phase timings, status/error counts, and
   exact input/flags. Do not use fleet wall time as the A/B signal.
4. Compare core XML and final output byte-for-byte. For split output, compare
   the complete output trees, not only the main page.
5. Run the narrow structural tests, then `cargo nextest run --workspace` within
   the concurrency limits in `docs/THERMALS.md`.
6. Land one cost-model change at a time. In particular, do not combine F1 with
   F2, or the simple pass-2 constructor fix with the font/rule ownership work.
7. Update this document with measured before/after results, exact witnesses,
   guard test names, and the commit. Move completed findings into
   `PERFORMANCE.md` closed history when the audit has no live residual.

Settled constraints still apply: do not retry SmallVec-backed `Tokens`, whole
`Whatsit` variant boxing, absorb-owned box streaming, dump precompilation,
PGO/`target-cpu` as a portability-breaking default, or wholesale libxml2 DOM
replacement without new evidence. Performance changes remain subject to the
byte-identical output gate.
