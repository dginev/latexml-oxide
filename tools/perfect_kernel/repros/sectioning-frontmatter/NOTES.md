# sectioning-frontmatter — root-cause notes (Checkpoint 1, 2026-09-03)

Binary: /home/deyan/data/pk_bin/latexml_oxide.b54l (HEAD d1dd27af3c).
All repros verified RED today; pdflatex oracle 0 errors on each.

## Candidate ranking (docs x error-lines)

| # | Mechanism | Docs (residue) | Err-lines | Class | Repro |
|---|---|---|---|---|---|
| A | `\cl@<ctr>` stored as State Value, not an expandable macro | contract-de, contract-en, afthesis/usethesis | 44+44+2 = 90 | SHARED (surpass) | cl_at_reset_macro_contract.tex, cl_at_reset_recursion_afthesis.tex |
| B | `\@maketitle`-recovery runs the class `\@maketitle`, which references binding-omitted frontmatter internals | ascelike/ascexmpl, resphilosophica/rpsample, ijmart/ijmsample (+likely mlacls, jourcl) | 2+4+2(+2+4) ~= 8-14 | RUST-ONLY | maketitle_recovery_authblk_ascelike.tex |
| C | Sectioning/block element inside a list item rejected by RelaxNG model (no auto-close) | ddphonism(9), phonrule(1), prerex(1), pdfmarginpar seed(1), contract's section-in-item | ~12 | SHARED | section_inside_item_ddphonism.tex |

## A — `\cl@<ctr>` is a Value, not a macro  (SHARED, pdflatex clean -> surpass approved)

`NewCounter` stores the reset list `\cl@$ctr` via `assign_value(...Tokens())`
(latexml_core/src/binding/counter/dialect.rs:106-108), line-faithful to Perl
`Package.pm:674` `AssignValue("\\cl\@$ctr" => Tokens())`. NEITHER engine makes
`\cl@$ctr` an expandable control sequence. Real latex.ltx `\@definecounter` does
`\global\expandafter\let\csname cl@#1\endcsname\@empty`, so `\cl@foo` IS a macro.
Raw package code that treats it as a macro breaks in BOTH engines:
  - contract.sty:336 `\edef\cl@Clause{\cl@Clause\cl@contractClause}` (counter at :353) -> `\cl@contractClause` undefined.
  - afthesis.cls:44-49 `\@removefromreset`, `\expandafter\edef\csname cl@#2\endcsname{\csname cl@#2\endcsname}` (called :74-76) -> `\cl@chapter expands into itself` (the `\edef` captures a self-reference when `\cl@chapter` is undefined-as-macro; 2nd call detects the loop).
Repro A1 (undefined) and A2 (recursion) both reproduced in Rust AND Perl.
Fix site (Checkpoint N): `new_counter` should ALSO define `\cl@$ctr` as an empty
expandable macro (mirroring `\@definecounter`'s `\let\@empty`) so raw code can
expand/redef it, while `\@addtoreset`/step_counter keep reading the Value list.
Tension to resolve: keep the Value list authoritative for LaTeXML resets; the macro
is only for raw expansion. Risk MED (counter subsystem). Guard: 0 errors + a
`<ltx:para>` present in A1's XML.

## B — `\@maketitle`-recovery reaches binding-omitted internals  (RUST-ONLY)

`\maketitle` (latex_constructs.rs:5741, locked) runs `\lx@deposit@maketitle`
(:5714-5717): `\ifx\@maketitle\@empty\else{...\@maketitle}\fi`. LaTeXML predefines
`\@maketitle` EMPTY (:5713) to recover `\g@addto@macro\@maketitle{...}`-injected
content (surpass-Perl, OXIDIZED_DESIGN #124). But a class that FULLY
`\renewcommand`s `\@maketitle` makes it non-empty, so the `\else` branch EXECUTES
the whole class title body — which references frontmatter internals that LaTeXML's
*bindings* (not raw) neutered:
  - ascelike.cls:406-411 `\AB@authlist`/`\AB@affillist` — authblk_sty.rs binds `\author`->`\lx@add@creator`, never builds authblk's `\AB@*` token lists (authblk.sty:109-110).
  - resphilosophica.cls:323 `\author@andify` + :358 `\@dedicatory` — amsart_cls.rs binds the user macros, omits amsart.cls:803 `\author@andify`.
  - ijmart.cls:245-247 `\def\@maketitle{\@origmaketitle \thankses...}`; `\thankses` is built by ijmart's `\thanks` (:181 `\protected@xdef\thankses`), but LaTeXML's locked `\thanks` (:5762) only does `\lx@add@pubnote`, never builds `\thankses`.
Perl does `\global\let\@maketitle\relax` and NEVER runs the class body -> clean.
Confirmed RUST-ONLY: repro errors in Rust, Perl clean, pdflatex clean.
Fix direction (Checkpoint N): make `\lx@deposit@maketitle` recover only the
*appended* injected content, not execute a wholesale class `\@maketitle`
redefinition (distinguish append-to-empty from full replace), OR wrap the recovery
so undefined frontmatter internals degrade rather than error. Risk MED (touches the
#124 witness arXiv:2506.23854 — must re-convert). Guard: 0 errors + `<ltx:title>`/
author frontmatter present, and #124 witness still recovers its injected figure.

## C — sectioning/block inside a list item  (SHARED, pdflatex clean -> surpass)

`document.rs:3149` openElement rejects `<ltx:subsection>` (and `<ltx:paragraph>`,
`<ltx:section>`) inside `<ltx:item>` because the RelaxNG model forbids it; TeX has
no such rule (a sectioning command inside a list just starts a heading). Both
engines error. Fix shape = document auto-close of the enclosing item/list (and the
para) before opening the sectioning element, mirroring Perl `Document.pm`
openElement autoclose, OR an inline run-in heading. Same site as the existing
`paragraph_inside_item_pdfmarginpar` seed. Members: ddphonism(9), phonrule,
prerex(paragraph-in-figure variant), contract(section-in-item). Risk MED-HIGH
(auto-close policy can mis-nest). Guard: 0 errors + `<ltx:subsection>` is a sibling
of, not a descendant of, `<ltx:item>`.

## Dead ends / out-of-scope one-offs (not grouped mechanisms)
- `\autopageref` (abntex2cite), `\pgfpagesuselayout` (beamerswitch) — hyperref/pgfpages, not frontmatter; other topics.
- `\markleft` (amscls-doc) — memoir/scrpage mark cmd; one-off.
- `\ams` (amslatex-primer/amshelp) — doc-local helper; one-off.
- `\digitalasset` (aastex701) — aastex701 frontmatter cmd missing in aastex binding; one-off (aastex-specific, could join B if reached via title).
- `\@authorgroup` (quantumview) — quantumarticle IS bound; etoolbox `\listxadd` author-group not built by the binding's `\author`; quantumarticle-binding-specific (B-flavoured but inside a bound class).

## Root B (Checkpoint #2) — RESOLVED RECOMMENDATION (2026-09-03, b54m)

Mechanism reconfirmed on b54m: `\maketitle` (latex_constructs.rs:5741) -> `\lx@deposit@maketitle`
(:5714-5717) EXECUTES the class/pkg `\@maketitle` when non-empty (surpass #124). ascelike/
resphilosophica/ijmart `\renewcommand`/`\def` `\@maketitle` to reference frontmatter internals
that LaTeXML's LOCKED kernel bindings for `\author`/`\thanks` deliberately never build ->
undefined-cs. Perl `\let\@maketitle\relax`, never runs it.

DECISIVE constraint: existing guard `frontmatter_titlepic_redefined_maketitle_figure_survives`
(06_cluster_frontmatter.rs:1472, witness arXiv:2606.25280) RELIES on running a WHOLESALE
`\renewcommand`'d `\@maketitle` (titlepic stores a teaser figure in `\@titlepic`, redefines
`\@maketitle` to inject it). So the discriminator is NOT append-vs-replace (both ascelike and
titlepic are wholesale `\renewcommand`) -> coordinator shapes (2) g@addto@macro-tracking and
(3) snapshot-suffix BOTH DROP the titlepic figure. REJECTED with evidence.

RECOMMENDATION = shape (1), refined: predefine each package's frontmatter ACCUMULATOR to its
package-initial EMPTY value at the package's binding site (BINDINGS OUTRANK RAW). Because the
locked kernel `\author`/`\thanks` emit SEMANTIC frontmatter and never fill these visual
accumulators, they stay empty, the class `\@maketitle` visual layout collapses to nothing, and
the semantic `<title>`/`<creator>` is the sole source -> NO duplication. Empirically validated
(b54m): predefining `\AB@authlist`/`\AB@affillist` empty makes the ascelike `\@maketitle` shape
convert with "No obvious problems", clean `<ltx:title>T` + `<ltx:creator><ltx:personname>Alice`,
empty `center` collapses, no ERROR. titlepic + #124 keep running (their bodies have no undefined
cs). NOT faithful-populate: amsart `\author@andify\authors` would emit author names as TEXT ->
DUPLICATED authors, so accumulators must be EMPTY, not filled.

Per-witness internals + site:
  - authblk (ascelike): `\AB@authlist`, `\AB@affillist` EMPTY in authblk_sty.rs (authblk.sty:109-110 init). VALIDATED clean.
  - amsart (resphilosophica): `\authors` EMPTY (so amsart.cls:357 `\ifx\@empty\authors` skips `\@setauthors`, never reaching `\author@andify`), `\@dedicatory` EMPTY, `\@setabstract` relax -> in amsart_cls.rs. `\author@andify` per amsart.cls:803 harmless once `\authors` empty. IMPLEMENTER MUST verify amsart binding leaves `\authors` empty (else author dup).
  - ijmart (unbound class): `\thankses` EMPTY at kernel near the locked `\thanks` (latex_constructs.rs:5752-5764), same rationale as `\shortauthor`/`\shorttitle` (:5664-5665) — a locked kernel binding shadows the class's own `\thanks` that would have built `\thankses`.

Guard spec (chosen shape):
  - maketitle_recovery_authblk_ascelike.tex: 0 Error/Fatal; XML has `<ltx:creator role="author">`
    with `<ltx:personname>Alice`; NO `<ltx:ERROR>` and NO `\AB@authlist` text.
  - OD #124 (tests/cluster_regressions/maketitle_injected_figure.tex + titlepic guard): UNCHANGED
    GREEN — injected/titlepic teaser figure still deposited (`labels="LABEL:fig:teaser"`, S0.F1).
Classification RUST-ONLY (Perl clean). Risk LOW-MED (per-package empties; amsart `\authors`
dup-check the only sharp edge). Expected gain: ascelike, resphilosophica, ijmart (+likely mlacls,
jourcl via the same path) ~ 5 docs / ~14 err.

## Separate deferred root (NOT Root B): contract `\the}`  (surfaced #1 after Root A)
contract-example-en 44->42 on b54m; new first error `Error:unexpected:\the} You can't use } after
\the` (tex_macro.rs:326, "Anonymous String" frame) + `\lx@tag@intags Attempt to end mode
restricted_horizontal`. This is contract's CLAUSE-NUMBER/tag machinery, not frontmatter/maketitle:
`\protected@edef\theClause{\@nameuse{\contract@env@type @Clauseformat}{\contract@number}}`
(contract.sty:534) with `\contract@number`/`\p@Clause`/`\theH..` expanding a bare `\the` next to a
`}`. Pre-existing (was error #3 before Root A). Own root — needs its own checkpoint.

## raw_class_author_flag_jinst (RED, 62k)
A raw class's `\renewcommand\author` carries a side effect — JINST.cls:753 sets `\@authortrue`, which its
`\AtBeginDocument` check (:1595) reads. The kernel `\author` is locked (Perl latex_constructs.pool.ltxml:1076, Rust
the same), so the redefinition is dropped with its flag and the class's own check errors. Since 62k the arXiv profile
loads shipped classes raw (DIVERGENCES #444), so JINST papers (1310.6454, 1504.01965) meet it; on OmniBus they did
not. SHARED with Perl under `rawclasses`; same lock as DIVERGENCES #253. Fix site: the locked-definition path, which
would have to run the class's redefinition for its effects while keeping the kernel's frontmatter capture.

## Journal styles neither in TeX Live nor shipped (64u): l-aa, epl2, OSA, REVTeX 2.1
Four run-336 first-error clusters (843 `\thesaurus`, 740 `\Name`, 759 `\address`, 750 `{instit}`) whose class or
style the paper names but neither TeX Live nor the paper has; Perl errors the same (SHARED), so each fix is a
beyond-Perl binding read from a copy found elsewhere in arXiv.
- l_aa_documentstyle_is_the_aa_class / l_aa_thesaurus_ends_at_par (GREEN, 64u): `l-aa`/`laa` = L-AA 3.0 (copy in
  astro-ph9601070), the `aa` class's predecessor: contrib `l_aa_cls.rs` loads the `aa` class binding and adds
  `\thesaurus` (l-aa.sty:453, a classification; read non-`\long`, so a `%`-commented brace ends at the `\par` as in
  pdflatex, astro-ph9405007) and `\picplace`. Bound as a class: `\documentstyle` prefers a `.cls` binding
  (latex_constructs.pool.ltxml:105-108), and the `aa` class brings plain `\cases` in displays (astro-ph9908044).
- epl_class_frontmatter_and_bibliography (GREEN, 64u): epl2.cls v1.1 (copy in 1201.0004): contrib `epl_cls.rs` for
  `epl2`/`epl`; `\institute` entries begin at their `\inst` marks (a `\\` inside one is a line break: 1803.06254).
- osa_style_marked_addresses (GREEN, 64u): opex3/ol2/osajnl/osajnl2 (copies in 0901.2165, 0901.2997, 0707.0735,
  0901.2793): contrib `osa_journal_sty.rs`; `\author` appends per call (0906.4563), `\address` is a marked list.
- revtex2_title_and_instit_environments (GREEN, 64u): REVTeX 2.1 (embedded in hep-lat9302012.tex:465-2231):
  `{title}`, `{instit}`, `\receipt`, `\nonum`, `\moreauthors`, `\tablenotes` added to revtex3_support_sty.rs, which
  both `\documentstyle[preprint,revtex]{aps}` and `[version2,aps]{revtex}` reach.
Review round 1 (GREEN, 64u): l_aa_keywords_without_argument (l-aa.sty:648 `\keywords` takes no argument),
l_aa_marked_institute_names_its_authors (a `$^n$`-marked `\institute` with marked names is a labelled list; one
institute every author's), epl_unmarked_institute_one_entry (an unmarked `\institute`'s `\\` is a line break),
osa_style_address_email_line (an email line is the marked author's email contact, named by its label; `$^{1,2*}$`
marks 1, 2 and *).
Residual (not this batch; RED revtex3_omnibus_author_group_address): under `\documentstyle[aps]{revtex}` the unknown options load OmniBus after revtex
(latex_constructs.pool.ltxml:137-153 `\compat@loadpackages`), whose `\author{A and B}` leaves an `annotate=new` affiliation on B only
(REVTeX 3 `\address` the same on the base). Mixed marks `$^{1,2*}$` are split by the OSA binding only (osa_journal_sty.rs `symbol_marks_apart`).
A line marked with several labels (`$^{1,2}$Dept`, 1406.0824) is one line a label in the OSA binding only
(osa_journal_sty.rs `lines_per_label`): the kernel's `\lx@set@frontmatter@label` takes the first label, as Perl's
(Base_Utility.pool.ltxml:568), on the request path (an author whose marks were taken as requests); where the marks stay
visible in the names (IEEEtran `\IEEEauthorblockA`, llncs `\institute`) relocation pairs by the marks shown, all of them.

## refused_dblarg_setter_reaches_its_internal (GREEN, 66d r2)

cmfne.cls:462-483 (2210.02246) is the amsart `\@dblarg` idiom over a locked `\author`: `\renewcommand{\author}[2][]{…}`
(refused, but `\@xargdef` still defines `\\author`, latex.ltx:1245-1257) and `\edef\author{\@dblarg\csname\string
\author\endcsname}` (refused; kept as `\lx@dropped@author`). The refused-setter replay (sect05.rs `replay_class_setter`)
runs `\@dblarg\\author[opt]{arg}`, which sets `\firstshortauthor` for the class's `\enddoc@text` line. 66c r3 skipped
the internal of a refused definition, so the replay's vocabulary gate dropped the call and the line was lost; restored
in 66d r2 (sect08.rs `define_optional_internal`, OD #567). Guard
`perfect_kernel_batch66b::refused_dblarg_setter_reaches_its_internal`.
