# Frontmatter on the digested representation — refactor design (archived 2026-11-08)

**Superseded (2026-11-08, DOC1).** The living doc is [`../parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`](../parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md). Its sections "The principle" (with the 2026-10-29 directive, task #423), "What already exists", "Gaps in the event model" (gaps 1-10, numbers kept), "What a rollback leaves (census, Z3)" and "Test suite and measurement" moved there under the same titles. Kept here as history: the status line of 2026-11-06, the 2026-10-29 spelling inventory (groups 3A-3E), the Stages 1-8 (Stage 1 landed 90a, 2 Z1-Z2, 3 Z3, 4 92b; 5-8 replaced by the plan's "The path": Route D, the strict ports, the flip, Z18-Z19) and the withdrawn audit item.

**Status (2026-11-06):** the old engine's removal before the merge is this refactor's Stages 7-8; the batches are Z0-Z23 in [`FRONTMATTER_SINGLE_ENGINE_PLAN.md`](FRONTMATTER_SINGLE_ENGINE_PLAN.md) "The path". Stages 1 and 4 landed; Stage 2's gaps 1, 2, 6, 8, 9 landed with Z1, its gaps 3, 4, 5, 7, 10 with Z2, Stage 3 with Z3; Stage 5 has not.

## What reads spelling today (inventory groups)

| Group | Where (representative) | Replaced by |
|---|---|---|
| Observer writer/comparator re-parse `tex` — **done (90a, OD #911)** | `writer/dom.rs`, `fmshadow/mod.rs`, `relocate.rs` `shown_marks`, `author_block/name_text.rs` `is_name_mark` now call `observer::marks::read_formula` | XMath shape (as digested and as parsed); `relocate::printed_marks` splits a mark's printed text |
| Writer cut walk-back — **done (Z3)** | `plan.rs` `piece_tokens`, `mark_formula_start`, `labelled_lead` | `writer::Field`: the formulas the digested list holds; a mark's own print; brace balance forward (Stage 3) |
| Binding pre-splitters | `defs/author_macros.rs:591` `ijcai_email_lines`; `defs/general.rs:211` `\lx@split@addresses` (brace list :233-244); `frontmatter_fields.rs:117,520,547,593,742`; `relocate.rs:693-717` revtex group rule | rehearse the section; Break (non-`group` role); printed-brace depth from ink; `resolver/labels.rs:446` `addresses_in` on printed text |
| Label keys | `frontmatter_api.rs:786,799` → `queue_digest.rs:212-300` `clean_frontmatter_labels` (FONT_SWITCH[19], spacing, `\ref{`); `insert.rs:689` `unanswered_mark_text` | `Mark.label` = printed text of the digested operand (**needs a user ruling**: departs from Perl's ToString labels) |
| Old reader `author_block/` (13,150 lines; 499 cs-literal / 404 `T_*` lines) | 3A name parting (195/202: `separators.rs:585`, `rows.rs:646,690,722`, `space_parted.rs:186`); 3B marks (78/81: `marks.rs:199`, `symbols.rs:66,473`); 3C line kinds (39/41: `line_kinds.rs:261`, `links.rs:318`); 3D e-mail/address (118/67: `email_owners.rs:385,625`, `email_block_calls.rs:613,908`); 3E notes (69/13: `notes.rs:88,473,617`) | Sep/Break/Layout events (3A); Mark (3B); Contact + Name text (3C — word lists stay, on printed text); Contact + ink (3D; Semiverbatim `\email` stays raw by Perl design); Note events, accents already Unicode (3E) |
| Per-class Rust parsers (38 files) | ptephy_cls, osa_journal_sty, amsppt, aistats, interspeech, epl, eptcs, … | the binding declares its commands' events (rehearsal bindings + `policy`); migrate with each family |

Six diverging switch lists disappear with their callers: SWITCHES[33] (88c), MARK_SWITCHES[32], FONT_SWITCH[19],
NAME_TEXT_COMMANDS[38], PLAIN_FONT_COMMANDS[8], FONT_WRAPPERS[16]. MARK_SWITCHES and MARK_SPACING[4] have only token
callers left since stage 1 (`relocate::script_marks` through `past_mark_switches`: the `\lx@frontmatter@sup` label keys,
`author_macros.rs:254`, and revtex's group rule, `content_marks`/`group_shares_the_mark`; `notes::leads_with_mark`):
they stay for stages 3–5.

## Stages (smallest risk first; each its own batch, red/green, census A/B)

1. **XML readers** (writer view, comparator, `shown_marks`, `is_name_mark`): read the XMath shape; merge the two
   copies into one reader. **Landed in 90a** (OD #911, `observer::marks::read_formula`, guards
   `perfect_kernel_batch90a`). The rule: a floating script is a mark unless the item printed right after it is a
   letter or digit; a blank box before `^` is no ink; subscripts stay in a base; a script on printed text is on a
   base; an application, digested or parsed, is read through only when LaTeXML lays it out as a row (an infix role,
   `POSTFIX`, `Apply:?:?`, a script on its base: Perl `Post/MathML.pm`) and holds a script, in print order, its silent
   operator none (`$Ann^{1}$` as `⁢(A, n, n¹)` reads as the digested `A n n ^1`); a two-dimensional layout (a
   fraction, a root, an accent, an enclosure, limits) is one printed item, and so is a big operator with its body
   (which Perl rows — read as one item on the safe side), so `$\frac{a}{b^{1}}$` ends
   on no script. The token-only callers of `MARK_SWITCHES`/`MARK_SPACING` (through `relocate::script_marks`: the
   `\lx@frontmatter@sup` label keys, `defs/author_macros.rs:254`; revtex's `content_marks`/`group_shares_the_mark`;
   `notes::leads_with_mark`) stay for stages 3–5. Settled 2026-10-29 (`agent_reports/2026-10-29_rf1_xmath_shapes.md`): the writer
   (`insert.rs:98` → `writer/mod.rs:63,73` → `dom.rs:79`) reads every formula *before* `finish_document`'s rewrite
   (`core_interface.rs:632`, the `tex_math.rs:2046` floating-script → `ltx:sup` rule) and math parse (`:654`), so it
   sees the digested shape: `XMApp[@role='FLOATSUPERSCRIPT']` (or `POSTSUPERSCRIPT` after base tokens) holding
   `XMArg[rule=Superscript]`, flat `XMTok`/`XMHint`/`XMText` siblings — chosen at digestion (`script_handler`,
   `tex_math.rs:121`). The comparator reads the final, parsed XML: a lone floating script is `ltx:sup`, and a script
   with a neighbour is `XMApp(XMTok[role=SUPERSCRIPTOP scriptpos=preN|postN], base, script)`, `XMDual`/`XMWrap` around
   lists. One reader takes both vocabularies: a mark is a floating script with no ink after it in its formula (any
   `XMHint` aside), or an `ltx:sup`; `preN` (a following base, ³He, `^a)`) and `postN`/`POSTSUPERSCRIPT` (base text plus
   a script) are not marks; a single `FLOATSUPERSCRIPT` whose `XMArg` holds only `XMTok` reads as the `ltx:sup` the
   rewrite will make. Font switches and spacing commands never appear — the six switch lists go.
2. **Event-model additions** (gaps 1–5, then 6-10): additive, shadow-only; checked offline with `ab_audit fmshadow --re-resolve`. **Gaps 1, 2, 6, 8, 9 landed in Z1** (guards `perfect_kernel_batchz1`), **3, 4, 5, 7, 10 in Z2** (`perfect_kernel_batchz2`).
3. **Writer cut**, **landed in Z3** (`writer/content.rs`, guards `perfect_kernel_batchz3`): a span's content is read from the digested list and its spans. Absorbs 88c (font-switched marks, 2105.07958; the isotope class; q17's mid-line unowned mark as text; the q18b/x2 cut path, 90a's RED repros, among them amsart's `$\,^{1}$` address
   lines, 0804.4669's shape, `amsart_address_spaced_marks_cut_with_their_lines`). `notes::leads_with_mark` stays, the old reader's, for Z18. What each walk-back read, and what reads it now:
   - `piece_tokens`, back from a span over shifts, braces and scripts: `$^{1}$` (the script is the Mark's span), `${}^*$Blackett` (the empty base), `\( ^{1} \)` with its spaces and `\protect` (hep-ph0110378, 2307.12984). It stopped at a switch (`$\rm ^{3}$`: no balanced run, a paper not written, 2105.07958). Now `Field::content_of`: a span takes the formula it starts or ends in whole, from `Outcome::Events::formulas` (each outermost formula's extent, `events::lex_with_formulas`), whatever is in it or opens it, a formula of no mark too (`$\alpha$ Lab`).
   - `piece_tokens`, on over closing braces and shifts: kept, as brace balance over the content, in one forward pass (`Field::balanced`): a group the span opens ends at its closing brace. A mark's own group, the Walker's fact that it prints its formulas alone (`Walker::note_a_formulas_group`, `Formula::alone_in`: spacing, a switch, a color, a list's comma beside it, no command printing before them, no word after: `{\bf $^{1}$} Univ X`, `{$^1$\,} Univ X`, 1709.01760's shape), starts at its opening brace; any other group a span closes is still no run, the fail-safe of a group going on after its mark (`\textit{$^{2}$H} Lab`, `cut_run_wrapped_isotope_is_entry_text`). Its environments too, in pairs (`\begin{ee}x\end{ee}`). The `\(` count stays by spelling: an unbalanced `\(` leaves no formula in the digested list to ask.
   - The walk's limit in `span_tokens`' lead, the mark's end: a closing brace inside the formula stopped it, so `${^\dagger}$ Corresponding author`, a legend, lost its printed mark (2605.10201's shape, `perfect_kernel_batch77n::authblk_glued_symbols_in_one_author_block`); the lead now starts past the whole formula.
   - `mark_formula_start`, for a mark in a wrapper (`\textbf{$^2$Tencent Hunyuan}`, 2605.25195; `{\it $^2$Univ B}`; `\textbf{\textsuperscript{2}…}`): `Field::own_print`, the formula's extent or the call's own position (`Event::own_at`), with only spaces from it to the entry's ink.
   - `mark_formula_start`, for where a group's switches stop (`{\itshape $^2$Univ B, City}`), and `labelled_lead`, back from an entry's ink to its marks (`$^2${\itshape Univ B}`): `Event::marks`, where the entry's leading marks print, and forward from there.
   - Still token-shaped, and not walk-backs: `entry_font` (a font by its boxes, the tokens between by kind), `container_wrap` and `cell_tokens` (a container's arguments, an environment's rows), which go with Z6. Gate: A/B popZ01.
4. **Labels as printed** (RULED 2026-10-29: printed labels, FIDELITY_CHARTER §4): every family; the marked-path guards; the `\lx@frontmatter@sup`
   label keys (`author_macros.rs:254` → `script_marks`). **Landed (92b, OD #919)** on the digested primitives, request
   and set side: `queue_digest::frontmatter_label_keys` reads an operand printing through a command by its speculative
   digestion's ink (`\inst{\ESO}` entry 2's, `\dag` ≡ `$\dagger$`; any spacing a label boundary, invisible
   characters and pictures no label, the observer's mark reader alike), Perl's ToString the fallback; the token readers
   before digestion (`place.rs`, `shared_row::marks_pick`, `queue.rs`) stay for stages 5 and 7.
5. **Pre-splitters onto the rehearsal**: ijcai e-mails, `\lx@split@addresses` → `addresses_in`, `\inst` marks,
   `\thanks`-as-affiliations, revtex's group rule (`content_marks`/`group_shares_the_mark`). With the last token
   caller gone, `MARK_SWITCHES` and `MARK_SPACING` go.
6. **Provenance as a schema attribute** (user 2026-10-29: the archival tier records how each creator and contact was
   decided as a schema attribute): one `LaTeXML.model`/RelaxNG attribute on `ltx:creator`/`ltx:contact` carrying the
   resolver's decision (spelled, marks, order, the group's only author, the corresponding author, the block, the
   class's own call); the observer's resolver already tags each decision (`owner:spelled`, `owner:order`,
   `owner:block`), so the written families emit it first and each family gains it as it is promoted below. A schema
   change refreshes the binding nets' RelaxNG copies (`binding_nets/scripts/rng/`).
7. **Retire the old reader family by family** through the per-family writer gate (S2b/S2c; ams_support written,
   aas_support's G5 held): notes, e-mails, marks, line kinds, name parting last.
   llncs and aa (task #429; `agent_reports/2026-10-29_s6_aa_llncs_gate.md`): in shadow, unclassified llncs 19.0 %,
   aa 23.3 % (90a). **Groundwork landed (92a, OD #913)**, the study's pieces 1, 2 and 7, each read from the digested
   list: a mark whose script holds `\ref`s printing nothing at digestion is one Mark carrying every key, each reference
   an item of its own whatever prints between them (`events::script_mark`/`script_ink`, `\ref{ia}\ref{ib}` and
   `\ref{ia},\ref{ib}` both `ref:ia,ref:ib`; the ref list split in `mark_labels`; an owner found by a key among a
   numbered entry's labels tagged `api:ref-label`; an ORCID link inside a mark the name's ORCID, not mark text,
   2407.07678, 2608.18287) — revtex's and amsart's written output reads every key too; a
   publication note's e-mail calls made with a binding's meaning (never a link it prints, nor a call of the paper's
   own `\email`, which stays the note's text: `ledger::email_meaning`, the meaning when the note is rehearsed — at
   the flush, `\maketitle` —, `EmailMeaning` on the event) reach the ledger, placed by where that binding prints them (`Policy::email_in_place`, the class's declaration: the
   digested list cannot tell) — printed in the note (the class's `\email`: llncs, aa, svjour), by FIDELITY_CHARTER
   §4's chain exactly, no elimination, the note's name (a piece, parted at `, ; : / &` "and" "or", that is one live
   creator's whole name) the corresponding author's designation (`answer_document_notes`, `Chain::DocumentNote`,
   `note_named`); routed elsewhere (amsart, REVTeX), the current author's whole own call (#420), the block's whole
   with no live author current; a
   frontmatter clear removes the API calls and
   the author blocks queued before it (`ledger::record_clear`, every real flush counted). Shadow census (92a r2,
   re-resolved; 92a r4 re-converted): unclassified llncs 81 of 463 papers (17.5 %, unchanged), aa 57 of 554 (10.3 %).
   **92b** (OD #917): G3, an entry's `\email` printing one address placed by FIDELITY_CHARTER §4's chain (the author it
   spells, the entry's only carrier, the block, the ONE address left the corresponding author's — one step,
   `Resolver::place_emails_left`, shared with the `_share` lines; `owner:inner-email`), and an e-mail call's
   line breaks read as such (`events::raw_lines`); the 85b `_share=authors` port, llncs's one `\institute` every live
   author's unless the class's `\inst{n}` picks it, its e-mail lines by the chain (`api:share:authors`); (OD #918) a
   numbered entry's owner found by a mark printed by hand, each request keeping its origin
   (`Resolver::class_requests`), tagged `ruling1:request`; a numbered entry whose every line a mark leads split at
   them (`line_led_pieces`); (OD #920) aa's `\author` still stacks through inst_support's, a ruling open (restoring
   Perl's clear there changed 162 OmniBus papers, mostly stacking classes). Shadow census (92b, re-converted):
   unclassified llncs 32 of 463 (6.9 %), aa 21 of 554 (3.8 %). **92c: aa and llncs written** (OD #922-#924): aa's own
   replacing `\author` keys its blocks `aa.cls` (piece 8); `written: true` for both; the writer writes an address group
   out (`Op::WriteOut`) and an empty call holds no paper back; a label glued to a symbol parted (`\inst{1*}`), a
   share entry no hand mark requests, a note's spelled address a change. Scorer aa 81.23 % → 100 %, llncs 78.98 % →
   94.89 % (left: 1901.07288, stage 5's pre-splitter; two scorer source limits); census unclassified llncs 25 (5.4 %),
   aa 18 (3.2 %). **92c r2** (review rev92c): an e-mail call's link is its printed text, one item per owner and text,
   a group written out only when its contact prints the addresses alone, added items in the reading's print order
   (OD #923, #924); aa's sole `\institute` is `_share=authors` as llncs's (OD #932). **92c r3** (review rev92c r2): a
   note's item prints after the author block's (aa's `\offprints` e-mail below the institutes), aa's `\email` the kernel's
   `\lx@add@email` so the shared row reads aa's sole institute as llncs's (a `\thanks` in it no line of it, an unread
   `e-mail:` line an e-mail), a group's domain after its brace its addresses' (OD #923, #924, #932). **93a: the kernel family written** (OD #925-#927): the kernel's own `\author` (`article`,
   `report`, `book` and every class whose bindings declare no conventions) `written`, a binding owning `\author` that
   declares none `UNDECLARED` (unwritten); the writer cuts a block's own marked lines today's joined, writes a legend
   line as the starred names' note, leaves an unrequested entry where today's put it; readings: a label several `\and`
   columns answer each column's, a compound symbol mark all its symbols', an order list agreeing with the block's
   every spelled address, a trailing mark a label only on a name. Scorer marks path 91.88 % → 92.53 %, `article`
   92.00 % → 92.39 %; left: the names the old reader parted otherwise (a writer re-part operation, 32 creators —
   **94a, OD #928**: `writer::repart` writes the creators again from the reading where it refines today's names, each
   name the run of today's digested personname it is (round 2: its fonts and notes kept), each item today's copied or
   made from its entry's tokens (no entry mark), only across a boundary the digested structure shows, kept only when it reads as the
   reading and holds every word today's held (round 3: declining on any doubt — no creator of invisible ink, no
   separator inside a written name; round 4: a creator of today's made an item only on structural evidence, a mark, a
   labelled line or row, an API call — never a word reading, a whole creator so read given back); marks path 92.53 % → 94.50 %, `article` 92.39 % → 95.48 %; left
   1304.1574 — **94b, OD #937**: an in-block `\affil{$^1$…}` reads its leading Mark as its label, 2405.08158 and
   1602.05576 —), raw files' in-place contact setters (**94b, OD #936**: the API's when the file's code prints the
   argument as a title-page line or appends it to its `\author`'s store), the scorer's source reading (11), other
   families' (44). **95b, OD #942: IEEEtran, acmart, authblk and the MNRAS family written** — acmart's and
   mn2e_support's own declarations, `owner:entry-address` a change tag, the comparator reading the take-out, a source-
   spelled label read to its first command, the writer keeping an item that prints a later mark its creator requests;
   archival 97.56 % → 98.15 % (`fm_scorer/run_95b_mn2e`); elsarticle unwritten (its `\author` is
   elsart_support_core's, shared with OmniBus through elsart_support).
8. **Per-class parsers** as each family is promoted.

## Superseded

`FRONTMATTER_ARCHITECTURE_AUDIT_2026-10-28.md` rank-7 ("export a token toolkit for the bindings", :108, :151)
conflicts with this directive and is withdrawn.

## From `FRONTMATTER_SINGLE_ENGINE_PLAN.md`, superseded (moved 2026-11-08)

The plan's own superseded parts: "The path"'s status of 2026-11-06, its original Z0-Z23 order with the event-gap
summary, the waves, the flip gate and the old plan's size estimate (Route D replaced them; the Z numbers stay names);
its Stages 0-7 of 2026-11-01 with their checks; its notes on bundle int126a. The plan keeps what is live of them
("Where it stands", "What the original order still holds").

### "Where it stands" (2026-11-06)

**Where it stands.** The observer on 98.83 / 98.55 %, off 97.83 / 97.35 %. The reading alone, its serialization artifact removed, 97.57 / 97.12 %: **-1.26 / -1.42, -280 creators net**, by class: no-block 0 / -118, declined +28 / -108, post-writer passes not in the reading +1 / -61, held 0 / -18, residual 0 / -4. On devh10k's disagreeing papers, declined is **+90 / -63**, the reading ahead; without no-block and the post-passes devh nets about -19. So most of the gap is two narrow classes. Token-spelling reads (`tokreads`, over-approximate): engine 1,962 (`author_block` 1,149, stores 399, relocate 127, `plan.rs` 116), bindings 556; on the guard's wider shapes at Z01, engine 2,264, `resolver/` 12, bindings 621, and Z2 widens them again and re-pins. Stages 2, 3 and 5 of the digested refactor have not landed.

### The original order (Route D replaced it)

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

### The old plan's size estimate

**Expected size under the old plan; Route D's is above (estimates, an over-count of what stays):** the engine deletes about 30.9k lines and adds about 3.6k, net about -27k of 144,114 (19 %); contrib about -3.2k. `author_block/` 19,964 → directory gone, about 2,500 moved. **Compile time is unmeasured** and may gain less than the line share; a clean `cargo build -p latexml_engine --timings` on Z19's parent and Z19 settles it.

### Stages (2026-11-01; superseded by "The path" above, kept for the mechanism detail)

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

### Interaction with bundle int126a

- 97c, 97d (r2), 97e and 97f (r2) all land before Stage 1, since they touch `blocks.rs`, `api.rs`, `mod.rs`,
  `lines.rs` and `post.rs`. Stage 0 can start at once.
- 97d's `collaborations.rs` is the last ownership rule allowed into the old reader; Stage 0's fence enforces that.
- 97f's M1 unwrap (`author_block/separators.rs:711-768`) is fallback-reader work. It stays until Stage 5.
- 97f r2's F1 and F3 are stopgaps that Stage 1 generalizes.
- 97f r2 also adds `\lx@add@contact@placed` emissions in `author_block/email_block_calls.rs` (`placed_email_line_calls`, `take_placed_line_emails`): Stage 0's fence baselines them as stopgaps (rev126a N2).
- SV1 (svproc's override deleted) returns in Stage 6, after Stage 3 gives inner e-mails the chain at the API.
