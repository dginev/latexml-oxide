# Perfect Kernel — the generalized kernel-capability program

**Approved by the user 2026-09-05**: *"prioritize a well-designed generalized
kernel support, faithful to the originals and with clear and well-defined
abstractions."* This document is the standing plan for that program. It is a
living worklist (no date in the name); each capability row carries its status.

It complements [`ARCHITECTURE_THEMES.md`](ARCHITECTURE_THEMES.md) (the six
recurring mechanisms distilled from batches 33–53). Where a capability *is* a
theme, the theme section holds the model and this file holds the landing plan;
where it is new (K1, K3, K4, K6), the model is here.

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
| K1 | Definition provenance + raw-load-then-overlay bindings | new (2 policy) | "already defined" 261 lines; stub-vs-raw internals (`\pdfstringdefPreHook` ×6, `\siunitx_number_format:nN`, `\ifGm@showframe` ×3, `\chemmacros_load_module:n`, PDF-API / `\LuaUL*` stubs) | 1 | OPEN — `is_latexml_predefinition_source` + `\lx@if@pooldefined` (56i) are the seed |
| K3 | lthooks as the single hook store | 6 | source2e, tikz-ext-manual, euclideangeometry (56i regression), every `\AddToHook{package/…}` no-op | 2 | OPEN — `\AtBeginDocument` routed (56i); the rest of the pool's private stores remain |
| K4 | Kernel templates and sockets backed by our constructors | new | ltx-talk ×10, tagpdf 113 lines, diffcoeff, every tagging-aware class | 2 | OPEN — inert declarations landed (56i) |
| K5 | One line-oriented raw reader | 4 | `\verb` EOL, listings line 1, `\tcbverbatimwrite`, fancyvrb, `\DocInput`, `.listing` round trips (25 docs) | 3 | OPEN |
| K2 | Semantic nest separate from the save stack | 1 (R9, approved 2026-09-02) | `unexpected:\endgroup` 49 docs, three off_save site patches (56g/56h), tutodoc, kblocks, nath | 4 | OPEN — decision brief = theme 1 |
| K6 | A consistent font-selection model for Unicode engines | 5 | polyglossia 136 lines, fontspec queries, `\mathitalicsmode` ×4, most lualatex manuals | 5 | OPEN — polyglossia TRUE stub (56i) is the anti-pattern to replace |
| K7 | One in-memory file model | 6 | VFS `./` (56i), `\jobname` round trips, `\IfFileExists`/`\openin`/`\file_full_name:n` gaps | 6 | half-landed (b42/b47/b50/56i) |
| K8 | Runaway cap that degrades instead of discarding | — | csvsimple-l3, forest-doc (pre-56i), euclideangeometry: 500 same-errors → 39-byte XML | 7 | OPEN |

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

**Design (2026-09-05, from the source survey).** Only `Expandable` carries a
`locator` today (`definition/expandable.rs:78`, set from `gullet::get_locator()`
in `binding/def/dialect.rs:333`); `Primitive`/`Constructor`/`Register` answer
`Object::get_locator() → None` (`common/object.rs:36`), which is why the
56i heuristic had to treat "no locator" as "ours". The origin is therefore
NOT derived from locators but recorded as its own field: a thread-local
`CURRENT_ORIGIN: Cell<DefinitionOrigin>` in `latexml_core::definition`,
set by RAII guard at the five loader seams — `dump_reader::load_from_str_
internal` (`Plain`/`LatexDump`, next to its existing `CURRENT_LOAD_CTX`),
`InnerPool!`/`LoadPool!` (`Pool`, `setup_binding_language.rs:60`),
`latexml_package::dispatch` (`Binding(name)`, `lib.rs:1159`), `content.rs::
input_definitions` for a raw `.sty`/`.cls`/`.def` (`File(path)`), and the
document mouth (`Document`) — and captured at construction by every
definition struct's `new`/`default` (the same moment `locator` is), so
`\let` shares the object and its origin travels with it. The trait gains
`Object::get_origin() -> DefinitionOrigin` (default `Unknown`) and
`DefinitionOrigin::is_latexml_owned()` = not `File`/`Document`. The dump's
`assign_internal('global')` apply keeps the dump origin (rule 2 of the
format boundary: the dump overwrites, and says so).

**Landing plan.** (1) `DefinitionOrigin` on `Definition`, set at the five
loader seams above; replace the locator heuristic
(`is_latexml_predefinition_source`) with `get_origin().is_latexml_owned()`. (2) `retract_pdf_api_stubs`-style retraction becomes
unnecessary: `\cs_new` over a `Binding` origin replaces silently. (3) Audit the
bindings that `Replace` a package whose internals raw code reaches for
(hyperref, siunitx, geometry, fontspec, chemformula, forest) and convert to
`Overlay` one at a time, each with its corpus witnesses re-run. Guards: a
repro per declarator over a pool name from a *class* (ltx-talk shape) and
over a *binding* name from a raw package (lua-ul shape).

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
`\@onefilewithoptions` seam). (3) Delete the private stores. Guards: the
56i `atbegindocument_joins_the_l3_begindocument_hook` and
`pgfmanual_toplevel_atbegindocument_runs_last`, plus a `package/x/after`
repro (DEMO-TUDaPhD `\@addchap`, P16-xii). The euclideangeometry regression
(a `\special_relax` marker inside a `\g__hook_` csname) is the first K3
correctness item: the label/argument must reach lthooks as *tokens*, never
through an expansion that can insert markers.

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

## Ordering

K1 → K3+K4 → K5 → K2 → K6 → K7/K8 (K7 and K8 are small and slot between
batches). Batch fixes continue in parallel, but a batch item that belongs
to a capability is landed *as* that capability's step, with its class-level
guard, not as a site patch.

## Status log

| Date | Row | Event |
|---|---|---|
| 2026-09-05 | all | Program approved by the user; K1/K3/K4 seeds from batch 56i recorded above |
| 2026-09-05 | K3 | Ordering fixed in 56j (L3 hook before the bindings' private store; bindings outrank raw). OPEN correctness item: lthooks' labeled `\exp_args:Nx` cleanup (latex.ltx:5375, 5401-5416) is not reproduced by the gullet — a `\noexpand`-family token surfaced inside `\csname g__hook_…`; parameter-bearing unlabeled chunks are pinned to the private store meanwhile (`hashful_begin_document_chunk_under_a_package_label`). |
| 2026-09-05 | K1 | Design fixed (thread-local origin captured at construction; five loader seams). Implementation next, after sweep #42. |
| 2026-09-05 | K6 | Persona decision pending with the user: DVI default (Perl's `\ifpdf` false / `\pdfoutput=0` = pdfTeX in DVI mode, right for the arXiv legacy), PDF mode as a per-document persona switched by document evidence. |
