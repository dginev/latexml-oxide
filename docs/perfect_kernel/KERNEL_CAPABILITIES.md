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
| 2026-09-06 | — | Correctness item (Gemini round 6, L9, locks): uspatent.cls redefines `\maketitle` to run `\patentStart` (which creates counter `parnum`); the class binding UNLOCKS `\maketitle` before the raw load. General form = the lock keeps the dropped definition's BODY (`<cs>:redefined@body`, beside 56z's `:redefined@nargs`) and the locked `\maketitle` runs it after `\lx@frontmatterhere` when `\maketitle:redefined` — one mechanism for every class that layers real work onto `\maketitle`, no per-class unlock. |
| 2026-09-06 | — | Correctness item (Gemini round 6, L1): the xcolor binding's `\XC@getcolor#1#2` is `\XC@edef#2{#1}`; xcolor.sty:1373-1390 normalises the spec through `\XC@getc@lor` (expression syntax, `\XC@@`-delimited scan) before `\aftergroupdef`. Enough for pstricks' `\pst@getcolor{name}`; a `red!50` expression reaches `\psk@…` unnormalised. |

| 2026-09-06 | — | Landed (batch 56ab, mode/group frames): a pgf graphics-state scope is an `<svg:g>` only, never a TeX group (pgfsys-common-pdf.def:37-38 `q`/`Q`; DIVERGENCES #212). Open root behind msc (21) and modernposter (14), PARKED with D12: `\pgf@maketext` (pgfcorescopes.code.tex:160-184) nests `\setbox\hbox\bgroup \pgfinterruptpicture(=\begingroup) \bgroup` and defers the box's `\egroup` through `\aftergroup`; with a `\vbox` shape part on top, a nested `\hbox` reader's `end_mode` (base_utilities.rs:3624) meets an `internal_vertical` frame. tex.web keeps the save stack (§274, `\endgroup` §1063 checks only `cur_group`) separate from the semantic nest (§211-218); `begin_mode`/`end_mode` (stomach.rs:1004-1103) fuse them. General fix = decouple the nest from the save stack (mission-level; no stopgap pop). `LXML_TRACE_BOUND_MODE=1` gives the backtrace. |
| 2026-09-06 | — | Correctness item (generalization pass, batch 56ab, alignment engine): the cell-head peek runs under an `AlignPeekMode` override (`internal_vertical` around the peek loop, tex_tables.rs) that reproduces tex.web's observable mode at every read — peek §15510, `\noalign` §15513, cell §15532. The structural form is `init_align`'s own: the alignment body BEGINS in `internal_vertical` (§15350) and each row enters `restricted_horizontal` at `init_row` (Perl TeX_Tables.pool.ltxml:181 would `beginMode` per row); then the explicit drop point and the override go. Side effect already carried: `\noalign` material digests in vmode (bound inside its own group). Settled: `MODE` is frame-local (`begin_mode` binds it in the box frame), so the override and its restore must run in the SAME save frame — a restore issued after `start_column` opened the cell group died with that group and left every later cell in vmode (16 goldens: spaces kept before `&`/`\\` in `tex=`, diagbox cells at the text width); the peek now ends the override before any row/cell group opens and `\omit` re-enters after its row group. |
| 2026-09-06 | — | Correctness item (generalization pass, batch 56ab, parameter types): `EqnoTag` is the first parameter that DIGESTS each token (`invoke_token`) up to a stop net and retracts the terminal; every `Until`/`XUntil`/`ProtectedXUntil`/`Expanded` type collects without digesting (base_parameter_types.rs). Seed for a general `DigestUntilRetract(stop)` type once a second display-end bounded digest appears (a `\tag`-style variant, or the `\noalign` group-end reader); single caller today, keep bespoke. |
