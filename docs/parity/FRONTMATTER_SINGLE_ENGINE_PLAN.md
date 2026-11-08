# Frontmatter: one ownership engine, one placement order (living plan)

**Why.** The user's direction (2026-11-01): "keep evolving our design of the Frontmatter API until it can accommodate
all variations used in arXiv. Prioritize refactoring to a better model of a single ownership engine, unified machinery
and API, unified placement order inside the API." This decides item 9 of
[`FRONTMATTER_ARCHITECTURE_AUDIT_2026-10-28.md`](FRONTMATTER_ARCHITECTURE_AUDIT_2026-10-28.md). The plan below comes
from a read-only design study (2026-11-01). Its stages are the frontmatter batches after bundle int126a. Per-shape
findings become their acceptance witnesses.

## The path: the old engine removed before the merge (user 2026-11-06)

**The goal (user 2026-11-06): "continue, and remove the old engine before the merge".** One engine, the other's code deleted, for simplicity and compile time; the bar is not the only measure of this work. **It completes [`FRONTMATTER_DIGESTED_REFACTOR.md`](FRONTMATTER_DIGESTED_REFACTOR.md)'s Stages 7-8: every decision reads the observer's events or the digested XML, never token or source spelling; tokens are used only as the content an event's span covers.** This section supersedes the Stages table below where they differ. Basis: `agent_reports/2026-11-06_engine_rebaseline_report.md`, `2026-11-06_engine_removal_report.md`: all of dev3 with int139c, observer on and off, devh10k as a less-tuned control, the frozen scorer (`fm_scorer/FROZEN.md`).

**The end state in Perl's terms, and the test for every batch (user 2026-11-06).** The user: "stay close to the perl latexml approach to architecture and data management, so that we can maintain and vet the code later. We want to be principled in the single engine for frontmatter, so I am not asking for fewer features than necessary for optimal results - just for compact and idiomatic design that also fits the LaTeXML paradigm."

| Perl LaTeXML (`Base_Utility.pool.ltxml`) | The end state |
|---|---|
| A binding's macros call `\lx@add@*` with tokens and keys | The same |
| `\lx@add@authors` (:687) decides a block's parts with `SplitTokens`, at the call, and emits `\lx@add@author`-style calls | **The one added step, at the flush (`place.rs:409`), as digested material doesn't exist at the call:** the macro queues the block, which is rehearsed, its marks and lines are read from what digestion made, and the same calls are emitted |
| `queueFrontMatter` (:272), `frontmatter_raw` | The same: `queue_front_matter` |
| `digestFrontMatter` (:799), `frontmatter`, `num_<tag>` | The same: `digest_front_matter` |
| `insertFrontMatter` (:824) | The same: `insert_frontmatter` |
| `relocateAnnotations` (:880), labels link annotations to owners | The same, with named passes of its kind after it |

**The questions every frontmatter batch and review answers:** (1) What is this step's Perl counterpart, and if it has none, is it part of the one added step? (2) Does its data live where Perl keeps it, in State values, API keys, tokens before digestion and boxes after, or does it add a parallel store (a thread-local, a private key, a struct mirroring a queue entry) that needs a stated reason? (3) Could a LaTeXML maintainer vet it by reading the Perl beside it? **Features stay:** no correct result is given up for size, and a rule the corpus needs lives in the added step, in one place. **The size can't match Perl's, the architecture can:** Perl's macro is about 60 lines because it gets the hard papers wrong (3 held papers, 25 creators: Perl 5 right, the old reader 23). So the added step is the first target for compaction: at the flip `observer/` keeps about 12k code lines (the resolver 6.7k, events 2.3k, ledger, marks, rehearsal), and the shadow's 0.8k and the ledger's parallel records are to justify themselves. **Today's departures to retire:** the old reader's 15.4k lines in place of that one macro, the writer's 5.9k, the private keys in the table below, 26 `thread_local`/`static` matches in `frontmatter/`, `author_block/` and `defs/frontmatter*` (not yet read one by one).

**Route D (user 2026-11-06, replaces the waves).** The user: "I am growing scared of the complexity of the refactor", and chose, on a second opinion's assessment of four routes (`agent_reports/2026-11-06_routes_report.md`): one engine by one switch for every paper with an author block, an experiment first, and both engines fenced if it fails. Rejected: the waves (20 batches, 30-40 M tokens, three live write paths across 14 batches, for a problem of mixed blocks that 1 of 9,781 papers has); the DOM writer as the only writer (it depends on the old reader's name partition, copied items, template attributes, owner map and name ink; under Perl's reader 10.5 % of 7,669 papers shred past what it can rejoin). **Where a line below conflicts with this paragraph or its table, this one stands; Z numbers stay as stable names.**

| Step | Mechanism | Gate |
|---|---|---|
| **S0** | **The experiment: a scratch branch on `pk/Z3` after Z3r2, never merged, about 150-300 lines behind an env switch.** At `place.rs:409`, replace block-made creators and contacts in `commands` with calls built from `Resolved`, for every family; about 1,000 stratified dev3 papers plus the witnesses. Variant 1: the reader still runs at call time. Variant 2: one whole-block queue entry per call, to count the papers whose API keys change, which prices the provisional queue. Also count how many of the 3,417 tests with creator XML move under the switch: they move at once at the flip | Real emit within 0.1 / 0.1 of the counterfactual on the same papers; XML outside the creators, ids masked, identical on at least 98 % of papers where the reading and the output agree; E and F don't rise. **A failure means route C: merge with both engines fenced, revisit after the merge** |
| | **Result (2026-11-07, `agent_reports/2026-11-07_S0_report.md`, scripts `kit/emit/`):** real emit within the gate: dev3 -0.02 / +0.09, devh10k +0.13 / +0.14. XML outside creators 834 of 834, 188 of 189. F 0; one new Error on one paper per sample, the honest `creator_not_a_name` on "on behalf of…" (1201.5726, 1301.6619), Z14's. **What the score can't see, which is the real size of Z4:** creator XML identical on only 52.8 % of agreeing papers (`sup` 941 → 518, `Math` 80 → 65, `@fontsize` 501 → 34; 211 items and 107 names from plain text); 805 of 3,417 tests move; 852 emitter lines + 941 reused writer lines. Scaled to all dev3: 98.31 per creator, under the floor before repairs. Passed: the route stands. | |
| **Z3** | **Done, in bundle int143b.** As below, with the rehearsal's read budget | Its review |
| **Z4** | **Two batches.** **Z4a:** the emitter for real behind its switch, faithful in markup: a name or contact is the tokens its span covers, cut at digested positions, never by spelling (no `past_accent`, `spelled` and the like from S0). **Z4b:** the whole-block queue reaches it: it answers the queue's readers (`frontmatter_fields.rs:561,568`, `affiliations.rs:1328,1427`, `sv_support_sty.rs:119,159`, 9 sites in `frontmatter_stores.rs`; 540 of 9,781 dumps carry keys computed that way or by the Cut) and repeat flushes (458 papers, 4.7 %) A cut's environment extents come from the digested list as `formula_extent`'s do, not from `environment_step`'s token spelling (Z4a, OD #980; re-review of Z3, `content.rs`: a `\let` copy and a macro pair go unseen, `\detokenize{\begin}` counts). **`role=speaker`, a creator call's keyvals on the Name it prints, done in F1** (Z14r2's finding: `writer/emit.rs` wrote `role=author` for a PoS `\speaker`, 108 PoS/JHEP papers in 20k; 0910.5686, 1007.2355, 1111.2428, 1111.6944). | Output byte-identical with the switch off. **Added 2026-11-07 (not yet ruled by the user; a stricter gate, so a safe default):** with it on, creator XML identical, ids masked, on at least 85 % of agreeing papers, `Math` not fewer, each other tag within 5 % or explained |
| **Z14** | PoS `appends` (R9) and the repairs: hep-ph0311131 (33 creators), 2603.28168 (16), 1412.5940 (13), 62 of the 126 that declined and held papers lose on dev3. **Done (`SYNC_STATUS.md`): at the flip dev3 99.02 / 98.61, devh10k 98.54 / 97.96, net +31 and +61; R9 by no `appends`. Z14r2 (review revZ14's blockers 2310.19837, 1009.2396, 2003.02512, each back to int143b's output or better): dev3 99.02 / 98.61 unchanged (+100 net without hep-ph0311131, the role heading's one paper), devh10k 98.56 / 97.99, net +63. Z14r3 (the bundle's full A/B: 1712.07118, 2203.17007, astro-ph0007423, hep-ph0108215, 1006.4832, 1605.02212 worse, each back to int143b's output or better; 28k A/B 0 worse): dev3 99.02 / 98.61, devh10k 98.56 / 97.99, as Z14r2** | The output moves only through the writer's gain (reading repairs change what the writer writes); the counterfactual moves |
| **Z17** | Held events, overflow, Perl's fallback; **the switch on, the old reader no longer decides** | **A bundle of its own (user 2026-11-07):** nothing else in it changes output, so its A/B, thousands of papers, is the switch alone; every other bundle keeps its 28k A/B, before and after the flip. **The flip's floor (user 2026-11-06):** dev3 at or above 98.5 / 98.2, devh10k net creators at or above 0, every worse paper read against its PDF. The floor allows the flip; the repairs' target stays today's 98.83 / 98.55 or better (user: not "fewer features than necessary for optimal results"). Projected 98.65 / 98.29 after the repairs and devh about 98.34 / 97.73 (before them, dev3 98.37 / 98.22) |
| **The safety net (user 2026-11-07: "Precision and recall are both crucial for great semantic markup of well-segmented and coordinated frontmatter annotation pieces")** | A check between the digested events and the calls the emitter writes, answered at the smallest unit that fails: a stray ink at the creator or contact whose span holds it, else the block's unclaimed text; a failed entry per entry; the whole-block creator only when the names fail. Checks: words both ways (lost; written more often than printed); each separator the events show (`Sep`, `Break`) a creator boundary or explained by a stated rule; marks and labelled contacts coordinated (no mark without its contact, no labelled contact no mark reaches). Perl's counterpart: `\lx@add@authors`' `{tabular}` test (`Base_Utility.pool.ltxml:693`), widened from "the block holds a tabular" to "the reading can't account for what TeX printed" | Measured, not assumed: per check, a confusion table against the reading written with the net off, frozen scorer, dev3 and devh10k (fired and helped / fired and hurt / fired, no change / missed while wrong / right). Z4br2: words both ways, the graded response, the misses classed by the check that would catch them; the separator and mark checks a batch of their own before the flip, sized by those misses. **The bar (user 2026-11-07, replacing that day's 98 % precision and recall: "yes, replace it with the three, keep 98 % on output"):** (1) *the output stays at 98 %*, per creator and per paper, on every prepared set (dev20k, dev3, devh10k, longtail3k), judged against the PDF-labelled truth set with intervals reported, the holdout v4 read once at the end; (2) *content is an invariant*: every word TeX printed in a block is in the output exactly once or counted in a Warning, checked deterministically on every paper by the output-side pass (no scorer), the scorer's lost-word counts a cross-check; (3) *no harm, a hard gate*: no paper the net fires on comes out worse, on every set, each firing read; (4) *structural self-knowledge* (segmentation, ownership) is a reported diagnostic, detection per creator with pooled intervals, not a gate. Why: a per-paper precision and recall at 1.8 % prevalence, judged by a scorer with its own limits, needs at least 149 wrong papers with no miss to show 98 % (236 with one), and moves as the reading improves. First measure (Z4br2, `=2`, the kit samples of dev3 / devh10k): precision 2 / 2, recall 2 / 47 and 0 / 36. **N1 (full sets, `=2`):** precision 30.9 / 42.4 / 42.1 / 38.8 %, recall 11.8 / 12.4 / 7.1 / 11.8 % (dev3, devh10k, longtail3k, dev20k), no firing hurt; of 80 misses read, 53 go wrong after the emit (the Kind step keeping a shared entry's addresses for every holder, digestion's footnote junk, a binding's lost `\institute` text, a block turned into no creator, marks on API institutes), 16 are the reading's own errors, 11 scorer limits. **So the net also checks the placed output:** one pass after `insertFrontMatter` / `relocateAnnotations`, the written creators' words per block and duplicate addresses within and across creators against the rehearsal's printed words; linear; it counts in a Warning, since placement is over (revN1; 2607.00684, 1808.10103 by hand). The next net batch after N1r2 **N1r2 (review revN1):** a note's written addresses cut out by position, a lost name a name failure (on a names line, or spelled by its line's or column's address), the reading and its checks made once while the ledger reads the same: dev3 10k 37.2 % (16 / 43) / 11.1 %, devh10k 47.1 % (24 / 51) / 10.7 %, longtail3k 53.3 % (8 / 15) / 7.1 %; fired and hurt 0, helped 7 / 15 / 1 (unchanged: the scorer decides no creator of a block written whole or kept with a note); dev3's loss alarms 31 → 19 (11 false gone; the one true, 2303.02975, a `\thanks` call written at the first flush counted lost at a later one, which a flush with nothing new now skips; with the ledger changed in between the count stands). **N1r3 (review revN1r2):** the reading and its checks on the ledger at its generation, none outliving its conversion; a call written once never counted lost later (devh's 1401.1489 false loss alarm gone); a lost name ranked against the written names (1908.02923 written whole); a line's other inks to its address's author (2203.10777, 1210.0352: no nameless creator): dev3 10k 37.2 % (16 / 43) / 11.1 %, devh10k 48.0 % (24 / 50) / 10.7 %; fired and hurt 0, helped 7 / 15. A diagnostic now, the bar the output's (user 2026-11-07); fired-and-hurt the gate. **N2 (bar (2)'s check, OD #980):** the content check on the placed output, the last step of `insert_frontmatter` under `=2`, counts and never answers; `=2` output byte-identical to int146a's (1,317 kit papers), off identical (E, F, W equal). Papers with a counted loss / excess, `=2`, against the pre-emit check's: dev3 10k 68 / 155 (23 / 5), devh10k 104 / 193 (36 / 3). Read, 20 of each per set: excess real 12 and 11 of 20 (an address a note keeps and the emit writes as a contact, an ORCID link written in the name and as a contact, 2607.00684's twice-held address), loss real 8 and 3 to 8 of 20; the false alarms are the reading's own text (accents a combining mark parts, `Boˇcní`; `--` against `–`; the rehearsal's command names in an ink, `lx@fm@obs@…`; a class's `\and` printing "and", which the rehearsal binds to an event printing nothing), a print counted twice where the page prints it twice and the output once, and address forms one side reads and the other doesn't. Silent words (`kit/emit/words.py`, kit lists) 139 → 130: what's left is marks' text (2204.09614's `24A`), kernel labels, and prints no ledger entry records (2005.05034's `zhanl@ihep.ac.cn`; SolarPhysics' raw `\author`, 1808.10103, 1306.3180, no block at all). Cost: under 1 % on the audit's six (the pass alone ≤ 0.91 %, 1507.02068) |
| **Strict porting (user 2026-11-08: "Adopt strict porting", after "are we refactoring strictly the algorithms in the old engine? I continue to be surprised at new kinds of failures")** | A readiness port translates the old engine's decision procedure; it doesn't re-derive one from witnesses. Per decision of the structure model (cut, kind, group, owner; first the address procedure E1 needs), a read-only **rule table**: each old rule's exact condition, its precedence (what runs first, what it overrides), `file:line`, its witnesses and tests, and the digested-event counterpart of its condition or **missing fact**. The port translates the table onto the reading in the same order; a departure is listed and ruled (main session or user), never invented. Each port also diffs **the decision** of both engines on the dev sets (both run on the same paper), every disagreement a ruled improvement or a bug; its review checks the port against the table line by line. Witness-driven re-derivation is what produced new failure kinds (RP2 took the routing test `starts_with_affiliation_mark` as a kind decision: 1610.05419; RP1 built a 343-line letter-alignment cutter instead of the old DOM cut `writer/apply.rs:914-1011`: 4 dev papers worse, the emitter past 2,000 lines). RP1 (`pk/P1` 645e93227c) stays unmerged as a draft; E1 is redone from its table. **Translation is the method, correctness the goal (user 2026-11-08: "we should be careful not to discard any improvements we made -- correctness is really the final goal, we should simply accelerate our work via first translating the original approaches in our refactored setup"):** a row where the new engine already differs and is better is marked *keep (improvement)*, never reverted to the old rule; every port keeps the 152 papers `=2` already does better (`scratch_main/readiness/better_at_2.tsv`: 135 by the scorer, 17 RD1's scorer-wrong) at least as good, and E1's redo keeps RP1's 28 fixed witnesses (`readiness/P1_fixed_must_keep.txt`), reusing RP1's tested pieces where they match a row. | each port: its table, every row mapped or ruled; the per-decision diff read |
| **`author_block/`'s survivors (user 2026-11-08: "stay focused on having only one engine for frontmatter; keep our high bar for quality over arXiv and require no regressions from our 98% goal")** | `author_block/mod.rs` glob-re-exports its submodules and `base_utilities/mod.rs` re-exports it all (`pub use author_block::*`), so a frontmatter file's `use super::*` reaches every name the old reader exports: the fence `perfect_kernel_batche0::the_one_engine_reaches_the_old_reader_only_through_its_survivors` counts **names** (a call, a path, a `use` list, a constant or type), **140 (file, name) pairs over `frontmatter/`, 51 pairs and 41 names in the new engine (`observer/`)** at int149a — not the 11 explicit `author_block::` paths an earlier count saw (main session's miss, 2026-11-08, found by RT1). A debt each: it may only fall; a new one fails, its decision belongs on the digested events in `observer/resolver/`. The new engine's 41, by destination at Z18: **text rules on printed ink → `frontmatter/classify.rs`, moved unchanged** (~26: `is_email_label`, `is_listed_local_part`, `is_local_char`, `ends_a_listed_run`, `write_out_unspaced_lists`, `names_a_contact`, `ADDRESS_GROUP_SEPARATORS`, `UNBRACED_LIST_SEPARATORS`, `CUT_LIST`, `is_pattern_address`, `is_entry_word_mailbox`, `lists_local_parts`, `is_name_qualifier`, `has_person_cue`, `opens_a_dedication`, `opens_with_deceased`, `reads_as_affiliation`, `classify_thanks`, `is_membership_part`, `non_name_word`, `name_count`, `reads_as_collaboration`, `reads_as_listed_collaboration`, `collaboration_heads`, `starts_with_affiliation_mark`, `as_printed`); **owner decisions → translated onto the reading (`observer/resolver/`) per the strict-porting row** (~7: `spelled_owners`, `entry_address_owners`, `names_correspondence`, `own_call_keeps`, `NameKeys`, `collaboration_line_framing`, `Framing`); **the emitter's vocabulary (Perl's API calls, marks) → into the emitter** (~7: `document_note_call`, `note_call_end`, `note_body`, `entry_mark`, `mark_label`, `rewrite_symbol_superscripts`, `split_wrapped_affiliation_marks`); **Perl's whole-block fallback** (`whole_block_author`, `Base_Utility.pool.ltxml:693`) → `frontmatter/fallback.rs`. The other 89 pairs are the insert-time passes both engines share (`insert.rs`, `place.rs`, `relocate.rs`, `shared_row/`, `queue*.rs`, `address_lists.rs`): each name they use takes the same destinations, the passes themselves Perl's (`insertFrontMatter`, `relocateAnnotations`) or Z19's. Open before Z18: (a) the transitive closure inside `author_block/` of what `=2` reaches; (b) the rules the reading re-implemented beside the old ones reconciled to one definition. **Order:** the readiness ports (RP1, RP2, … — not "Segmentation is digested"'s P1-P6) translate the old mechanisms with real-paper reach before the flip (the strict-porting row); then the flip; then Z18 moves the survivors and deletes the rest of `author_block/`, Z19 the old writer's passes, and the tests compaction runs. | the fence green; at Z18 `author_block/` gone, byte-identical |
| **Z18, Z19** | Delete what the flip leaves unused (user 2026-11-06, "delete what's unused"): the reader's own closure, at least 4.8k lines, the writer's general DOM operations, about 4.2k, the passes the emitter replaced, 1-2k. The compile measure | Byte-identical; each batch net-deletes |
| **Compaction (user 2026-11-07: "I don't think we need a hard budget for size, but we should aim to end the refactor with deletions of old code and abstractions that compact the new code")** | After Z18-Z19 and the segmentation port: a read-only audit of all frontmatter code for duplication and idiom, then a writer that applies it, byte-identical. No size budget; every bundle row reports the frontmatter line count (non-blank, non-comment: int127a 36,477; int143b 47,435; int145a 50,182). Every review treats a decision on rule strings, a spelled word list deciding a kind, or a flag threaded where a type would carry the meaning, as a finding. **Tests too (user 2026-11-07: "when we delete the old engine and compact the code, we should also do a session compacting tests into unified bundles for better performance"):** the old engine's own tests go with it, guards pinning its behaviour move to the reading's expectations, and the frontmatter guard modules (the `perfect_kernel_batch*` files) are consolidated into unified bundles by concern, each assertion kept, as the 2026-08 `cluster_*` consolidation did; measure nextest wall and CPU before and after; the fixture-sweep binaries stay separate (`114_streaming_*`, `tex_tests!` stubs) | The old engine deleted; the new code's abstractions shared, not repeated; output byte-identical |
| **~~Later, stoppable~~ Before the merge (user 2026-11-07, option C)** | **All token segmentation ported onto the digested reading, after the flip:** the section "Segmentation is digested" below, P1-P6. Replaces "later, stoppable" | Per batch, as the stop rules |

**Dropped:** Z5, Z11, Z12, Z13, Z15, Z16 as separate flips; Z7, Z8, Z9. Papers with no block keep Perl's API path as they are (-118 creators if the reading took them), and the post-writer passes stay as named passes (-61): the waves would have paid to port both. **At the flip about 44k of 53.9k frontmatter lines remain, 38-39k if the later batches finish** (estimates, `author_block/` by static name reach; the old plan reached 34.3k). Removing the reader's three entry points frees only 4,793 of its lines; 10,536 survive through other callers, 6,762 of them through 25 names in core bindings. So `author_block/` stays as a helper library at the flip, and token helpers stay with it: decisions come from the reading, while the digested refactor's "never token spelling" holds for `resolver/` at once and for the helpers as the later batches go.

**Segmentation is digested (user 2026-11-07: option C of the decision page https://claude.ai/artifact/RpPwXXbkusiSSBavqWBfYz, then "adopt that rule and add it to the plan. Stay principled, keeping in mind we need to be capable of handling the full breadth of diversity of the LaTeX ecosystem, as seen in arXiv and beyond").** Every decision about where a name, mark, line, entry, kind or owner is comes from the digested reading, before the merge. Evidence: `agent_reports/2026-11-07_segaudit_report.md` (`author_block/`), `agent_reports/2026-11-07_bindsplit_report.md` (the bindings).

- **The rule. Tokens carry, digestion decides, tokens are emitted.** *Argument passing* stays on tokens: the class delivers separate arguments by separate macro calls (`\IEEEauthorblockN{…}\IEEEauthorblockA{…}`, `\author[short]{long}`, one `\affiliation{…}` per author), and a binding hands each to Perl's API. *Segmentation* is several items inside one argument, and it is cut from the digested events, **even where the separator is the class's own `\and`**: a token split sees the spelling before expansion, so a separator in a macro, a `\let` copy or a class wrapper parts the authors otherwise than TeX (the bindsplit probes om1, ln1, ie1, ep1; pdflatex prints two authors in each).
- **The mechanism exists.** The rehearsal binds the author line's vocabulary to event constructors, `ALWAYS_BOUND` (`frontmatter/observer/rehearsal.rs:312`): `\and`, `\And`, `\AND` → `\lx@fm@obs@and…` (`DefConstructor`, prints nothing, `rehearsal.rs:106`), `\\`, `\newline`, `\linebreak` → a break, `\par`. The walker reads their boxes as `Sep(And)`, `Break` (`events/mod.rs:1120`). This is Perl's `@authorsplits` (`Base_Utility.pool.ltxml:675-680`) applied to what TeX expanded instead of to the spelling. `\author{Ann Able \and Bob Baker}` digests to `Name "Ann Able" [0,8]`, `Sep and [9,10]`, `Name "Bob Baker" [10,19]`; with `\and` inside `\newcommand\aut{…}` the same three events, spans `[0,1]` (the one source token `\aut`).
- **Breadth is the test of a rule.** A rule names a meaning or a digested fact, never a class or a spelling; what is class-specific is the binding's declaration of its own structure (its separators, columns, sections: `Policy::block_breaks`, `BOUND_IN_A_BLOCK`), as Perl's bindings declare theirs. Every separator-like rule is probed on its spellings: literal, macro-held, `\let` copy, `\csname`, a class wrapper, an active character. Every batch is validated beyond what it was tuned on: **the bundle's full A/B (28,354 papers) is the gate, not popZ01**, which missed six real regressions on 2026-11-07; plus the shape census on dev20k and longtail3k, devh10k once, the TeX Live manuals sweep, and holdout v4 once at the end.

| Step | Mechanism | Witnesses / reach |
|---|---|---|
| **P1** | The reading cuts a creator call at its own `Sep(And)` events, as it cuts a numbered institute list (`resolver/api.rs:385-400`, `labelseq`) | ln1 (llncs, the event is there at nest 1, the reading keeps the call whole by `api:creator`), om1, ie1, ep1 |
| **P2** | Bindings stop pre-splitting by spelling: the `\lx@splitting{…}{\and}` family (`inst_support`, aa, `sv_support`, webofc, ptephy), llncs, the IEEEtran grid and wrap, EPTCS cells, `note_and_numbering`, nowfnt pass their block; the 11 gates on a literal `\and` decide from `Sep` events; the segmentation sites go (`ieeetran_cls.rs` `printed_refmarks`, `name_memberships`, pasj00, gretsi, interact's rewrite) | OmniBus ~1,060 dev20k papers, aa 314, llncs 378, IEEEtran 1,282: one class family per batch, each through the full A/B |
| **P3** | The emitter cuts inside a macro's expansion: when a block's events all point at one macro token, it writes each piece from the expansion's tokens the rehearsal has, not from plain text | ar1 (`\author{\aut}`), S0's "a macro's span" items |
| **P4** | `author_block/`'s roots that cut before the reading (segaudit group A) and decide after it (group C), the five with a digested fact first: `queued_author_marks`, `binding_cut_list_attr` with `affiliation_is_marked`, `marker_leads`, `leads_with_mark`, `affiliation_calls` with `class_numbered_institutes` | dev3 papers changed 201, 86, 153, 61, 110 |
| **P5** | The facts that don't exist yet, each before its port: an `\IEEEmembership` grade digested as a contact, a framing rule for "on behalf of", the surname match of `placed_contact_owner` | ie3, col1 |
| **P6** | Delete what each step leaves unused; report the real line count | Each batch net-deletes |

Stays on tokens (argument passing, or a text rule on ink): `email_addresses`, `reads_as_affiliation`, `note_body`, `row_email_owners`, `email_calls`, and each binding's reading of its own separate arguments.

**Stop rules, per batch (user-accepted 2026-11-06):** more than 3 fix rounds or 3 M tokens: stop and report, no fourth round. A round whose findings are all hostile-only: REDs, and ship. Real emit more than 0.1 below the counterfactual: stop. A Fatal added on dev3, or any of the 18 documents up more than 1 % in `instructions:u`: stop. An emitter over 2,000 code lines or 10 named rewrites: stop and reassess. (Counted on the emitter's own code, not the helpers it reuses nor the reading's checks: a trigger to stop and reassess with the user, not a cap; user 2026-11-07. At int145a: 1,730 lines, 9 named rewrites.) After the flip, a batch that doesn't net-delete states why.

**The boundary is Perl's Frontmatter API (user 2026-11-06).** The engine relies as much as possible on the original LaTeXML API, its LaTeX macro dialect, and follows its design: a binding or the emitter speaks `\lx@add@*` and `\lx@annotate@frontmatter` with their keys, entries queue, digest and insert, and labels link annotations to their owners through `relocateAnnotations` (`Base_Utility.pool.ltxml:503-536`, `:736-740`, `:891-900`). The engine reads what those calls printed and decides owners; it answers in the same dialect (`\lx@add@creator`, `\lx@add@contact`, `label=`, `annotations=`), not through a parallel path. **Minor rewriting is fine for hard cases that don't fit the API yet (user 2026-11-06), until the design generalizes to fit them: the aim is a clean architecture, with no special-casing of constructs that should be standard.** So a rewrite is named, small, and for a shape the API can't yet express; a standard construct goes through the general path, and if it can't, the design is what changes. All 38 of Perl's macros are in use. An extension is allowed where it has Perl's shape, a macro with keys feeding the same queue, and is opt-in data; it's the exception, and it stays only with a stated reason. Today there are 16: `\lx@add@authors@append`, `@list`, `@adaptive`, `\lx@add@author@split`, `@keyvals`, `\lx@add@contact@placed`, `@list`, `\lx@add@affiliation@marked`, `@shared`, `\lx@add@store@affiliations`, `@authorblock`, `\lx@add@frontmatter@container`, `\lx@add@creator@rdf`, `\lx@add@aas@email`, `\lx@add@email@metadata`, `\lx@add@date@halved`; and the keys beyond Perl's: 60 pinned rows at Z01 (78 uses), `_grouped`, `_markpiece`, `_share`, `owner=`, `_labelprints`, `_zone`, `_store`, `_net`, `_unsplit`, `_mark`, `_marks`, `_markshown`, `_sequence`, `_unmarked`, `_withsup`, `_cutlist` among them (the API guard, `perfect_kernel_batche0`; Z2 closes its unparsed shapes). Each one retires with the reader or becomes a key on Perl's own call, or stays with its reason recorded; the classification comes at Z4, and Z20-Z22 move the bindings.

**The rehearsal must leave no trace and stay cheap (user 2026-11-06).** Today only papers whose blocks are all a written family's are rehearsed; after the flips every paper is, so the rehearsal becomes part of the default path.
- *Side effects.* The rehearsal rolls back assignments (`AssignmentJournalGuard`), holds diagnostics, mutes output, freezes the API, puts back the field ledger, and since 108b ends no document (`gullet::speculative_digestion`). 108b's review found both failure kinds in a speculative run: `\closing{\end{document}}` ended the real document (fixed), and a looping stored argument took the whole run's timeout (since R1 the title-page check's runs have a read budget, `gullet::read_budget`, guard `perfect_kernel_batchr1::a_looping_stored_argument_is_budgeted`; the rehearsal and the stores' sentinel runs have none yet). **Before Z4:** an isolation guard from a hostile suite (`\label`, `\refstepcounter`, `\global\def`, an undefined macro, an unbalanced `\if`, group or math, `\input` missing or real, `\write18`, `\openout`, a figure, a section, `\end{document}`, `\stop`, `\@@end`, a definition in the block), each with the observer on and off: output outside the creators byte-identical, and E and F equal; a census of state the journal doesn't cover (files, image loads, ids, caches); and one step budget for every speculative digestion (the check's mechanism, `gullet::read_budget`, extended), ending in a counted outcome, never a Fatal the real run wouldn't have.
- *Performance.* From the 2026-10-31 audit: frontmatter is 0.5-0.9 % of fleet CPU, about 1 % of a median paper, 3.5-15.6 % of many-author papers and 41 % at 975 authors, so the tail is what's hot, and a rehearsal costs 0.4-0.7 of the real digestion of the same content. **Every flip batch reports `instructions:u` on the audit's 18 documents against base; at Z19 each must be at or below today's.** The old reader and passes leaving pay for part of it; what the rehearsal adds to median papers is unmeasured so far.
- *Digest once: decided against (measured 2026-11-06, instructions on 48 papers, median and tail; int142a).* **Keep the rehearsal; Z4 isn't gated on it.** An author block can't be digested once: the event constructors print nothing and real meanings leave no box to part at; a group crossing a separator carries over (`\bf Ann \and Bob` turns Bob bold, unlike the per-entry group); `Rehearsal::Skipped` (Fatal, NotUndone) is what allows a fallback, and adding a rollback recreates the rehearsal; spans are per box while the lexer cuts inside text boxes. API content could be read from the real boxes only by splitting the flush (`\thanks`, `\footnote`, `\label` go to the API and leave no box; the rehearsal runs at `place.rs:409` before the real run at `:564`, and the real digestion depends on the emitter's decisions, which turns circular once decisions read events), for about 0.03 % of the fleet. So the isolation guard and a step budget are needed either way. **What the rehearsal costs:** every paper with a block is already rehearsed, as no family declares `written: false`, so the flips add it only to no-block papers: median block paper 0.04 %, no-block paper with API calls +0.14 %, fleet about 0.05 % today and +0.007 % added; many-author papers 1.45-10.7 %. The retiring code is 38.37 % at 975 authors, 5.51 % at 592, 0.12 % on a median paper, so the end state is plausibly faster on the median and clearly on the tail; the emitter's real cost is unmeasured until it exists. **New, not in the 2026-10-31 audit:** `author_block/affiliations.rs:311` `cut_marked_contact` is quadratic, 28.75 % at 975 authors (3.25 G), retiring with the Cut at Z6. `rehearse` still has no read budget: Z4's prerequisite, by `gullet::read_budget`, with a conversion total of its own, as many-author papers rehearse per call.

**The API keys that aren't Perl's, and what each becomes (review revZ2r2, by Perl grep and the keys' comments, `Base_Utility.pool.ltxml`; rows 4, 6, 8, 9 not read call by call).** The guard pins them (`perfect_kernel_batche0`), and the deletion batches work from this table: none of them looks permanent.

| Keys | Perl's? | How it's said, and fate |
|---|---|---|
| `property`, `content`, `xml:lang` | Yes: every option reaches `openElement` (L851-852) | On the Perl list since Z2 |
| `accumulate` | No, but it means Perl's default: `@now` always pushes (L404), replacing is an explicit clear (L576) | A bare call; dies once OD #154's rule (`frontmatter_api.rs:240`) moves to the clear |
| `owner` | No | `sequence` = `labelseq=author` (L489, L409); `first` = `label=author:1`; the flush's "unless proved" dies with the old engine |
| `mark`, `_mark`, `_marks`, `_unmarked`, `_bymark`, `_answers`, `_entry`, `_linemark`, `_linelist`, `_linefirst`, `_linklabel`, `_seqlabel`, `_sequence` | No | `label=`, `annotations=`, `labelseq=` |
| `_share`, `_group`, `_grouped` | No | `annotate=all`, `annotate=N` (L516-520) |
| `_labelprints`, `_markshown`, `_linemarkshown`, `_refmarkshown`, `_textmarks`, `_entrysymbol`, `_withsup`, `_markpiece`, `_kinded` | No key | The mark stays in the content; the keys die |
| `_command_form` | No | Perl's `\lx@add@abstract` (L612) vs `\lx@begin@abstract` (L621) |
| `_store`, `_released_store`, `_net`, `_unsplit`, `_cutlist`, `_queued`, `_late`, `_late_mark`, `_latecall`, `_zone`, `_printedbefore`, `_mergednames`, `_for`, `_source` | No | Die with the stores, the Cut and relocation |
| `_deposit`, `_inserted`, `_titlepage`, `_preamble`, `_paragraph_break` | Not API | Flow state; out of the pin or die |
| `_noautoclose`, `_nowrap`, `_scopebegin`, `_font`, `_xmkey`, `_qname`, `_split_ref` | Not frontmatter | XML attributes, the first five Perl's own (`acmart.cls.ltxml:103`); `_answers` also has a DOM half |

**Where it stands.** The observer on 98.83 / 98.55 %, off 97.83 / 97.35 %. The reading alone, its serialization artifact removed, 97.57 / 97.12 %: **-1.26 / -1.42, -280 creators net**, by class: no-block 0 / -118, declined +28 / -108, post-writer passes not in the reading +1 / -61, held 0 / -18, residual 0 / -4. On devh10k's disagreeing papers, declined is **+90 / -63**, the reading ahead; without no-block and the post-passes devh nets about -19. So most of the gap is two narrow classes. Token-spelling reads (`tokreads`, over-approximate): engine 1,962 (`author_block` 1,149, stores 399, relocate 127, `plan.rs` 116), bindings 556; on the guard's wider shapes at Z01, engine 2,264, `resolver/` 12, bindings 621, and Z2 widens them again and re-pins. Stages 2, 3 and 5 of the digested refactor have not landed.

**The original order, superseded by Route D above and kept for each mechanism's detail and witnesses:** measure, event gaps, content from spans, emitter on the safest families, repairs tied to each family's flip, delete. Every gain or loss in the tables is a counterfactual score until that batch's real emit.

| # | Mechanism | Risk | Output | Witnesses |
|---|---|---|---|---|
| **Z0** (done) | Writer outcomes in `fmshadow`, read-back, fence and pass-list guards, the reading-alone tool with the rdf copy, the family gate table, the token-read check | EASY | byte-identical | 2012.14750, 2201.02056, 2606.18623, 1507.02068 |
| **Z1** (done; `is_font_state` and the span rule's holes, revz01, are Z2's part 1) | Event gaps 1, 2, 6, 8, 9, additive, checked with `--re-resolve` | MED, low | byte-identical | 2105.07958, 0804.4669 |
| **Z2** (done) | Revz01's findings, gaps 3, 4, 5, 7, 10, then the writer's regressions on events ("de Naurois", an IEEE `1st` name, A*STAR) | MED, low | witnesses and reach | 2606.16069, 1702.08280, 2401.15006, 2609.01831 |
| **Z3** | The digested refactor's Stage 3: `content_of(span)` replaces the walk-backs (`piece_tokens` `plan.rs:379`, `mark_formula_start` `:1298`) | MED | 88c's REDs | 1410.8534, astro-ph9807043 |
| **Z4** | **The emitter at `place.rs:409`, wave 1.** Flipped papers skip `author_macros.rs:44,88,100,108,117`; output outside `ltx:creator` byte-identical on them | HARD, high | 132 papers | a wave 1 sample |
| **Z5** | Wave 2 | MED | 478 papers | 2206.00684, 2608.25509 |
| **Z6** | Cut and Kind on events (`affiliations.rs:311,:459`, `relocate.rs:1195-1642`) | HARD, high | reach of 99a, 104c, 107e | 1810.01941, 2502.01754, hep-ex0106079 |
| **Z7** | Classifiers on text; `text_tokens` gone | EASY-MED | byte-identical | `--re-resolve` |
| **Z8** | E-mail lists on events: address lines, lists, relink (the post-passes P1-P4) | MED-HARD | papers read differently | 1106.2694, 2009.14655, gr-qc0207058 |
| **Z9** | Kinds and notes: legend, dedication, link, grade, role notes, marks (P5-P10) | MED | same | 2608.22772, 0910.1386, 2511.18438 |
| **Z10** | Stage 5's pre-splitters (`ijcai_email_lines`, `\lx@split@addresses`, revtex's rule, `leading_line_label`); `MARK_SWITCHES` goes | MED | their reach | 1901.07288, 0909.0058 |
| **Z11** | Wave 3, revtex | HARD by reach | 1,752 papers | 1508.05940, nlin0410054, gr-qc0611116 |
| **Z12** | Wave 4 | HARD | 1,451 papers | 1302.1274, 2311.04097 |
| **Z13** | Wave 5: `annotations=`, ORCID | MED | 499 papers | 2508.09752, 2408.15810, 1807.05355 |
| **Z14** | PoS `appends` (R9) and wave 6 | MED-HARD | 712 papers | 1509.01480, 1412.5940, 2107.01943 |
| **Z15** | Wave 7 | HARD | 725 papers | 2305.13191, 1610.05419 |
| **Z16** | Wave 8, with name heads | HARD, high | 2,418 papers | 2606.18623, hep-ph0311131 |
| **Z17** | Held events, overflow, Perl's fallback; **old path off** | MED | under 1 % | 2603.28168, 2012.14750, 2201.02056 |
| **Z18** | **Delete the reader,** move the survivors to `frontmatter/classify.rs` | EASY | byte-identical | gates |
| **Z19** | **Delete DOM operations (but the named hard-case rewrites), old passes, `shared_row`;** the compile measure | EASY | byte-identical | gates |
| **Z20** | Core splits: llncs, aa, sv_support, inst_support, webofc | MED | per binding | 2206.00684, 1912.05868 |
| **Z21-22** | Contrib parsers | MED | per binding | per-class nets |
| **Z23** | The stores' token deciders on events | MED-HARD | store reach | 2607.20672, nlin0310001 |

**The event gaps.** The digested refactor's 1-5, and: 6, unprinted contacts (aastex7's rdf `\email`); 7, mid-line hand-printed labels (100a's RED); 8, a declaration event with a span (`definitions_at_author`, `email_lines.rs:609`), which the emitter must keep; 9, partial events under held errors; 10, llncs `\orcidID` as a contact.

**The waves, dropped by Route D; the figures still size each family** (per paper, on a new `Policy.emits` replacing `written`, `policy.rs:96`; a mixed paper waits for its last family; decline rate does not predict the gap):

| Wave | Families | Papers on dev3 | Dev3 fix / break | Devh fix / break |
|---|---|---:|---:|---:|
| 1 | iopart, jpconf, jheppub, jcappub, jinstpub | 132 | 0 / 0 | 0 / 0 |
| 2 | mn2e, aa, svjour, inst_support, ijcai, acmart, book, report | 478 | +3 / 0 | +16 / -8 |
| 3 | revtex | 1,752 | +3 / -8 | +39 / -6 |
| 4 | ams, elsart, llncs | 1,451 | +2 / -16 | +6 / -7 |
| 5 | no-block | 499 | 0 / -118 | +4 / -119 |
| 6 | undeclared, authblk | 712 | +5 / -47 | +6 / -40 |
| 7 | IEEEtran | 725 | +4 / -56 | +2 / -35 |
| 8 | kernel | 2,418 | +12 / -64 | +30 / -22 |

**The flip gate, replaced by Route D's floor.** A family flips when real emitted output is at or above the then-current base on dev3 (frozen scorer) and net creators are at or above 0 on devh10k, with every worse paper read against the PDF. No code is deleted before wave 8. One holdout v4 read at the last flip, no lower than the pre-flip reading of the same draw.

**Definition of done (each one mechanical):**
1. **By Route D, at the merge:** the old reader decides nothing, and all it leaves unused is gone; `author_block/`'s helpers stay until the later, stoppable batches. **At the end of those:** `author_block/` is gone; old passes and the writer's general DOM operations are gone, leaving only named hard-case rewrites, each with its witness and what would let the API absorb it. What remains: `frontmatter/classify.rs` (text classifiers, the owner chain), the emitter, Perl's `\lx@add@authors` for papers with no events.
2. The final fallback: with events the reading always answers, uncertain text whole; without events (Fatal, NotUndone, JournalUnavailable) Perl's reader, about 60 lines. The cautious split goes. Held papers are repaired, not sent to the fallback (3 such papers, 25 creators: old right 23, Perl 5, reading 5).
3. The token-read check at its floor: about 60 in the engine (rehearsal's table, ledger stamping, the Perl API boundary), `resolver/` 0, decision code elsewhere 0. **It counts segmentation by spelling; argument passing into the API is exempt (user 2026-11-06, the digested refactor's principle 4).** The floor's figure is re-set accordingly when the later batches reach it.
4. The fence holds until then: `\lx@add@` emissions and token reads in `author_block/` can't rise; a new ownership rule lands in `resolver/`.
5. Passes after the emitter may change kind and text, never who holds an item (guard and test-profile invariant).
6. Dev3 D3 not below base at any push before the flip, which the off switch guarantees, and at the flip Route D's floor; the holdout read once.
7. The rehearsal: the isolation guard green, every speculative digestion under the step budget, the 18 documents at or below today's instructions at Z19.
8. The API boundary: every binding and the emitter reach the document through Perl's dialect and the kept extensions, the named hard-case rewrites apart. A guard pins the list of macro names and keys outside Perl's: a new one fails until justified, and each batch that retires one removes it from the list.

**Expected size under the old plan; Route D's is above (estimates, an over-count of what stays):** the engine deletes about 30.9k lines and adds about 3.6k, net about -27k of 144,114 (19 %); contrib about -3.2k. `author_block/` 19,964 → directory gone, about 2,500 moved. **Compile time is unmeasured** and may gain less than the line share; a clean `cargo build -p latexml_engine --timings` on Z19's parent and Z19 settles it.

**The 98 % bar at the merge:** decided after the v4 read (user 2026-11-06); 98 / 98 stays the stated bar, nothing lowered in advance. **The truth set (user 2026-11-06):** the v4 read comes with one: every paper the frozen scorer flags plus about 500 it passes, authors, affiliations and e-mails recorded from the PDF and source alone by two independent agent reads, disagreements to the user; labelling agents never see the converter's output or the verdict, the main session sees only counts. A pilot on dev3 settles the format and cost.

**Open, proceeding on the recommendation unless the user says otherwise (main session 2026-11-06):** the floor's figure, re-set at Z0 from a hand-audited count; stores' reads of class code exempt and counted apart, content lines must move; `queue_digest.rs:661`'s ToString fallback kept only for non-printing operands; R6, R10, R11, R12, R15, R16, R17 as tentatively written, at the batch that makes each real; the flip gate's tolerance zero, replaced by Route D's floor (user 2026-11-06).

## The diagnosis

- **Two ownership engines decide the same things.**
  - The old one: the author-block reader (`base_utilities/author_block/`, ≈14.2k lines) plus the insert-time passes
    (`frontmatter/insert.rs:88-99`, `relocate.rs`, `place.rs`, `shared_row.rs`).
  - The new one: the observer (`frontmatter/observer/`: lexer, ledger, resolver ≈7.7k, writer ≈6.4k).
- **The observer only patches.**
  - Since 97e its writer has the last word for every family. But it rewrites the old reader's DOM after insert
    (`insert.rs:97`), and only where a closed set of operations explains each difference (`writer/plan.rs:57-95`).
  - It can build only two kinds of item from tokens: affiliations and legends (`plan.rs:46-53`).
  - When it declines, the old reader's output stands. So every capability the writer cannot express has to be built
    in the old reader too: 97d's collaborations live in `author_block/collaborations.rs` and again in
    `resolver/lines.rs:533-595`.
- **Rules that exist twice** (old reader / observer):

  | Rule | Old reader | Observer |
  |---|---|---|
  | respell e-mails | `insert.rs:141` | `post.rs:541` |
  | amsart upfront grid | `insert.rs:290` | `post.rs:402` |
  | label matching | `relocate.rs:72-120` | `labels.rs:196` |
  | annotate walk | `defs/frontmatter_api.rs:620-800` | `resolver/api.rs:48-133,600-760` |
  | author order | `place.rs:42,152` | `post.rs:110` |
  | corresponding author | `email_block_calls.rs:251`, `place.rs:206` | `resolver/mod.rs:566`, `post.rs:1649` |
  | shared institute | `shared_row.rs:54-160` | `post.rs:969,1525,1551` |
  | an entry's inner e-mail | `frontmatter_api.rs:530-560` (inherits the entry's label: rev97cd C1) | `post.rs:1616` (the charter's chain) |
  | revtex group rule | `relocate.rs:769-816` | `_grouped` hint |
  | collaborations | `author_block/collaborations.rs` | `lines.rs` + `mod.rs` attach |
  | leading mark as label | `author_macros.rs:127`, `revtex4_support_sty.rs:1006` | `api:leading-mark` |
  | marked names split | `names_split.rs` | `lines.rs:467` |
- **Binding-private splits.**
  - In the core bindings: llncs (`llncs_cls.rs:134-261`), aa (`aa_cls.rs:64-100`, `\lx@add@author@split` reaches
    the collaboration reader one `\and` piece at a time since 103b — a heading over later pieces covers none of them:
    2206.00684), sv_support (`sv_support_sty.rs:79-143`, an inner `\email` inherits
    the piece's label: C1), inst_support, webofc, revtex/aastex `group_or_marked_affiliation` (a binding deciding
    "marks or run" by token tests: F1).
  - Contrib parsers: ptephy, osa_journal, epl, eptcs, aistats, interspeech, gretsi, l_aa, apa7, cpc_hepnp (≈3.9k
    lines).
  - Duplicated primitives: group end ×3, skip spaces ×3, argument readers ×6, spacing tests ×5, top-level splits ×3,
    footnote-symbol tests ×6, leading-mark tests ×3, e-mail line parsers ×6.
- **`annotate=` modes stand in for evidence.** About 95 calls choose a mode: `run` 42, `new` 35, `all` 16, `last`,
  `N`. Only the `@marked` calls and `group_or_marked_affiliation` test marks before the convention; every other call
  applies its convention even where marks decide (the F1 class). Some `new` calls are classes that group:
  quantumarticle.cls:1164-1170 adds each `\affiliation` to every author of `\@authorgroup`, REVTeX's `run`. The
  `annotate={N}` counts in `dispatch.rs:751,1614`, `collaborations.rs:554`, `line_kinds.rs:751` and
  `frontmatter_stores.rs:5856` are the old reader's own decisions encoded as counts.

## The target model

- **Engine.** Rehearsal and lexer → resolver (one placement order) → **emitter at the flush** → Perl's queue, digest,
  insert and `relocateAnnotations`.
  - The writer's DOM operations retire. `writer/dom.rs` stays for the comparator and a read-back check.
  - The old reader is frozen as the fallback producer, for blocks never observed (`observer/ledger.rs:67-80`) and for
    the emitter's declines. In the end it is Perl's `\lx@add@authors` (`Base_Utility.pool.ltxml:687-727`) plus the
    cautious split.
- **API (Perl's calls unchanged).**
  - Reserved owner labels `fm:<k>` ride on Perl's own keys: `annotations={fm:k}` on a creator, `label={fm:k1,fm:k2}`
    on a contact (several owners clone, as in Perl), `fm:block` for the block (OD #159's orphan path). `relocate.rs`
    matches them exactly, as it does `author:N`.
  - An owner table generalizes `owner=sequence` (`frontmatter_api.rs:504-519`, `place.rs:42`).
  - `annotate=` keeps Perl's `''`/N/`all`/`new`, ours `run[:N]`, and gains `sequence` and `share` (replacing
    `owner=sequence` and `_share=authors`). It means step 3 only.
  - Retire `_grouped`, `_markpiece` and `Policy.written`.
  - Institute lists go through Perl's `\lx@add@affiliations[…]` with a `split=` key; one block per binding `\author`
    (`\lx@add@authors@list`).
- **The placement order, for every item** (affiliation, address, e-mail, note, collaboration, ORCID, URL):
  - **0. What the item is.** One item per printed entry; marked lists cut at their marks; addresses taken out
    (OD #938); emptied notes dropped (#916). An annotation inside a name's own ink is that name's candidate.
  - **1. Labels and marks.**
    - The item's labels: `label=`, `labelseq=`, its `\label`, its leading printed mark, a hand-printed leading number
      when names request numbers.
    - The names' requests: shown marks, `annotations=`, `author:N`, `fuzzy:<name>`.
    - Matching: exact → prefix-stripped (Perl `Base_Utility.pool.ltxml:891-900`) → glued-label parting → fuzzy
      surname. Every requester gets a copy.
    - For an e-mail, its own label gives the candidates and spelling picks within them. A label inherited from an
      enclosing entry is not evidence (#917). A labelled item nobody answers goes to step 4, never to a convention.
  - **2. Spelling.** An e-mail or URL spelling exactly one live author as printed; base letters confirm an owner and
    never pick one (#420). A contact opening with one author's whole name. A note naming one creator.
  - **3. The declared convention,** for undecided items only: the binding's `annotate=`, plus the block conventions in
    `Policy`:
    - an unmarked line under the names row is that row's (R12);
    - an entry's only carrier takes its inner e-mail;
    - a sole author takes everything;
    - a collaboration covers its `\and` group since the last one, or a heading covers the names after it.
  - **4. The block,** kept once at block level (#159; the 2026-11-01 boxed-block ruling).
  - **Order within a creator:** print order.
- **What a binding declares:** only what printing cannot tell. That is the rehearsal vocabulary
  (`observer/rehearsal.rs:244-277`), `email_in_place`, `title_marks`, section markers, the `Policy` block conventions,
  and a per-command `annotate=`. Nothing for steps 1, 2 or 4.
- **Perl's shape.** Steps 1 and 3 are Perl's own (`Base_Utility.pool.ltxml:503-536`, `:891-900`, `:736-740`). Step 4's
  keep-on-block is OD #159. These go beyond Perl and need OD entries: step 2, leading marks as labels on every call,
  the requester skip (ruling R1), the `fm:` labels and the emitter (superseding #837's S2b writer), and per-call
  positions.

## The structure model (user-accepted 2026-11-01)

A block is read in four steps, and a fifth writes it. Each residual belongs to the step where it first goes wrong: a cut
error that shows downstream as a wrong owner is a **Cut** residual.

| Step | Question | Today |
|---|---|---|
| **Cut** | Where does a printed piece end? | about 9 readers per signal family: old reader (3A), observer lexer and `lines.rs`, writer `cut`/`repart.rs`, binding pre-splitters, scorer |
| **Kind** | What is the piece: person, affiliation, address, e-mail, note, collaboration, remark, spacer? | inventoried (FRONTMATTER_DIGESTED_REFACTOR.md 3B-3E), not unified; two `reads_as_names` (`author_block/marks.rs:106` on tokens, `observer/resolver/lines.rs:381` on text), three collaboration word lists |
| **Group** | Which entries form a unit: `\and` column, names row, run, collaboration group, the block? | 14 sites with four meanings (owner, scope, positional domain, temporal run). Both engines model it inside (`resolver/mod.rs:311,313` `group`/`column`; `author_block/line_kinds.rs:23-33` `LinkOwner::Group`); it is lost at the API (`annotate=N`) and in the observer's output (`Resolved{creators,block}` `resolver/mod.rs:85-88`, writer `Owner{Creator,Block}` `writer/plan.rs:41`) |
| **Link** | Which entry, group or block owns each item? | ruled (steps 1→4 above), **run by neither engine**: both apply step 3 → 1 → 2 (`insert.rs:89-92`: `distribute_upfront_contacts` → `relocate_annotations` → `respell_emails`; `resolver/mod.rs:765-774`: `distribute_upfront` → `answer_pending` → `answer_sequence` → `respell_emails` → `sole_author_takes_the_block`); about 10 owner chains, each with its own order |
| **Emit** | Is the reading written? | the writer gates each family and declines whole papers on one unexplained difference (W3) |

**Signals are read once.** Today a mark is read by about 17 mechanisms in 6 representations (tokens, source strings,
macro bodies, digested ink, events, DOM); a line break by about 8; `\and`, comma/"and", wide space, label key, brace
depth and spacer line by 5-9 each, with a witnessed disagreement for most pairs. The target: one read into the events,
every step consuming them; the scorer adopts the same signals (R13).

**Data model: two trees and a map.**
- **Names:** Block → Group* → Entry. Groups nest: an `\and` column holds names rows, and a row holds runs; a
  collaboration group sits inside a column.
- **Prints:** Block → Section → Item → inner Item. A section (ijcai `\affiliations`, REVTeX `\affiliation` lines,
  amsart address blocks) is where items are cut and kinded; it owns nothing. The inner item (a note's contact, an
  entry's inner e-mail) never inherits its parent's owner (C1, #917); a sub-item carries BESIII's `(A)` (R6).
- **Link:** Item → Owner ∈ {Entry, Group, Block}, decided in the ruled order. A Group is also a **scope**, the
  candidate set for steps 2 and 3 ("the group's only author", spelling within a group).

**One fan-out, `expand(owner)`, at the flush.** It turns an owner into members:
- Entry → itself.
- Group → its live members in print order. For an item printed apart from the names, members whose own marks
  request that role are skipped: R1, applied at expansion and never baked into membership.
- Block → the item stays once at block level (#159).

It is emitted as Perl's label fan-out: each member requests `fm:g<k>`, the item carries `label=fm:g<k>`, and
`relocate_annotations` clones per target (Perl `Base_Utility.pool.ltxml:891-900`), keeping today's per-target dedup
(`relocate.rs:23-33,353-369`). `share_row_contacts` already works this way with `row:shared`/`row:atN`
(`shared_row.rs:96-154`), so no new insert machinery is needed.

**What collapses into a Group owner:**
- `_share=authors`;
- `annotate=all/N/new/run[:N]`, in both engines (`frontmatter_api.rs:621-826`, `resolver/api.rs:48-100`);
- REVTeX/aastex's group rule and `_grouped` (`revtex4_support_sty.rs:1013`, `relocate.rs:769-1013`);
- the collaborations' `annotate=N/new` (`collaborations.rs:699-712`);
- R12 rows (`dispatch.rs:802,1733`, `LinkOwner::Group`, the observer's `R12:row-above`);
- relocate's fuzzy multi-owner (`relocate.rs:236-271`);
- authblk's `TheNote` runs;
- the `fm:block` shared creator.

These become Group scopes: `_group`/`_marks` (`place.rs:129-137`), `frontmatter_group_first`
(`author_block/affiliations.rs:497`) and `Policy::call_is_group`.

**What does not fit:**
- **Positional pairing** (`insert.rs:290`, `relocate.rs:272-283`, the observer's `distribute_upfront`) is 1:1, not a
  fan-out. It stays a step-3 convention yielding Entry owners, legal only as a declared sequence (R10).
- **Runs are temporal:** membership depends on call order and on step-1 answers. This works only with the R1 skip
  inside `expand`.
- **One block across calls** (PoS/JHEP, R9; appending raw classes): settled by Z14 with no declaration. PoS.cls:666 sets each
  call in a `\vtop` of its own, so a block per call is what it prints; R9's witnesses lost their names to `\speaker` printing none in
  the rehearsal (`BOUND_IN_A_BLOCK`).
- **A collaboration's Kind** (R7) must be settled before Group.
- **Raw-class stores** come before the model: a coverage gap (the dev lost-words census family A; FIDELITY_CHARTER
  §4 "No printed frontmatter word is lost").

**Counter-example triage (user 2026-11-01).** Every counter-example a review or census finds is a test of the shared
segmentation and reassembly algorithm, and is first classed:
- **(a) Missing generality over TeX constructs.** The same printed thing in another spelling:
  - a mark after a spacing nucleus (`${\,\!}^1`, `{}^1`, `\mbox{${\ }^{1}$}`);
  - a one-argument line wrapper (`\centerline{…}`);
  - a digit run with a sub-letter (`23A`);
  - an environment name or `\item[…]` argument that prints nothing.

  Fix it once, in the step it belongs to (Cut, Kind, Group, Link) or in the API, through a reader both engines share.
  Never as a per-shape exception in one binding.
- **(b) Authoring that contradicts the macros' own assumptions.** Give it a RED repro and a witness in
  DEFERRED_FAMILIES.md "Pathological authoring". No recovery.

Briefs and review reports class each counter-example (a) or (b).

**Residuals by step** (2026-11-01 mechanism map over SYNC, OPEN_ROWS, the plan's rulings, the reviews and the dev
lost-words census; classes deduplicated, paper counts from different censuses):

| Step | Classes | Pathological, deferred |
|---|---|---|
| Cut | 19 | 2 |
| Kind | 18 | 2 |
| Group | 6 | 0 |
| Link | 11 | 0 |
| Emit | 5 | 0 |

The five classes that cost the most papers, pathological cases excluded:
1. Raw-class and document stores (Kind/coverage): 69 of 101 real lost-words papers.
2. Emit, writer gate shut or whole-paper declines: 29 + 5 papers.
3. Old-reader cuts the writer cannot override (M1, W2, M5, M7, Cut): 29 papers.
4. Convention before marks, and label inheritance (F1, C1, Link order): 21 papers.
5. IEEE block grades and `\thanks` contents (Kind): 11 papers.

Group residuals are few and mostly synthetic. Group costs complexity (14 sites) rather than papers, which is why it
is unified in Stages 1-3 rather than given a stage of its own. The four pathological ones are in DEFERRED_FAMILIES.md
"Pathological authoring" (and "Names stacked under names").

## Stages (2026-11-01; superseded by "The path" above, kept for the mechanism detail)

| # | Stage | Effort / risk | Output | Acceptance witnesses |
|---|---|---|---|---|
| 0 | **Measure and fence.** Per-family writer outcomes and `SkipReason` counts in `ab_audit fmshadow`; a drift guard failing on any new `\lx@add@*` emission in `author_block/`; audit item 2's switches for `insert.rs:290`, `:141`, `relocate.rs:753-816`, `shared_row.rs:54`, `place.rs:42`; a census of unobserved giant lists | EASY, none | byte-identical on `popint125a_s10k` | 2012.14750, 1507.02068, 1512.08197, 2201.02056, 2007.02542 (observed or not) |
| 1 | **One placement order in the reading, and one structure** (design study 2026-11-01, `agent_reports/2026-11-01_stage1_design.md`). A literal reorder of `resolver/mod.rs:765-774` changes correct output and fixes nothing: each pass already runs its own steps in the ruled order on disjoint items (counter-example: the amsart grid with crossed e-mails). The one real violation is R1 at call time (`resolver/api.rs:716-873`: a convention attached before step 1 is known). **1a:** `Owner{Entry,Group,Block}` and `Resolved.groups` in the reading; apart Group conventions (`annotate=all/new/run/N`) held at call time and expanded after the labels (`expand_conventions`) with the R1 skip through one `label_carriers` reader shared with `answer_pending`; tag `ruling1:marks-win`; the old engine waits for Stage 3 (its passes are already label-first). **1b:** hand-printed numbers on every entry through the shared `relocate::leading_line_label` (`api:printed-number`), census first. Gate: `fmshadow --re-resolve` shows no untagged difference. **Step 0's Cut is shared (batch 99a, OD #954):** `\lx@annotate@frontmatter` cuts an affiliation, alternate affiliation or address holding two or more entries led by requested marks into one contact per entry before it is queued, so both engines read the entries (`affiliations::cut_marked_contact`, the author blocks' cutter `entry_calls`, every mark spelling through the shared readers) | MED, medium | F1's 14 go by their marks | 1810.01941, 1202.0629, astro-ph0108212, 1911.09695, 2006.10291, 0909.0058, 1203.1623, 1503.02933, 1604.05346, 1612.02554, 1302.2646, astro-ph0702360, 1105.5884, astro-ph0607224; 1808.01089, astro-ph0105133; astro-ph0408475. Must not move: amsart grid 1302.1274, 1610.05835, 2204.13910, 2205.05923, 2507.07513, 2311.04097; 0903.2366, 2303.09899; 2403.06760, 1804.01623, 2001.01892, astro-ph0611722, 0910.4381; C1's seven |
| 2 | **Step 3 declared, one vocabulary.** `owner=sequence` → `annotate=sequence`, `_share=authors` → `annotate=share` (aliases kept); `new` → `run` where the class groups (quantumarticle first; check acmart, ieeetran, sv, rmaa, aguplus, czipreprint, apa6, kluwer, ws_procl against their class code); elsarticle declares body title marks; `\IEEEauthorrefmark` into IEEEtran's declaration | EASY-MED, low-medium | per flipped class, against pdflatex | 1108.0201; a quantumarticle `\author{A}\author{B}\affiliation{X}\affiliation{Y}` paper |
| 3 | **Owners at the flush.** Owner table built at `place.rs:403`, a Group owner emitted through `expand` (`fm:g<k>` on the item and on each member, R1 at expansion) (after `shadow_flush`, before `share_row_contacts`), consumed in `frontmatter_api.rs:472-800`; `relocate.rs` matches `fm:` exactly; `fm:` items skip `insert.rs:290`, `:141`, relocate's held/group rules and the inner-e-mail inheritance; the writer's Copy/Remove become no-ops for them; switch `owner_table` | HARD, high. **From here ownership has one home** | print order; declined papers get the reading's owners | C1's seven (2101.04527, 2206.03859, 1911.11805, 2102.01452, 2404.15833, 2505.18180, 2507.18248) without the writer; F1's 14; 1009.1371; 2212.06489 (table size); 2103.04797; 1303.7097; 0911.5235 |
| 4a | **In-block items and new kinds from the reading.** The writer's segmentation (`cut` `plan.rs:463`, legends, roles, write-out, `cut_entry_addresses` `:1429`, `block_span_tokens` `:542` with 97e's fold) becomes queue rewrites; new kinds: boxed e-mail items, collaboration contacts, block-level contacts; the ledger's `inside` calls dequeued | HARD, high | pinned output of `batch97a/97d/97f` changes, each with its PDF evidence | names M1 (2605.09461, 2507.07465, 2609.36484, 2603.10436, 2608.20401); W2 (2410.09484, 2507.05173, 2105.11228, 1009.4816, 2512.19748); W3 (2503.10804, 1903.03238, 2512.20289); 0910.2149, hep-ex0011032, 2501.13770, 1210.5341; 1508.03930, 1603.09150, 1509.06963, 2007.02542, 0909.3044; BESIII `40A` (R6) |
| 4b | **Names from the reading.** `writer/repart.rs` moves to the flush; `insert.rs:97` becomes a read-back Warning; a binding `appends` declaration for PoS/JHEP's one `\@author` (`jhep_cls.rs:183`, PoS.cls:666) | HARD, high | `merged_creators` Errors must not rise | M7 (2412.03072, 1901.08289, 2510.01161); M8 (1409.4999, 2501.09580); M5 (2107.12423, 2603.24245, 2003.04668, 2101.00440, 1708.03035, 2402.14804); 2105.09506; 1711.04132 |
| 5 | **The old engine becomes the fallback.** Positions per call (`ledger.rs:67-69`), so giant lists are observed (done in F1, with the emitter's switch on: OD #837); the old passes run only without a table; the writer's DOM operations, `keep_as_today` and `Policy.written` deleted | MED, medium | unobserved papers only | 2012.14750; the giant lists Stage 0 finds unobserved |
| 6 | **Binding-private splits retire,** one binding per batch by reach: llncs, aa, sv_support (svproc's override goes: SV1 returns), inst_support, webofc onto `\lx@add@authors@list` / `\lx@add@affiliations[split=…]`; `group_or_marked_affiliation` → `annotate=run`; the contrib parsers onto declarations | EASY-MED each | per binding, reach-complete A/B | 2206.00684, 1912.05868, 2512.08803; 2608.25509 + C1's svproc papers; 2610.23498; 2203.01270; 0910.4381 |
| 7 | **The fallback becomes Perl's.** `author_block/` shrinks to `Base_Utility.pool.ltxml:687-727`, the cautious split and a `frontmatter::classify` module; the API's Rust-only attach modes go | MED, low | Stage 5's residual unobserved papers | — |

Checks at every stage: `ab_audit fmshadow --re-resolve` on every family's census first, then the A/B (stratified
subset plus the reach), `fm_checks.py` (it flags shared addresses and affiliation growth), the review's `cr.py` /
`order.py`, the 3,000-sample D3 (no drop), the dev subsets, the gates. Every guard whose pinned output changes is
listed with its PDF evidence.

## Rulings

| # | Question | Recommendation | Status |
|---|---|---|---|
| R1 | May a convention give an unmarked item to a name whose marks request items of that role? | Never for items printed apart from the block; in-block lines under the names stay with the row | **RULED 2026-11-01: marks win** (FIDELITY_CHARTER §4) |
| R2 | Pairing by position | Hand-printed numbers count everywhere (#918); bare position only under a declared `labelseq` | **RULED 2026-11-01: printed numbers only** |
| R3 | Does a class convention ever outrank a mark? | No (a class's printed numbering is a label) | **RULED 2026-11-01: no** (with R1) |
| R4 | Must every difference from today's output still carry a change tag, or does the ruled order license it, with the comparator kept as the census gate? | The ruled order licenses it (Stage 3 onward) | **RULED 2026-11-01: the ruled order licenses; 2026-11-06: opened shape by shape,** since opening it whole measures +8 / -38 creators |
| R5 | The fallback when the reading is uncertain | The old reader until Stage 7, then the reading with the uncertain text whole | **RULED 2026-11-06: the old reader stands until Z17; then the reading with uncertain text whole, Perl's reader only without events** |
| R6 | BESIII sub-labels: does `48A` request sub-entry (A) of entry 48? | Yes where entry 48 prints `(A)` sub-entries, else entry 48 whole | **TENTATIVE 2026-11-01** (main session, user away; revisit): as recommended, at Stage 4a (sub-items) |
| R7 | revtex `\collaboration` / `\affiliation{(… Collaboration)}`: a contact (Perl's aastex) or a creator (Perl's revtex)? | A contact, as the 2026-11-01 collaboration ruling makes one for every collaboration; the same for an aa/llncs/svjour `\and` piece `X Collaboration:` (98a: a creator today, as base 97b) | **RULED 2026-11-06: a contact** |
| R8 | Contact order within a creator | Print order (97d D1 chose it) | **RULED 2026-11-01: print order** |
| R9 | PoS/JHEP one `\@author` across calls: lift the difficulty deferral for Stage 4b? | Yes, through the binding's `appends` declaration | **RULED 2026-11-06: lifted** (Z14; 39 of dev3's 261 wrong creators). **Done by Z14 with no `appends`:** see "What does not fit" |
| R10 | amsart's upfront N·m grid and numbered institutes equal to the authors: a declared sequence under R2, or bare position? | A declared sequence where the class's own code pairs them (Stage 1's "must not move" witnesses) | **TENTATIVE 2026-11-01** (main session, user away; revisit): as recommended |
| R11 | Does the R1 requester skip apply inside a Group owner at expansion, including in-block lines under the row? | At expansion, for items printed apart from the names; in-block lines under the row stay the row's (R1's exemption) | **TENTATIVE 2026-11-01** (main session, user away; revisit): as recommended |
| R12 | A Group in the XML: Perl's clones only, or also a group id in the provenance attribute? | Clones only, as Perl does; the id is a later beyond-Perl option | **TENTATIVE 2026-11-01** (main session, user away; revisit): as recommended |
| R13 | Should the scorer read the ruled signals (symbol marks as labels, break kinds) so that its disagreements read as converter defects? | Yes, one signal reading shared by converter and scorer | **TENTATIVE 2026-11-01** (main session, user away; revisit): as recommended |
| R14 | Emit: decline per item or per paper (W3)? | Per item: write what the reading explains, fall back for the rest | **RULED 2026-11-06: per item** |
| R15 | (N-B) A Group whose every member requests the role: does it stand (today, 77n t02), or go to the block once (strict R1)? | Stands; measure strict behind the `r1_at_expansion` switch, then ask | **TENTATIVE 2026-11-01** (main session, user away; revisit) |
| R16 | (N-A) A labelled item nobody answers, with one name: the sole author (OD #159) or the block? | The sole author, as today | **TENTATIVE 2026-11-01** (main session, user away; revisit) |
| R17 | (N-D) The run walk's call-time stop (`resolver/api.rs:89-118`) bakes R1 into membership (`A$^1$, B, C$^1$, D`: B gets nothing) | Keep in Stage 1, remove in Stage 2 | **TENTATIVE 2026-11-01** (main session, user away; revisit) |

## Interaction with bundle int126a

- 97c, 97d (r2), 97e and 97f (r2) all land before Stage 1, since they touch `blocks.rs`, `api.rs`, `mod.rs`,
  `lines.rs` and `post.rs`. Stage 0 can start at once.
- 97d's `collaborations.rs` is the last ownership rule allowed into the old reader; Stage 0's fence enforces that.
- 97f's M1 unwrap (`author_block/separators.rs:711-768`) is fallback-reader work. It stays until Stage 5.
- 97f r2's F1 and F3 are stopgaps that Stage 1 generalizes.
- 97f r2 also adds `\lx@add@contact@placed` emissions in `author_block/email_block_calls.rs` (`placed_email_line_calls`, `take_placed_line_emails`): Stage 0's fence baselines them as stopgaps (rev126a N2).
- SV1 (svproc's override deleted) returns in Stage 6, after Stage 3 gives inner e-mails the chain at the API.
