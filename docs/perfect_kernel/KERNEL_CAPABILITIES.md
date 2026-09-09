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
| K1 | Definition provenance + raw-load-then-overlay bindings | new (2 policy) | "already defined" 261 lines; stub-vs-raw internals (`\pdfstringdefPreHook` ×6, `\siunitx_number_format:nN`, `\ifGm@showframe` ×3, `\chemmacros_load_module:n`, PDF-API / `\LuaUL*` stubs) | 1 | **Step 1 LANDED** (batch 56l: `DefinitionOrigin` on every definition, five loader seams, `\lx@if@pooldefined` consumes it); steps 2-3 (retraction retirement, overlay audit) OPEN |
| K3 | lthooks as the single hook store | 6 | source2e, tikz-ext-manual, euclideangeometry (56i regression), every `\AddToHook{package/…}` no-op | 2 | OPEN — `\AtBeginDocument` routed (56i); the rest of the pool's private stores remain |
| K4 | Kernel templates and sockets backed by our constructors | new | ltx-talk ×10, tagpdf 113 lines, diffcoeff, every tagging-aware class | 2 | OPEN — inert declarations landed (56i) |
| K5 | One line-oriented raw reader | 4 | `\verb` EOL, listings line 1, `\tcbverbatimwrite`, fancyvrb, `\DocInput`, `.listing` round trips (25 docs) | 3 | OPEN |
| K2 | Semantic nest separate from the save stack | 1 (R9, approved 2026-09-02) | `unexpected:\endgroup` 49 docs, three off_save site patches (56g/56h), tutodoc, kblocks, nath | 4 | OPEN — decision brief = theme 1 |
| K6 | A consistent font-selection model for Unicode engines | 5 | polyglossia 136 lines, fontspec queries, `\mathitalicsmode` ×4, most lualatex manuals | 5 | OPEN — polyglossia TRUE stub (56i) is the anti-pattern to replace |
| K7 | One in-memory file model | 6 | VFS `./` (56i), `\jobname` round trips, `\IfFileExists`/`\openin`/`\file_full_name:n` gaps | 6 | half-landed (b42/b47/b50/56i) |
| K8 | Runaway cap that degrades instead of discarding | — | csvsimple-l3, forest-doc (pre-56i), euclideangeometry: 500 same-errors → 39-byte XML | 7 | OPEN |
| K10 | A pdfTeX byte mouth (256-entry catcode table over U+0000..U+00FF) | new (7 CJK/kotex manuals, D9) | cjk-ko-doc, kotex-doc, kotex-utf-doc, oblivoir-simpledoc, sample-bxcjkjatype-beamer (`\가`/`\japanese`/`\ifx 가가`) | 8 | steps 1+2 landed (56bl, 2026-09-09); step 3 deferred |

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
`push_stack_frame(false)` + `BOUND_MODE` bound in that frame (stomach.rs:1004-
1043); `end_mode` demands the top frame be the mode frame (:1103, error :1118);
`egroup`/`endgroup` gate on `BOUND_MODE`-on-top + `groupNonBoxing` (:741-903);
the box reader's terminal is the raw frame depth (base_utilities.rs:3616).
Batch 56z added `lx@group@code` for `bounded` environments only.

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

**Open question before Stage 1 lands.** pdflatex is clean on the witnesses, so
real TeX's save stack never drifts there; the drift source in our engine
(which push/pop differs — `\tracinggroups=1` on pdflatex vs
`LXML_TRACE_FRAMES=1`) is being pinpointed; a local missing pop would land
first. Design notes: `~/data/pk_agents/w22/mode-frames/NOTES.md`.

**Risk.** LOW (Stage 0), MED (1-2: `AlignPeekMode`, `$…$`, `INNER_BOX`/
`\ifinner`, `\aftergroup` order), HIGH (3).

## K10 — A pdfTeX byte mouth

**Goal.** The last non-policy oracle-clean residue (D9: cjk-ko-doc, kotex-doc,
kotex-utf-doc, oblivoir-simpledoc, sample-bxcjkjatype-beamer) converts. All
five are SHARED with Perl (no CJK/kotex binding exists in either engine).

**Source of truth.** pdfTeX reads a UTF-8 file byte by byte: tex.web §30-31
(`buffer: array of ASCII_code`, `input_ln` decodes nothing), §230-232 (a
256-entry `\catcode` table), §341-356 (`get_next`: one byte, one token;
a control symbol from a single byte, `^^ea` = byte 0xEA). Proven on plain
pdftex (`~/data/pk_agents/w22/byte-mouth/repros/pdftex_byte_probe.tex`):
`가` = three tokens 234/176/128, `\ifx 가가` is FALSE, `\catcode234`=12.
LuaTeX/XeTeX read Unicode scalars (one token U+AC00) — the split that
dhucs-trivcj.sty:18 `\ifx 가가` probes. cjkutf8-josa.sty:176-194 defines
control SYMBOLS over the first byte (`\DeclareRobustCommand*\^^ea[2]`) and
reads the next two bytes as arguments; kotexutf.sty:40-41, CJK.sty:896-910 and
`UTF8.bdg` make 0x80..0xFF active. Perl's Mouth.pm:145-163 splits on `\X`
grapheme clusters and keys catcodes on the code point — no byte mode.

**Mechanism.** The 256-entry table is ALREADY our `catcode: HashMap<char>`
on U+0000..U+00FF: `\catcode"EA` and a `^^ea` in a `.sty` both address
U+00EA today. Byte mode is therefore a per-Mouth flag (sibling of
`saved_at_cc`, mouth.rs:100) under which `decode_bytes` (mouth.rs:586) takes
the Latin-1 branch that already exists at :598 (`b as char`): `\가` lexes to
`\<U+00EA>` = the josa definition, `\ifx 가가` compares U+00EA/U+00B0 → the
legacy branch. Output text needs the bytes reassembled into scalars at the
funnels the packages themselves use: CJK.sty:896-968 `\CJK@XX/XXX/XXXX`
(`\csname CJK@\number\`#1…`), `\CJKchar`, `\Unicode`, kotexutf-core:143-217
`\@@ucs` — net-new bindings (bindings outrank raw) that UTF-8-combine their
byte arguments and emit the scalar.

**Trigger.** Package-scoped opt-in: the CJK/CJKutf8/kotexutf bindings set a
global `BYTE_MOUTH` (gated `!LUATEX_PROFILE`); nested mouths inherit it.
Faithful (those packages ARE the byte-catcode switchers), package-keyed not
hangul-keyed. A GLOBAL byte mouth under the pdfTeX persona is rejected: real
utf8.def:157-170 makes 0xC2..0xFD active (pdflatex `\catcode234`=13 under
utf8), our `utf8_def.rs` deliberately keeps native code points (bibarts,
`utf8_input_keeps_latin1_code_points_other`), and a global mode would split
every accented Latin character. Phase-3 generalization (auto-trigger on an
`assign_catcode` to U+0080..U+00FF while `!LUATEX_PROFILE`) waits for that
invariant's guard against latin1 + `\DeclareFontEncoding{T1}`.

**Steps.** Phase 1 (MED): mouth `byte_mode` + the bindings that set it; CJK,
kotexutf and dhucs run raw; guards `josa_min.tex` (0 errors, josa text
present; today `Error:undefined:\가`) and the trivcj probe (`\japanese`
defined). Phase 2 (MED): the reassembly bindings; guard = a cjk-ko-doc
syllable serializes as its real scalar. Expected gain ≤ 5 docs; bxcoloremoji
and gentombow are the `\pdfoutput` persona (K6), not this. Notes
`~/data/pk_agents/w22/byte-mouth/NOTES.md`.

**Dead ends.** Global byte mouth (above); defining `\japanese`/josa directly
(hangul special case, then needs kotex's font infrastructure); reassembling
byte runs inside the mouth (the probes need the bytes as separate tokens).

## Ordering

K1 → K3+K4 → K5 → K2 → K6 → K7/K8 (K7 and K8 are small and slot between
batches); K9 runs as its own staged batch once the drift source is known; K10 last (surpass-tier, package-scoped). Batch fixes continue in parallel, but a batch item that belongs
to a capability is landed *as* that capability's step, with its class-level
guard, not as a site patch.

## Status log

| Date | Row | Event |
|---|---|---|
| 2026-09-05 | all | Program approved by the user; K1/K3/K4 seeds from batch 56i recorded above |
| 2026-09-05 | K3 | Ordering fixed in 56j (L3 hook before the bindings' private store; bindings outrank raw). OPEN correctness item: lthooks' labeled `\exp_args:Nx` cleanup (latex.ltx:5375, 5401-5416) is not reproduced by the gullet — a `\noexpand`-family token surfaced inside `\csname g__hook_…`; parameter-bearing unlabeled chunks are pinned to the private store meanwhile (`hashful_begin_document_chunk_under_a_package_label`). |
| 2026-09-05 | K1 | Design fixed (thread-local origin captured at construction; five loader seams). Implementation next, after sweep #42. |
| 2026-09-05 | K1 | Step 1 landed (batch 56l): `latexml_core::definition::origin::{DefinitionOrigin, OriginGuard, with_origin, current_origin}`; `Object::get_origin()`; fields on Expandable/Primitive/Constructor/Register/Conditional/MathPrimitive; seams in `dump_reader::load_from_str_internal` (Plain/LatexDump), `InnerPool!` (Pool), `input_definitions` (Pool for `.pool`, Format for latex.ltx/plain.tex/expl3-code.tex, else File), `latexml_package::dispatch` / `latexml_contrib::dispatch` (Binding, or Pool for `.pool` entries); the thread-local default is Document. The 56i locator heuristic is deleted. Guard `raw_double_declaration_still_errors` pins the other side: two raw declarations still error. |
| 2026-09-05 | K5 | OPEN item from the perf lane: `\scantokens` never marks its mouth for `\everyeof` (etex.rs:440-463 unwired, PLANS P15), so spreadtab's `\ST_eat_to_nil` sentinel loop (spreadtab.sty:315-323, :1492) reads past EOF until the cap (spreadtab-en/-fr; SHARED — Perl's `\everyeof` is unused too). The raw-line reader's `\scantokens` pseudo-file budget and this everyeof insertion are the same mechanism; land together, bounded to the pseudo-file (the two earlier naive designs regressed l3doc/spath3/litetable/zref). Repro `perf_scantokens_everyeof_eatloop_hang.tex`. |
| 2026-09-05 | K8 | Finding: the streaming driver ALREADY has the K8 property — `convert_streaming` keeps the document built so far when digestion stops on a resource fatal (core_interface.rs:1215/1241) — but the eager path (the sweep's and cortex's default) discards everything. Auto-streaming is decided up front from `projected_source_bytes × 1900` against the 75 % fuse (`bin/latexml_oxide.rs::resolve_streaming`); a pgf-dense manual (glossaries-user: 1.5 MB source, ~1,500 tcolorbox pictures at ~0.6 MB retained each) projects as "fits" and dies eager at 4.8 GB with a 39-byte XML. Levers, in order: (1) A/B the three memory-fatal manuals with `--streaming` (expected: complete, bounded); (2) a picture-aware projection term (count of picture-opening constructs × measured per-picture retention) or a mid-digestion failover into streaming when RSS passes half the fuse; (3) free `\lxSVG@insertpicture`'s `content_box`/arg after construction so the eager path retains less per picture. |
| 2026-09-05 | K8 | A/B done (user request): the three manuals with `--streaming` (sweep43 binary, `--max-memory=6144`): glossaries-user 256 s / peak 4.76 GB / **9.99 MB XML, 2,178 pictures kept**; datatool-user 361 s / 4.76 GB / 11.1 MB, 2,270 pictures; glossaries-extra-manual 367 s / 4.75 GB / 10.1 MB, 2,129 pictures — each still hits the MemoryBudget fuse (1 Fatal) but keeps the partial document (eager: 39 bytes). The per-fragment log shows the growth is NOT the DOM (libxml C-live ≈ 150 MB throughout) nor the digest box list: RSS climbs 1.7 → 3.2 GB across fragments while `node_boxes` grows 63k → 148k — a Rust-side per-picture retention that streaming cannot spill. Next: identify that holder (node_boxes / whatsit content / arena) and free it after construction. |
| 2026-09-05 | K1 | Step 2 landed (batch 56n): `retract_pdf_api_stubs`/`PDF_API_STUBS` and the lua-ul hook re-declarations retired — Pool-origin stubs are replaced/kept quietly by the K1-aware declarators (guard `tagpdf_base_redeclares_the_stubbed_api_cleanly` unchanged). Step 3 (overlay-binding audit) open: every "binding then raw overlay" site (dhucs, lua-ul) is now a candidate for the same origin-based leniency instead of hand-written `\@ifundefined` overlays. |
| 2026-09-05 | K8 | Measured (batch 56o probe, glossaries-user, `--streaming --max-memory=6144`, debug binary): the retention is STALE `node_boxes` entries — `LXML_NODE_BOXES_SWEEP=20000` sweeps 61,108 → 384 entries at the first spill and again 54,140 → 384 at the second, and peak RSS drops **4.76 GB → 1.72 GB**. But the run then timed out at 800 s on fragment 2 (the earlier run finished pass 1 in 250 s): with RSS under the soft watermark the box-budget yields are rare and each fragment is huge; something in the big-fragment regime is superlinear (suspects: the pending box list, absorb of a large fragment). Control run with `--max-memory=2048` (watermark yields every fragment: 1,024 fragments, 32 segments): still bounded at ~1.03 GB and still ~10× slower (killed at 12 min, user cap for probes is now 3 min) — so the slowness comes with the sweep itself (or the mass drop of the swept `Digested` trees), not from the big-fragment regime. Next: a `perf record` of a 3-minute sweep-on run to attribute it before choosing the trigger. Lever once the slowness is attributed: sweep by WEIGHT (or after every spill — the post-spill spine mark is cheap) instead of the 1M-entry count gate. The 56m picture purge is a settled dead end (shared `Digested`, and `arrange_panels` reads the picture box). |
| 2026-09-05 | K8 | Attributed (Gemini T1, merged): the count-gated `sweep_stale_node_boxes` marks the WHOLE live DOM on every yield when nothing was spilled (4-5 ms per mark × thousands of yields = the 10× slowdown), while a post-spill mark is microseconds and `retain` frees thousands of trees in ~50 ms. Next step (Gemini G1): gate the sweep on `runs_spilled > 0` with a growth fallback, measure on glossaries-user (≤ 3-minute probes), then lower the count threshold. |
| 2026-09-05 | K7 | Confirmed by Gemini T5: `\AtBeginDocument` bodies run in three stores (raw `#`-bearing chunks first, then the L3 `begindocument` pool, then the bindings' private store) where latex.ltx keeps ONE FIFO `\@begindocumenthook`; italian.ldf's hook (nested `##1`) therefore runs before a class hook registered earlier. Design: a single ordered list with a per-entry route (raw-chunk vs L3), replayed in registration order. |
| 2026-09-05 | K7 | Landed (batch 56r): a `#`-bearing unlabeled `\AtBeginDocument` chunk is stored under a fresh `\lx@bdhook@N` macro (a no-parameter body keeps its `#` tokens verbatim) and THAT name is what lthooks receives, so raw hooks run in registration order like latex.ltx's single `\@begindocumenthook` (verifica/italian shape and hep-paper shape both pass). The bindings' private store still fires LAST by design (bindings outrank raw: cleveref). The separate first-fired store remains only for formats without lthooks. Guard `begin_document_hooks_run_in_registration_order`. |
| 2026-09-05 | K6 | Persona decision pending with the user: DVI default (Perl's `\ifpdf` false / `\pdfoutput=0` = pdfTeX in DVI mode, right for the arXiv legacy), PDF mode as a per-document persona switched by document evidence. |
| 2026-09-05 | K3 | Landed (batch 56s): the kernel fused `{verbatim}` constructor unreads `\end{verbatim}` ahead of the rest of its line so the CURRENT `\end` macro runs (latex.ltx:15438 `\@xverbatim`); the fused `\end{verbatim}` is a no-op. knowledge.sty's scope-area push/pop pair is balanced again (knowledge 1→0; RUST-ONLY, Perl never registers the area because it lacks `\verbatim`/`\endverbatim`). Guard `perfect_kernel_batch56::package_state_prtec_psfragx_knowledge` (hooked-`\end` control). Same class to audit: every binding that reads its own environment body. |
| 2026-09-05 | K3 | Audit complete (Gemini round 3 + batch 56s): every self-terminating environment reader — kernel `{verbatim}`, verbatim.sty, alltt, fancyvrb, listings `lstlisting`, minted, tcolorbox listings, comment.sty — hands `\end{X}` to the CURRENT `\end` macro exactly once after its element closes, so `env/X/after` and a hooked `\end` fire (guards `perfect_kernel_gemini::*_self_terminating_hands_to_end`). comment.sty additionally matches pdflatex's whole-line rule (OXIDIZED_DESIGN #133 retracted). |
| 2026-09-05 | K3 | Correctness item, recorded not landed: the gullet's x-expansion of `\exp_not:N\foo` (via `\use:x`, expl3-code.tex:2654) leaves a PERSISTENT noexpand-family token (`\special_relax…`) where TeX's one-shot `no_expand_flag` collapses to the live token; inside lthooks' `\exp_args:Nc`→`\csname g__hook_…` (latex.ltx:5416-5420) the csname builder (gullet.rs:2297) rejects it. Batch 56u removed the trigger (`\DeclareMathOperator` no longer full-expands protected bodies) but any full expansion that reaches ltcmd's `\__cmd_check_definable_aux:nN` (latex.ltx:4385-4396) can still hit it. Fix = make the marker collapse when stored/rescanned (tex.web §369 `no_expand_flag`, one token only); repro `tools/perfect_kernel/repros/macro-state/euclideangeometry-man_special_relax_hook.tex` (green now via 56u; a `\DeclareMathOperator`-free trigger is still to be written). |
| 2026-09-06 | K3 | Landed (batch 56x): `\begin`/`\end` (sect01.rs) and DefEnvironment's begin/end constructors (dialect.rs) fire `env/<name>/{before,begin,end,after}` from the lthooks store (latex.ltx:15347/15362/15388/15391); emitted in the kernel's own token shapes (`\romannumeral\IfHookEmptyTF{env/X/end}{\expandafter\z@}{\z@\UseHook…}` for `end`: an empty hook vanishes at expansion time — any unexpandable token before `\endtabular`'s implicit `\crcr` leaked the last cell's group); DefEnvironment asks `\IfHookEmptyTF` before digesting. etoolbox's private `@environment@*` store still fires; migrating it onto `\AddToHook` is the next K3 step. |
| 2026-09-06 | K6 | Landed (batch 56x), the first model piece: the font FILE a fontspec selection resolves to (`coverage::resolve_fontspec_file` — file name, ls-R stem, or the OpenType `name` table's family) is recorded as `FONTSPEC_FONTFILE` by `\fontspec`/`\setmainfont`/the `\newfontface` definers, and `\iffontchar` answers from its `cmap` (TFM `char_info` for `\font`-declared fonts). The declared-family table and the fontspec conditionals stay open. |
| 2026-09-06 | K1 | Landed (batch 56x): a file that un-marks itself loaded (fontenc.sty tail) declares `<file>_unmarks_itself`; the loader's two `_loaded` gates, the option-clash check, `\ver@` and `\@ifl@aded` honour it — the loader seam for "loaded" is the file's own report, as in latex.ltx. |
| 2026-09-06 | K7 | Correctness item (batch 56x): an environment's closing tag closes only what its own replacement opened (`maybe_close_element` in `exec_ops` and codegen `emit_ops`) — content auto-closing its container is the document model's move, not an error (DIVERGENCES #202). |
| 2026-09-06 | K1 | Correctness item (generalization pass, batch 56x): the loader's notion of "loaded" is a private `_loaded` flag plus the per-file `<file>_unmarks_itself` declaration; latex.ltx's is `\ver@<file>` (`\IfFileLoadedTF` :18463-18469, `\@pr@videpackage` :18471-18490, `\@ifl@aded`). Re-root `already_handled`/`_load_binding`/`\@ifl@aded` on `\ver@` (both the binding and the raw loader must set it) and drop the flag — any self-relaxing file then reloads by itself. MED; own batch. |
| 2026-09-06 | K3 | Correctness item (batch 56x): the env hooks are emitted only when the L3 hook system exists and its `\IfHookEmptyTF` reports code — a bare no-op `\UseHook{env/X/after}` after `\end{X}` of a LINE-READING environment (comment.sty's included comments, `tests/tokenize/comment`) adds a stray newline token, although the same tokens are inert in running text. Trace the comment reader's `unread_expansion(\end{X})` + mouth position with the hook tokens present; then the gate can go and the kernel shapes fire unconditionally as latex.ltx does. |


| 2026-09-06 | K1 | Correctness item (batch 56z): a binding cannot pass an option to ITS OWN raw load — `\PassOptionsToPackage{hidethumbs}{thumbs}` digested inside `thumbs_sty.rs` before `InputDefinitions!(noltxml)` never reached kvoptions' `\ProcessKeyvalOptions*` (the option list is fixed by `before_input_handle_options` when the binding load begins). The binding applies the package's own hide branch after the load instead. Fix = let a binding's raw re-entry re-read `\opt@<file>` (the K1 `\ver@`/option model). |
| 2026-09-06 | — | Landed (batch 56z, no K row: mode/group frames): `\end{document}` abandons open groups tex.web §1335-style — stack frames + `boxing` popped together, `\aftergroup` discarded, one warning — and a `bounded => true` primitive closes only a frame carrying its own group code (`lx@group@code`, written by the opener as tex.web §274 does; checked as §1068 checks `cur_group`); the document close is lenient about elements those groups left open (OXIDIZED_DESIGN #207). |
| 2026-09-06 | — | Landed (batch 56z, no K row: alignment engine): alignment cell mode follows `\@classz` (latex.ltx:16550/16561), a raw `\hbox\bgroup$…\@tabarray` scaffold closes through `\endtabular`'s `$\egroup`, and `ReadAlignmentTemplate` `\edef`-expands the template (OXIDIZED_DESIGN #206). |

| 2026-09-06 | — | Landed (batch 56aa, mode/group frames): a `{` group is digested as a NESTED body (`tex_box.rs` `{` primitive → `digest_next_body(None)`; Perl too, TeX_Box.pool.ltxml:30-41), so a bounded body's terminal arriving inside a brace group never reaches the body's loop. Mechanism: the until-body records its terminal + boxing depth on its frame; the terminal primitive, invoked deeper, abandons the groups above the body (§1335 shape, one Warning; LaTeX `\@checkend` latex.ltx:15394 analogue) and flags the loop (`until_terminal_inside_group`, stomach.rs). `LXML_TRACE_TERMINAL=1` shows which loop reads a token. |
\n
| 2026-09-06 | — | Correctness item (generalization pass, batch 56z, alignment engine): `ReadAlignmentTemplate` expands column macros IN PLACE against the live gullet, so an invoked `\csname`/`\expandafter` can over-read past the template's `}` into the document (nicematrix `V{3cm}` → 1002 errors) — the reason for the interim denylist. General algorithm = latex.ltx:16564/16632's own shape: capture the balanced template group first, then `\edef`-expand INSIDE that captured list on its own mouth; every expandable is then safe and the denylist goes. Moderate restructure of `alignment.rs::read_alignment_template` (expansion is interleaved with the `nopens` scan). |
| 2026-09-06 | — | Correctness item (generalization pass, batch 56z, frontmatter): real TeX stores `\author`'s body unexpanded (`\gdef\@author{#1}`) and expands it at `\maketitle` inside the class's own scope (beamerbasetitle.sty:148/233 `\def\insertauthor{\def\inst{…}#2}`, bfhsciposter's title-box hook); our `\author` (and Perl's Frontmatter model) digests the argument at the setter. The kernel `\providecommand\inst` is a legitimate default under that model; the general fix is deferred frontmatter expansion within the class group — the whole eager collection model. |
| 2026-09-06 | — | Correctness item (generalization pass, batch 56z, luatex persona): functional LuaTeX primitives are added one by one beside the ad-hoc `\luatexversion` defs (`\glet` in 56z, `\csstring` earlier). General home = one `LUATEX_PROFILE`-gated installer table (`\glet`, `\expanded`, `\gtoksapp`/`\etoksapp`/`\gtokspre`/`\etokspre`, `\Uchar`, `\begincsname`, `\lastnamedcs`, each with its faithful body), still EXCLUDING the engine-detection probes (`\directlua`, `\luatexversion`, `\tex_directlua:D`) per `project_lua_bridge_directive`. LOW priority consolidation. |
| 2026-09-06 | K3 | Landed (Gemini round 6, L2/L3): etoolbox's `\AtBeginEnvironment`/`\AtEndEnvironment`/`\BeforeBeginEnvironment`/`\AfterEndEnvironment` and `\apptocmd`/`\pretocmd` on a constructor-backed `\X`/`\endX` route onto the lthooks `env/X/…` store when it exists. Special case recorded: environments whose name starts with `verbatim` keep the private `@environment@…` store (the line-reader path does not run the kernel env hooks) — retire with the K3 line-reader item. |
| 2026-09-06 | — | Correctness item (Gemini round 6, L9, locks): uspatent.cls redefines `\maketitle` to run `\patentStart` (which creates counter `parnum`); the class binding UNLOCKS `\maketitle` before the raw load. General form = the lock keeps the dropped definition's BODY (`<cs>:redefined@body`, beside 56z's `:redefined@nargs`) and the locked `\maketitle` runs it after `\lx@frontmatterhere` when `\maketitle:redefined` — one mechanism for every class that layers real work onto `\maketitle`, no per-class unlock. SUPERSEDED: attempted as Gemini round 7 M2 and REVERTED at the merge (2026-09-06 row below) — a generic raw replay ran resphilosophica.cls:331 against the amsart binding; the per-class binding unlock stands. |
| 2026-09-06 | — | Correctness item (Gemini round 6, L1): the xcolor binding's `\XC@getcolor#1#2` is `\XC@edef#2{#1}`; xcolor.sty:1373-1390 normalises the spec through `\XC@getc@lor` (expression syntax, `\XC@@`-delimited scan) before `\aftergroupdef`. Enough for pstricks' `\pst@getcolor{name}`; a `red!50` expression reaches `\psk@…` unnormalised. |

| 2026-09-06 | — | Landed (batch 56ab, mode/group frames): a pgf graphics-state scope is an `<svg:g>` only, never a TeX group (pgfsys-common-pdf.def:37-38 `q`/`Q`; DIVERGENCES #212). Open root behind msc (21) and modernposter (14), PARKED with D12: `\pgf@maketext` (pgfcorescopes.code.tex:160-184) nests `\setbox\hbox\bgroup \pgfinterruptpicture(=\begingroup) \bgroup` and defers the box's `\egroup` through `\aftergroup`; with a `\vbox` shape part on top, a nested `\hbox` reader's `end_mode` (base_utilities.rs:3624) meets an `internal_vertical` frame. tex.web keeps the save stack (§274, `\endgroup` §1063 checks only `cur_group`) separate from the semantic nest (§211-218); `begin_mode`/`end_mode` (stomach.rs:1004-1103) fuse them. General fix = decouple the nest from the save stack (mission-level; no stopgap pop). `LXML_TRACE_BOUND_MODE=1` gives the backtrace. |
| 2026-09-06 | — | Correctness item (generalization pass, batch 56ab, alignment engine): the cell-head peek runs under an `AlignPeekMode` override (`internal_vertical` around the peek loop, tex_tables.rs) that reproduces tex.web's observable mode at every read — peek §15510, `\noalign` §15513, cell §15532. The structural form is `init_align`'s own: the alignment body BEGINS in `internal_vertical` (§15350) and each row enters `restricted_horizontal` at `init_row` (Perl TeX_Tables.pool.ltxml:181 would `beginMode` per row); then the explicit drop point and the override go. Side effect already carried: `\noalign` material digests in vmode (bound inside its own group). Settled: `MODE` is frame-local (`begin_mode` binds it in the box frame), so the override and its restore must run in the SAME save frame — a restore issued after `start_column` opened the cell group died with that group and left every later cell in vmode (16 goldens: spaces kept before `&`/`\\` in `tex=`, diagbox cells at the text width); the peek now ends the override before any row/cell group opens and `\omit` re-enters after its row group. |
| 2026-09-06 | — | Correctness item (generalization pass, batch 56ab, parameter types): `EqnoTag` is the first parameter that DIGESTS each token (`invoke_token`) up to a stop net and retracts the terminal; every `Until`/`XUntil`/`ProtectedXUntil`/`Expanded` type collects without digesting (base_parameter_types.rs). Seed for a general `DigestUntilRetract(stop)` type once a second display-end bounded digest appears (a `\tag`-style variant, or the `\noalign` group-end reader); single caller today, keep bespoke. |
| 2026-09-06 | K1 | Landed (Gemini round-7 fixups): `\LoadClassWithOptions` = latex.ltx:18637 `\@loadwithoptions` — the calling file's `\opt@\@currname.\@currext` list goes to the loaded class (`calling_file_options`, shared with `\RequirePackageWithOptions`; the document-class list is the fallback for bindings delegating outside a file load). The K1 `\ver@`/`\opt@` re-rooting item stays open; this is one more seam that reads `\opt@` the way latex.ltx does. |
| 2026-09-06 | — | REVERTED at the merge (Gemini round 7 M2, macro state): replaying a class's dropped `\maketitle` body raw after the kernel's frontmatter deposit ran resphilosophica.cls:331's body (`\@setcopyright`, `\andify\shortauthors`, `\@maketitle@hook`) against the amsart BINDING, which defines none of them (guard `amsart_maketitle_internals_are_defined`) — the unseen-class backfire of a generic replay. Rule kept: the lock drops a class `\maketitle`; a class owns it only through its binding's explicit unlock (`uspatent_cls.rs` restored: `\maketitle:locked=false` + raw load — the class body is complete only when its base chain is raw or bound with the internals). General form unchanged: the class's definition should be the whole `\maketitle` with the kernel's frontmatter deposit hooked in, which needs the deferred-frontmatter model (`\author` digested at `\maketitle` in the class's scope). |
| 2026-09-06 | — | Correctness item (Gemini round 7 M4, graphics): the DVI persona has no PDF page count — `\pdflastximagepages` is a stub (pdftex.rs) and l3graphics' `\__graphics_backend_get_pagecount:n` answered by `extractbb -O` (expl3-code.tex:33334-33350) now returns l3's fallback 1 (expl3-code.tex:33348). `latexml_core/src/util/image.rs` parses PDF page BOXES but exposes no page COUNT; general fix = one PDF page-count reader feeding both `\pdflastximagepages` and the l3 backend hook (notebeamer `pages=-` loops over it). Trap found at the merge: the hook was written under `\ExplSyntaxOn` at latexml.sty top level; latexml.sty's body runs WHILE the format loads, so that pulled a raw expl3.sty whose backend selection loaded l3backend-dvips.def at preload time — a later `\pdfoutput=1`/`\sys_load_backend:n{pdftex}` hit "Backend configuration already set" (`backend_load_follows_pdfoutput_and_prior_choice`). Rule: latexml.sty top-level code uses `\csname` for expl3 names, never `\ExplSyntaxOn`. |
| 2026-09-06 | K9 | Landed (batch 56ac): every stack frame carries a serial (`lx@frame@id`, Stage 0) and the box reader ends on its OWN frame, not on the save-stack depth (tex.web §1068 dispatches on `cur_group`). The msc "crossing" root: `\globaldefs=1` (msc.sty:2616) globalized the frame bookkeeping through `assign_internal`'s override (Perl State.pm:144-151 too) — the records now bind via `assign_local_unconditional`, exempt from `\globaldefs`/`\global` (tex.web §1214 vs §274; KPE #209, DIVERGENCES #215): msc 21→0. Settled dead end: `\endgroup` off_save insertion of the box's `}` (Stage 2 shape) — the inserted brace reaches the nested consumer, not the box reader owning the frame; modernposter (14) and psmatrix/dsptricks stay open (Stages 1/3). |
| 2026-09-06 | K9 | Stage 3 scoped (psmatrix root-causer): the row/column boundaries are BOXING save frames (`start_row`/`start_column` `bgroup`, alignment.rs:420/453; Perl Alignment.pm:218/242 identical) where tex.web has nest levels (§15532/§15560), and the `\halign` itself is a bounded-env `bgroup` + `begin_mode` where tex.web has two non-boxing `align_group`s (§15336/§15338). A template straddling a box (pst-node.tex:1215/1222) crosses them. Fix shape: nest levels for row/column (invisible to `\endgroup`/`\egroup`/`}` matching), align-coded non-boxing frames closed only by `end_column`/`end_row`/`fin_align`. HIGH risk (53_alignment goldens) — own batch; repro `repros/kernel-alignment/psmatrix_single_cell_halign_boxing.tex`, control `\vbox{\halign{\begingroup#\endgroup\cr X\cr}}`. |
| 2026-09-06 | K3 | Landed (batch 56ad, hooks): every document hook runs INLINE in the galley — `begindocument/end` and etoolbox's `\AfterEndPreamble` store are unread at the end of `\begin{document}`, and `\end{document}` expands to the `\AtEndDocument` list + the `enddocument` lthook + `\lx@finalize@document` (latex.ltx:15255-15259, etoolbox.sty:1774-1776). A box/environment opened from one hook and closed from the other reads the body as its contents (modernposter 14→0; KPE #211, DIVERGENCES #217). Remaining isolated digests at `\begin{document}`: the legacy `@at@begin@document` stores (raw-param chunks, the L3 pool, the bindings' private store) and the babel activation — same principle applies if a witness opens a body-spanning construct from `\AtBeginDocument`. |
| 2026-09-06 | — | Landed (batch 56ae, alignment engine, generalization A): ONE `\halign` after-digest (`tex_tables.rs` `halign_after_digest`: mode frame, template parse, `to` width, the caller's bindings, body digest, align-state balance for a `{` opener only — tex.web §347) shared by the tabular primitive and the SVG matrix `\lxSVG@halign`; the SVG copy had kept an unconditional decrement after batch 54 guarded the tabular one, so every tikz-cd/pgf matrix inside an amsmath cell left the cell's hidden `$` math open. Also the amsmath binding now initializes `\maxcolumn@widths` (amsmath.sty:1896) — a register raw packages read (cryptocode). |
| 2026-09-06 | K9 | Stage 3 RETIRED for psmatrix (batch 56af): the "row/column boundaries are boxing frames" diagnosis rested on a cell whose u-part never opened its box — `pstricks_support_sty.rs` stubbed `\pst@object{}` as `#1`, so `\psm@beginnode` typeset its name and the v-part closers hit our cell frame. With pstricks.tex's real dispatch the psmatrix template nests properly and the frames balance. Lesson (twice today): a "structural" mismatch whose first error is a stray closer — check the OPENER's binding for a stub before designing a nest/save-stack split. K9 stays as the design for a real straddle; no witness needs it now. Cf. K1 step 3 (overlay-binding audit): every "binding then raw overlay" site is a candidate for exactly this stub-shadows-dispatch defect. |
| 2026-09-06 | K1 | Step 3 first pass done (8 most-hit stubs, sweep 57): 0 retire, 2 silent drops closed (batch 56aj: datetime named dates, mdframed subtitles). Method: rank stubs by (oracle-clean docs hitting them × content risk), probe the main construct of each for body/structure survival, read the real .sty's top for engine primitives before proposing a retire. Next candidates by hit count: pst-plot (17 docs, refusing stub), forest (12, side goal R8b), chemnum (11), pst-all (5). |
| 2026-09-07 | K6 | Engine persona, new data point (batches 56ak/56am): with the §484 halt faithful, a wrong-identity run of a LuaLaTeX/XeLaTeX-authored doc dies in seconds — so the persona decision can be made AFTER the fact by retrying under `luatex` (harness now does; 46 sweep-59 docs). A production converter should adopt the same one-retry rule rather than guess the engine up front; `\RequirePDFTeX` has no corpus users, so the reverse retry is not needed. |
| 2026-09-09 | K6 | Persona measurement on sweep 62: of the 805 lualatex-oracle docs whose oracle was NOT clean (so they run under the pdfTeX identity), 125 load a fontspec-family package (`fontspec`, `luacode`, `unicode-math`, `polyglossia`, `luatexja`) — 45 are already S0/S1, 59 status 2, 18 fatal, 3 timeouts. Their first errors split into LuaTeX-identity gaps (`\setmainfont`/`\setmonofont`/`\newfontfamily` 32, `\directlua` 5, `\luatexversion` 1 — a source-grep persona rule would recover roughly a dozen) and engines we never impersonate (`\newXeTeXintercharclass`/`\XeTeXcharclass` 6, `\epTeXinputencoding` 6, bidi/xepersian fatals). A persona rule is a harness lever measured in its own sweep (one lever per run), not folded into a code sweep. |
| 2026-09-09 | K6 | Second persona signal in the harness (`run_doc.sh`): pgf's graph-drawing library gate ("You need to run LuaTeX to use the graph drawing library", pgflibrarygraphdrawing.code.tex:16-24) now triggers the one-shot luatex retry like the §484 terminal-read halt. tikz-ext-manual (lualatex oracle exit 1, so the clean-oracle gate ran it as pdfTeX) had 14 of its 16 errors from that gate; under the luatex identity our kernel renders the force layout (0 errors on the repro) where Perl-under-luatex still fails pgf's gate (15). Rule confirmed: decide the identity by an unambiguous engine-required signal, never by the oracle engine alone. |
| 2026-09-09 | K6 | `\pdfoutput=0` (the DVI persona, `pdftex.rs:11`, Perl pdfTeX.pool.ltxml:23) now has three named pdflatex-clean casualties: l2tabu (scrbase `\ifpdfoutput` false branch, 2 errors), gentombow (gentombow.sty:582 driver branch, 1) and bxcoloremoji-shortnames (bxcoloremoji.sty:461 graphicx gate, 3). All SHARED with Perl. A PDF-mode persona (`\pdfoutput=1` with the graphics/color/hyperref driver selection that follows) is the pending decision; note `latexml_sty/mod.rs:468` already sets `\outputmode=1`, so the two answers disagree today. |
| 2026-09-09 | K6 | Fourth `\pdfoutput=0` casualty in the pdflatex-clean set: scanpages-doc (scanpages.sty `\GenericError{Must be processed with pdf[la]tex!}` under the DVI persona, 1 error). Same pending persona decision. |
| 2026-09-09 | K10 | Designed (root-causer design study): per-Mouth byte mode over the existing U+0000..U+00FF catcode entries, package-scoped trigger from the net-new CJK/kotexutf bindings, reassembly at the packages' own `\CJK@XXX`/`\@@ucs` funnels. Global byte mode rejected (utf8.def activation vs our native-code-point design). Two phases, ≤ 5 docs. |
| 2026-09-09 | K10 | Landed (batch 56bl): steps 1+2 — `enable_pdftex_byte_mouth` (mouth `bytes` decode + utf8.def:174-190 activation with parameterless protected lead bytes over `\lx@utfviii@N@octets` readers, sticky across inputenc reloads, `\DeclareUnicodeCharacter` → `\u8:<bytes>`), triggers in kotexutf/dhucs/CJK, funnels `\unihangulchar`/`\CJK@XXX` emit the character. Lessons: the byte itself must stay parameterless (packages re-wrap it with `\unexpanded\expandafter{~}`); every `.dfu` loaded after the switch addresses bytes, not characters. Blast radius = 53 corpus docs loading CJK/dhucs/kotexutf (log scan); step 3 (auto-trigger on a byte catcode assignment) still deferred. |
| 2026-09-07 | K1 | Step 3 pass two (batch 56an): pst-plot refusing stub → argument-consuming binding; pst-all → the real require chain; chemnum/forest KEEP with named blockers. Running tally of the audit: 12 stubs examined, 2 retired-in-spirit (pst-plot, pst-all), 2 silent drops closed (56aj), 0 raw-load retirements. Next by hit count: libertinehologopatch (27 docs, 0 oracle-clean), listings (13, 0), nag/tabu/xr KEEP. |

