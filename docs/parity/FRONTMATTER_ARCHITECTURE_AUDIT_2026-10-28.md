# Frontmatter architecture audit (2026-10-28)

> A dated diagnostic snapshot (perfect_kernel `501e8ba4f1`), not a live worklist: the live rows are in
> `docs/SYNC_STATUS.md` ("Frontmatter architecture: the audit's plan"). Read it before changing how frontmatter is
> queued, segmented, owned, placed or written.

**The principle audited** (user, 2026-10-28): "I want us to be very careful in using the Frontmatter API, so that any
changes can globally propagate to all bindings. The split and reorganization logic we are adding for hard special cases
needs to be compartmentalized and local so that we can reliably continue to provide high quality conversions on
documents using previously unseen packages and classes" — clarified: "the 'local only' rule I mentioned was about
localizing treatments for special cases. I definitely want us to have generalized abstractions for reorganizing
frontmatter into semantic XML, at all stages of processing". As rules:
- **R1** bindings feed frontmatter only through the Perl-shaped Frontmatter API (`\lx@add@*`, the queue, placement at
  `\lx@frontmatterhere`; Base_Utility.pool.ltxml:272-446, :824-944), so a fix in the API reaches every binding;
- **R2** a treatment for one class's or one witness's shape is local — in that class's binding, or behind data a binding
  sets through the API (the `_share` opt-in, OD #901) — never a branch inside a general mechanism or keyed on a class name
  in generic code;
- **R3** general mechanisms that reorganize frontmatter into semantic XML are wanted at every stage; they are general (not
  accumulations of special cases), fail-safe, documented in OXIDIZED_DESIGN_DIVERGENCES, and switchable as one unit.

## Bottom line

- **The pipeline matches Perl's design, and the Rust-only API extensions are opt-in.** Bindings queue frontmatter through `\lx@add@*` / `\lx@annotate@frontmatter`, the flush digests the queue, and the placement step inserts it.
  - About 50 Rust-only `\lx@…` API entries exist. They are carried as data a binding sets: `\lx@add@contact@placed[owner=sequence, mark=address]`, `_share=authors`, `\lx@add@affiliation@shared`, `\lx@title@inst`, `declare_fnsymbol_title_block`.
  - `share_row_contacts` (`frontmatter/shared_row.rs:55-110`, called at `place.rs:428`) is the model. It returns at once unless an entry carries `_share`, and it hands the queue back unchanged when nothing is shaped.
- **The R2 problem is concentrated in three places:**
  1. the observer's policy table, keyed on binding file names in generic code;
  2. class vocabulary hard-coded in the generic author-block reader, in about 7 near-duplicate lists;
  3. six contrib bindings that each re-implement the reader's token primitives.
- **R1 bypasses are few.** About 12 contrib bindings plus svmult write `<ltx:classification>`, `<ltx:abstract>` or `<ltx:creator>` in place, at the call site.
- **No general mechanism has a kill switch except the observer** (`LATEXML_FM_SHADOW=0`). In particular, 86g has none.

## 1. Map of the frontmatter mechanisms

| Mechanism | Entry point | Trigger | Perl counterpart | Size (lines) | Verdict |
|---|---|---|---|---|---|
| Queue / dequeue / digest | `queue.rs:18` `queue_front_matter`, `place.rs:403` `digest_front_matter` | the API | `queueFrontMatter` / `digestFrontMatter`, Base_Utility.pool.ltxml:272-446 | 1.1k | R1 ✓. Pre-clearing the queue is documented (KPE #30). |
| Sequenced contacts | `place.rs:42` `sequenced_contact_labels` | `owner=sequence` (jheppub, iopart, lmcs, osa) | jheppub `labelseq=author` | ~350 | R2 ✓ (opt-in). Witness-heavy, but only on that opt-in path. |
| Row-shared contact | `shared_row.rs:55` (+ `shared_row/` 1.2k) | `_share` (llncs, revtex, sv, amsppt) | none (KPE #1195) | 1.5k | R2 ✓, the model. |
| Kernel author reader | `author_block/dispatch.rs:19` via `\lx@add@authors*` (`author_macros.rs:40-130`) | every `\author` | `\lx@add@authors`, Base_Utility.pool.ltxml:687-727 (~40 lines) | **13.1k** | General by design (Perl's one global heuristic, expanded). R2 ✗ in places (§3a). |
| Raw-class store reroute (K11) | `frontmatter_stores.rs:1397` `reroute_raw_class_stores`, `:4297` `harvest_stores` | raw-loaded classes, setter name in a 655-class survey table (`latexml_core/src/binding/store_setters.rs:22`) + a body check | OmniBus.cls.ltxml:62-247 | 4.7k | General ✓. No class-name keys in its code (grep: none outside comments). |
| Title-page deposit, speculative runs | `sect05.rs:1336-1614` `\lx@deposit@maketitle` | every `\maketitle` | none (#124, #265) | ~400 | General ✓. |
| Deposit `\insert` | `tex_inserts.rs:30` | `lx_depositing_frontmatter_fields` only | Perl: no-op | 50 | General ✓, gated. |
| Placement | `place.rs:782` `place_frontmatter` | every flush | `\lx@frontmatterhere` / `\lx@frontmatter@fallback` (:918, :924) | ~200 | General, documented (#247/#262). No switch. |
| Late-abstract cover (86g) | `cover_order.rs:206`, called from `cover_order.rs:160,179` and `author_macros.rs:523` | every document with a late abstract | none (#903) | 382 | General (§4). No switch. |
| Title layout, promotion, ink | `titlepage_layout.rs`, `field_ink.rs` | every document | none (#244/#246/#761) | 1.5k | General, user-ruled. No switch. |
| Insert-time passes | `insert.rs:93-104`: `coalesce_empty_creators`, `distribute_upfront_contacts` (:276), `respell_emails` (:140), `show_unanswered_marks`, `drop_empty_contacts` | every document | only `relocateAnnotations` | 720 + relocate 780 | General. `distribute_upfront_contacts` is position-based (§3a-5). |
| Observer (M3) and writer (S2b) | `observer/`, `insert.rs:103` `write_observed_frontmatter` | every document, rewriting output only for families with `written: true` | none (#837) | **10.2k** | **R2 ✗**: the policy table is keyed on binding names (§3a-1). The only mechanism with a switch. |

## 2. API bypasses (finding c)

All of these are SUSPECTED by reading: a construct placed at the call site rather than queued.

1. **Contrib bindings with in-place frontmatter constructors.** The element is built where the environment stands, so it is not queued, not ordered by `FRONTMATTER_ELEMENTS`, and not reached by API fixes.
   - `<ltx:classification>` constructors:
     - `latexml_contrib/src/siamart_cls.rs:93,101,163,167,171`
     - `interact_cls.rs:102,174,178,182`
     - `jmlr_cls.rs:170`, `jmlr2e_sty.rs:131`
     - `ieeeojcsys_cls.rs:52`, `ieeetaes_cls.rs:55`
     - `agujournal2019_cls.rs:68`
   - `optica_article_cls.rs:37`: `{abstract*}` builds `<ltx:abstract>` in place.
   - `dmtcs_episciences_cls.rs:66`: `^ <ltx:contact role='affiliation'>`.
   - Perl's own bindings almost never do this: grep finds 5 such lines in all of `LaTeXML/lib/LaTeXML/Package/*.ltxml`, 4 of them moderncv and 1 commented out in acmart.
   - **Fix:** route through `\lx@add@classification[scheme=…]` (or `\lx@add@frontmatter@until`), `\lx@begin@abstract` and `\lx@add@affiliation`. EASY. Low risk, but placement will change, so run an A/B on these classes.
2. **`latexml_package/src/package/svmult_cls.rs:281`** (`\lx@sv@chapauthor`) builds `<ltx:creator><ltx:personname>` directly. It is Rust-only: Perl's svmult.cls.ltxml has no `\chapauthor`. It needs either a sectional creator API or a divergence entry. EASY.
3. **Direct pushes into the digested `frontmatter` hash:**
   - `lxrdfa_sty.rs:242` and `omnibus_cls.rs:28` mirror Perl (lxRDFa.sty.ltxml:190, OmniBus.cls.ltxml:128), so this is parity.
   - `latexml_contrib/src/siamltex_cls.rs:25` copies the OmniBus pattern. It should use `\lx@add@frontmatter@until{ltx:classification}`. NIT.
4. **Read-only queue queries** (`authblk_sty.rs:212,242`, `amsppt_sty.rs:134,139`: `queued_creator_count`, `queued_authors_carry_entry_mark`) are fine. They are API helpers.

The rest of the sampled bindings are clean under R1: llncs, revtex4_support, aas/ams_support, ieeetran, elsarticle, acmart, jheppub, pos, rmaa, imsart/ims_support, omnibus, fcs, dgruyter. Their only direct XML is non-frontmatter (bibliography titles, proof titles).

## 3a. Special cases inside general mechanisms (R2)

1. **[Highest] The observer policy table is keyed on binding file names in generic code.**
   - Where: `observer/policy.rs:98-260`, with entries `"revtex4_support.sty"`, `"llncs.cls"`, `"aa.cls"`, `"IEEEtran.cls"`, `"jheppub.sty"`, `"iopart_support.sty"`, … `policy_for` / `has_entry` are read by `ledger.rs:319,384`.
   - The design says so on purpose: "notes decision 2: no policy keyval in the binding files" (`policy.rs:4`).
   - `written: true` (`policy.rs:108` revtex, `:164` ams) makes the default build rewrite those families' frontmatter, from generic code.
   - This is exactly the rule the user excluded: a binding fix does not move its policy, and a new binding has to edit generic code.
   - **Should live in:** each binding, as declared data, for example `\lx@frontmatter@conventions{and_inline,label_sequence,email_owner=sequence,written}` assigned at load. `policy_for` then reads state, and `KERNEL` stays the default.
   - Effort EASY–MEDIUM. Risk low; guard with an identical `fmshadow` census before and after.
2. **Class command vocabulary hard-coded in the generic author reader.** Each entry is keyed on a command name and sometimes its meaning, never on a class name, which is why raw copies benefit. Each is still one class's convention, embedded in a branch:
   - llncs/aa `\inst`, `\fnmsep`/`\fnsep`, `\institutename`:
     - `separators.rs:35-70` (`marks_before_glued_commas`), `:542-548`, `:566-583` (`inst_takes_back_space`, which reads the meaning `\unskip`)
     - `email_lines.rs:545`, `affiliations.rs:146` (`class_numbered_institutes`, which checks `\institutename`'s body for `\def\and` + `\stepcounter`)
     - `space_parted.rs:313,347`, `queue_digest.rs:48` (re-`\let`s `\fnmsep`/`\fnsep` in every digest)
   - ACM SIG `\alignauthor`/`\affaddr`: `separators.rs:843-960` (`unbrace_acm_author_columns`, run for every `\author` at `dispatch.rs`, the `lines_prepared` step).
   - IJCAI `\affiliations`/`\emails`: `separators.rs:605` plus the early branch at `dispatch.rs:52-58`, which hands off to `\lx@ijcai@authorsplit`, defined generically at `author_macros.rs:156-161`.
   - IEEE `\IEEEauthorrefmark`, `\authorrefmark`: `marks.rs:166-171`. Elsevier `\thanksref` and AAS `\altaffilmark`: `notes.rs:474-479`, `email_lines.rs:600-606`, `name_text.rs:625`, `separators.rs:493`.
   - **Should live in:** one registry of author-block conventions (mark commands, annotation commands, mark separators, column commands, section markers). Bindings declare their entries at load (`\lx@title@inst` already shows the pattern: "its binding declare[s] that meaning … which decides first", `separators.rs:563-565`). The defaults for raw classes stay in one table, each row naming its origin class.
3. **Near-duplicate vocabulary lists.** These are the same mechanism behind finding 2. They have already drifted: `NAME_ANNOTATIONS` includes `\inst` and `\IEEEauthorrefmark`, while `take_author_annotations`' list has neither; `mark_commands()` has neither `\inst` nor `\altaffilmark`.
   - Locations: `marks.rs:166`, `notes.rs:474`, `email_lines.rs:545` and `:602`, `name_text.rs:625`, `separators.rs:493`.
   - Merging them into the registry, with per-role sets, is EASY. The risk is real: wherever the lists disagree today, behaviour moves, so run an A/B.
4. **Witness-motivated rules inside the reader** (`tabular_juggles_as_line_breaks` `separators.rs:785`, `join_icons_to_links` `links.rs:89`, `rewrite_in_word_superscripts` `symbols.rs:566`, …) are mostly written as shape rules, on meaning or token structure, not as witness text. Read individually, they are general rules derived from witnesses.
   - The real issue is the one the repo's own design note names (`docs/parity/AUTHOR_BLOCK_LABELING_DESIGN.md:18-30`): line labelling uses hand-written English lexicons. Examples are `notes.rs:17-41` (correspond / equal contribution / grant words), `links.rs:204-223,665-682` and `name_text.rs:195`.
   - These are general but language-bound, and accretive. I found no rule keyed to one witness's literal text.
5. **`distribute_upfront_contacts` (`insert.rs:276`) runs on every document and assigns by position.** It hands the last author's contact pile out as an N×m grid. It is gated on a strictly regular pile, so it is fail-safe. It sits in tension with FIDELITY_CHARTER §4 "never by bare position" (`FIDELITY_CHARTER.md:81`) unless the grid counts as "consistent list evidence". Record which one applies. EASY (doc).

## 3b. Missing shared abstractions

1. **Bindings re-implement the reader's token primitives.** These bindings carry private parsers that duplicate `author_block`'s:
   - `latexml_contrib/src/ptephy_cls.rs` (936 lines): `footnote_symbol` at :84 duplicates `symbols.rs:12` (`is_footnote_symbol_operand`); it also has its own `mark_run`, `email_calls`, `argument_at`, `skip_spaces`.
   - `osa_journal_sty.rs` (856): its own e-mail-line parser at :754, duplicating `email_lines.rs`.
   - `epl_cls.rs` (536), `eptcs_cls.rs`, `aistats_sty.rs`, `interspeech_cls.rs`.
   - Twelve copies of `group_end` / `skip_spaces` / `argument_at` / `read_argument` exist across the tree.
   - Cause: the reader's primitives are `pub(in crate::base_utilities)`, so bindings cannot reach them.
   - Under the clarified R2, the bindings' local *policy* is legitimate. The duplicated *primitives* are what stops fixes from propagating.
   - **Fix:** export a small `frontmatter::tokens` toolkit (mark-operand reader, footnote-symbol test, e-mail-line splitter, argument reader) and move bindings onto it as they are touched. HARD in total (~2.5k lines), moderate risk; do it per binding.
2. **Two ownership engines run side by side.** One is today's reader plus the insert-time passes (`author_block` 13.1k + `relocate.rs`/`insert.rs`); the other is the observer's resolver and writer (10.2k). The writer overrides the first for 2 families. Every ownership rule now has two homes. Converging them is the user's architecture decision. HARD.
3. **The IJCAI sectioned split is a general "sectioned author block" mechanism named after one class** (`\lx@ijcai@*`, `author_macros.rs:156`). Rename it to `\lx@add@authors@sectioned` and put its markers in the registry. EASY.

## 4. 86g cover placement: provenance vs typography

The decision is made at `cover_order.rs:279-283`. A node is cover if it `is_titlepage_layout` and any of these hold: it carries `_deposit`, an open deposit precedes it, or it is not body prose (`!is_body_prose`, `:332`, with `is_set_apart` `:349` meaning `align`, `ltx_align_*` or `ltx_noindent`).

- **Provenance part:** the `_deposit` stamp set by `\lx@deposit@begin/end` (`sect05.rs:1336-1344, 1502-1503`), `_inserted` from deposit inserts (`tex_inserts.rs:30`), the `_titlepage` store provenance with its fail-safe (`:36`, `:256`), and `_late` (`:320`). These are general: time and source provenance, with no shape keyed to one class.
- **Typography part:** the `is_body_prose` / `is_set_apart` test. Also a general rule, but it carries the risk:
  - CONFIRMED probe: an article with `\maketitle`, then `\noindent In this paper we study … body prose …`, then a `center` line, then `{abstract}`, then `\section` (pdflatex: 0 `!` errors).
  - Command: 86g-rel and int116a-rel, flags as briefed; both runs 0 errors, 0 warnings.
  - int116a gives `abstract, para.ltx_noindent, para, section`. 86g wraps both paragraphs as `<titlepage class="ltx_titlepage_continued">` ahead of the abstract.
  - The order now matches the print, and nothing is lost. But body prose is labelled as title page, because `\noindent` alone counts as "set apart".
  - There is no CSS for `titlepage` (`latexml_post/resources/CSS/`), so the rendering only differs by the XSLT title display (`LaTeXML-structure-xhtml.xsl:377-379`).

**A/B r86g5 (299 changed papers):**
- 295 gain a continued titlepage; A has none. The other 4 changed only through the `\insert` and slug fixes: 1705.10455, 2110.04708, 2312.05616, 2501.04820.
- **Who:** 262 run plain `article` (plus packages); 36 run raw `nature*.cls`; 1 runs OmniBus. Package loads in `log.B`:
  - 199 `aaai*.sty`, whose `\@maketitle` prints `\affiliations`, i.e. deposit;
  - 10 `arxiv.sty`;
  - about 86 others, roughly 36 nature `{affiliations}` lists and about 50 hand-set centred or `\noindent` lines.
- **What admits them** (I classified each cover child in `out.B.xml` with the same test the code uses):
  - **289 of 295 are admitted by typography alone.** About 209 by alignment, 39 by `ltx_noindent`, about 42 as box-only paragraphs.
  - **Only 6 strictly need provenance**, all arxiv.sty ("A Preprint" header and date in an indented-shaped paragraph): 1911.09939, 2402.16192, 2503.06162, 2506.13256, 2604.23415, 2609.30787.
- **So:** provenance is the true source for about 209 covers (AAAI, arxiv.sty), but redundant for 203 of them. Typography is the only basis for about 86 (nature plus hand-set).
- Caveat: `_deposit` is stripped from the output, so "deposit-sourced" is inferred from which packages loaded. A switch would make the split exact.

Verdict: neither part is a special case in disguise. Remaining gaps:
- `\noindent` = cover is the soft spot, affecting the 39 papers admitted by `ltx_noindent` alone. Most are nature's lists, which are lists and would stay admitted.
- No kill switch: `lx_late_abstract_placed` is only a once-per-document flag.
- The smfart/`printed_before_the_cover` handling is expressed through `_late`, not through the class.

## 5. Ranked plan

| # | Action | Kind | Effort | Risk |
|---|---|---|---|---|
| 1 | Policy table → binding-declared conventions (§3a-1); `written` becomes a binding flag | move into bindings | EASY–MED | Low (census-identical guard) |
| 2 | One kill switch per general reorganizer: a `FmFeature` set read once (env plus state override for tests). Covers 86g cover order, #262 placement, #246 wrap, `distribute_upfront_contacts`, `respell_emails`, K11 reroute, deposit keep. Default unchanged. | switches | EASY | None; also makes A/B attribution exact |
| 3 | Author-convention registry: merge the ~7 lists, bindings declare entries, raw-class defaults kept with origin tags (§3a-2/3) | move into bindings + merge | MED | Medium where the lists disagree; A/B |
| 4 | 86g: a `\noindent` paragraph with sentence-length running text is not cover unless deposit-stamped; lists and centred/box lines unchanged | fail-safe tightening | EASY | About 39 papers to re-check |
| 5 | Contrib in-place constructors → API (§2 item 1); svmult chapauthor → API or OD entry | R1 | EASY each | Placement shifts; A/B per class |
| 6 | Rename IJCAI sectioned split to a generic API; markers into the registry | generalize | EASY | Low |
| 7 | Export the token toolkit; migrate ptephy, osa, epl, eptcs, aistats, interspeech as they are touched | merge duplicates | HARD | Moderate, per binding |
| 8 | `distribute_upfront_contacts`: charter exception recorded, or gated behind data | doc / gate | EASY | Low |
| 9 | Converge the two ownership engines (reader + passes vs observer) | architecture | HARD | High; user decision |

Perl-parity note: Perl's `insertFrontMatter` (Base_Utility.pool.ltxml:824-844) only places what bindings queued, in fixed element order, and then runs `relocateAnnotations`. Perl has two global heuristics: `\lx@add@authors` (:687-727) and OmniBus's table. In Rust, `author_block` and `frontmatter_stores` are their expansions. Placement, cover order, title layout, field ink and the insert-time passes are all beyond Perl, and need the switches in item 2 to stay "switchable as one unit".

## Not checked

- `frontmatter/observer/resolver/*` and `writer/*` in detail, beyond the policy keying.
- `field_ink.rs` and `titlepage_layout.rs` rule by rule.
- `sect05.rs` speculative-run internals.
- The `_share` emitters in revtex, sv and amsppt, beyond the grep.
- The contrib bindings beyond the brief's sample, except the grep sweeps in §2 and §3b.
- None of the §2 bypasses converted.

My scratch scripts and probe are in `/home/deyan/.claude/jobs/356a0c47/tmp/audit_fm/`: `scripts/cls.py`, `why.py`, `tally.py`, and the probe in `p1/`.
