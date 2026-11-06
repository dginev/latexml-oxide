# Frontmatter: one ownership engine, one placement order (living plan)

**Why.** The user's direction (2026-11-01): "keep evolving our design of the Frontmatter API until it can accommodate
all variations used in arXiv. Prioritize refactoring to a better model of a single ownership engine, unified machinery
and API, unified placement order inside the API." This decides item 9 of
[`FRONTMATTER_ARCHITECTURE_AUDIT_2026-10-28.md`](FRONTMATTER_ARCHITECTURE_AUDIT_2026-10-28.md). The plan below comes
from a read-only design study (2026-11-01). Its stages are the frontmatter batches after bundle int126a. Per-shape
findings become their acceptance witnesses.

## The path: the old engine removed before the merge (user 2026-11-06)

**The goal (user 2026-11-06): "continue, and remove the old engine before the merge".** One engine, the other's code deleted, for simplicity and compile time; the bar is not the only measure of this work. **It completes [`FRONTMATTER_DIGESTED_REFACTOR.md`](FRONTMATTER_DIGESTED_REFACTOR.md)'s Stages 7-8: every decision reads the observer's events or the digested XML, never token or source spelling; tokens are used only as the content an event's span covers.** This section supersedes the Stages table below where they differ. Basis: `agent_reports/2026-11-06_engine_rebaseline_report.md`, `2026-11-06_engine_removal_report.md`: all of dev3 with int139c, observer on and off, devh10k as a less-tuned control, the frozen scorer (`fm_scorer/FROZEN.md`).

**The boundary is Perl's Frontmatter API (user 2026-11-06).** The engine relies as much as possible on the original LaTeXML API, its LaTeX macro dialect, and follows its design: a binding or the emitter speaks `\lx@add@*` and `\lx@annotate@frontmatter` with their keys, entries queue, digest and insert, and labels link annotations to their owners through `relocateAnnotations` (`Base_Utility.pool.ltxml:503-536`, `:736-740`, `:891-900`). The engine reads what those calls printed and decides owners; it answers in the same dialect (`\lx@add@creator`, `\lx@add@contact`, `label=`, `annotations=`), never by DOM surgery and never through a parallel path. All 40 of Perl's macros are in use. An extension is allowed where it has Perl's shape, a macro with keys feeding the same queue, and is opt-in data; it's the exception, and it stays only with a stated reason. Today there are 16: `\lx@add@authors@append`, `@list`, `@adaptive`, `\lx@add@author@split`, `@keyvals`, `\lx@add@contact@placed`, `@list`, `\lx@add@affiliation@marked`, `@shared`, `\lx@add@store@affiliations`, `@authorblock`, `\lx@add@frontmatter@container`, `\lx@add@creator@rdf`, `\lx@add@aas@email`, `\lx@add@email@metadata`, `\lx@add@date@halved`; and the keys `_grouped`, `_markpiece`, `_share`, `owner=`. Each one retires with the reader or becomes a key on Perl's own call, or stays with its reason recorded; the classification comes at Z4, and Z20-Z22 move the bindings.

**Where it stands.** The observer on 98.83 / 98.55 %, off 97.83 / 97.35 %. The reading alone, its serialization artifact removed, 97.57 / 97.12 %: **-1.26 / -1.42, -280 creators net**, by class: no-block 0 / -118, declined +28 / -108, post-writer passes not in the reading +1 / -61, held 0 / -18, residual 0 / -4. On devh10k's disagreeing papers, declined is **+90 / -63**, the reading ahead; without no-block and the post-passes devh nets about -19. So most of the gap is two narrow classes. Token-spelling reads (`tokreads`, over-approximate): engine 1,962 (`author_block` 1,149, stores 399, relocate 127, `plan.rs` 116), bindings 556. Stages 2, 3 and 5 of the digested refactor have not landed.

**Order: measure, event gaps, content from spans, emitter on the safest families, repairs tied to each family's flip, delete.** Every gain or loss in the tables is a counterfactual score until that batch's real emit.

| # | Mechanism | Risk | Output | Witnesses |
|---|---|---|---|---|
| **Z0** | Writer outcomes in `fmshadow`, read-back, fence and pass-list guards, the reading-alone tool with the rdf copy, the family gate table, the token-read check | EASY | byte-identical | 2012.14750, 2201.02056, 2606.18623, 1507.02068 |
| **Z1** | Event gaps 1, 2, 6, 8, 9, additive, checked with `--re-resolve` | MED, low | byte-identical | 2105.07958, 0804.4669 |
| **Z2** | Gaps 3, 4, 5, 7, then the writer's regressions on events ("de Naurois", an IEEE `1st` name, A*STAR) | MED, low | witnesses and reach | 2606.16069, 1702.08280, 2401.15006, 2609.01831 |
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
| **Z19** | **Delete DOM operations, old passes, `shared_row`;** the compile measure | EASY | byte-identical | gates |
| **Z20** | Core splits: llncs, aa, sv_support, inst_support, webofc | MED | per binding | 2206.00684, 1912.05868 |
| **Z21-22** | Contrib parsers | MED | per binding | per-class nets |
| **Z23** | The stores' token deciders on events | MED-HARD | store reach | 2607.20672, nlin0310001 |

**The event gaps.** The digested refactor's 1-5, and: 6, unprinted contacts (aastex7's rdf `\email`); 7, mid-line hand-printed labels (100a's RED); 8, a declaration event with a span (`definitions_at_author`, `email_lines.rs:609`), which the emitter must keep; 9, partial events under held errors; 10, llncs `\orcidID` as a contact.

**The waves** (per paper, on a new `Policy.emits` replacing `written`, `policy.rs:96`; a mixed paper waits for its last family; decline rate does not predict the gap):

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

**The flip gate.** A family flips when real emitted output is at or above the then-current base on dev3 (frozen scorer) and net creators are at or above 0 on devh10k, with every worse paper read against the PDF. No code is deleted before wave 8. One holdout v4 read at the last flip, no lower than the pre-flip reading of the same draw.

**Definition of done (each one mechanical):**
1. `author_block/` is gone; old passes and the writer's DOM operations are gone. What remains: `frontmatter/classify.rs` (text classifiers, the owner chain), the emitter, Perl's `\lx@add@authors` for papers with no events.
2. The final fallback: with events the reading always answers, uncertain text whole; without events (Fatal, NotUndone, JournalUnavailable) Perl's reader, about 60 lines. The cautious split goes. Held papers are repaired, not sent to the fallback (3 such papers, 25 creators: old right 23, Perl 5, reading 5).
3. The token-read check at its floor: about 60 in the engine (rehearsal's table, ledger stamping, the Perl API boundary), `resolver/` 0, decision code elsewhere 0.
4. The fence holds until then: `\lx@add@` emissions and token reads in `author_block/` can't rise; a new ownership rule lands in `resolver/`.
5. Passes after the emitter may change kind and text, never who holds an item (guard and test-profile invariant).
6. Dev3 D3 not below base at any push; the holdout read once.
7. The API boundary: every binding and the emitter reach the document only through Perl's dialect and the kept extensions. A guard pins the list of macro names and keys outside Perl's: a new one fails until justified, and each batch that retires one removes it from the list.

**Expected size (estimates, an over-count of what stays):** the engine deletes about 30.9k lines and adds about 3.6k, net about -27k of 144,114 (19 %); contrib about -3.2k. `author_block/` 19,964 → directory gone, about 2,500 moved. **Compile time is unmeasured** and may gain less than the line share; a clean `cargo build -p latexml_engine --timings` on Z19's parent and Z19 settles it.

**The 98 % bar at the merge:** decided after the v4 read (user 2026-11-06); 98 / 98 stays the stated bar, nothing lowered in advance. **The truth set (user 2026-11-06):** the v4 read comes with one: every paper the frozen scorer flags plus about 500 it passes, authors, affiliations and e-mails recorded from the PDF and source alone by two independent agent reads, disagreements to the user; labelling agents never see the converter's output or the verdict, the main session sees only counts. A pilot on dev3 settles the format and cost.

**Open, proceeding on the recommendation unless the user says otherwise (main session 2026-11-06):** the floor's figure, re-set at Z0 from a hand-audited count; stores' reads of class code exempt and counted apart, content lines must move; `queue_digest.rs:661`'s ToString fallback kept only for non-printing operands; R6, R10, R11, R12, R15, R16, R17 as tentatively written, at the batch that makes each real; the flip gate's tolerance zero; no dip in dev3 at a push.

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
- **One block across calls** (PoS/JHEP, R9; appending raw classes) needs a declared block boundary (Stage 4b
  `appends`).
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
| 5 | **The old engine becomes the fallback.** Positions per call (`ledger.rs:67-69`), so giant lists are observed; the old passes run only without a table; the writer's DOM operations, `keep_as_today` and `Policy.written` deleted | MED, medium | unobserved papers only | 2012.14750; the giant lists Stage 0 finds unobserved |
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
| R9 | PoS/JHEP one `\@author` across calls: lift the difficulty deferral for Stage 4b? | Yes, through the binding's `appends` declaration | **RULED 2026-11-06: lifted** (Z14; 39 of dev3's 261 wrong creators) |
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
