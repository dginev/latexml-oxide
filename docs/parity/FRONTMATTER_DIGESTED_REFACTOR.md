# Frontmatter on the digested representation — refactor design (living)

**Directive (user, 2026-10-29).** Frontmatter feature work is frozen. Every segmentation and ownership decision moves
onto the *digested* representation, "the exact motivation for converting LaTeX to XML in the first place: reduce the
natural variations from authors into a compact semantic representation with markup and Unicode". The task is #423;
this doc is its living plan. The inventory it rests on is `~/data/pk_agents/main/agent_reports/2026-10-29_fm_inventory.md`
(read-only sweep of 3c65814c1a; 29 rows, every pointer `file:line`).

**Status (2026-11-06):** the old engine's removal before the merge is this refactor's Stages 7-8; the batches are Z0-Z23 in [`FRONTMATTER_SINGLE_ENGINE_PLAN.md`](FRONTMATTER_SINGLE_ENGINE_PLAN.md) "The path". Stages 1 and 4 landed; Stage 2's gaps 1, 2, 6, 8, 9 landed with Z1, its gaps 3, 4, 5, 7, 10 with Z2, Stage 3 with Z3; Stage 5 has not.

## The principle

1. A decision reads **what TeX made of the input**: the observer's events, or the digested XML (`ltx:sup`,
   `ltx:break`, `ltx:note`, XMath structure, font attributes, Unicode characters). It never reads TeX token or source
   spelling. After digestion `\{`, `\lbrace`, `\textbraceleft` are one "{"; `\rm`, `\bf`, `\scriptstyle` are a font
   property, never ink; `\hspace{1mm}` is glue.
2. What digestion cannot settle (a prescript `${}^{3}$He` and a mark ³ print the same) is decided by a **structural
   fail-safe**, never by a finer reading: uncertain stays as written (the readability tier, `FIDELITY_CHARTER`).
3. The binding API (`\lx@add@*`, its labels) stays the parity boundary with Perl. Perl itself splits by spelling
   (`Base_Utility.pool.ltxml:679-726`, `ToString(...) =~ /\{tabular\}/`; labels as source text, `cleanFrontmatterLabels`
   :365), so the migration is a documented divergence under OD #837, entry by entry.
4. **Tokens are natural at the API; segmentation is what moves (user 2026-11-06).** "My main goal was to do segmentation on complex TeX, such as the various macros for marks, on the digested layer - to avoid dealing with TeX's full diversity via heuristics. But when it comes to argument passing into the Frontmatter API, the token data is natural." As in Perl, whose `\lx@annotate@frontmatter` queues its arguments without looking inside (`Base_Utility.pool.ltxml:470-474`). So a binding reading its own class's arguments and passing them to the API is not a defect, nor are tokens as an item's content. The defect is reading marks, lines, kinds and owners out of those tokens by spelling: `author_block/` holds 781 control-sequence tests, 277 distinct, and the Cut and Kind steps Rust added at the API's entry (OD #954, #956).

## What already exists

- **Rehearsal** (`frontmatter/observer/rehearsal.rs`): each queued block digested through the speculative-run engine,
  assignments rolled back, the API frozen. Its binding table (`rehearsal.rs:233-264`: `\and` → column, `\\` → break,
  `\inst` → mark) works by `\let`, as Perl's `@authorsplits` does, and stays.
- **Event lexer** (`frontmatter/observer/events/`): the digested list read into print-order events with field
  provenance — Name, Mark (any raised box; footnote symbols by the printed character; its formula or group, base and
  end, Z1), Sep (column, comma, "and", gap with width), Break (with role), Contact, Note (inner marks and contacts),
  Layout (box, cell, font change); beside them Declaration and the held errors (Z1).
- **Already compliant:** the lexer's tests on printed Unicode (`lexer.rs:66,112-116,307,319,363`), the resolver
  (`resolver/`), `fmshadow/classes.rs`, `insert.rs` `respell_emails`, `cover_order.rs`, `field_ink.rs`,
  `titlepage_layout.rs`. Font switches never become ink (`font_key`, `events/mod.rs:426`).

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

## Gaps in the event model (to add, shadow-only first)

Gaps 1-10 are in the dump, read from the digested list, its spans and the rehearsal's notes; only gap 5's `ordinal` has a reader. Guards `perfect_kernel_batchz1`, `perfect_kernel_batchz2`, the dump's reader `fmshadow::tests::a_dump_reads_the_marks_formula_and_an_old_one_reads_none`; repros `sectioning-frontmatter/event_*`.

1. A Mark's **base** — **done (Z1):** `Event::formula` (`events::MarkFormula`), dump `base`, `ends_formula`. In a formula `base` is `floating` or `posted`, by what prints before the mark there, the rule `marks::read_formula` reads the XMath by (spacing, a blank box, a font switch and an empty group are no base; past the first script, punctuation and symbols neither). `ends_formula`: no letter or digit prints after it there. A prescript and a mark print the same, so `${}^{3}$He` reads floating and ending its formula, `${}^{3}\mathrm{He}$` not ending it: the fail-safe's input, no decision (2105.07958, 0804.4669). A mark outside a formula has no base.
2. `formula_span` — **done (Z1, tightened Z3):** the whole formula's span, its opening shift (the box is stamped by the closing shift, whose invocation builds it, so `document::FIELD_FORMULA_START_PROPERTY` records the opening's under the notes alone) to past its closing shift, read in one step from those two (`events::formula_extent`): no walk of its boxes, and no first box's cover, which is the wrapper's in `\textbf{$^2$Univ B}` (the Mark's own span stays so); else the innermost group's or box's; none at the block's top level. A group's span stops at its last box: its closing brace is covered by no box. An API call's content has no positions: `formula_span` null, base and `ends_formula` read.
3. **Break kind** — **done (Z2):** `Event::break_kind` (`events::BreakKind`), dump `break`: `par` on a Break `\par` made, none on a line break, its `role` as it was: a `c` column prints nothing for `\par` (tex.web §1094; `\lx@fm@obs@par`, bound as `\lx@fm@obs@break` is). A binding-declared section (ijcai's `\emails`, `\affiliations`) or column (OmniBus's `\alignauthor`, in the conventions of inst_support, which owns its `\author`; the kernel declares none): `Policy::block_breaks`, each call noted where the expansion is heard (`field_notes::FieldBreak`, `FieldInvocation::Expanded`), dump `breaks` on the block, beside its events, which it does not part: what the command prints stays what the class makes it (OmniBus's no-op; ijcai's split takes `\affiliations` out before the rehearsal, so only a call in the names' part reaches one). Guards `perfect_kernel_batchz2::{a_break_says_what_made_it, a_break_a_binding_declares_stands_beside_the_events}`; repros `event_break_*`. A raw class's own `\alignauthor` (sig-alternate.cls) declares nothing: it reads as the tabular's end and fill it prints. `\endgraf`, `\let\pp\par` and `\@@par` give no Break: RED `event_a_par_by_another_name_is_a_par_break`.
4. **Printed-brace depth** — **done (Z2):** `Event::braces`, dump `braces`, on a Break or a comma: the `{` printed and not closed since the column's start, by whatever printed them (`\{`, a formula's `\lbrace`, `\textbraceleft`), a line break no end of them (`\{ann, bob,\\ cat\}@x.org`), a TeX group none. Guard `a_break_and_a_comma_say_their_printed_braces`.
5. **Ordinal glue** — **done (Z2):** `Event::ordinal`, dump `ordinal`, on a Mark printing `st`, `nd`, `rd` or `th` right after a Name ending in a digit, glued or spaced, in text or a formula's script. A reading rule of printed text, no token read. Guard `an_ordinal_says_so`. 2609.01831's source has no ordinal: the shape it was cited for is the repro `affiliation_line_superscript_ordinal_is_not_a_mark`.
6. **Unprinted contacts** — **done (Z1), one residual:** `ledger::UnprintedCall`, dump top-level `unprinted`: each `ltx:rdf` call annotating a creator a Contact with `Event::unprinted` holding what it records, `role` the property, `text` empty; beside `Ledger::entries`, so no index moves and the reading does not see them (2506.18556, 2507.12222; OD #479). Residual: a block's `\email` under AASTeX 7 (`\author{Jane Doe, \email{x}}`) is still a printing Contact in the block's events: the rehearsal binds `\email` before the class decides, by `[show]` and its state. Its real call is in `unprinted` beside it. Minimal input: repro `aastex7_email_inside_author`; none in the 373 AASTeX 7 papers of OD #479.
7. **Mid-line hand-printed labels** — **done (Z2):** `Event::printed_labels` (`events::PrintedLabel`), dump `printed_labels` on a Name: each word `a)`, `12)` it prints, one or two letters or digits with no bracket open before it, with which word, whether first on its line, and its span. Beside the Name's text, which stays whole: the Cut does not read them yet (100a's RED, hep-ex0106079). Guard `a_name_says_its_printed_labels`.
8. **Declarations** — **done (Z1, Z2):** `Outcome::Events::declarations`, dump `declarations` on the block, beside its events: `EventKind::Declaration`, its span the call's and what leads to it (`\global`, `\long`, `\expandafter`: a read of the span's content is the definition), `label` the definees, each once, `role` `global`, `nest` its groups. A definition makes no box, so it is not read from a box: it is read from the meaning assignment itself (`state::begin_meaning_log`) and the invocation that made it (`observer/field_notes.rs`, called through core `token::FieldNotesHook` so that no line of it sits in the core's codegen units), never from the tokens' spelling or a list of names.
   - **What the log holds:** what the token being invoked assigned, with a field provenance, and it alone (`MeaningAssigned::by`): not `\begin`'s hooks, nor `\size@update`, which `\selectfont` puts back with none and which 2608.17767's `\small\itshape` read as `\itshape`'s.
   - **Whose it is:** a definer invoked directly declares what it assigns, unless it reads no parameter (`\selectfont`, whose `pickup_font` loads a font and sets `\font@name`: it names no definee). A call of a macro the paper or a file it read defined (`Document`, `File`) declares what its expansion assigns. In the run of a kernel's or a binding's macro it reached (`FieldNotes::kernel_runs`, told by the pushback falling below what it held before that expansion) it declares only what its arguments name and, the document's own, what its body or a document's macro in its body names (`names_in_a_papers_macro`): `\newcommand\m{\newcount\qq}` and `\newcommand\m{\@ifundefined{qq}{\def\qq{1}}{}}` declare `\qq`, `\newcommand\lxaff{\small\itshape}` nothing, `\itshape` being made by `\reinstall@nfss@defs` at `\begin{document}` under the document's origin. A kernel's or a binding's macro called directly (`\bfseries`, `\centering`, `\newcount`) declares only the names the field handed it (`\nn` of `\newcount\nn`).
   - **Where it starts:** at the prefixes and `\expandafter`s invoked right before it (`FieldNotes::lead`), by invocations, not positions: a `\global` a `\let` or a test read leads nothing, a `\global` waiting past a `\relax` or a paper's macro does.
   - **Boxes:** one digested while another field token is being invoked around it is that box's and none (`stomach::field_token_encloses`: a formula, `\thanks`, `\emph`, `\mbox`, `\parbox`, an environment, `\sbox`). A plain group's stays, and so does one in `\textbf`, whose macro opens a group (`\textbf{Ann\def\x{1}Bob}`: `nest` 1), and a macro printing ink has none (a box inside the reach of its call takes it back). Right before a box it stands: that box's cover holds the boxless invocation before it, which no longer matters.
   - **The origin of `\itshape`** stays the document's: the run above is what a paper's macro needs anyway, and 41 readers of the origin make the change no cheap fix (Z2's report).
   - **Where it ends:** one past what the invocation read. A definer whose next token is no later one of the field (a macro's own `\relax`, the end of an argument) read as far as the call around it read, and stops before a closing brace of what opened before it: `\newcommand\pass[1]{#1\relax}\pass{\def\x{1}}` [2,7].
   - **Settled for a replay (Z3, `field_notes::settle_declarations`):** the spans are in order, balanced, and none reads what another reads. A definition in the body of a macro the field defined runs at the macro's call, its tokens at their own positions inside the braces of what defined them: `\def\m{\newcount\qq}\m`'s `\qq` is `\m`'s call [6,7], not [3,7] over `\def\m…`'s [0,6]; with nothing of its own left, `\def\x{\def\y{1}\def\z{2}}\x`'s `\y` goes with `\z`'s call. A macro that peeked at the next token (`\mstar\mfile`, `\@ifstar`) ends where that token's own call starts.
   - **Limits:** what is no meaning (`\setcounter`); a macro of a `.sty` or `.cls` the paper loaded, in a kernel macro's run, declares only what its arguments name, since a raw package's body names its own state (`\makecell`'s `\tabg@pe`: 2203.03195, 2502.11387), so `\newcommand\m{\newcount\qq}` in such a file read raw is unread (`\input{defs}` is the document's, and reads as declared); a name a kernel macro builds, handed to it as letters (`\@namedef{qq}{1}` directly, RED `event_declaration_a_kernel_macro_building_a_name`; through a paper's macro it reads, `\@namedef`'s run having ended when its `\def` comes); a raw package's own assignments read as the paper's (`\ifthenelse`'s `\@tempa`, beside the real `\x`: RED `event_declaration_through_ifthenelse`); a macro assigning at expansion time, before the next invocation (a binding's `DefMacro` closure); a paper's macro that defines and then makes a box is that box's (`\def\a{M}\small`); a raw class's own font switch reads as the paper's (`\small` defining `\@listi`: 1905.04513, 2110.04730, 2410.03307, 2412.17598, 2609.38967; 2609.35561's `\rmdef@ult`: 6 of dev3's 10,000), a span the emitter would replay unharmed.
   Guards `perfect_kernel_batchz1::a_definition_in_the_block_is_an_event_with_its_span`, `a_declarations_span_holds_what_leads_to_it`; `perfect_kernel_batchz2::{a_font_switch_declares_nothing_and_a_name_is_no_reason, a_definition_is_the_blocks_unless_a_box_holds_it, a_declarations_lead_is_invocations_not_positions}`.
9. **Held errors** — **done (Z1):** the block keeps all its events, as it always did: a rehearsal goes on past an error it holds (`Rehearsal::Done`), so the events are whole, not partial. What was missing is added: `Event::held` on the events an error's position touches, `Outcome::Events::held` the errors by position and log line, `ApiCall::errors` the count for a call's content, which has no positions. Kinds: **Done with held Errors or Warnings: all events, marked.** Fatal (`Skipped(Fatal)`, digestion returned an error, so no digested list), NotUndone (the state cannot be trusted) and JournalUnavailable (no digestion): none, unchanged. Overflow and Empty: none, no positions or no tokens. **Root cause of the study's held papers (2603.28168, 1405.3507, 2201.04449):** the error is the observer's own, not the document's: its constructors digest the API's `_cutlist=…` options and a `\corref` label, `_` in text mode. Census in `SYNC_STATUS.md`'s Z1 row.
10. **llncs `\orcidID` as a contact** — **done, no change (Z2):** in a block `\orcidID`, `\lx@add@orcid`, reads as a Contact of role `orcid`, its span the call, no name's text; in an API creator's content (2408.15810's `\orcidlink`, the `\author` of llncs) a Contact too. The resolver's API reading (`api:creator`) prints it in the name: Z13's. Guard `an_orcid_is_a_contact_in_the_events`.

**The writer's regressions, read from events (Z2, E1 of the re-baseline; `perfect_kernel_batchz2`):**
- A control space or a tie is its font's word space at any width in em (`events::is_word_space_of_its_font`): aa's `~` at `\tiny` is 0.347 em, typewriter's `\ ` 0.5, each cut a name (1702.08280 "M. de", "Naurois"). A wider skip still parts two names.
- A symbol glued between two words on a line a mark leads, an entry, is the word's (`lexer::glue_in_word_marks`), whatever else shows that symbol: 2401.15006's `$^6$A$^*$STAR`, `$^1$IHPC A$^*$STAR`. On a line no mark leads it stays a mark: `Cher$^*$Bob Baker`, with a legend or none.
- An address every carrier spells alone, under each by an entry of its own, is each entry's (`answer_inner_emails`): `\IEEEauthorblockN{Ann Lee}\IEEEauthorblockA{lee@a.org}\and\IEEEauthorblockN{Bob Lee}\IEEEauthorblockA{lee@a.org}`, as the PDF prints it. One entry two Lees share stays the block's.
- The last `\author` wins in a binding's body too (`queue::begin_author_body`, IEEEtran's blocks, which queue each creator and contact at their `\author` where Perl's holds them in one deferred creator): 2606.16069's two calls, 4 creators in the PDF and Perl, 8 before. A plain call is such a body too (`\lx@IEEE@author@plain`): `\lx@add@authors` takes back a creator, not the contacts a marked call queued flat. A replayed class setter leaves the body as it stands (`frontmatter_frozen_for_replay`). Guards `the_last_author_call_wins_in_the_blocks_path`, `a_plain_author_call_is_a_body_like_the_blocks`, `a_class_setters_replay_leaves_the_author_body`.
- An ordinal is its entry's text (`Event::ordinal`, `events_text`, `entries_of`, `mark_led_pieces`), on any line but a names line, whose `1st` stays the name's prefix (`join_ordinals`, 2412.10792, 2202.06596). Today's view reads it too (`writer::dom::ordinal_suffix`), so a call holding one no longer declines the paper. `2609.01831` has no ordinal in its source: its cited shape is the repro `reading_ordinal_in_a_call_reads_as_todays`. RED: `ordinal_in_a_group_or_with_a_font_keeps_its_letters`, `ordinal_names_on_two_lines_are_two_names`.

## What a rollback leaves (census, Z3)

A rehearsal runs a block's tokens and undoes them; so do the title-page checks (`sect05.rs` `speculative_run`) and `queue_digest::speculative_label_ink`. The hostile suite (`perfect_kernel_batchz3::the_rehearsal_leaves_no_trace_*`, repro `observer_rehearsal_leaves_no_trace`; the study's, `kit/rehearsal_iso.py`) runs each input with the observer on and off, each in a directory of its own, and compares the XML outside the creators, PIs kept, what was written, E, F and W. What runs in the block runs twice with it on, so an input that builds on its own value (`\global\advance`, `\read`, `\fontdimen2\font=\dimexpr\fontdimen2\font+1pt`) shows what a rollback left behind; an idempotent one hides it, unless it shows what it found before what it repeats (`\xdef\qd{\pdfuniformdeviate 1000}\pdfsetrandomseed 5`). On and off agree where the title-page check's run is as speculative as the rehearsal, so a row may need pdflatex's value (`\read` before `\closein`). A package's macros are defined in the preamble: in `\author`'s argument `_` is tokenized already, and tikz fails in the body, so such a row ran nothing. The study's inputs, on article, revtex4-1, amsart, a raw class and a class printing no author, with `\maketitle` at the top level, in a group, an environment and braces, in the block, in a note, in a cut `\affiliation`, and in a first `\author` a second replaces (rehearsed, never printed), are the probes behind this table; the guard keeps one row per mechanism. Each guard row of `the_rehearsal_leaves_no_trace_outside_its_journal` goes red on `int142a`, `ISO_GUARD`.

| State | What keeps it | Test or reason |
|---|---|---|
| Values, meanings, catcodes and the other code tables, `\count`, `\dimen`, `\skip`, `\toks`, `\box` bindings, counters, VFS files (`\openout`, `\write`), `\aftergroup`, `\AtEndDocument`, `afterAssignment` | `AssignmentJournalGuard` | `…_of_its_assignments`, `…_in_files` |
| A font's parameter array, edited in place | **Fixed (Z3):** saved as a copy (`journal_save`, `set_font_parameter`). It ran twice | `…_outside_its_journal` `fontdimen` |
| A box register's `\wd`, `\ht`, `\dp`, properties of the box, which no binding holds | **Fixed (Z3):** `journal_box_dimension`, put back at the rollback. It ran twice | `box width` |
| An `\openin` stream's mouth: `\read`, `\readline`, `\closein`, the `\openin` that closes it | **Fixed (Z3):** the first change under a journal works on a copy (`state::input_stream_to_change`, `Mouth::copy_for_a_rollback`, a file reopened at its offset). A rehearsal took the document's next line | `read`, `readline`, `closein`, `openin` |
| The pdfTeX generator (`\pdfuniformdeviate`, `\pdfsetrandomseed`; a thread-local) | **Fixed (Z3):** a speculative digestion has a generator of its own. An Info quoting the title page's words may show another draw: words of a run no one prints | `random`, `random seed` |
| Keyvals' "seen" set of missing keys (a thread-local) | **Fixed (Z3):** none is recorded in a speculative digestion. The document's own Warning went unsaid | `missing key` |
| Groups, `\begingroup`, math, boxes and modes left open | Unwound (`pop_stack_frame`, `box_lists_mark`); frames changed under the journal say NotUndone, never seen in the study | `…_of_the_flow` |
| Conditionals, the tokens being invoked | `StomachMark`; a sequence's blocks share conditionals, put back after the last | `if`, `fi` |
| Mouth stack, pushback | `reading_from_mouth`, a mouth of its own; a macro reading past the block's end meets its end (`\textbf`, `\def`, `\count255=`). R1 closes by depth a mouth a stopped `\scantokens` left | the study; R1's `check_budget_run_leaves_no_trace` |
| Document end (`\end{document}`, `\stop`, `\@@end`, `\bye`) | `gullet::speculative_digestion` | `…_of_the_flow` |
| Prefixes (`\global` last in a block) | The block's own group, at `egroup` | the study: none shows |
| Diagnostics, their counters, `LAST_ERROR_KEY`, the too-many latch | `DiagnosticsHold`, `save_diagnostic_gates` | every row's E, F, W |
| Terminal and log output, an `\input`'s progress note | `SuppressLogOutputGuard`, `SilenceProgressNotesGuard` | the study's log diff: only the Info above |
| Field ledger, pending cover, stamps on boxes | `FieldLedgerScope` | `observer_rehearsal_leaves_a_used_box_unchanged` |
| The frontmatter queue, the ledger | Frozen (`lx_replaying_class_setter`), `rehearsing()` | the study's `\title` and `\maketitle` in a block |
| `\write18` | Not run by this engine, in either run | `write18` |
| The arena | Append-only; a name is never read back as state, only its value | Reason |
| Caches: TFM, glyph coverage, kpathsea memo, image dimensions, `NOEXPAND_FAMILY_MEMO`, font maps | Memos of what the input and the tree say, the same for either run | Reason |
| Thread-local counters: `FRAME_SERIAL`, `ABANDONED_FRAMES`, `ENDS_INSIDE_GROUPS`, `NEXT_FRAME_ID`, `SPLICE_SERIAL`, `RAW_REFSTEP_SERIALS` | Each an identity or a tally: compared for equality, or read as a difference across one run, never printed | Reason; `refstepcounter` |
| `PENDING_STEPS`, `REFSTEP_PROPS` (a counter's last step) | Keyed by counter, overwritten by the next step; the real digestion steps the same counter first | Reason; `refstepcounter` |
| `RAW_READ_PATHS` | **Fixed (Z3r2):** a speculative run's raw read is no read (`load_tex_definitions`, as `<request>_raw_loaded`, undone with it). A block nothing prints read a stand-in's copy, and the document's own `\input` skipped it (`read_stand_in_copy`: `\author{First\input{aas_macros.sty}}\author{Ann}\maketitle \input{aas_macros.sty}\kmsx`, undefined) | `a_rehearsals_read_is_no_read_of_a_copy` |
| `IN_PROGRESS`, the class and package load depths, `MODEL`, `tag_properties`, `pending_resources` | A package a block loads or a resource it asks for is the real run's too: the same set, in the same order, the first being the rehearsal's | Reason; `input real`, the study's `\input{lipsum.sty}` |
| A binding's document-wide state: pict2e's graph; chemnum's counter and first uses; `ANIM_STACK`; `BLX`, `BBL_ENTRY`, `BBL_DATALISTS`, `BBL_READ`; a stub's `REPORTED` | **Fixed (Z3r2):** one value of the Value table (`state::DocumentState`), bound globally, as the package's own macros and registers are (pict2e.sty:186 `\global\pIIe@GRAPH`), so a rollback puts it back; in place outside a journal, a copy at the first change under one. A path stroked once more at the body's `\strokepath`, on and off; `\cmpd`'s first use spent (`<ref>` with no target) and its number; a category filed; a stub's Warning unsaid | `a_bindings_state_stands_for_one_run` |
| The Lua interpreter (`lua_bridge::LUA`) | **Fixed (Z3r2):** a speculative run evaluates no Lua: no rollback reaches an interpreter's globals. What a chunk prints is not in the rehearsal's reading, so a cut that needs it can go too: `$^{1}$\luaexec{tex.print("Univ X")}, $^{2}$Univ Y`, two entries on the base, one now, nothing lost (RED `rehearsal_no_lua_loses_a_cut_red`) | `a_speculative_run_evaluates_no_lua` |
| The observer's note of a binding's load (`OWNERS`, `CLASS_BINDINGS`, `CLASS_AUTHOR`, `LOADED`) | **Fixed (Z3r2):** a speculative load is not noted; the document's own is | Reason |
| `NICE_RECTS`, `NICE_FIRST_ROW`, `NICE_FIRST_COL`, `NICE_NAME` | Set at each `\begin{NiceMatrix}`'s `\lx@nice@setopts`, read by that matrix alone: no later run reads a rehearsal's | Reason; `nice` |
| `PENDING_FOREST_TREES`, `LIBRARIES_LATER` | A hand-off between two hooks of one `\begin{forest}`, a LIFO: an abandoned run's stays below, never popped. Filled and taken within one package load | Reason; `forest` |
| `CACHE` (minted), `SCRIPT_CACHE`, `UNIT_DEFN_CACHE`, `six_pin!`, datatool's regexes, `VARIABLE`, `KERNEL_CS`, `IN_THREAD_CACHE` | Memos of what the tree or a key's value says | Reason |
| `COLLAPSING_UNITS`, `CURRENT_SCRIPT`, `CTOR_CTX`, `WHATSIT_CTX`, `MOVEABLE_BOX_DEPTH`, `READING_LABEL`, `WIRED_COUNT` | Pushed and popped around one call, on its failure too (drop guards, `result` taken before `?`) | Reason |
| `ESCAPE_SEQ`, `BODY_SEQ` (listings) | Serials: part of a key (`lx@lst@escape@<id>`, journaled) and of a virtual file's name (`lx-<env>-<n>.tmp`), printed in no XML; a rehearsal moves them, the log's progress note alone shows it | Reason; `lst` |
| `BIB_ENTRIES`, `CURRENT_ENTRY_KEY`, `WANTED_KEYS` | Keyed by the entry's key, its value what the same read registers; each bibliography's read registers what it then reads | Reason; `bibliography_file`, `blx` |
| `INTARRAYS` (expl3's Lua) | Dead: the native bindings are off, expl3-code.tex keeps them as font parameters, journaled | Reason; `e_intarray` |
| `ENDTRIVLIST_OWNER`, `NET_HOOKS_BEFORE`, `REPLAY_CAPTURED`, `REPLAY_BODY_STORES`, `BUILDING_FORMAT`, `NATIVE_OVERRIDE` (datatool, pgfkeys), `SPLICE_SERIAL`, `SPLICE_STARTS`, `BLOCK_CONTENT`, `FIELD_NOTES`, `LEDGER` | A hand-off within one invocation; around `\begin{document}`'s hooks, a replay or a format build; a test's choice; the absorption no rehearsal makes; the observer's own, frozen while it rehearses | Reason |
| `DOCUMENT_CALLS`, `STORE_CALLS`, `DROPPED_ARGUMENTS` (the title-page check's notes) | Watched only inside the check's own window | R1's; the study's raw class |

**The suite, on Z3r2** (`kit/rehearsal_iso.py`): 1,670 rows on 5 classes `bad 0`; 668 in a first `\author` a second replaces, `bad 0`; 334 in a cut `\affiliation` under two authors, `bad 0`, and under one, the writer's path, 5 rows, below. An anonymous `xml:id` numbering on from the creators', which differ, is named `ids`, no difference of content: 7 rows.

**A cut field's second digestion.** The writer digests a cut entry again for real, after the queue's digestion of the whole call (`writer::apply`, `digest_frontmatter_item`). So what the content errs on is said twice: `ifthen` E 2 on, 1 off, `setbox_used` 4/2, `unbalanced_noalign` 2/1; a Warning in `unbalanced_left`, `jobname` 2/1. None is lost, the XML is the same; a hold would hide what only the piece errs on. RED `writer_second_digestion_says_an_error_twice_red`. It moves auto-numbered `xml:id`s past the title (`logical-block`'s `id4`/`id5`): `font`, `equation`, `itemize`, `trivlist2`, 9 rows more. A run that ended inside an environment lost its end, in an Error (`long_tab`, `nice`, `newenv`, `tabular`): **fixed**, `writer::Field::balanced` holds them in pairs.

**A run's environments, and the groups it starts at (review revZ3r2).** A mark's formula that opens a group at the field's start or right after a line break starts that group at its brace, by what digestion made (`Walker::note_a_formulas_group`, `Formula::opens_line_in`, 2005.08290); after a comma it stays the fail-safe, so `Univ A, {$^{2}$H} Lab` stays Ann's. Left, each cut on the base and uncut now, nothing lost: RED `affiliation_cut_group_siblings_stay_uncut_red` (`{\textsuperscript{1}} Univ X`, `{$^1$2} Univ X`, `{$^{1}$ Univ X,} Moscow, {$^{2}$Univ Y,} Moscow`; `~\\ {\textsuperscript{1}Univ X,} Moscow\\ …` onto a nameless creator). A run's environments come from their meanings and the digestion's frames since OD #979 (`field_notes::EnvironmentStep`, `writer::Field::environments`): a macro pair and a `\let` copy count, a `\begin` under `\detokenize` or `\ifx` doesn't (`perfect_kernel_batchz4::an_environment_by_its_frames`).

**A loop is no trace but a cost: the read budget.** `rehearsal::rehearsal_budget`, R1's mechanism (`gullet::read_budget`): 8,000,000 reads a rehearsal, three times that a conversion (`LATEXML_REHEARSAL_READ_BUDGET`, 0 for none). Past it the run is abandoned and rolled back, `SkipReason::OverBudget`, never a Fatal, and one `Warning:frontmatter:rehearsal_budget` at the document's close counts them. By the data (popZ01, 1,707 papers rehearsing, `--debug=frontmatter`'s `FRONT a rehearsal read`): dearest rehearsal 166 reads at the median, 8,417 at the 99th percentile, 3,359,702 at most (2608.18287; 2011.00785 3,314,577); a conversion 731, 20,444, 3,360,403. The many-author papers are far under the total: 1901.01540 (1,202 authors) 2,637 rehearsals 118,996 reads, 2103.04797 (975) 2,136 and 92,093, 2005.08745 (592) 668 and 13,009. A block looping where nothing prints it (`\author{First \qloop}\author{Ann}`) cost 76.7 G instructions on Z3, now 32 G. Guard `a_looping_block_ends_in_the_rehearsal_budget`.

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

## Test suite and measurement

- Guards: `cluster_package_guards/perfect_kernel_batch{fms2a*,fms2b,78e,78f,64zc,81,86i,88b,90a,92a}`,
  `fmshadow/tests.rs` (30),
  `author_split_tests.rs` (32), `114_streaming_structure`. Repros: `tools/perfect_kernel/repros/class-bindings/` and
  `sectioning-frontmatter/` (name groups: `*and*` 103, `*symbol*` 66, `*thanks*`/`*footnote*` 30, `*email*` 27, …).
- The held batches' witnesses and probes join the suite: 88a (`pk/88a`, #422: e-mail spelling, t06–t11, q01–q12,
  2606.02915), 88c (`pk/88c` 377529655a, never merged: 2105.07958, 1410.8534, 1804.10344, 1810.00507,
  astro-ph9807043, 2002.12524, 2006.00536; its 20 repros incl. the isotope and amsart-legend probes f4/f8/f9/a4/h2),
  88d (#421), aas G5 (#390), 86h (#412), 86e, 86f (#405).
- Each stage: red/green repros first; the scorer's two tiers on the 3,000 sample; a census A/B with nothing worse.

## Superseded

`FRONTMATTER_ARCHITECTURE_AUDIT_2026-10-28.md` rank-7 ("export a token toolkit for the bindings", :108, :151)
conflicts with this directive and is withdrawn.
