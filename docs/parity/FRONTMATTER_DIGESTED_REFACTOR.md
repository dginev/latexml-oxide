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
| Observer writer/comparator re-parse `tex` | `writer/dom.rs:79`, `fmshadow/mod.rs:100` (two diverging copies); `relocate.rs:489` `shown_marks` (MARK_SWITCHES[32]); `author_block/name_text.rs:1224` `is_name_mark` | XMath shape: `XMApp[@role='FLOATSUPERSCRIPT']` last non-hint child = a mark; `POSTSUPERSCRIPT` base = text (`tex_math.rs:1980,2004`); XMHint = spacing |
| Writer cut walk-back | 88c `plan.rs` `piece_tokens`/`mark_formula_ends` (not merged) | event `Mark.formula_span` + `Mark.ends_formula` (to add) |
| Binding pre-splitters | `defs/author_macros.rs:591` `ijcai_email_lines`; `defs/general.rs:211` `\lx@split@addresses` (brace list :233-244); `frontmatter_fields.rs:117,520,547,593,742`; `relocate.rs:693-717` revtex group rule | rehearse the section; Break (non-`group` role); printed-brace depth from ink; `resolver/labels.rs:446` `addresses_in` on printed text |
| Label keys | `frontmatter_api.rs:786,799` → `queue_digest.rs:212-300` `clean_frontmatter_labels` (FONT_SWITCH[19], spacing, `\ref{`); `insert.rs:689` `unanswered_mark_text` | `Mark.label` = printed text of the digested operand (**needs a user ruling**: departs from Perl's ToString labels) |
| Old reader `author_block/` (13,150 lines; 499 cs-literal / 404 `T_*` lines) | 3A name parting (195/202: `separators.rs:585`, `rows.rs:646,690,722`, `space_parted.rs:186`); 3B marks (78/81: `marks.rs:199`, `symbols.rs:66,473`); 3C line kinds (39/41: `line_kinds.rs:261`, `links.rs:318`); 3D e-mail/address (118/67: `email_owners.rs:385,625`, `email_block_calls.rs:613,908`); 3E notes (69/13: `notes.rs:88,473,617`) | Sep/Break/Layout events (3A); Mark (3B); Contact + Name text (3C — word lists stay, on printed text); Contact + ink (3D; Semiverbatim `\email` stays raw by Perl design); Note events, accents already Unicode (3E) |
| Per-class Rust parsers (38 files) | ptephy_cls, osa_journal_sty, amsppt, aistats, interspeech, epl, eptcs, … | the binding declares its commands' events (rehearsal bindings + `policy`); migrate with each family |

Six diverging switch lists disappear with their callers: SWITCHES[33] (88c), MARK_SWITCHES[32], FONT_SWITCH[19],
NAME_TEXT_COMMANDS[38], PLAIN_FONT_COMMANDS[8], FONT_WRAPPERS[16].

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
   copies into one reader. First confirm which XMath the writer (`insert.rs:98`) and `fmshadow` (final XML) each see.
2. **Event-model additions** (gaps 1–5): additive, shadow-only; checked offline with `ab_audit fmshadow --re-resolve`.
3. **Writer cut** on `formula_span`/`ends_formula`; absorbs 88c (font-switched marks, 2105.07958; the isotope class;
   q17's mid-line unowned mark as text). Gate: the revtex/ams census A/B.
4. **Labels as printed** (after the user's ruling): every family; the marked-path guards.
5. **Pre-splitters onto the rehearsal**: ijcai e-mails, `\lx@split@addresses` → `addresses_in`, `\inst` marks,
   `\thanks`-as-affiliations, revtex's group rule.
6. **Retire the old reader family by family** through the per-family writer gate (S2b/S2c; ams_support written,
   aas_support's G5 held): notes, e-mails, marks, line kinds, name parting last.
7. **Per-class parsers** as each family is promoted.

## Test suite and measurement

- Guards: `cluster_package_guards/perfect_kernel_batch{fms2a*,fms2b,78e,78f,64zc,81,86i,88b}`, `fmshadow/tests.rs` (30),
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
