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
| K1 | Definition provenance + raw-load-then-overlay bindings | new (2 policy) | "already defined" 261 lines; stub-vs-raw internals (`\pdfstringdefPreHook` ×6, `\siunitx_number_format:nN`, `\ifGm@showframe` ×3, `\chemmacros_load_module:n`, PDF-API / `\LuaUL*` stubs) | 1 | **Step 1 LANDED** (batch 56l: `DefinitionOrigin` on every definition, five loader seams, `\lx@if@pooldefined` consumes it); steps 2-3 (retraction retirement, overlay audit) OPEN |
| K3 | lthooks as the single hook store | 6 | source2e, tikz-ext-manual, euclideangeometry (56i regression), every `\AddToHook{package/…}` no-op | 2 | OPEN — `\AtBeginDocument` routed (56i); the rest of the pool's private stores remain |
| K4 | Kernel templates and sockets backed by our constructors | new | ltx-talk ×10, tagpdf 113 lines, diffcoeff, every tagging-aware class | 2 | OPEN — inert declarations landed (56i) |
| K5 | One line-oriented raw reader | 4 | `\verb` EOL, listings line 1, `\tcbverbatimwrite`, fancyvrb, `\DocInput`, `.listing` round trips (25 docs) | 3 | OPEN |
| K2 | Semantic nest separate from the save stack | 1 (R9, approved 2026-09-02) | `unexpected:\endgroup` 49 docs, three off_save site patches (56g/56h), tutodoc, kblocks, nath | 4 | OPEN — decision brief = theme 1 |
| K6 | A consistent font-selection model for Unicode engines | 5 | polyglossia 136 lines, fontspec queries, `\mathitalicsmode` ×4, most lualatex manuals | 5 | persona LANDED (56id, DIVERGENCES #285: `\ifpdf` follows `\pdfoutput`, 1 unless the source ships EPS/PS) and the font-file record (56x); OPEN — the polyglossia TRUE stub (56i) is the anti-pattern to replace |
| K7 | One in-memory file model | 6 | VFS `./` (56i), `\jobname` round trips, `\IfFileExists`/`\openin`/`\file_full_name:n` gaps | 6 | half-landed (b42/b47/b50/56i) |
| K8 | Runaway cap that degrades instead of discarding | — | csvsimple-l3, forest-doc (pre-56i), euclideangeometry: 500 same-errors → 39-byte XML | 7 | OPEN |
| K9 | Group codes on every frame; closers dispatch on the group code | 1 | the fused mode-frame family (DIFFICULT_CASES D12: msc, modernposter, dsptricks/psmatrix) | staged | Stage 0 LANDED (56ac: frame serials, the box reader ends on its own frame; msc 21 → 0); stage 3 retired for psmatrix (56af); no witness needs the rest now |
| K13 | Binding-conformance detector: the real macros' signatures and prologues against the bindings | 7, 2b | arydshln `\hdashline[..]`/`;{..}` (56jx), `\makebox`/`\raisebox` enterHorizontal lost (54n→56kb), the "arity long tail" | after the current goal, first | IN PROGRESS: design recorded, stage 0 (`DeclaredMode`) LANDED 57b |
| K14 | Two-phase typed parameters: read the macro's argument, then parse the type inside it | 7 | 56jm, 56jr/56ju (and 56jr's three sweep-#126 regressions), 56jw A-D, 56jz, 56kf's first cut (every `{Dimension}` read as a `\setlength` operand; now the declared `SetlengthDimension`) | after K13 | OPEN (recorded 2026-09-26) |
| K15 | A typed tail of the horizontal list (glue, kern, penalty, char, box) | 8 | 56jy trim, babel-french `;`, the paragraph text-node split, `\@bsphack`/`\xspace` spacing | after K13 | OPEN (recorded 2026-09-26) |
| K16 | Bibliographies from the style's programs: a native `.bst` interpreter; biblatex from its declarations | 9 | abntex2cite 80.5 → 99.4 % measured, biblatex-chicago/apa samples, every future formatter row | own sessions | OPEN (recorded 2026-09-26) |
| K17 | A fixed, stratified manual regression net per batch | 10 | 56jr/56js regressions found five batches late (sweep #126) | cheapest; any time | LANDED (phase 57, 2026-09-27): `manual_net.{tsv,sh}`, `manual_net_select.py`, `manual_net_compare.py` |
| K18 | Typed box sizes: a box's measured size is its constructed size | 11 | `\framebox[w]` 12.08 vs 10.0pt, `\raisebox` height, `\parbox` 0.1pt rounding, `\resizebox`/`\scalebox`/`\rotatebox` sizes (56kf side finding); `\makebox[w]` contents line-broken at w | step 1 (the four + the width leak) 56kj; step 2: `\height` etc. bound to the box (`\raisebox[h][d]`), `\Gscale@div`, makecell/diagbox, the sizer audit | STEP 1 LANDED (56kj); step 2: `\height` binding LANDED (56kl), `\resizebox`/`\Gscale@div`/makecell/diagbox/audit OPEN |
| K10 | A pdfTeX byte mouth (256-entry catcode table over U+0000..U+00FF) | new (7 CJK/kotex manuals, D9) | cjk-ko-doc, kotex-doc, kotex-utf-doc, oblivoir-simpledoc, sample-bxcjkjatype-beamer (`\가`/`\japanese`/`\ifx 가가`) | 8 | steps 1+2 landed (56bl, 2026-09-09); step 3 deferred |
| K11 | Raw classes' title-page stores reroute to the frontmatter API | surpass (user-approved 2026-09-18) | jpsj2, ptptex; 390 of 655 TL classes define store setters | — | steps 1-2 LANDED (56di, 56dj); OPEN: two-argument `[short]{long}` setters, list-append stores (`\g@addto@macro`) |
| K12 | pTeX control-word letters under a pLaTeX class | CJK (D9) | jsarticle `\if西暦`, ptex-manual `\黄マーカー` | — | LANDED (56ds, `PTEX_PROFILE` + upTeX `kcatcode`); the pTeX residual is PARKED, SHARED with Perl |

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

**Mechanism (as landed, batch 56dj).** After a raw `.cls` loads
(`latexml_core/src/binding/content.rs`, the `-h@@k`/after-hook site; bindings
still precede raw, and a binding's own raw self-load never passes
`handleoptions`, so it does not fire): `\lx@class@loaded@raw{name}` runs
`frontmatter_stores::reroute_raw_class_stores` — for each setter of the
explicit 45-name table that the class defined as a one-argument macro whose
body is a pure store, REPLACE the setter with the API form of its kind, and,
once any store was captured, `\let\@maketitle\@empty` exactly as Perl's locked
`\maketitle` discards it. Off-table stores are therefore dropped with the
`\@maketitle` that read them (Perl parity; the `\lx@deposit@maketitle` surpass,
which recovered them as body paragraphs, was the mis-ordering this capability
replaces) — the `only_table_names_with_store_bodies_are_rerouted` guard pins
that a `\logo` store leaves no trace. Names on the exclusion list are simply
not in the table. One whole-element guard per kind, plus the ptptex fixture on
the general rule. Inside author content `\inst{n}` is always the affiliation
link request (`\lx@author@withinst` `\def`s it, so a rerouted document-level
`\inst` list setter cannot shadow it).

**Steps.** (1) survey → table + exclusions + corpus coverage (running); (2) the
detector on the macro body + the table, behind the raw-class seam; (3) kinds in
corpus order, each with its guard and a Perl-parity note (Perl loses them);
(4) retire the per-class bindings whose only content is store rerouting
(ptptex first), keep those with non-store shapes.

## Ordering

K1 → K3+K4 → K5 → K2 → K6 → K7/K8 (K7 and K8 are small and slot between
batches); K9 runs as its own staged batch once the drift source is known; K10 last (surpass-tier, package-scoped); K11 (surpass-tier, frontmatter) runs
as its own staged batches once the survey lands. K13–K17 (recorded 2026-09-26) wait for dedicated sessions
after the current large goal: K17 is cheapest; K13 first among the kernel ones, since its worklist
sizes K14 and theme 2b; K15 and K16 are independent. Batch fixes continue in parallel, but a batch item that belongs
to a capability is landed *as* that capability's step, with its class-level
guard, not as a site patch.

## Status log

Rows through 2026-09-24 are in [`archive/KERNEL_CAPABILITIES_STATUS_LOG_2026-09-24.md`](archive/KERNEL_CAPABILITIES_STATUS_LOG_2026-09-24.md).

| Date | Row | Event |
|---|---|---|
| 2026-09-27 | K13 | Stage 0 landed (batch 57b): `DeclaredMode` on `Primitive`/`Constructor`, filled by every mode-taking definer, which now builds its prologue closures from it; `Definition::declared_mode()`. Behavior-neutral: the manual net (1000 manuals, 970 repros) is byte-identical. Guard `perfect_kernel_batch56::a_definition_keeps_its_declared_mode`. Next: stage 1, the walker and comparator on arydshln and the kernel box family. |
| 2026-09-26 | K13–K17 | Recorded from batches 56jm–56kc and sweep #126 (ARCHITECTURE_THEMES themes 7–10, 2b), on the user's standing practice: record as met, implement in dedicated sessions after the current large goal. |
| 2026-09-25 | K8 | Batch 56it supersedes the `runs_spilled > 0` sweep gate below: once a large block stays resident (a glossary of ~3,000 definitions flushed at the root), a mark after every spilling yield costs O(resident DOM) each time (datatool-user 11.6 s, glossaries-extra-manual 82.4 s of marking). The sweep now runs when `node_boxes` grows by `max(last/8, 256)` over the size after the last sweep (capped by `LXML_NODE_BOXES_SWEEP`, default 50,000), the baseline drops with every spill that purges below it, and the finishing yield sweeps. Spill-time purging is unchanged. Guards `perfect_kernel_gemini::{resident_glossary_does_not_sweep_every_yield, align_stale_node_boxes_are_swept, spill_gated_node_boxes_stays_bounded}`. |

## K12 — pTeX control-word letters under a pLaTeX class

**Batch 56ds (2026-09-18).** pTeX/upTeX give every non-ASCII code point a
`kcatcode` (upTeX manual: 16 kanji, 17 kana, 18 other kchar, 19 hangul, 20
modifier; ASCII always Latin) and a control word after `\` extends over kanji,
kana (and, upTeX, hangul) characters — `\if西暦` is one control sequence
(jsarticle.cls:1927 `\newif\if西暦`; ptex-manual `\黄マーカー`, where the JIS-row
`ー` (kcatcode 18) ends it under pTeX but not upTeX). Both LaTeXML engines tokenize
kanji as OTHER (Perl State.pm:113-115, Mouth.pm:163), so `\newif` names the bare
`\if` and lets it to `\iffalse` (Package.pm:1220; batch 56do made Rust faithful to
that), and every later `\if` in the job is false: chuushaku 73 errors, gentombow,
plext, qworld, and sample-bxjaprnind's `.`-delimited tokenizer (svn-prov.sty:87-107)
running off the end of input into the batch-52 `Until:` Fatal. Perl cannot fix it;
this surpasses Perl.

**Mechanism.** `state::is_ptex_kanji_letter` is the block set of the kanji, kana
and hangul kcatcodes (Hiragana/Katakana U+3040-30FF and extensions, CJK radicals,
Bopomofo, Kanbun, CJK Unified Ideographs and extensions A-J, compatibility
ideographs, Hangul Jamo and syllables, fullwidth digits and Latin; upTeX manual
`01uptex_doc_utf8.txt:491-533`) — NOT the Unicode letter class, so
Latin-1/Greek/Cyrillic (kcatcode 15, inputenc bytes under pTeX) and the
kcatcode-18 symbols stay OTHER: the U+3000 block including 々〆〇 (`\foo々Y` is
`\foo` then text in platex and uplatex), ー U+30FC under pTeX, halfwidth ー
U+FF70. The set is upTeX's; pTeX proper joins only BMP kanji and kana (not
Extension A, Bopomofo, Hangul), an over-approximation no witness reaches — a
second, narrower profile if a pLaTeX2e witness ever needs it. `\ifcat` on a
kanji reports LETTER here where real pTeX reports the kanji catcode: the LETTER
default is the model's means to the control-word scan, not asserted. `unicode_letter_catcode_default` (state.rs) letters
those code points under `PTEX_PROFILE` exactly as the LuaTeX profile letters the
L/M class (circledtext's precedent); the mouth's control-word scan
(`handle_escape`) needs no change, and `\string`/`\csname` follow. The profile is
raised by `\NeedsTeXFormat{pLaTeX2e}`/`{upLaTeX2e}` (sect05.rs; jsarticle.cls:14,
before the class's kanji control words) and never by a `LaTeX2e` document, so
pdfTeX documents keep kanji OTHER, as pdflatex does. Dump-safe: the formats never
declare pLaTeX2e.

**Residual, PARKED (shared with Perl):** the pTeX engine primitives `\kanjiskip`,
`\xkanjiskip`, `\prebreakpenalty`, `\postbreakpenalty`, `\inhibitxspcode`,
`\xspcode`, `\jis`, `\jfam`, `\iftombow`, `\pfmtversion`, `\hour`, `\minute`,
`\Cwd`/`\Cvs`/`\Cht`/`\Chs`/`\Cdp` — the standing rule never defines them.

**Guard:** `perfect_kernel_batch56::kanji_control_words_under_platex` (jsarticle:
`\if西暦` true, `\csname if西暦\endcsname` the same token, `\foo々Y` leaves `\foo`
intact, no Fatal, no `\西`; article: kanji OTHER). Witnesses: js1 (jsarticle +
tikz) 28 → 27 errors (the parked set), sample-bxjaprnind Fatal → 62 errors = Perl's
62 and completes, chuushaku-sample 45 → 44 (Perl 56). `\NeedsTeXFormat` is now
non-expandable like real LaTeX's (Perl's is an empty macro); nothing `\ifx`es it.

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

**Design (phase 57, 2026-09-27; Plan agent, file:line to re-verify when implementing).**
- *Real side, in process, not `\meaning`.* `\meaning` loses a `\newcommand` default, and grepping
  the `.sty` misses `\let`/`\csname`/expl3 definitions. A preamble-only session (the LSP pattern:
  `reset_thread_state`, `Converter::from_config`, `digest_content_with_provenance`) runs
  `\documentclass{article}` → a mark → `require_package(P, noltxml)` → a mark. The marks take
  `stage_snapshot`s (state.rs), and their `Rc::ptr_eq` diff is exactly what P installed. A raw
  `\newcommand` is our primitive (`convert_latex_args`, content.rs), so it yields one `Expandable` with
  an `Optional` first parameter, and no `\@protected@testopt` wrapper, except in the kernel dump. The
  kernel is a pseudo-package read from the dump (`load_native_dump`, `collect_meaning_keys`).
- *Binding side.* A second session with `\usepackage{P}` looks up the same keys; an origin of `File`
  means a raw passthrough, conformant by construction. `def_constructor`/`def_primitive` (dialect.rs)
  turn `mode`/`enter_horizontal`/`leave_horizontal`/`bounded` into anonymous closures, so **stage 0**
  adds a `DeclaredMode` record to `Constructor`/`Primitive` and a `declared_mode()` trait method
  (behavior-neutral).
- *One chain walker for both sides* (bounded symbolic evaluation of a body): robust `\protect\X␣`,
  `\@testopt`/`\@protected@testopt` optionals, the named peeks (`\@ifnextchar`, `\@ifstar`,
  `\peek_meaning:NTF`, …), xparse `\__cmd_start:nNNnnn{spec}`, tail calls reading past the body,
  the mode prologue (`\leavevmode`, `\par`, `\@bsphack`, `\ifmmode`, `\hmode@bgroup`), and ambiguous
  branches.
- *Comparison.* The signature normalizes to M / O(default) / S / T / D / L / UB, plus a prologue set.
  Mismatch classes: HIGH (OPT_MISSING, OPT_EXTRA, ARITY, DELIM, PROLOGUE_ENTERH/LEAVEH_MISSING),
  MED, LOW (SCAN_KIND feeds K14). Accepted differences are allowlisted with their DIVERGENCES entry.
- *Tool.* `latexml_oxide/src/conformance/{view,walk,compare}.rs`; a `binding_audit` bin gated on
  `test-utils`; `tools/perfect_kernel/binding_conformance.sh`, one subprocess per package. Output is
  `conformance.tsv`, ranked by the corpus documents that load the package, then severity.
- *Stages.*
  0. `DeclaredMode`.
  1. Walker and comparator on arydshln and the kernel box family (`\makebox`, `\mbox`, `\framebox`,
     `\fbox`, `\raisebox`, `\parbox`, `\rule`, `\textcolor`, `\colorbox`).
  2. Driver and weights over the 523 binding packages and classes the corpus loads.
  3. Classes, environments, the allowlist, and triage into batches and K14.
- *Stage 0 landed (57b).* `latexml_core::definition::DeclaredMode` (mode after `text` is lowered, enter/leave
  horizontal, bounded, require/forbid math) on `Primitive` and `Constructor`, filled by `def_primitive`,
  `def_constructor`, `def_environment` (both `\begin{env}` and `\env`) and the DefMath constructors
  (`transfer_common_constructor_options`), read through `Definition::declared_mode()`. The definers build
  their prologue closures from the record (`command_declared_mode` lowers `text`), so the record cannot
  drift from what digestion does. Guard `perfect_kernel_batch56::a_definition_keeps_its_declared_mode`, one
  witness per definer path. What stage 1 must know about it:
  - it holds what the options compile to: an environment that names no mode records
    `restricted_horizontal`; `bounded` is dropped when a mode is given (`\@makebox` declares it and records
    `false`); DefMath's `nogroup` defaults on, so math constructors record no group;
  - it covers the opening side only; the closers (`\end{env}`, `\endenv`) undo it;
  - a robust command's record is on its `\cs␣` body, not on the `\protect` wrapper `\cs`;
  - about 25 bindings perform a prologue by hand inside their closures (`enter_horizontal()` in
    tex_glue.rs, tex_character.rs, `\leavevmode` in plain_bootstrap.rs, …); the record does not see them, so
    the comparator must read those closures' prologues as unknown, not as missing;
  - `None` on a primitive or constructor (one built by hand, as pgfsys's shadings are) means none declared.
  The prologue order is Perl's (`DefPrimitiveI` Package.pm:1303-1309): math checks, enter, leave, mode
  or group. No definition declares both enter and leave (7 `leaveHorizontal` in Perl's pools, none with
  `enterHorizontal` or `mode => 'text'`), so the comparator reports a record with both as a finding.
- *Validation.* Mutation tests: re-applying the pre-56jx `Let!("\\hdashline","\\hline")` must give
  OPT_MISSING, and a `\@makebox` without enter_horizontal (pre-56kb) must give
  PROLOGUE_ENTERH_MISSING. HEAD must flag neither.

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

**Landed (phase 57).** `manual_net_select.py <sweep> corpus.tsv > manual_net.tsv` picks the set greedily from a
sweep: first for coverage of what the corpus loads — every binding (`(Loading …)`) and every raw file read as
definitions (`(Processing definitions …)`, the raw `.sty`/`.cls`/`.def`) — then of what it produces (XML element
names and `class` values, user-named theorem/listing/float families counted once), per √seconds, at most two
manuals a bundle, from manuals that ended with status 0-2 in under 45 s (the heaviest manuals are outside the net).
From sweep #128: 1,000 manuals (the cap) covering 4,078 of the 4,230 load features and 420 of the 475 output
features of 2,275 candidates, 1,125 s serial. `manual_net.sh <binA> <dumpsA> <binB> <dumpsB> <out> [jobs]` checks
its arguments (fresh outdir, executables, both dumps; dump-override variables unset), runs both sides on cores
64-127 with a fixed `SOURCE_DATE_EPOCH`, validates both sides' XML (`validate.sh`; outside the 8 GB cap, under
which jing's JVMs fail to start and count as invalid) and runs every repro topic with both binaries. `manual_net_compare.py <out> [--recall]` fails toward flagging: a manual missing from a side, an
incomplete repro run and an unscorable recall are regressions, as are a worse status, a new Fatal, more errors,
valid → invalid, words down >1 %, recall down, a Math `tex=` change and a newly failing repro. Words up >1 % and
runs slower by >50 % and 5 s are listed. First run (56kw3 → 56kx5, the pre-review net): tallies identical; the byte
diffs were timestamps, `\time`-seeded shuffles and `\meaning` heap addresses, now pinned or normalized; it caught
`runaway_space_group_loop`'s count-less `% expect:` (fixed). Null run (A = B = 56kx5, `~/data/pk_agents/w70/net_null`): 1,000 manuals and 968 repros, CHANGED and REGRESSIONS
(none), 7 minutes wall at 24 jobs on 64 cores. Re-select the
set when the corpus changes, not per batch.

## K18 — Typed box sizes: a box's measured size is its constructed size

**Model:** theme 11.
**Abstraction:** a whatsit's `width`/`height`/`depth` properties are typed `Dimension`s, rendered to attribute text only when the XML is written (`Stored::to_attribute`). Every box constructor has a sizer that carries its own geometry: the frame inside `\framebox[w]`'s width, `\raisebox`'s raise and its optional height and depth, the scale and rotation of the graphics boxes, following graphics.sty's `\Gscale@div` (tex.web §455). The list's own width wins over a width inherited from the whatsit.

**Invariant:** `\wd`/`\ht`/`\dp` of a box equal pdflatex's to the sp where the XML attribute already does, and no size key holds a `Stored::String`.

**Landing:**
1. **Step 1:** the four items of the RED repro, with the width-leak fix as a prerequisite for `\framebox`.
2. **Step 2:** bind `\width`/`\height`/`\depth`/`\totalheight` to the box while a box command reads its size arguments (latex.ltx `\@begin@tempboxa`; 0pt stubs at sect12.rs, as Perl's pool.ltxml:4644-4647), then honour `\raisebox`'s `[height][depth]`. `\Gscale@div`'s arithmetic for `\resizebox`. A debug-build warning in `compute_size_and_cache` for any String size, and the audit of the constructors without a sizer (79 `sizer =>` sites, 54 engine + 25 package/contrib, against 945 `DefConstructor!`, 364 + 581). makecell and diagbox follow.

**Class guard:** the RED repro turns green, and `\makebox[1em]{aaa bbb ccc ddd}` measures 6.94444pt / 0pt.

**Status:** step 1 LANDED in 56kj (guards `perfect_kernel_batch56::{box_sizes_are_their_constructed_sizes, a_box_width_does_not_break_its_contents}`): `\framebox[w]`, `\parbox`, `\raisebox` (Perl's `raisedSizer`), `\scalebox`/`\resizebox`/`\rotatebox`/`\reflectbox` and rotating.sty (sharing `rotated_properties`), and the width leak, measure pdflatex's sizes (DIVERGENCES #327). Step 2, first part LANDED in 56kl (guard `perfect_kernel_batch56::box_size_arguments_measure_the_box`, DIVERGENCES #328): `TempboxaDimension` size arguments are read with the box set (`within_tempboxa`), so `\width`/`\height`/`\depth`/`\totalheight` measure it for `\makebox`, `\framebox`, `\parbox`'s height, `\raisebox`'s `[height][depth]` (its sizer applies them) and `\Gscale@box@dd`/`@dddd`. Open for step 2: `\raisebox`'s raise (needs `yoffset` to render with reserved space, not `position:relative`; RED repro `raisebox_raise_measures_the_box.tex`, 13 papers); `\resizebox{\width}` and `\resizebox*` (`GraphixDimension`/`\Gscale@@box`, 4 papers); `\pic@raisebox`, `{minipage}`'s [height]; `\resizebox{1em}` is 10.00002pt (Perl's) where pdflatex gives 10.00081pt through `\Gscale@div`; makecell and diagbox; the String-size detector and the sizer audit.

**Risk:** LOW for (a)-(c), with attributes byte-identical. MED for the width leak, since measured heights of multi-word boxes change and small numeric goldens may move.

