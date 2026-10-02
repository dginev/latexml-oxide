# Perfect Kernel — the generalized kernel-capability program

**Approved by the user 2026-09-05**: *"prioritize a well-designed generalized
kernel support, faithful to the originals and with clear and well-defined
abstractions."* This document is the standing plan for that program. It is a
living worklist (no date in the name); each capability row carries its status.

It complements [`ARCHITECTURE_THEMES.md`](ARCHITECTURE_THEMES.md) (the recurring
mechanisms distilled from batches 33–53, and themes 7–10 and 2b from batches 56jm–56kc). Where a capability *is* a
theme, the theme section holds the model and this file holds the landing plan;
where it is new (K1, K3, K4, K6), the model is here.

**Standing practice (user, 2026-09-26):** an architectural insight is recorded as it is met — the
mechanism in ARCHITECTURE_THEMES.md, the landing plan here — and the open capabilities are implemented in
dedicated sessions once a large goal completes, not inside the batch that found them.

**Design rules for every capability**

1. **Faithful to the originals.** The model is tex.web / latex.ltx / the real
   `.sty`, cited by file:line. Perl LaTeXML is the reference for the *XML*
   shape, not for kernel mechanics it emulates loosely. A divergence from Perl
   is recorded in `docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md`.
2. **One abstraction, one owner.** Each capability names its Rust type or
   module and the invariant it maintains. Bindings consume the abstraction;
   they never re-implement it (the `\verb`/listings/tcolorbox readers of
   batch 56i were six re-implementations of K5).
3. **Mechanism over symptom.** A fix that only closes the witness is not
   landed under this program; it goes to the batch fix log. A program row
   lands with the class-level guard (a repro from a *different* package than
   the witness).
4. **Additive first, rewrite last.** K1, K3, K4, K5, K7 are additive; K2 is a
   rewrite on its own branch with its own sweep gate.

## Summary

| # | Capability | Theme | Retires (evidence) | Order | Status |
|---|---|---|---|---|---|
| K1 | Definition provenance + raw-load-then-overlay bindings | new (2 policy) | "already defined" 261 lines; stub-vs-raw internals (`\pdfstringdefPreHook` ×6, `\siunitx_number_format:nN`, `\ifGm@showframe` ×3, `\chemmacros_load_module:n`, PDF-API / `\LuaUL*` stubs) | 1 | **Steps 1-2 LANDED** (56l `DefinitionOrigin`; 56n retraction retired; 59g `Fallback` origin: OmniBus yields to the document, DIVERGENCES #408); step 3 (Overlay audit: hyperref, siunitx, geometry, fontspec, chemformula, forest) DEFERRED |
| K3 | lthooks as the single hook store | 6 | source2e, tikz-ext-manual, euclideangeometry (56i regression), every `\AddToHook{package/…}` no-op | 2 | Steps 1 (56i, `\AtBeginDocument` routed), 2 (54c `use_load_hooks`), 4-kernel (56s) LANDED; step 3 (delete the private stores) + step-4 binding audit OPEN; no in-scope witness |
| K4 | Kernel templates and sockets backed by our constructors | new | ltx-talk ×10, tagpdf 113 lines, diffcoeff, every tagging-aware class | 2 | OPEN — inert declarations landed (56i); no in-scope witness with errors (s134) |
| K5 | One line-oriented raw reader | 4 | `\verb` EOL, listings line 1, `\tcbverbatimwrite`, fancyvrb, `\DocInput`, `.listing` round trips (25 docs) | 3 | OPEN |
| K2 | Semantic nest separate from the save stack | 1 (R9, approved 2026-09-02) | `unexpected:\endgroup` 49 docs, three off_save site patches (56g/56h), tutodoc, kblocks, nath | 4 | APPROVED (R9, 2026-09-02); deferred (re-steer 2026-10-01). Its in-scope witness kaytannollista-latexia (165 'Attempt to close a group that switched to mode internal_vertical') was a binding seam, fixed by 59h (a raw `\@endfloatbox` closes only a box that was opened, DIVERGENCES #409) |
| K6 | A consistent font-selection model for Unicode engines | 5 | polyglossia 136 lines, fontspec queries, `\mathitalicsmode` ×4, most lualatex manuals | 5 | persona LANDED (56id, DIVERGENCES #285: `\ifpdf` follows `\pdfoutput`, 1 unless the source ships EPS/PS) and the font-file record (56x); OPEN — the polyglossia TRUE stub (56i) is the anti-pattern to replace |
| K7 | One in-memory file model | 6 | VFS `./` (56i), `\jobname` round trips, `\IfFileExists`/`\openin`/`\file_full_name:n` gaps | 6 | half-landed (b42/b47/b50/55m/56i) |
| K8 | Runaway cap that degrades instead of discarding | — | csvsimple-l3, forest-doc (pre-56i), euclideangeometry: 500 same-errors → 39-byte XML | 7 | OPEN — no in-scope witness (canaries only); deferred |
| K9 | Group codes on every frame; closers dispatch on the group code | 1 | the fused mode-frame family (DIFFICULT_CASES D12: msc, modernposter, dsptricks/psmatrix) | staged | Stage 0 LANDED (56ac: frame serials, the box reader ends on its own frame; msc 21 → 0); stage 3 retired for psmatrix (56af); no in-scope witness needs the rest (kaytannollista-latexia was the floatrow seam, 59h) |
| K13 | Binding-conformance detector: the real macros' signatures and prologues against the bindings | 7, 2b | arydshln `\hdashline[..]`/`;{..}` (56jx), `\makebox`/`\raisebox` enterHorizontal lost (54n→56kb), the "arity long tail" | after the current goal, first | Stages 0-2 LANDED (57b, 57c, 57h); the 60 complete HIGH findings landed or verified in 57i/57j/57ah/57aj; stage 3 DEFERRED (re-steer 2026-10-01; resumes with the post-goal generalization audit) |
| K14 | Two-phase typed parameters: read the macro's argument, then parse the type inside it | 7 | 56jm, 56jr/56ju (and 56jr's three sweep-#126 regressions), 56jw A-D, 56jz, 56kf's first cut (every `{Dimension}` read as a `\setlength` operand; now the declared `SetlengthDimension`) | after K13 | OPEN, deferred (re-steer 2026-10-01) |
| K15 | A typed tail of the horizontal list (glue, kern, penalty, char, box) | 8 | 56jy trim, babel-french `;`, the paragraph text-node split, `\@bsphack`/`\xspace` spacing | after K13 | OPEN, deferred (re-steer 2026-10-01); french highpunct fixed at its site (gemini-13 Q6, c393accb65, guard `perfect_kernel_gemini::french_high_punctuation_unskips_the_space`) |
| K16 | Bibliographies from the style's programs: a native `.bst` interpreter; biblatex from its declarations | 9 | abntex2cite 86.9 % at s134 (57cm), 99.4 % with bibtex's `.bbl`, biblatex-chicago/apa samples, every future formatter row | own sessions | OPEN (recorded 2026-09-26) |
| K17 | A fixed, stratified manual regression net per batch | 10 | 56jr/56js regressions found five batches late (sweep #126) | cheapest; any time | LANDED (phase 57, 2026-09-27): `manual_net.{tsv,sh}`, `manual_net_select.py`, `manual_net_compare.py` |
| K18 | Typed box sizes: a box's measured size is its constructed size | 11 | `\framebox[w]` 12.08 vs 10.0pt, `\raisebox` height, `\parbox` 0.1pt rounding, `\resizebox`/`\scalebox`/`\rotatebox` sizes (56kf side finding); `\makebox[w]` contents line-broken at w | step 1 (the four + the width leak) 56kj; step 2: `\height` etc. bound to the box (`\raisebox[h][d]`), `\Gscale@div`, makecell/diagbox, the sizer audit | STEP 1 LANDED (56kj); step 2: `\height` binding LANDED (56kl), `\resizebox`/`\Gscale@div`/makecell/diagbox/audit OPEN |
| K19 | Order-free math ranking: pragmas rank readings by violation count; a reversed enumeration reads alike | 12 | 57ao A/B (≈2,550 formulas moved by order), `(f(x)+1)(g(x)+1)`, rc59c bigop operand, `\{(0,6),(1,4)\}` | after the current math train | step 1 LANDED (57av); PARKED with the math stream (user 2026-10-02) |
| K10 | A pdfTeX byte mouth (256-entry catcode table over U+0000..U+00FF) | new (7 CJK/kotex manuals, D9) | cjk-ko-doc, kotex-doc, kotex-utf-doc, oblivoir-simpledoc, sample-bxcjkjatype-beamer (`\가`/`\japanese`/`\ifx 가가`) | 8 | steps 1+2 LANDED (56bl); step 3 dropped — witnesses out of scope (CJK, 2026-10-01) |
| K11 | Raw classes' title-page stores reroute to the frontmatter API | surpass (user-approved 2026-09-18) | jpsj2, ptptex; 390 of 655 TL classes define store setters | — | steps 1-2 LANDED (56di, 56dj); extended 58m, 58q, 59d, 59e; OPEN: list-append stores other than `\@author` (verify a witness first) |
| K12 | pTeX control-word letters under a pLaTeX class | CJK (D9) | jsarticle `\if西暦`, ptex-manual `\黄マーカー` | — | LANDED (56ds, `PTEX_PROFILE` + upTeX `kcatcode`); the pTeX residual is PARKED, SHARED with Perl; witnesses out of scope (CJK, 2026-10-01) |

## K1 — Definition provenance and raw-load-then-overlay bindings

**Goal.** A new `.sty`/`.cls` works at the raw level by default; a binding only
says which constructs become XML.

**Source of truth.** latex.ltx `\newcommand` → `\@ifdefinable` (:1006);
expl3-code.tex:2031 `\__kernel_chk_if_free_cs:N`; latex.ltx:4820
`\NewDocumentCommand`, :4872 `\__cmd_new_env:nnnn`. Perl:
`isDefinableLaTeX` (latex_constructs.pool.ltxml:2512) — the leniency the
pool needs because it pre-defines article-level names for class-less input.

**Abstraction.** `DefinitionOrigin { Plain, LatexDump, Pool, Binding(name),
File(path), Document }` stored on every definition (today inferred from
locator shapes in `latex_constructs/mod.rs::is_latexml_predefinition_source`).
One predicate, `origin.is_latexml_owned()`, consulted by `\newcommand`,
`\cs_new`, ltcmd, `\@ifdefinable`, the dump reader and `\let`-retraction. A
binding declares its shape: `Overlay` (load the real file raw, then define
constructors, locked) or `Replace` (today's default; to be justified per
binding). The overlay form is what tcblistingscore, tagpdf-base,
pdfmanagement, polyglossia and circuitikz already do by hand.

**Design as landed (56l):** [`archive/KERNEL_CAPABILITIES_LANDED_2026-10-02.md`](archive/KERNEL_CAPABILITIES_LANDED_2026-10-02.md) (the 2026-09-05 source-survey design).

**Landing.** (1) LANDED 56l, (2) LANDED 56n. (3) Audit the bindings that `Replace` a package whose internals raw code reaches for (hyperref, siunitx, geometry, fontspec, chemformula, forest) and convert to `Overlay` one at a time, witnesses re-run. Guards: a repro per declarator over a pool name from a class (ltx-talk shape) and over a binding name from a raw package (lua-ul shape).

**Risk.** LOW for (1)–(2); MED per binding in (3) (overlay exposes the real
package's internals to our constructors — the tcolorbox listing family was
the template).

## K2 — Semantic nest separate from the save stack

Model, witnesses and fix shape: [`ARCHITECTURE_THEMES.md` §1](ARCHITECTURE_THEMES.md#1-grouping-and-mode-are-one-stack-tex-keeps-two)
(tex.web §211–219 nest, §268–284 save stack, §1064–1069 `off_save`). The
three `off_save` site patches of batches 56g/56h (`stomach.rs::endgroup`,
`digest_next_body`, `egroup`) are the symptom form; the program form is one
`off_save` routine on the nest. Own branch, own sweep gate (S0∧S1 must not
drop on the oracle-clean slice). Approved as R9 on 2026-09-02.

## K3 — lthooks as the single hook store

**Goal.** Every hook a raw package or class can see is the L3 one.

**Source of truth.** latex.ltx lthooks: `\AtBeginDocument` = `\AddToHook
{begindocument}` (:18901), `\AtEndDocument`, `\AtBeginDvi`, `\AtEndPreamble`/
`\AfterEndPreamble`, `package/<name>/after`, `file/<name>/after`, `env/<name>/
{before,begin,end,after}`, `shipout/*`; label = current file name, `top-level`
runs last. Perl keeps private stores (`@at@begin@document`,
latex_constructs.pool.ltxml:296-297) that ignore labels — the source2e
`\RemoveFromHook` miss and the tikz-ext-manual ordering bug.

**Abstraction.** `hooks::gput(hook, label, code)` in `latexml_engine` that
calls `\hook_gput_code:nnn` when the L3 system exists and the private store
only on the format-less path; the Rust `AtBeginDocument()` helper
(`prelude.rs:28`) and every binding's `RawTeX!(r"\AtBeginDocument{…}")` go
through it. `\begin{document}` (sect02.rs) fires only `\hook_use:n
{begindocument}`; the private store is drained into the hook at format end.

**Landing plan.** (1) Move `AtBeginDocument()`/`AtEndDocument()` helpers onto
`hooks::gput` with the binding's package name as label. (2) Make the package
loader fire `package/<name>/after` and `file/<name>/after` (theme 6's
`\@onefilewithoptions` seam) — LANDED 54c (`use_load_hooks`, guard
`perfect_kernel_batch54::package_after_hook_fires_for_a_binding_load`). (3) Delete the private stores. Guards: the
56i `atbegindocument_joins_the_l3_begindocument_hook` and
`pgfmanual_toplevel_atbegindocument_runs_last`. The euclideangeometry regression
(a `\special_relax` marker inside a `\g__hook_` csname) is the first K3
correctness item: the label/argument must reach lthooks as *tokens*, never
through an expansion that can insert markers. (4) Self-terminating fused
environments must hand their terminator back to the current `\end` macro so
`env/<name>/after` and any hooked `\end` fire: the kernel `{verbatim}` does
(batch 56s, DIVERGENCES #199); audit the bindings that read their own body
(listings, fancyvrb, minted, comment) for the same shape.

**Risk.** MED — ordering of begin-document code changes for every document
(lthooks order is the faithful one; goldens that encoded the old order are
re-baselined, not patched around).

## K4 — Kernel templates and sockets backed by our constructors

**Goal.** A class written against the 2024+ kernel (`\DeclareInstance
{blockenv}{myenv}`, `\EditInstance{item}{basic}`, `\AssignSocketPlug`) gets
real structure, not inert declarations.

**Source of truth.** latex.ltx lttemplates (`\NewTemplateType`,
`\DeclareTemplateInterface/Code`, `\DeclareInstance`, `\UseInstance`) and
ltsockets (`\NewSocket` … `\UseSocket`, :7316 top-level check, :7405
undeclared error); latex-lab-testphase-block.sty:96–170 (types + interfaces),
:202–330 (`blockenv` display code, `\endblockenv`), :1180–1473 (instances);
latex-lab-testphase-minipage.sty:47–50 (sockets).

**Abstraction.** Template types `blockenv`/`list`/`item`/`para`/`block` are
declared once (the 56i bindings) and their *code* maps onto the pool's list
machinery: `blockenv` display → open the element named by `tag-name`
(itemize/enumerate/description/quote/quotation/center/theorem/verbatim/Div)
with the `inner-instance` selecting the item style; `\endblockenv` closes it
(`end_mode` on the K2 nest). Sockets are raw and stay raw; the tagging ones
(`tagsupport/*`) are declared, no-op plugs.

**Landing plan.** (1) `blockenv` code bodies call `\lx@blockenv@begin{tag}` /
`\endblockenv` → the pool's begin/end list constructors. (2) `item` instances
feed `\makelabel`. (3) Map `\DeclareInstance{blockenv}{X}` onto a
`DefEnvironment`-equivalent so `\begin{X}` works. Guards: the 56i
`testphase_tagging_sockets_and_block_templates_are_declared` plus a repro
that declares a `blockenv` instance and asserts the element it opens.

**Risk.** MED — list-structure fidelity; keep the enumitem/paralist goldens
green.

## K5 — One line-oriented raw reader

**Goal.** Every verbatim-family construct reads lines through one reader.

**Source of truth.** tex.web §343–360 (`get_next`, `state`, `\endlinechar`
§360), latex.ltx:15504–15510 `\verb@eol@error`, verbatim.sty:107–112
`\verbatim@start`, lstmisc.sty:45–64 (write-file tee), doc.sty
`\MakePercentIgnore`, tcolorbox.sty:2726–2735, fancyvrb.sty:418–421.

**Abstraction.** `mouth::RawLines` — a reader over the *current* mouth with:
`from_column_zero()` (the line the pushback was probed from, OXIDIZED #162),
`until(pattern)` (regex on the raw line, remainder pushed back as tokens
under the caller's regime), `regime(CatcodeRegime)` (verbatim / semiverbatim
/ obeylines: what the EOL yields — active `^^M`, space, `\par`), honouring
`\endlinechar` and the `\scantokens` pseudo-file budget. Bindings (listings,
fancyvrb, verbatim, tcolorbox, doc, minted, `\verb`) call it; none keeps a
private scanner.

**Landing plan.** (1) Extract today's `listings_read_raw_lines_with_outer`,
`read_verb_invocation`'s scan and `verbatim@` into `RawLines`. (2) Port the
other five callers. Guards: the existing #162 guards, `verb_ended_by_end_of
_line_recovers`, `forest_docinput_lstenv_writefile_gobbles_doc_percent`, the
`tcbverbatimwrite_*` pair — all kept, plus one per ported caller.

**Risk.** LOW–MED (the readers are well guarded; the risk is in `\scantokens`
budget interplay).

## K6 — A consistent font-selection model for Unicode engines

**Goal.** Packages that *ask about* fonts get consistent answers from one
state, not per-package TRUE/FALSE stubs.

**Source of truth.** NFSS (latex.ltx fontdef/`\selectfont`, `\fontencoding`),
fontspec (`\fontspec_if_script:nTF`, `\fontspec_if_language`, `\l_fontspec
_family_tl`, `\newfontfamily`, `\setmainfont` → family declarations),
unicode-math (`\setmathfont`, `\mathitalicsmode`), polyglossia.sty:632–677
(the script check), LuaTeX manual §7 (math-code primitives). The 56i
polyglossia binding answers TRUE unconditionally — correct for the witnesses,
not a model.

**Abstraction.** `FontModel` in `latexml_engine`: the NFSS state plus a
declared-family table populated by `\setmainfont`/`\newfontfamily`
(name, features, scripts declared or implied by the name), answering the
fontspec conditionals from that table ("a declared font supports the
scripts its declaration named; the default cmr answers as the oracle's
default font would"). Engine persona (theme 5) decides which primitives
exist (`\mathitalicsmode`, `\Umathcode`).

**Landing plan.** (1) Family table + the five fontspec conditionals. (2)
polyglossia's binding reduces to loading raw. (3) unicode-math surface.
Guards: `polyglossia_script_check_passes_without_font` kept, plus a
`\newfontfamily\greekfont[Script=Greek]` repro.

**Risk.** MED — touches every lualatex document's preamble.

## K7 — One in-memory file model

Model: [`ARCHITECTURE_THEMES.md` §6](ARCHITECTURE_THEMES.md#6-file-loading-and-file-io-bypass-the-kernel).
`latexml_core::binding::virtual_files` is the store; the program item is
that *every* existence/read/write primitive consults it first with one key
normalization (`vfs_key`, 56i) and one search order, and that the loader
runs latex.ltx's `\@onefilewithoptions` (also K3's step 2). Half-landed.

## K8 — Runaway cap that degrades instead of discarding

**Goal.** A document that fires the same error 500 times keeps its output.

**Source of truth.** tex.web §1283 (`error_count`, 100 → `history=fatal`) is
per *paragraph*/interaction, not per document; Perl's `MaxErrors` fatal is a
beyond-TeX guard. Ours (`TooManyErrors:MaxLimit`, the same-error runaway cap)
aborts the whole conversion, leaving a 39-byte XML (csvsimple-l3,
euclideangeometry, forest-doc before 56i).

**Abstraction.** The cap becomes a *suppression*: after N identical errors
the construct that raises them is neutralized for the rest of the document
(its definition replaced by the error-once no-op), the count is reported
once, and conversion continues. Fatal stays Fatal for genuine kernel faults.

**Risk.** LOW; beyond-Perl reliability lever, recorded as a divergence.

## K9 — Group codes on every frame; closers dispatch on the group code

**Goal.** The parked "fused mode-frame family" (DIFFICULT_CASES D12: msc,
modernposter, dsptricks/psmatrix — a `\begingroup`/`\endgroup` pair or a
deferred `\egroup` straddling a box or cell boundary) stops cascading.

**Source of truth.** tex.web keeps the semantic nest (§211-218 `push_nest`/
`pop_nest`: mode + current list) apart from the save stack (§274
`new_save_level(group_code)`); a box pushes both (§1083-1085 `scan_spec` then
`push_nest`) and is packaged when its GROUP closes (§1085-1086 `package`:
`unsave` → `hpack` → `pop_nest`). `}` dispatches on `cur_group` only
(§1068 `handle_right_brace`: `simple_group` → `unsave`, `semi_simple_group` →
`extra_right_brace`, box groups → `package`); `\endgroup` checks
`cur_group = semi_simple_group` only, else `off_save` (§1064-1065) inserts the
matching closer with "Missing } inserted". Perl fuses the two
(Stomach.pm:330-335 admits it: modes "are NOT correlated to grouping").

**Today.** One `State` frame stack carries groups AND modes: `begin_mode` =
`push_stack_frame(false)` + `BOUND_MODE` bound in that frame (stomach.rs `begin_mode`); `end_mode` demands the top
frame be the mode frame; `egroup`/`endgroup` gate on `BOUND_MODE`-on-top + `groupNonBoxing`; the box reader's
terminal is the raw frame depth (base_utilities.rs). Batch 56z added `lx@group@code` for `bounded` environments only.

**Abstraction (staged; each stage = guard + control).**
- Stage 0, no behaviour change: every opener writes its group code
  (`simple`, `semi_simple`, `hbox`/`vbox`/`vtop`, `math`/`display`/
  `math_left`, `align`), generalizing 56z's `lx@group@code`.
- Stage 1 (msc, modernposter): `end_mode` closes intervening `semi_simple`
  frames the tex.web way (one "Missing } inserted"-class warning) instead of
  refusing; the box reader keys its terminal on the box's OWN group level.
- Stage 2: `\endgroup`/`}`/`\egroup` dispatch on the group code with
  `off_save`/`extra_right_brace` recovery, unified with the existing
  "Missing $ inserted" path (stomach.rs:851-877).
- Stage 3 (psmatrix/dsptricks): alignment cell-boundary group codes — the
  template's `\begingroup…##…\endgroup` pairs close on the save stack
  without exposing the `\halign` box frame.

**Before Stage 1.** Stage 0 resolved msc (56ac). Trace the next in-scope witness first (`\tracinggroups=1` on
pdflatex vs `LXML_TRACE_FRAMES=1`); a local missing pop lands first. Notes `~/data/pk_agents/w22/mode-frames/NOTES.md`.

**Risk.** LOW (Stage 0), MED (1-2: `AlignPeekMode`, `$…$`, `INNER_BOX`/
`\ifinner`, `\aftergroup` order), HIGH (3).

## K10 — A pdfTeX byte mouth

Steps 1+2 LANDED 56bl (package-scoped byte mouth, `decode_byte_mouth_runs`); step 3 dropped (witnesses CJK, out of scope 2026-10-01). Dead end: a global byte mouth under the pdfTeX persona (utf8.def:157-170 vs `utf8_input_keeps_latin1_code_points_other`). Detail: [`archive/KERNEL_CAPABILITIES_LANDED_2026-10-02.md`](archive/KERNEL_CAPABILITIES_LANDED_2026-10-02.md).

## K11 — Raw classes' title-page stores reroute to the frontmatter API

**Goal (user, 2026-09-18, APPROVED — surpass Perl).** "Both `\author` and
`\inst` need to use the latexml provided Frontmatter API so that we can recover
high-quality semantic XML annotating the metadata kinds" and "we need to surpass
perl and detect these common mechanisms rerouting them to the Frontmatter API".
Under LaTeXML's locked `\maketitle` a raw class's `\@maketitle` never runs, so
everything the class STORED for it — affiliations (`\inst`, `\institute`,
`\affiliation`, `\address`), abstract (`\abst`, `\abstract`), dates by role
(`\recdate`, `\received`, `\accepted`), publication info (`\pubinfo`,
`\journal`), `\subtitle`, `\keywords`, `\email` — is lost in Perl (its ~200
class bindings are the only route) and was deposited as body paragraphs by the
Rust `\lx@deposit@maketitle` surpass (sect05.rs:948), presentational and, on
manptp, ahead of the frontmatter. Batch 56di landed the two per-command forms
(`\inst` in an author = `\lx@request@frontmatter@annotation[affiliation]`,
llncs.cls.ltxml:50; a ptptex binding for its five setters); K11 is the general
rule that makes such bindings unnecessary for the common shape.

**Source of truth.** The store convention is mechanical: a class setter whose
body is a pure store — `\gdef\@NAME{#1}`, `\def\@NAME{#1}`, `\long`/`\global`
variants, `\newcommand\NAME[1]{\gdef\@NAME{#1}}` (ptptex.cls:616,
jpsj2.cls:842, gammas.cls:189 …) — is a metadata setter by construction, a
check on the macro's token body, never a guess. The KIND comes from a surveyed
NAME → kind table (evidence: every TL `.cls` scanned; Perl's own class
bindings' setter → `\lx@add@*` mappings; survey
`~/data/pk_agents/w23/frontmatter_stores/`). The frontmatter API is Perl's
(Base_Utility.pool.ltxml:547-640): `\lx@add@{title,subtitle,author,affiliation,
date[role],abstract,keywords,pubnote,…}`, `\lx@add@affiliations` with `$^n$`
labels linked from the authors' `$^{n,}$` (`\lx@author@withsup` /
`\lx@affiliation@withsup`).

**Mechanism (as landed: 56dj, extended 58m, 58q, 59d, 59e).** After a raw `.cls` loads (the
`latexml_core/src/binding/content.rs` after-hook site; a binding's own raw self-load does not fire it),
`\lx@class@loaded@raw{name}` runs `frontmatter_stores::reroute_raw_class_stores`: each setter of the NAME → kind
table that the class defined as a pure store — one argument, or `[optional]{mandatory}` (58m, bfhlayout
`\institute`) — is rerouted to the frontmatter API, and once any store was captured `\let\@maketitle\@empty`, as
Perl's locked `\maketitle` discards it; off-table stores are dropped with it (Perl parity; guard
`only_table_names_with_store_bodies_are_rerouted`). Since 58m the stores are read where the class reads them:
creator stores (affiliation, address, e-mail, ORCID; editor, speaker) at `\maketitle`, the others when set. 58q
leaves a binding's own setters alone (`latexml_core::binding::store_setters`, KPE #418/#419); 59d hands what code
appends to `\@author` to the creator (DIVERGENCES #405); 59e lets a later `\maketitle` typeset the authors set
after the last one (DIVERGENCES #406). Inside author content `\inst{n}` is the affiliation link request
(`\lx@author@withinst`). The 56dj text and the original steps are in the archive.

**Open.** List-append stores other than `\@author` (verify a witness first); retire the per-class bindings whose only
content is store rerouting (ptptex first), keeping those with non-store shapes.

## Ordering

Capabilities are not scheduled on their own (user re-steer 2026-10-01): a batch item from the scoreboard (S3 missing words → schema-invalid → timeouts, in-scope manuals) that belongs to a capability lands as that capability's step, with its class guard, not as a site patch. The remaining steps resume with the whole-branch generalization audit after the goal. Math (K19) is parked with the math stream (2026-10-02).

## Status log

Rows through 2026-09-24: [`archive/KERNEL_CAPABILITIES_STATUS_LOG_2026-09-24.md`](archive/KERNEL_CAPABILITIES_STATUS_LOG_2026-09-24.md); 2026-09-25 to 09-27 (K13 stages 0-2, K13–K17 recorded, the `node_boxes` sweep cadence): [`archive/KERNEL_CAPABILITIES_LANDED_2026-10-02.md`](archive/KERNEL_CAPABILITIES_LANDED_2026-10-02.md). Since 2026-10-01 a capability step lands only as a scoreboard batch's fix (see Ordering).

## K12 — pTeX control-word letters under a pLaTeX class

LANDED 56ds (`PTEX_PROFILE` from `\NeedsTeXFormat{pLaTeX2e}`, `state::is_ptex_kanji_letter`; guard `perfect_kernel_batch56::kanji_control_words_under_platex`). pTeX primitive residual (`\kanjiskip` …) PARKED, SHARED; witnesses out of scope (CJK, 2026-10-01). Detail: [`archive/KERNEL_CAPABILITIES_LANDED_2026-10-02.md`](archive/KERNEL_CAPABILITIES_LANDED_2026-10-02.md).

## K13 — Binding-conformance detector

**Model:** ARCHITECTURE_THEMES theme 7 (typed parameters are claims about how TeX reads) and 2b
(replacing a macro drops its mode transitions).

**Abstraction.** A read-only audit tool (`tools/perfect_kernel/`), not an engine change: for each
package the corpus loads, load it raw in a scratch state and record every public macro's real signature
— parameter text and delimiters, `\newcommand` arity and optional default, xparse argument spec — and
its body prologue (`\leavevmode`, `\par`, `\@bsphack`, `\ifmmode`); read the binding's declared
parameters and constructor options (`mode`, `enter_horizontal`, `leave_horizontal`) for the same control
sequences; report every mismatch ranked by the number of corpus documents that load the package.

**Invariant.** A binding that replaces a macro reads the same arguments and makes the same mode
transition as the macro, or the difference is recorded in OXIDIZED_DESIGN_DIVERGENCES.

**Landing.** (1) Signature extraction from the raw load (our engine's own definitions after a raw
`\usepackage`); (2) binding-side extraction (the definition's `Parameters` and constructor options);
(3) the ranked report, run once over the corpus's packages; its rows then become ordinary batches or
feed K14. **Class guard:** the report flags the known cases (arydshln's `\hdashline` optional, a
`\makebox` without enterHorizontal on the 56ka binary). **Risk:** LOW (read-only).

**As landed (57b, 57c, 57h; design and stage notes: [`archive/KERNEL_CAPABILITIES_LANDED_2026-10-02.md`](archive/KERNEL_CAPABILITIES_LANDED_2026-10-02.md)).**
- Stage 0: `latexml_core::definition::DeclaredMode` on `Primitive`/`Constructor`, read through `Definition::declared_mode()` (guard `perfect_kernel_batch56::a_definition_keeps_its_declared_mode`).
- Stage 1: `latexml::conformance` (`latexml_oxide/src/conformance/`, `test-utils`): `view_of(cs)` + `compare(raw, binding)` (guards `binding_conformance::{kernel_box_family_conforms_to_latex_ltx, arydshln_binding_conforms_to_the_sty, makebox_without_entering_horizontal_is_reported, hdashline_without_its_optional_is_reported, delimited_parameters_read_on_past_their_given_items}`).
- Stage 2: `audit_package`/`real_views` (`audit.rs`), the `binding_audit` bin, `tools/perfect_kernel/binding_conformance.sh` + `binding_conformance_rank.py` (guards `binding_conformance::{the_walker_follows_tex_through_body_assignments, the_walker_reads_what_real_bodies_hand_on, a_package_audit_reads_both_sides}`). Run 57h: 612 bindings, 611 audited (ajmacros' raw load never finishes, DIFFICULT_CASES §D9); 74,250 public macros; HIGH 2,119 / MED 5,704 / LOW 16,858; 60 complete HIGH findings → landed/verified 57i, 57j, 57ah, 57aj (SYNC_STATUS).
- Known walker limits: continuations handed to expl3 conditionals (`\IfBooleanTF`, `\autoref`), run-time `\futurelet` peeks (amsmath `\FN@`), a group opened before the reader (soul `\caps`), context-bound definitions (`\aligned`), real sides reaching LaTeXML definitions (eufrak `\mathfrak`); expandable-ltcmd defaults not read.
- Stage 3 (environments' `\endX`, classes, the allowlist): DEFERRED (re-steer 2026-10-01).
- Validation: re-applying pre-56jx `Let!("\\hdashline","\\hline")` gives OPT_MISSING; a `\@makebox` without enter_horizontal (pre-56kb) gives PROLOGUE_ENTERH_MISSING; HEAD flags neither.

## K14 — Two-phase typed parameters

**Model:** theme 7. **Abstraction:** `latexml_core::parameter` — a binding for a LaTeX *macro* reads its
argument token list exactly as the real macro would (undelimited, delimited, or the kernel's peek), then
parses the typed value (`Dimension`, `Number`, `Semiverbatim`, …) from that list in an isolated mouth,
with one central policy for what remains (56jr's ArgumentTails, generalized). Only bindings of
*primitives* keep scanning the live gullet, as TeX's primitives do.

**Invariant.** A macro binding's parameters never consume input the real macro would not, and never
leave input it would have consumed; expansion happens where the real macro's body would expand.

**Landing.** Per parameter type, in the order K13's report ranks them; each step removes a per-reader
rest rule. **Class guard:** the 56jz template overrun, 56jr's braced rest, and the three sweep-#126
regressions (tkz-grapheur, bxcalc, PixelArtTikz) as repros from different packages. **Risk:** MED–HIGH;
a primitive's scan legitimately crosses into the following tokens, so the primitive/macro line must be
exact.

## K15 — A typed tail of the horizontal list

**Model:** theme 8. **Abstraction:** the stomach keeps the last item of the current horizontal list with
its kind and amount (glue including inter-word space, kern, penalty, char, box), maintained where
material is appended; `\unskip`/`\unkern`/`\unpenalty` (tex.web §1105), `\lastskip`/`\lastkern`/
`\lastpenalty` (§424) and `\removelastskip` act on it. Glue becomes spaces only at the XML boundary,
under one trim rule that replaces `trim_node_right_whitespace` and its dependence on how the DOM split
the text.

**Invariant.** A tail operation sees what TeX's list would hold, independent of text-node splitting.

**Landing.** (1) The tail record and `\unskip`/`\lastskip`; (2) the boundary trim; (3) retire the
per-site whitespace patches. **Class guard:** RED repros `babel-lang/french_highpunct_unskips_space.tex`
and `block-model/paragraph_trim_text_node_split.tex`, plus a `\@bsphack`/`\@esphack` case. **Risk:**
MED; whitespace golden churn.

## K16 — Bibliographies from the style's programs

**Model:** theme 9. **Abstraction:** (a) `latexml_engine`/`latexml_post`: a native `.bst` interpreter —
BibTeX's stack VM with its built-in functions (btxdoc, btxhak) — that writes the `.bbl` the existing
`.bbl` reader digests, so a `.bib` + `.bst` document gets pdflatex's reference text for any style; (b)
the biblatex formatter takes the style's declarations (`\DeclareBibliographyAlias`, `.lbx` bibstrings,
`\DeclareFieldFormat`) instead of hard-coded FMT_SPEC rows — batch 56kc's alias hook is the first
instance.

**Invariant.** A reference list reads as the style prints it; formatter rows are no longer the way a
style feature is added.

**Landing.** (a) is standalone and closes the DEFERRED_FAMILIES `.bib`+`.bst`-without-`.bbl` family
(abntex2cite 80.5 → 99.4 % measured with bibtex's own `.bbl`); (b) per declaration kind. Running
biblatex's drivers raw over the `.bbl` would be exact but loses the per-field markup — needs a ruling.
**Risk:** MED (a is new code but isolated).

## K17 — A fixed, stratified manual regression net per batch

**Model:** theme 10. **Abstraction:** a fixed set of ~200 manuals chosen for package diversity (one or two
per package family, weighted to the mechanisms batches touch), run per batch beside the arXiv A/B, plus
the whole repro catalog (`repros.sh`), each against the previous binary with byte-diff classification
(the arXiv harness's `tex=` fingerprint included).

**Invariant.** A lateral regression is caught at the batch that causes it, not at the next sweep.

**Landing.** Select the set from the census and the sweep history; a runner and a comparer in
`tools/perfect_kernel/`; the gate ladder's L2 then names it. **Risk:** LOW; ~15 min on 64 cores per batch.

**Landed (57a; re-selected 2026-09-29 for the scope ruling).** `manual_net_select.py <sweep> corpus.tsv > manual_net.tsv` picks greedily for load coverage (`(Loading …)`, `(Processing definitions …)`) then output coverage, per √seconds, ≤2 manuals a bundle, from in-scope manuals that ended status 0-2 in < 45 s: from sweep #130, 797 manuals, 3,149/3,201 load and 395/402 output features, 1,202 s serial. `manual_net.sh <binA> <dumpsA> <binB> <dumpsB> <out> [jobs]` runs both sides on cores 64-127 with a fixed `SOURCE_DATE_EPOCH`, validates both sides (outside the 8 GB cap — jing JVMs fail under it) and runs every repro topic; `manual_net_compare.py <out> [--recall]` fails toward flagging (missing manual, incomplete repro run, unscorable recall, worse status, new Fatal, more errors, valid→invalid, words −1 %, recall down, Math `tex=` change, newly failing repro). Null run A = B (56kx5): no changes, 7 min at 24 jobs. Re-select when the corpus changes.

## K18 — Typed box sizes: a box's measured size is its constructed size

**Model:** theme 11.
**Abstraction:** a whatsit's `width`/`height`/`depth` properties are typed `Dimension`s, rendered to attribute text only when the XML is written (`Stored::to_attribute`). Every box constructor has a sizer that carries its own geometry: the frame inside `\framebox[w]`'s width, `\raisebox`'s raise and its optional height and depth, the scale and rotation of the graphics boxes, following graphics.sty's `\Gscale@div` (tex.web §455). The list's own width wins over a width inherited from the whatsit.

**Invariant:** `\wd`/`\ht`/`\dp` of a box equal pdflatex's to the sp where the XML attribute already does, and no size key holds a `Stored::String`.

**Landing:**
1. **Step 1:** the four items of the RED repro, with the width-leak fix as a prerequisite for `\framebox`.
2. **Step 2:** bind `\width`/`\height`/`\depth`/`\totalheight` to the box while a box command reads its size arguments (latex.ltx `\@begin@tempboxa`; 0pt stubs at sect12.rs, as Perl's pool.ltxml:4644-4647), then honour `\raisebox`'s `[height][depth]`. `\Gscale@div`'s arithmetic for `\resizebox`. A debug-build warning in `compute_size_and_cache` for any String size, and the audit of the constructors without a sizer (79 `sizer =>` sites, 54 engine + 25 package/contrib, against 945 `DefConstructor!`, 364 + 581). makecell and diagbox follow.

**Class guard:** the RED repro turns green, and `\makebox[1em]{aaa bbb ccc ddd}` measures 6.94444pt / 0pt.

**Status:** step 1 LANDED in 56kj (guards `perfect_kernel_batch56::{box_sizes_are_their_constructed_sizes, a_box_width_does_not_break_its_contents}`): `\framebox[w]`, `\parbox`, `\raisebox` (Perl's `raisedSizer`), `\scalebox`/`\resizebox`/`\rotatebox`/`\reflectbox` and rotating.sty (sharing `rotated_properties`), and the width leak, measure pdflatex's sizes (DIVERGENCES #327). Step 2, first part LANDED in 56kl (guard `perfect_kernel_batch56::box_size_arguments_measure_the_box`, DIVERGENCES #328): `TempboxaDimension` size arguments are read with the box set (`within_tempboxa`), so `\width`/`\height`/`\depth`/`\totalheight` measure it for `\makebox`, `\framebox`, `\parbox`'s height, `\raisebox`'s `[height][depth]` (its sizer applies them) and `\Gscale@box@dd`/`@dddd`. Open for step 2: `\raisebox`'s raise (needs `yoffset` to render with reserved space, not `position:relative`; RED repro `raisebox_raise_measures_the_box.tex`, 13 papers); `\resizebox{\width}` and `\resizebox*` (`GraphixDimension`/`\Gscale@@box`, 4 papers); `\pic@raisebox`, `{minipage}`'s [height]; `\resizebox{1em}` is 10.00002pt (Perl's) where pdflatex gives 10.00081pt through `\Gscale@div`; makecell and diagbox; the String-size detector and the sizer audit.

**58b:** a second width leak closed — a text group `{…}` in a paragraph was a List with `width=\hsize` and no mode, measured as an `\hsize`-wide paragraph of its own (every measured box holding an inline group: vbox 30.94pt for pdflatex's 6.94pt, parbox 17.92+12.92pt); it is now a `horizontal` List that `flatten_paragraph` opens (guard `perfect_kernel_batch58::text_group_is_measured_in_its_line`).

**Risk:** LOW for (a)-(c), with attributes byte-identical. MED for the width leak, since measured heights of multi-word boxes change and small numeric goldens may move.

## K19 — Order-free math ranking

**Status:** PARKED with the math stream (user 2026-10-02); step 1 LANDED 57av.

**Model:** theme 12.
**Abstraction:** a reading's rank is the tuple of its violation counts under the student pragmas, in their
order (`student_defaults`); `soft_prune_choices` keeps the readings with the lowest count instead of all when
none is clean, and the root-only multi-tree pragmas (`prefer_named_interval_at_root`,
`prefer_non_self_wrapping_root`, `prefer_distributed_relation_at_root`) count over the whole tree.

**Invariant:** the chosen reading does not depend on the order the route enumerated the readings in: the tree
route and pure ASF read a formula alike, and so does the same route with its readings reversed.

**Landing:**
1. Count-based `soft_prune_choices` (RED repro `math-parse/function_application_beside_a_fenced_factor`); A/B
   with a Perl sample — #18 applies `f(x)` where a shared failure hid it.
2. The detector: a test-only override that reverses the readings before ranking; run it over the grouped
   parse goldens, then a corpus scan, and list every formula whose reading moves.
3. The root-only pragmas to whole-tree counts, or retired by the Fence port (#154; 57au landed it for bracket and
   brace lists, so no wrapped interval or `set@(set@(…))` reading exists there any more; paren pairs remain).

**Class guard:** the RED repro turns green; the grouped parse goldens read identically in reversed order.

**Risk:** MED — every formula whose readings all failed some pragma may move; A/B and Perl sample per step.
