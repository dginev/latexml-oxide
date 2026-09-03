# Streaming the CORE stage — fragmented conversion for arbitrarily large documents

**Date:** 2026-07-29
**Status:** IMPLEMENTED on `feat-streaming-xml-core` (S2–S5: stomach yield,
spill/placeholder/purge, pass-2 driver, splice assembly, `--streaming` flag +
auto-when-doomed), byte-identical to eager across the full suite (1781/0),
the eight forced-streaming sweep suites, and the 19.8 MB real witness
(613,104,457-byte outputs `cmp`-equal). **The linear-RSS premise below was
subsequently HALVED by a leak fix**, not obsoleted: most of the "DOM cost"
was rust-libxml discard semantics (unlink never frees a doc-owned node;
`append_tree` copies and the sources were abandoned — ~1.4 MB/formula).
With `Node::free_subtree` (libxml 0.3.17) + `Document::discard_subtree` at
the math-parser/replace_tree/spill discard sites, measured on the 19.8 MB
witness (release, same host): **eager 63.7 → 34.9 GB peak** (wall 8:27
unchanged); **streamed at a 24 GB cap: completes at 16.97 GB peak, 8:28**
(pre-fix: hard SIGKILL at 24.6 GB; pre-fix streaming overhead +38% — gone).
Streaming also avoids eager's 3 libxml2-XPath node-set-capacity errors on
the 613 MB output.
**PRODUCTION PROOF (2026-07-30): the 131 MB witness itself CONVERTS** at
`--streaming --max-memory 48000`: exit 0, **peak RSS 28.1 GB** (pass 1
bounded ~15 GB over 25M yields / 482k segments; pass 2 ~19.5 GB steady),
wall 1:10, **2.66 GB well-formed XML**, zero duplicate ids, one real error
(`{nowrap}` undefined — issue 297's binding, not a streaming defect). The
composed levers beyond the leak fix: the `node_boxes` stale-entry sweep
(build-time discards — alignment rearrangement above all — leaked pinned
`Digested` trees), nested spills kept NESTED (literal placeholders +
recursive assembly splice — inlining had rebuilt an 841 MB and a 1.85 GB
segment out of chapter shells), the spilled-id dedup half
(`Document::spilled_ids`), `malloc_trim`/`mi_collect` give-back, and a
source-scaled runaway-token backstop (×200/byte).

**PERF CONSOLIDATION (2026-07-31, branch `perf-streaming-pass2-segment-coalescing`):
the same witness/run now completes in 32:56 wall / 1942.9 s user CPU** (from
1:10:29 / 4125.8 s — −53 %; ≈14.8 s/MB, below the 21 s/MB eager curve), peak
31.5 GB @ the same 48 GB cap, output byte-identical to the unmodified base
commit (md5-verified against a paired control build). Log: 3,142,509 → 26,283
lines (−99.2 %). Levers, in landed order: pass 2 shares one label index instead
of copying 28,068 labels × 459k segments (12.9 G String allocs); segment
telemetry gated like pass 1's; the **soft-yield floor** (the dominant
structural fix — see the derivation table); logger stops emitting a blank line
per record (45.8 % of the log; also process-correct now, `AtomicBool` under the
stderr lock); streaming phases report to telemetry (`Digest` was 0 µs on every
streamed job, `formulae` clobbered — phase-sum/wall now 0.99–1.00); **flat
spill serialization** (51.2 % of intermediate bytes were indentation). Measured
phase split of the 32:56 run: MathParse 41.1 %, Build 28.6 %, Digest 22.4 %,
Rewrite 5.4 %, Serialize 2.3 % — MathParse (523,676 formulae) is the only
remaining ≥40 % block; the next lever of that size is ambiguity reduction, not
constant-factor work. Warning counts are NOT segmentation-invariant (11,414 /
11,397 / 11,390 across runs, output identical) — per-segment log duplication.
Known cosmetic residual: the Fatal-partial path splices flat segments into an
indented spine (well-formed; no test covers partial output).
**Companion to** [`STREAMING_POST_DESIGN_2026-07-06.md`](STREAMING_POST_DESIGN_2026-07-06.md),
which covers the *post-processing* half (splitting an already-built DOM) and was
deferred with the condition *"Revisit only if a <64 GB target appears."* That
condition has arrived from the other direction — the **document** grew, not the
machine.

**2026-09-03 residual handoff:** do not reimplement this design. The remaining
structural work is the core-to-post writer/file handoff plus pass-2
per-segment allocation removal in
[`PERFORMANCE_AUDIT_2026-09-03.md`](PERFORMANCE_AUDIT_2026-09-03.md) F3/F4.
The current `ConversionResponse` still holds full core XML as a `String`, and
each pass-2 fragment clones conversion-global font/rule data.
**Witness:** Nasser Abbasi's ODE notes, `flat_index.tex` — **131 MB, 5,050,933
lines**, 84,233 `\subsubsection`, 105,921 `align` environments, 158,338 display
`\[`. Reported 2026-07-28 against rc4; reproduced on stock Linux Mint 22.3 +
TL2026, so not a WSL artifact. Archive:
`https://12000.org/tmp/july_28_2026/RC3_crash_july_28_2026.zip`.

---

## 1. The measured problem

The core stage — digest → build → rewrite — holds the entire document at once.
Peak RSS scales **linearly** with source size, with no leak and no degradation:
marginal cost is flat at ~21 s/MB and **~1.84 GB of RSS per MB of source** across
a 3x span (prefixes of the witness at 3.5 / 5.8 / 8.4 / 9.9 MB).

| source | wall | peak RSS |
|---|---|---|
| 3.5 MB | 59 s | 5.8 GB |
| 5.8 MB | 108 s | 10.4 GB |
| 8.4 MB | 166 s | 15.4 GB |
| 9.9 MB | 200 s | 18.2 GB |

Extrapolated to the full 131 MB: **~241 GB peak, ~46 min of CPU.** The reporter
has 96 GB. **Memory is the binding constraint; time is not.** The conversion was
never going to finish — the box-list guard merely stopped it after ~7 hours, and
(before `387564a5d6`) reported "3 warnings" over a 0-byte file.

### 1.1 Where the memory is

Attributed with `--features dhat-heap` (needs
`--config 'profile.release.strip="none"' --config 'profile.release.debug=1'`, or
dhat captures a single `[root]` frame). On a 1.5 MB content-dominated slice of
the witness: 2.35 GB RSS, 1.0 GB Rust live heap, 18.6 MB core XML, 249,846 DOM
element nodes.

| component | share | note |
|---|---|---|
| **libxml2 DOM + allocator** | **1,346 MB (57 %)** | ~5.7 KB per DOM node; invisible to dhat (libxml2 uses `xmlMalloc`) |
| `SymHashMap<Stored>` property tables | 150 MB | ~195 k tables, 1312 B each |
| `Rc<DigestedData>` boxes | 54 MB | 144 B each |
| `VecDeque<Stored>` State value stack | 43 MB | 157,711 pushes |
| serialization buffers | 58 MB | transient |
| `Rc<Constructor>` per `invoke_token` | 21 MB | 77,902 allocations — *per invocation*, worth a look |
| `Vec<alignment::cell::Cell>::to_vec` | 16 MB | table cells cloned |

**Node count is what math costs us.** `XMTok` 75,974 + `XMApp` 30,074 — the
`XM*` family is ~55 % of all nodes. Plain prose of the same byte size produces
~30x fewer nodes.

### 1.2 Why constant factors cannot close the gap

`perf(memory)` (PR #432) removed the largest *Rust-side* allocation — a deep
`Font` clone per absorbed box, 47 % of peak on plain prose — for a **2.1x** win
there (6360 → 2974 MB on 800k words). On the witness it was worth **3-4 %**
(18876 → 18227 MB), because that content is dominated by the C-side DOM.

Even eliminating **every** Rust allocation leaves 1,346 MB per 1.5 MB ≈ 0.9 GB/MB
→ **118 GB** for the witness. Still over 96, with no margin, and that bound is
unreachable in practice. The remaining levers (property-table sizing, the
per-invocation `Constructor`, cell cloning) sum to well under 1.3x.

**Conclusion: the architecture must change. Optimisation is a complement, not a
substitute.**

---

## 2. Target architecture — fragmented core conversion

Process the document in **fragments**, completing and releasing each before
starting the next, so peak memory is bounded by *fragment* size rather than
*document* size. With disk for intermediates, document size becomes bounded by
disk, not RAM.

The witness supplies its own natural seam: `--splitat=subsubsection`, i.e.
**84,233 fragments**. At the current 1.84 GB/MB a 5 MB fragment peaks ~9 GB.

```
  pass 1 (streaming)                        pass 2 (streaming)
  ─────────────────────                     ──────────────────────
  digest fragment  ─┐                       TextReader over pass-1 XML
  build fragment    │  per fragment:        expand_to_document() per fragment
  local rewrites    │  bounded RAM          apply global rewrites + crossref
  serialize to disk │                       serialize final output
  free              ─┘  + emit label/id     free
                        records to an
                        on-disk index
```

### 2.1 Substrate: `TextReader` (we own it)

`~/git/rust-libxml` — **we maintain this library in full**; changes go in the
fork, then publish + dep bump ([[rust-libxml-owns-libxml-ffi]]). The streaming
reader landed in `86d67597` / `e3323da3` and already exposes what pass 2 needs:

* `TextReader::from_file(path, options)`
* `read()` / `read_next()` / `read_to_next(want)` — advance, optionally to a
  predicate match
* `node_type()` / `is_element()` / `depth()` / `local_name()` / `namespace_uri()`
* **`expand() -> Option<RoNode>`** — borrowed, zero-copy, valid only until the
  next advance
* **`expand_to_document() -> Option<Document>`** — an **owned, mutable**
  `Document` for the current subtree

`xmlTextReaderExpand` is the key primitive: it materialises **only the current
node's subtree** while the rest of the file stays unparsed. That is exactly the
"partially warm DOM" this design needs — a real, queryable, *mutable* DOM for one
fragment at a time.

### 2.2 The hard part: `DefRewrite` on a partial DOM

Rewrites run in the **core** stage (`core_interface.rs:443-470`), starting from
`get_root_element()`. Two properties decide feasibility:

**Encouraging.** `rewrite.rs:413-416` already evaluates
`document.findnodes(xpath, Some(tree))` — XPath **against a subtree context** —
falling back to whole-document only when the subtree match is empty. So the
matching machinery is already subtree-capable; a fragment DOM can be the context.

**The blocker is far smaller than it looks — census 2026-07-29.**
`core_interface.rs:448` calls `document.load_labels_for_rewrite()` before
applying rules, building a document-global `rewrite_labels` map. But only one
rule form actually *consumes* it:

| scope form | data needed | resolvable inside a fragment? |
|---|---|---|
| `label:<name>` | the global `rewrite_labels` map | **no** — needs the pass-1 index |
| `id:<xml:id>` | an `xml:id` anywhere in the document | only if it lies in this fragment |
| everything else (`select`, `xpath`, `regexp`, `match`, `attributes`, `replace`) | the subtree only | **yes** — already `findnodes(xpath, Some(tree))` |

And `scope => 'label:…'` occurs **4 times in the whole repository, every one a
test fixture** (`tests/helpers/scopemacro_src.rs`,
`tests/math/simplemath.latexml` x2, `tests/grouping/scopemacro.latexml`). Zero
in `latexml_engine`, `latexml_package`, `latexml_contrib` — and **zero in
upstream Perl's own bindings** (`LaTeXML/lib`). On a real document (a 0.6 MB
slice of the witness) only **7 rules run at all, ~10 ms total**, none scoped.

So the production rewrite corpus is **essentially entirely subtree-local**: pass
1 can apply nearly every rule inline per fragment, and only the rare
`label:`/`id:` scope need defer. Two caveats that must not be waved away:

* A `label:` scope whose target is missing compiles to `Ignore`
  (`rewrite.rs:285-289`) rather than erroring — so a fragment that cannot
  resolve one would **silently drop the rule**. Streaming needs a real deferral
  path, not a "rare enough to skip" argument; silence is exactly how a corpus
  regression hides.
* `id:` scope walks `findnodes("descendant-or-self::*")` and filters Rust-side,
  because `@xml:id` XPath fails in rust-libxml (flagged at the site as an L2
  workaround). That is O(document) per rule, and an inefficiency worth fixing in
  our own fork independently of streaming.

Hence the two-pass shape: pass 1 emits a **label/id index** to disk; pass 2
loads that index (or queries it) and applies the rules that need it. Rules that
are provably subtree-local can run in pass 1 and never touch disk. Classifying
rules into local vs global is the first real design task, and it needs
enumerating what our actual `\lxDeclare` / `DefMathRewrite` corpus uses —
`select_count` and scope resolution are the suspicious ones.

### 2.3 Cross-reference resolution out of RAM

Same treatment: `\label`/`\ref`, TOC, bibliography keys and the `ids` map become
an **on-disk index** (a sorted side file, or SQLite if we want range queries)
written in pass 1 and read in pass 2. The post half already has an O(n) CrossRef
(`4ec2587993`, 42 min 50 s → 2 min 18 s on the 614 MB witness) — the data
structure is fine, it is the *residency* that must change.

---

## 3a. The user-facing interface (settled 2026-07-30)

**`--max-memory <MiB>` is the budget and the only memory number.** Everything
else derives from it, so the two cannot contradict each other:

| derived quantity | rule | why that number |
|---|---|---|
| cooperative fuse | 75 % of the budget | graceful Fatal with partial output before the hard watchdog |
| spill watermark | a third of the fuse | a HALF steadied pass 1 ~5 GB under the fuse on the 131 MB witness and the bookkeeping creep then walked it in and died |
| fragment budget | budget ÷ 8, in boxes | leaves room for the DOM built from those boxes (~1.4x) plus creep |
| soft-yield floor | 1024 boxes before the RSS-triggered yield may fire (`stomach.rs::soft_yield_min_boxes`, calibration env `LATEXML_SOFT_YIELD_MIN_BOXES`) | the RSS trigger is a LEVEL test with no hysteresis: a document whose resident floor sits above the watermark latches it on and yields at EVERY seam — 24,051,712 yields / 459k segments averaging 5.5 KB on the witness, vs 1,507 / 6,050 with the floor. Guard `115_soft_yield_floor` (red-tested) |
| floor waiver | floor ignored at RSS ≥ watermark + (fuse − watermark)/2 | the RSS branch exists for content whose per-box cost dwarfs the 2416 B/box the budget assumes; 1024 such boxes are unbounded, so under real pressure per-seam yielding returns. Guard `soft_yield_floor_waiver_boundaries` |
| pass-2 segment chunk | watermark ÷ 72, clamped 4-32 MB (calibration env `LATEXML_SEGMENT_CHUNK_MIB`) | measured serialized-to-DOM expansion, so pass 2 respects the budget pass 1 did. NOTE: with the floor at 1024 the ceiling never binds (segments run ~0.4 KB × floor); the 32 MB MAX's "re-parse dominates" claim is unsubstantiated — the pass-2 tail is linear |
| spill serialization | FLAT (`Document::spill_flat`) — no indentation in the intermediate | 51.2 % of segment bytes were indentation+newlines: generated, written, re-parsed into ~40 M text nodes, deleted (`unlink` = orphaned, never freed), regenerated. Output formatting untouched — pass-2 fragments re-serialize normally. Removing it cut the witness 44:56 → 32:56 |

Two escapes, no third memory flag: **`--streaming`** forces fragmentation,
**`--streaming=false`** forces the eager path (before this, auto-activation
could not be turned off at all). A `--spill-at` watermark flag was considered
and **rejected**: lowering `--max-memory` already moves the watermark and the
ceiling together, preserving the validated ratio, whereas an independent
watermark could be set above the fuse — a run that Fatals before it ever
spills.

**`--max-memory=0`** lifts the death ceiling only. Spilling still engages,
judged against the ceiling that would have been derived and with a watermark
taken from physical RAM (RAM ÷ 8): *"do not kill me"* is not *"let the machine
run out"*. Before this the arithmetic degenerated — `projected > 0` always
fired and `0 / 8 / 2416` floored to a ONE-BOX budget, i.e. a spill after every
box, with no watermark at all.

**Activation compares against the fuse, and estimates the DOCUMENT.** Against
the ceiling there was a band (0.75-1.0x) judged "fits" that the fuse then
killed — 8.1-10.8 MB of source on a 16 GB machine. And the estimate read the
main file's length, so a 2 KB `index.tex` that `\input`s a thousand chapters
projected as 2 KB; it now sums the source tree's `.tex`/`.ltx`/`.bbl` bytes
when the main file names an inclusion command — gated on that command so a
self-contained paper among unused alternates keeps the eager path. Guards:
`streaming_activation_tests` in `bin/latexml_oxide.rs`. Known limitation: an
inclusion assembled by macro expansion names no literal command and needs an
explicit `--streaming`.

**Why the projection survives at all**, rather than arming the fragmented
driver for every document: measured on 16 stratified sandbox papers (three
interleaved rounds each), always-on is FREE in time — 16/16 byte-identical,
median wall -1.3 % (p90 +7.5 %, inside per-paper spread), peak RSS +0.2 % —
but the interleaved driver differs from eager in root-hook ordering,
per-step resource folding and deferred frontmatter. That difference is
byte-identity-tested across the sweep suites, not true by construction, so
ordinary documents keep the eager path (user ruling 2026-07-30).

## 3. Memory-ceiling policy (user requirement, 2026-07-29)

Build XML in RAM up to a ceiling, then spill to disk:

* **Ceiling = min(64 GiB, HALF of machine RAM)**, floored at 2048 MiB
  (`watchdog::default_ceiling_mib`). The rule was 90 % until 2026-07-30, which
  is laptop-hostile once you follow it through: the cooperative fuse rides at
  75 % of the ceiling, so a 16 GB machine let one conversion reach **10.8 GiB**
  before complaining — long after the user's session started swapping. Half of
  RAM is the 16 GB-baseline design point (8 GiB ceiling / 6 GiB fuse / 2 GiB
  spill watermark) and lands a 96 GB host on **48 GiB**, the ceiling under
  which the 131 MB witness was measured to convert (28.1 GB peak).
  Machine RAM comes from `sysconf(_SC_PHYS_PAGES)` (Windows:
  `GlobalMemoryStatusEx`). **Known gap: that probe reports the HOST's RAM and is
  blind to cgroup limits**, so a memory-limited container picks a ceiling far
  above its real budget and is OOM-killed instead of getting the graceful
  Fatal — task #157.
* **Before spilling, verify disk headroom.** If neither RAM nor disk suffices,
  raise a `Fatal` that *names the shortfall and the requirement* — the user can
  add swap or free disk, but only if we tell them how much. A silent OOM kill is
  the current behaviour and is unacceptable.
* Swap is a fallback, not a plan: this host has 258 GB RAM but only 8 GB swap, so
  "let it swap" would not have saved the witness.

Related bug, independent of this design — **FIXED** (task #140, merged): the
Build phase had no cooperative memory guard, so `--max-memory` on a large
document produced a hard watchdog abort (exit 137, no output, no `Fatal:` line)
rather than a graceful one. `document.rs` now checks alongside the three
`stomach.rs` digestion sites.

---

## 4. Staged plan

| stage | deliverable | gate |
|---|---|---|
| **S0** | Machine-aware ceiling + swap/disk check + `Fatal` naming the shortfall | unit tests on the derivation; no behaviour change under an explicit `--max-memory` |
| **S1** | Cooperative memory guard inside the Build/absorb loop | a large document degrades to a graceful `Fatal` with partial output instead of exit 137 |
| **S2** | ~~Classify rewrite rules local vs global~~ **done** (census above). Remaining: route the rare `label:`/`id:` scope through the pass-1 index instead of silently `Ignore`-ing an unresolved one | full suite green; `\lxDeclare` corpus unchanged in output |
| **S3** | Pass-1 fragmented digest→build→serialize with an on-disk label/id index | byte-identical output vs the eager path on the existing test corpus |
| **S4** | Pass-2 `TextReader` + `expand_to_document` for global rewrites and crossref | ditto, plus the 614 MB `index.xml` witness |
| **S5** | Retire the eager path, or keep it for small documents behind a size threshold | measured crossover point |

**Parity gate throughout** (canvas-triage golden rules): output must be
byte-identical to the eager path on the whole test corpus. A streaming rewrite
that silently drops a rule is exactly the kind of regression that greens a test
suite and corrupts a corpus.

**Settled dead-end — dropping the digested tree at the Build boundary.**
`core_interface.rs` takes `digested: Digested` by value and last reads it at the
`absorb` call, so the whole graph stays alive through Rewriting, Math Parsing and
Finalizing; adding `drop(digested)` right after Build looks like a free
multi-hundred-MB win. **It is not: measured 2026-07-29, interleaved A/B, release
profile, 4.5 MB corpus — 6078/6073 MB baseline vs 6089/6080 MB with the drop,
i.e. noise.** The reason is the whole point of S3: **peak RSS is reached _inside_
Build**, so nothing done _after_ Build can lower it. Fragmenting has to happen
within the digest→build loop, not around it. Do not re-attempt the outer drop.

---

## 5. Open questions

1. ~~Which rewrite rules are genuinely global?~~ **ANSWERED 2026-07-29** — see
   §2.2. Only `scope => 'label:'`/`'id:'`, and `label:` appears solely in test
   fixtures. S2 is therefore far cheaper than budgeted, and S3 can apply rules
   inline in pass 1 for the whole production corpus.
2. **Fragment boundary when the user did not ask for `--splitat`.** A document
   with no natural seam still needs bounding — section? A byte budget with a
   safe cut point? What happens to an alignment or math environment straddling
   the boundary?
3. ~~**Is the `Rc<Constructor>` per `invoke_token` a real defect?**~~ **ANSWERED
   2026-07-29, and the premise was wrong.** The *lookup* never allocated:
   `Stored::Constructor` is already `Rc<Constructor>` (`common/store.rs`), so
   `lookup_digestable_definition`'s `front.clone()` is a refcount bump. The
   allocation was one level down, in `Constructor::invoke_primitive`, which built
   the Whatsit's definition back-reference as `Rc::new(self.clone())` — a
   **retained** deep clone (Whatsits live for the whole Build), 1:1 with
   invocations, plus a placeholder `Rc<Expandable>` from `..Whatsit::default()`
   that was overwritten immediately. Both fixed by routing the invoker's existing
   handle through `Constructor::invoke_primitive_shared`. Measured: peak RSS
   6074/6081 → 6041/6044 MB on a 4.5 MB math+prose corpus, wall unchanged,
   output byte-identical. **Real but small — ~0.6%**, so it is a tidy-up, not a
   lever on the 131 MB problem.

   The same dhat run reorders the priorities for anyone optimizing constant
   factors here (debug profile, math-dense fixture, so treat as shape not
   absolute):
   * `State::assign_internal`'s per-symbol `VecDeque<Stored>` growth is the
     single largest byte consumer (52.2 MB / 181k blocks), and the State's own
     top-level table adds 64.5 MB in 31 `with_capacity`/rehash blocks — together
     ~26% of all bytes, from the binding store alone.
   * The math parser (marpa ASF + `translate_node`) is 28.7% of blocks, 25.5% of
     bytes, dominated by one 56-byte `Rc<Node<ByteToken>>` per parse node.
   * The libxml/document layer is **block-heavy, byte-light** — ~44% of blocks
     for ~8.6% of bytes — almost entirely 4–7 byte `String` copies out of
     `Node::get_name` / `attr_node_value` / `get_properties`, i.e. FFI round
     trips that re-copy tag and attribute names on every query. Caching qnames
     would cut allocation *count*, not peak bytes.
   * Caveat: dhat's stack depth cap (24) truncated the two largest block sites
     (383k blocks of `format!`, 62k of `HashMap<String,String>::insert`) to no
     project frame, so their callers are unattributed. Re-profile with a deeper
     backtrace before chasing them.
4. **Can the libxml2 DOM be made cheaper per node**, or is 5.7 KB/node simply
   what a `XMTok`-dense tree costs? If the latter, fragment size is bounded by
   node count rather than byte count, and the fragment policy must reflect that.
5. **Does `expand_to_document`'s copy cost dominate** for fragments with many
   small nodes? `expand()` is zero-copy but read-only; rewrites need mutability.

---

## 6. Reproduction

```bash
# harness: bytes of RAM per source word, seconds per run
#   (plain prose isolates the per-box cost from the per-node cost)
python3 -c "…"  # see the prefix builders in this doc's history
/usr/bin/time -f "%e %M" latexml_oxide --timeout=0 --max-memory=0 \
  --dest=out.xml --nocomments w800000.tex

# heap attribution (symbols are required — release strips them)
cargo build --release --features dhat-heap --bin latexml_oxide \
  --config 'profile.release.strip="none"' --config 'profile.release.debug=1'
```

Prefixes of the witness are cut at `\section` boundaries (1,735 of them, spread
from line 2,325 to 5,048,704) so each run is minutes rather than the ~7 hours a
full run costs. Note the front **1.65 M lines** are a single enormous lookup
table — `\hline` rows whose cells hold `minipage` + `\[\begin{array}…\]` — which
is why the witness accumulates 1.6 M undrained boxes during digestion where
ordinary paragraphs flush continuously.
