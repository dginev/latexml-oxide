# Frontmatter on the digested representation — refactor design (living)

**Directive (user, 2026-10-29).** Frontmatter feature work is frozen. Every segmentation and ownership decision moves
onto the *digested* representation, "the exact motivation for converting LaTeX to XML in the first place: reduce the
natural variations from authors into a compact semantic representation with markup and Unicode". The task is #423;
this doc is its living plan. The inventory it rests on is `~/data/pk_agents/main/agent_reports/2026-10-29_fm_inventory.md`
(read-only sweep of 3c65814c1a; 29 rows, every pointer `file:line`).

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

## What already exists

- **Rehearsal** (`frontmatter/observer/rehearsal.rs`): each queued block digested through the speculative-run engine,
  assignments rolled back, the API frozen. Its binding table (`rehearsal.rs:233-264`: `\and` → column, `\\` → break,
  `\inst` → mark) works by `\let`, as Perl's `@authorsplits` does, and stays.
- **Event lexer** (`frontmatter/observer/events/`): the digested list read into print-order events with field
  provenance — Name, Mark (any raised box; footnote symbols by the printed character), Sep (column, comma, "and",
  gap with width), Break (with role), Contact, Note (inner marks and contacts), Layout (box, cell, font change).
- **Already compliant:** the lexer's tests on printed Unicode (`lexer.rs:66,112-116,307,319,363`), the resolver
  (`resolver/`), `fmshadow/classes.rs`, `insert.rs` `respell_emails`, `cover_order.rs`, `field_ink.rs`,
  `titlepage_layout.rs`. Font switches never become ink (`font_key`, `events/mod.rs:426`).

## What reads spelling today (inventory groups)

| Group | Where (representative) | Replaced by |
|---|---|---|
| Observer writer/comparator re-parse `tex` — **done (90a, OD #911)** | `writer/dom.rs`, `fmshadow/mod.rs`, `relocate.rs` `shown_marks`, `author_block/name_text.rs` `is_name_mark` now call `observer::marks::read_formula` | XMath shape (as digested and as parsed); `relocate::printed_marks` splits a mark's printed text |
| Writer cut walk-back | 88c `plan.rs` `piece_tokens`/`mark_formula_ends` (not merged) | event `Mark.formula_span` + `Mark.ends_formula` (to add) |
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

1. A Mark's **base**: floating vs posted on ink, and **ink after it** in the same formula (`ends_formula`); today
   `events/mod.rs:800` lumps every superscript.
2. `formula_span`: the enclosing formula's or group's span (a Mark spans only its script, `deep_span` :807).
3. **Break kind**: `\par` vs line break (`\par` in a `c` column prints nothing, tex.web §1094); a binding-declared
   section break (ijcai `\emails`/`\affiliations`); ACM `\alignauthor` columns.
4. **Printed-brace depth** of a Break or Comma (ijcai, grouped local parts).
5. **Ordinal glue**: a digit with a glued lowercase superscript (`5\textsuperscript{th}`, 2609.01831).

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
2. **Event-model additions** (gaps 1–5): additive, shadow-only; checked offline with `ab_audit fmshadow --re-resolve`.
3. **Writer cut** on `formula_span`/`ends_formula`; absorbs 88c (font-switched marks, 2105.07958; the isotope class;
   q17's mid-line unowned mark as text; the q18b/x2 cut path, 90a's RED repros, among them amsart's `$\,^{1}$` address
   lines, 0804.4669's shape, `amsart_address_spaced_marks_cut_with_their_lines`) and `notes::leads_with_mark`. Gate: the
   revtex/ams census A/B.
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
   reading and holds every word today's held (round 3: declining on any doubt — no person's name made an item, no
   creator of invisible ink, no separator inside a written name); marks path 92.53 % → 94.50 %, `article` 92.39 % → 95.48 %; left
   1304.1574 — **94b, OD #937**: an in-block `\affil{$^1$…}` reads its leading Mark as its label, 2405.08158 and
   1602.05576 —), raw files' in-place contact setters (**94b, OD #936**: the API's when the file's code prints the
   argument as a title-page line or appends it to its `\author`'s store), the scorer's source reading (11), other
   families' (44).
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
