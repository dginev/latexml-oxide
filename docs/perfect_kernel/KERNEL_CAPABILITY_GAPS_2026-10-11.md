# Kernel capability gaps behind the open work — 2026-10-11

A point-in-time study of the open perfect-kernel work, run-336 Error/Fatal clusters, html_feedback rows and RED
repros, grouped by the missing abstraction, algorithm or workflow behind several specific gaps. The question was the
user's (2026-10-11); code paths are relative to the repository at `be7deec245` (64x/64zc/64zd were in flight). The
landing plans live in [`KERNEL_CAPABILITIES.md`](KERNEL_CAPABILITIES.md) (K20–K24); this file keeps the evidence.

**User context (2026-10-11).**
- Version drift of `.sty`/`.cls` across 1990–2026, and author-shipped copies of known TeX Live or journal files with
  small edits, are acknowledged limits of distilling XML from LaTeX: the semantic XML an author's edit was meant for
  cannot be inferred. C1 below detects and preserves; it does not infer.
- The author-block model (C4) looks fruitful but needs very careful testing (shadow mode first).
- The missing tex.web algorithms (C5) are the core focus of the perfect-kernel work.

**Decided order (user, 2026-10-11):** manual sweep #149 → P0 → C5 (G4 → G2 → G3 → G1) → C2, with the C1 read-only
audit in parallel → C4 in shadow mode once P0 has its frontmatter extractor.

Count bases (reports and tables under `~/data/pk_agents/main/agent_reports/`):
- run 336's first-error clusters (`2026-10-10_run336_error_first_clusters_all.tsv`, 181,145 keyed tasks);
- the 222 RED repros, by header (sectioning-frontmatter 49, boxes-groups 27, captions-floats 22, graphics-tikz 14,
  singletons 12, index-bib 11, macro-state 10, math-parse 10, kernel-alignment 9, block-model 8, loader 7, …);
- the HF rows; KPE #520–585; OD #440–500;
- the 64-series payload reports; HANDOFF 63t–64zb.

## 0. Conclusion — invest in this order

1. **P0 — one audited A/B comparator** (workflow; a prerequisite; 1–2 sessions; LOW risk).
   - At least 7 regressions or near misses this week came from the tooling (§7).
   - C2 and C4 can only land behind byte-identical gates.
2. **C1 — binding provenance + ownership** (version drift, author-shipped copies).
   - `undefined:` is 63% of first errors (114,371 tasks, 8,479 clusters).
   - After this week's ~20 fixed clusters, 227 clusters of ≥100 papers still hold ≈54k tasks.
   - Start read-only: a corpus-copy conformance audit plus a FileIdentity census. Then an owned-set overlay per family.
3. **C5 — gullet expansion fidelity**, one program for four tex.web deviations.
   - The four: the expansion granularity of peeking macros; `\left`/`\middle`/`\right` as macros; §395 extra `}`;
     column ends matched by meaning.
   - About 400–700 papers, 8 REDs, rows 64n/64o/64v and D1's residual.
   - Each piece is small, specified by tex.web, and retires a per-reader patch.
4. **C2 — environments as TeX does them.**
   - The group kind and the mode come from the begin code.
   - Document-content environments stream (open/close) instead of capturing `#body`.
   - About 5k first errors and 15 REDs. HIGH risk; staged behind P0.
5. **C4 — frontmatter author-block event model.**
   - The largest effort sink: 49 REDs, 20+ batches, more than 30 NO-SHIP review rounds. Small error mass (≈335 first errors).
   - Shadow mode first, with the guard corpus as oracle.

Next tier:
- **C3 — placement policy for misnested structure:** 11,012 `malformed:` first errors; roots untriaged, so sample first.
- **C6 — content-aware emptiness.**
- **C7 — string values for attributes.**
- **C8 — bibliography state:** mostly subsumed by C2 and C3.

## 1. Ranking by open specific gaps explained

| # | capability | open gaps explained (basis) | Perl shares? | risk / cost |
|---|---|---|---|---|
| C1 | binding provenance + ownership | ≈54k first errors in 227 open `undefined:` clusters of ≥100 papers (+29.8k in clusters of 10–99); ~15 REDs; HF3 residual, HF11; KPE ~18 "binding lacks what the file defines"; rows: 31 JHEP3 copies, ws packages, epsfig, float.sty/floatrow, PoS variant | yes | audit LOW; overlay MED–HIGH per family |
| C2 | environment group/mode/element | M1 ≈2,060 (64y patched; 82/150 of the sample still erroring); `}` 2,642 + `\endgroup` 2,301 (overlapping); `\endlx@list` 839 (partly 64p); `$$` in mode-less boxed environments ≈1,700 (64 sites left); 15 REDs; HF6, HF5?; rows "#body audit" and "list deferred-close" | yes (Package.pm:1902) | HIGH / large |
| C3 | placement of misnested structure (not seeded) | `malformed:` 11,012 first errors (section 1,644, bibitem 1,598, TOC 683, paragraph 581, theorem 552; toccaption/break already fixed) | yes | MED / medium, after triage |
| C4 | frontmatter author-block model | 49 REDs; HF2 residual 16; A2 rows 7; KPE #521–559 (~24), OD ~15; ≈335 first errors | no (Perl: one silent creator; this is beyond-Perl work) | HIGH / large |
| C5 | gullet expansion fidelity | V1b ~61, X1 62 (+24 xymatrix), `\span` ~127, part of Mouth:EoF 338, xskak; 8 REDs; rows 64n/64o/64v; OD #500 residual | yes, except `\left` (Rust-only) | MED / medium |
| C6 | content-aware emptiness and size | KPE #526, #581; OD #462, #467, #490; HF1/HF8; 64x graphics-only phantoms; ~8 REDs; K18 (79 sizers for 945 constructors) | yes | MED / medium |
| C7 | string values for attributes | ~8 REDs; 60 `˙` contacts (64zc); M8 ruling; HF7 natbib label 3, `\preprint` 1; SYNC F2 / 64j | yes | LOW–MED / small |
| C8 | bibliography state | 1,598 `malformed:ltx:bibitem`, split across C2, C3 and bindings; 7 REDs; KPE #512, 513, 569, 585, 587–593 | yes | — (subsumed) |
| P0 | audited comparator + integration numbering | ≥7 missed or near-missed regressions; 122 ad-hoc compare scripts; 3 renumberings | n/a | LOW / small |

## 2. C1 — Binding provenance and ownership (seed 1 reframed as version drift + author overrides)

**Gaps explained**
- **This week's binding gaps (≈14k papers fixed):**
  - 64h: aipproc 428, elsart 367, tcilatex 537;
  - 64k: JINST/compositio 899;
  - 64qa: ws 1,078, iopart 1,058, revtex 1,023, svmult 960, mdpi 765, WileyNJD 701, ptephy 204;
  - 64qb: paspconf 860, caption2 843, `\macc@` 817, `\sidecaption` 186;
  - 64u: l-aa 843, EPL 740, OSA 759, REVTeX 2 750;
  - 64w: axodraw 1,507;
  - 64x: PoS 155, `{centering}` 1,078.
- **Still open (≥340 each; some are faithful):** `\diagramstyle` 658, `\linkable` 641, `\citeauthoryear` 621,
  `\ifCUPmtlplainloaded` 615, `\bBigg@` 600, `\psfig` 596, `\AtlasRefCode` 547, `\pub` 546, `\readRCS` 528,
  `\newsymbol` 505, `\titlecomment` 460, `\declaretheorem` 459, `\setcaptionwidth` 436, `\newarrow` 434, `\dates` 431,
  `\bibselect` 426, `{claimproof}` 373, `\sanitize@url` 371, `\AuthorCallLimit` 370, `\usetkzobj` 361, `{@proof}` 352.
- **Author-shipped variants:**
  - 1104.0806 (edited opex3, 5 errors);
  - neurips_2024 (HF3, 24 errors);
  - 31 renamed JHEP3 copies;
  - JINST copies with a path prefix or new name (64k);
  - 11 Interspeech copies edited to final (63w);
  - 63 shipped axodraw copies;
  - the PoS variant in 1708.06608;
  - an edited ieeeconf copy (ruling R3);
  - IEEEtranTCOM (OD #444).
- **Binding surplus that overrides the paper:**
  - OSA abbreviations (64u r3 blocker: 0909.4968, 1104.3859);
  - PoS loading JHEP journal macros (~155);
  - IEEEtran's `\proof` alias over the author's;
  - EPL's boxed `{widetext}` (0810.4982: 1 → 14 errors).
- **Locks against raw code:**
  - HF11 (pgf shapes against the locked amstext `\text`);
  - 64p (`\renewenvironment` lock read, 132);
  - 64x (`\endabstract` file lock);
  - OD #482 (refused setters replayed);
  - D1 / OD #500.
- **REDs:**
  - loader/{family_wrapper_loads_family_class, emulated_package_is_not_loaded, package_options_keep_protected_macros,
    typog_patch_itemize, listings_textstyle_hook_patch};
  - singletons/{omnibus_alias_shadows_a_local_package, ws_procs_psfig, showexpl_result_keys_and_locked_commands,
    hyperref_language_option_names};
  - block-model/ieeetrantools_ieeeproof_any_class;
  - index-bib/{biblatex_ext_oa_patches_warn, ifbibmacroundef_knows_the_standard_bibmacros};
  - expl3/l3chk_siunitx_keys_set_l3keys_empty;
  - sectioning-frontmatter/revtex3_omnibus_author_group_address.
- **Vintage drift (not seeded):** TeX Live itself moved. These are "faithful" against TL 2025 but not against the tree
  arXiv compiled with: `\usetkzobj` 361, JINST under TL 2025 39, l3backend loaded twice 71, ctex fandol 107.

**Patch sites today**
- one binding per family (`latexml_package/src/package/*_cls.rs`, `latexml_contrib/src/*`);
- `latexml_contrib/src/shipped_class.rs` `requested_shipped_class` (64k);
- `class_binding_alternate` (OD #444);
- OmniBus provide-only (64zd);
- `state.rs` lock reads (`name_carries_lock`, 64p);
- K13 `latexml_oxide/src/conformance/audit.rs`, which audits TL packages only. Its stage 3 (classes) was deferred, and
  most journal classes are not in TL: `kpsewhich` finds none of ws-procs, iopart, svmult, mdpi, WileyNJD, ptephy, PoS,
  JHEP, JINST.

**Perl** shares the weakness: bindings replace files, ar5iv raw-loads styles but not classes, and OmniBus guesses.

**Design (faithful to K1 "Overlay" and the class-family ruling)**

(a) **FileIdentity** for every class and package resolution:
- what it records:
  - the requested name and the resolved path;
  - the provenance: binding-only, TL, shipped, shipped variant (renamed or path-prefixed), or legacy-absent;
  - the `\Provides*` version and date;
  - a content hash;
- one Info line per load;
- bindings can query it, which generalizes `requested_shipped_class`, AASTeX's `\gridline` 6+ detection and
  Interspeech's copy-reading.

(b) **Owned set vs emulation layer.** Each binding declares an owned set: the names and environments whose XML
semantics it defines, i.e. today's locks plus the frontmatter hooks.
- When the real file is present (TL or shipped), raw-load it and install only the owned set over it.
- When it is absent, install both layers.
- "Bindings outrank raw" and `localrawclasses` hold unchanged for owned names.

(c) **Shipped-copy delta.** Compute the delta between a shipped copy and its canonical copy by loading both raw in
scratch states with the K13 walker.
- Delta in non-owned names: the copy stands (abbreviations, toggles, new macros: 1104.0806, neurips_2024, Interspeech).
- Delta in owned names: keep the binding's semantics and emit one Info naming them.
- Renamed copies route to the family binding by content fingerprint (the JHEP3 row).

(d) **Corpus-copy conformance audit**, read-only (K13 stage 3):
- per binding, gather the copies arXiv papers ship;
- cluster them by hash and version, and raw-load each;
- rank the public names the binding lacks or adds, by paper count.

This would have listed `\ioptwocol`, `\toctitle`, `\history`, `\state`, `\bodymatter` and the PoS/OSA surplus before
run 336.

(e) **Legacy-absent styles keep their emulation**, validated against corpus copies, as 64u did by hand. The paper date
(64s) can select vintage behaviour only after a ruling, because it changes what "pdflatex succeeds" means.

**Achievable**
- **Can:** detect the version, variant or override; keep non-semantic author edits; flag edits to owned macros.
- **Cannot:** infer the XML an author's edited `\maketitle`/`\author` was meant for. Those keep the binding's semantics,
  and the Info says so.

**Subsumes** the HF3 residual, the "bindings load what the class loads" row, the JHEP3 row, surplus collisions, and
future 64q-style batches (they become audit rows).

**Migration**
1. FileIdentity logging, with no behaviour change.
2. A census: what share of this week's ≈14k fixed papers ship the file? This confirms or kills (b).
3. The corpus-copy audit.
4. Overlay one shipped-heavy family (iopart or svmult); the A/B must be byte-identical except for the intended changes.
5. Owned-set declarations for the frontmatter bindings.
6. The delta handler.

**Evidence** that would confirm the design:
- the shipping share from step 2;
- recall of the audit on the 20 hand-fixed gaps of this week;
- one byte-identical overlay family A/B.

## 3. C2 — Environment = TeX group + mode + XML element, decoupled (seed 2)

**Gaps**
- **M1.** 64y emulates `off_save` from frame markers. All 17/17 witnesses are clean, but 82 of the 150-paper sample
  still error (41 frames with no initiator, `\lx@list`, table/picture). `unexpected:\endlx@list` 839, partly fixed in 64p.
- **M2.** `$$` inside a mode-less `#body` DefEnvironment is not display math: the body is boxed in restricted
  horizontal mode, and `tex_math.rs:610-612` treats `$$` as display only under a vertical BOUND_MODE.
  - 64ra fixed four environments (≈1,700 papers);
  - 64 sites remain (census in `payload_64ra.md`);
  - adding one more caused the EPL regression.
- 2205.07302: the list group is closed by `\lx@mung@bibliography` (KPE #588).
- HF6: acmart's `\Collect@Body` abstract is digested at once.
- HF5: a font switch over a tcolorbox body is lost (not yet root-caused; a captured-body candidate).
- REDs:
  - boxes-groups/{atbegin_hook_group_above_env_frame, bare_theorem_ended_by_enclosing_end,
    amsthm_bare_proof_closed_by_brace, amsthm_proof_closed_by_endgroup, footnote_bare_quote,
    macro_env_ended_by_inner_bare_end, bare_doublespace_closed_by_brace, screenplay_abstract_unclosed,
    bibliography_in_minipage_in_quote_keeps_the_quote, mal_picture_env_par_kernel_control};
  - block-model/setspace_display_math_in_doublespace;
  - kernel-alignment/captured_body_interline_changed_inside;
  - sectioning-frontmatter/bibitems_in_a_list_keep_the_lists_settings;
  - index-bib/{class_bibliography_as_enumerate, class_bibliography_end_prints_text};
  - loader/multicols_afterpackage_group.
- 158 frontmatter first errors read "Attempt to end mode restricted_horizontal".

**Sites**
- `latexml_core/src/stomach.rs:1013-1092` `close_environment_frame`; `:1108-1158` `egroup`; `:1191-1214` the
  `endgroup` band-aid (`INTERPRETING_DEFINITIONS`);
- `binding/def/dialect.rs:1420` `environment_tex_group`, plus 64y's manual `tex_group` audit of 216 sites;
- `latex_constructs/sect06.rs:412-433`: lists are captured `#body` whatsits;
- `sect11.rs:578`: the mung's list branch.

**Perl** shares this.
- `Package.pm:1899-1990`: `captureBody`, `$mode='restricted_horizontal' if !$mode` (:1902), and `\begin` = bgroup +
  beginMode.
- `Stomach.pm:330-335` says modes are not correlated with grouping.

**TeX**
- `\begin{X}` = `\begingroup` + `\csname X\endcsname`.
- `\end{X}` = `\csname endX\endcsname` + `\@checkend` + `\endgroup`.
- The group kind comes from the begin code (tex.web §269); the mode changes only if that code changes it.

**Design: two body models for DefEnvironment.**
- `Captured`, as today, for boxes that need their body as a unit: minipage, tabular, floats, picture, tcolorbox.
- `Streamed` for the rest:
  - begin = a semi_simple group (K9 group code) + a constructor that opens the element;
  - end = `maybe_close_element` + `\lx@checkend` + endgroup.

Further:
- **No default mode.** A mode-less environment inherits the current mode, which fixes the 64 `#body` sites as a class.
- **Independent stacks.** The XML element stack and the save stack become independent:
  - the mung can close the list element and leave the TeX group to the author's `\end`;
  - a group closer ending an environment becomes tex.web §1064 `off_save`, not a marker search;
  - 64y's group-kind table becomes derivable.

**Subsumes** the "#body audit" row, the "list deferred-close" row, M1's residuals, and most boxes-groups REDs.

**Migration**
1. P0.
2. For each census site whose real source is a macro pair, drop the boxed mode (A/B per batch). Then make "no mode" the
   default, with an allowlist of `Captured` environments.
3. Stream itemize/enumerate/description (2205.07302, the 64p mung shapes, list goldens byte-identical).
4. Then quote, quotation, center, flush*, the theorems and proof.
5. Retire the marker path in `close_environment_frame` once group codes carry the kinds (K9 stages 1–2).

**Risk**: HIGH. Ids, `after_construct` hooks that read `#body`, reversions and sizers all depend on the current model.

**Evidence**
- byte-identical XML on 300 random + 100 theorem/list-heavy papers;
- the REDs above go green;
- the M1 sample drops from 82/150 to ≤20.

## 4. C5 — Gullet expansion fidelity (seed 6, in depth)

### G1. Expansion granularity of peeking macros

**tex.web**
- `get_x_token` §380, `expand` §366, `macro_call` §389.
- `\let`/`\futurelet` are executed (§1210–1221), never expanded.
- These readers stop at the first unexpandable token: `scan_int` §440–445, `\if` operands §506, `align_peek` §785
  (e-TeX `get_x_or_protected`), `scan_left_brace` §403, `\csname` §372, `\edef` §477.
- latex.ltx `\@ifnextchar` (:1756-1760) begins with `\let\reserved@d=#1`. `\@ifstar`, `\@testopt` and every
  `\newcommand` optional argument (`\@protected@testopt`) go through it.

**Rust** has closures that peek while expanding:
- `latex_constructs/sect13.rs:1535` `\@ifstar`, `:1557` `\@testopt`, `:2080` `\@ifnextchar`;
- binding-level `\newcommand` optional arguments (`sect08.rs:61`, an `Optional` parameter read at expansion);
- `cancel_sty.rs:53-62`;
- D1's `call_owner` peek (`definition/expandable.rs:305-313`).

Four per-reader patches compensate:
- `gullet.rs:1780` `scan_stops_at_futurelet_peek` (number scans);
- `gullet.rs:1752` `optional_arg_protected` (`\protected@edef`);
- `tex_tables.rs:1088-1148` `AlignPeekMode` (fixes the mode only);
- the `\if` patch proposed in the header of RED ifnextchar_inside_if_operand.

**Explains**
- V1b `_` at the head of an alignment cell, ~61 papers (64n);
- RED expansion-primitives/ifnextchar_inside_if_operand (xskak, 12 errors);
- RED macro-state/m5_locked_name_in_expansion_only_context (OD #500 residual);
- earlier patched instances: egpeirce, the sweep-16 OOM.

**Perl** shares it (`latex_constructs.pool.ltxml:5636`).

**Design**
- **Marking:** a definition whose TeX first step is an unexpandable assignment is marked `first_step_unexpandable`
  (this generalizes `peeks_by_futurelet`).
- **One-step expansion:** it yields a single unexpandable token, `\lx@peek⟨id⟩`. Executing it runs the closure's peek
  and pushes back the chosen branch.
- **Readers:** every reader then stops where TeX stops at `\let`.
- **Expansion-only contexts:** there the token reverts to the call as written, which fixes D1's `\meaning`/`\write`
  residual.
- **The alternative, to measure first:** latex.ltx's own `\@ifnextchar` from the dump, keeping closures only where
  profiling requires them.

**Tests** (minimal expansion chains, the pdflatex log per test, whole output pinned):
1. `\if Z\@ifnextchar*{A}{B}\else E\fi` → "E", 0 errors (the RED).
2. `\count@=1\@ifstar{7}{5}*\the\count@` → "71" (keeps the guard `ifnextchar_scans` green).
3. `\newcommand\two[1][2]{#1}\count@=1\two\the\count@` → "21".
4. `$\begin{array}{c}\cancel{x_1}\end{array}$` → 0 errors (the V1b repro).
5. `\def\text{V}\edef\x{\text{z}}\meaning\x` → no `\lx@locked@` (the m5 RED).

### G2. `\left` / `\middle` / `\right` are macros

**tex.web**
- One unexpandable `left_right` (§1188).
- `math_left_right` (§1191) with `scan_delimiter` (§1160).
- `\left` pushes `math_left_group`; `\right`/`\middle` check `cur_group`, and otherwise give "Extra \right." (§1192).

**Rust**
- `\left` is an expandable `DefMacro` that reads an `XToken` (`tex_math.rs:1077-1116`) and trampolines to
  `\lx@delim@left` + `\lx@hidden@bgroup`.
- `\right` is a constructor unreading `\lx@hidden@egroup@right \lx@right` (`:1131-1136`, `:1118-1127`).
- `\middle` has no group check (`etex.rs:529`).

**Perl** is faithful (`DefConstructor('\left TeXDelimiter')`, `TeX_Math.pool.ltxml:773`). The deviation is Rust-only:
`TeXDelimiter` was never ported (WISDOM #44).

**Explains**
- X1: 62 IfLimit Fatals. An unbraced `\left\{` in an `\xymatrix` cell is read ahead by the cell scanner, and the
  if-stack desyncs.
- Part of `unexpected:\right` 48.
- 2605.21750's PushbackLimit (`\let\@left\left`, patched by renaming).
- RED math-parse/left_delimiter_outside_the_map (~400 warnings in 7 papers).

Ruled 2026-10-10; queued as 64v.

**Design**
- Port `TeXDelimiter` as §1160:
  - `get_x_token` past blanks and `\relax`;
  - `\delcode`, `\delimiter`, `\Udelimiter`, and `\let`-equivalents, matched by meaning.
- All three become constructors.
- `\left` pushes a frame with group code `math_left` (K9 stage 0).
- `\middle` and `\right` check that frame; a mismatch gives the §1192 Error and discards the delimiter.

**Tests**
1. `\edef\x{\left(}\show\x` → `macro:->\left (`.
2. mathtools' `\let\@left\left\def\left{\mathopen{}\mathclose\bgroup\@left}` → 0 errors.
3. `$a\right)$` → 1 Error, as in pdflatex.
4. `$\left(a\middle|b\right)$` → the whole XMath pinned.
5. The X1 xymatrix repro.

**Status: LANDED 65d.** `scan_delimiter` matches by meaning (a `\delcode` character, `\delimiter`/`\Udelimiter` with its
code, a math symbol); text mode takes any token, as Perl; `\middle` dispatches to `\lx@delim@middle`; a document
`{`/`\begingroup` between `\left` and `\right` is §1064 `off_save` (OD #530). The X1 witnesses convert (48 Fatals → 0;
109 witnesses' errors 3541 → 826); RED math-parse/left_delimiter_outside_the_map is green. Guards
`perfect_kernel_batch65d::*`.

### G3. A delimited argument meets an unmatched `}`

**tex.web**
- §392 scans the parameter.
- §395: back_input the `}`, "Argument of \x has an extra }", incr align_state, insert `\par`.
- §396: a non-`\long` macro is abandoned; a `\long` one scans on with the `\par`.

**Rust**
- `gullet.rs:2668-2803` `read_until` pushes the `}` into the argument (`:2728-2730`, `:2779-2781`) and reads to EOF.
- `base_parameter_types.rs:313-326` then aborts with Fatal Mouth:EoF.

**Perl** reads the same way (`Gullet.pm:654-665`). The Fatal is Rust-only.

**Explains** part of Fatal Mouth:EoF 338: `\endcaption` 67, `\NOT` 46, `\endtitle` 29, `\@checkend` 22. (`Until:\do` 72
was mostly iosart2x, fixed in 64k.) Queued as 64o.

**Design** On a catcode-2 token at depth 0:
- unread it and emit the Error;
- non-`\long`: take the 62h `abandon_argument_at_par` path;
- `\long`: append `\par` and continue;
- adjust the align count as §395 does.

**Tests**
- `\def\a#1\stop{[#1]}{\a x}y\stop` and its `\long` twin, against pdflatex logs (error counts and surviving text pinned
  whole);
- the V3b repro.

### G4. Column ends recognized by meaning

**tex.web** §342/§789 insert the v-template when `get_next` yields `cur_cmd` ∈ {tab_mark, car_ret} at align_state = 0.
`\span` is tab_mark; a `\def\span` is a call, not a column end.

**Rust** `gullet.rs:4542-4565` `is_column_end` compares the current meanings, so a redefined `\span` still ends the cell
(`Gullet.pm:266-277` does the same). Insertion is checked only in `read_x_token` (`gullet.rs:1973-1996`).

**Explains** M4: ~127 papers (math0004102).

**Design** Compare against frozen primitive meanings (`\lx@prim@span`, …), i.e. the command code.

**Test** `~/data/pk_agents/main/agent_reports/rc_struct/repro_span_redef.tex` (pdflatex 0).

**Status: LANDED 65b.** The implemented design is equality on the defining cs of a Primitive/Constructor (OD #522),
not frozen copies. Frozen `\lx@span`/`\lx@frozen@cr` (Base-pool `\let` copies) serve only the machinery's own
emissions (OD #523, #524). W1 (179 `\span` redefiners): errors 1730 → 839. Guards `perfect_kernel_batch65::*`.

### G5. `scan_toks` ordering

**tex.web**
- §473: `scanner_status:=absorbing` before §403 hunts the brace.
- §474–475: "Missing { inserted" when a `}` ends a parameter text.

**REDs**, manual corpus only; fold them into the same harness:
- expansion-primitives/uppercase_brace_hunt_file_end;
- expansion-primitives/def_param_text_right_brace_storecmd (`tex_macro.rs:91` `UntilBrace` reads on).

### G6. Not gullet algorithms (seed 6 partly unsupported)

- **PushbackLimit/IfLimit loops** (674/217): 64p traced 280 of 508 to bindings and locks.
- **`insert_relax`** (`definition/conditional.rs:341`, `:381`) uses the redefinable `\relax`, where §379 uses
  `frozen_relax`. Real, but no witness.
- **§392 (a `\par` in a non-`\long` argument)** landed in 62h. RED
  parameter-conditional/delimited_argument_ending_its_paragraph_is_a_runaway still says RED; re-grade it.
- **Untriaged clusters that may belong here:** `misdefined:#` 772 (`#` doubling in stored arguments, §476–477?),
  `Stomach:Recursion` 418, `unexpected:\noalign` 1488. Sample them first.

**Program**: one branch and one harness of expansion-chain repros, ordered by risk: G4 → G2 → G3 → G1. Each step gets:
- an arXiv A/B on the cluster witnesses, 300 random papers and manual_net;
- a perf check for G1, whose closures are on the hot path.

## 5. C4 — Frontmatter author-block event model (seed 3) — test strategy first

**Oracle**
- 466 frontmatter guard functions (`06_cluster_frontmatter.rs` 49; `perfect_kernel_batch61/63/64*`);
- 459 `sectioning-frontmatter` repros, 49 of them RED;
- the A/B corpora already run: HF2 2,089; 64k 2,918; aastex7 373; Interspeech 146; 64u 1,139.

**Stages**
- **Stage 0:** extend P0 with a frontmatter-tuple extractor (creator, name, affiliations, contacts, notes, marks).
  Snapshot ~10k frontmatter-bearing papers by class family.
- **Stage 1:** Layer A (the lexer, below) runs as a pure function in shadow mode beside today's path, logging
  disagreements, with no output change.
- **Stage 2:** migrate one family at a time, new and small first: OSA/EPL/l-aa, interact, elsarticle/cas, IEEE, acmart,
  and the generic article last.
  - Gate: byte-identical on the family corpus, plus every frontmatter guard.
  - Any intended change needs a RED→GREEN repro with a ruling.
- **Stage 3:** delete the superseded heuristics.

**Gaps**
- 49 REDs (`author_*`, `affiliation_*`, contacts, thanks, IEEE);
- HF2 residual 16 (86 → 16 after 63y–64d);
- A2 rows: an.cls, entics, `$(a)$` marks, sectsty, title-cite tooltip, llncs;
- KPE #521–559 (~24);
- OD #445, 459–466, 470–472, 478–479, 482.

**Sites**: `latexml_engine/src/base_utilities.rs` (13,870 lines; 65 of 502 commits since 09-25):
- splitter: `split_author_line` :9414, `split_marked_pieces` :11269, `marker_leads` :11389,
  `names_list_continues` :12094;
- owners: `run_in_author_order` :3670, `placed_contact_owner` :3091, `\lx@add@contact@placed` :1102,
  `block_email_calls` :12599, `email_command_calls` :12786;
- DOM repair: `relocate_annotations` :5941, `show_unanswered_marks` :5882.

**Perl** emits one silent creator (KPE #521). This is beyond-Perl work with no TeX oracle for "a name", which is why
per-shape fixes don't converge.

**Model**
- **Layer A, the lexer:** an author block becomes events, each with its source span:
  - Name;
  - Mark(kind, label);
  - Sep(and | comma | `\and` | `\\` | `\AND`);
  - Affil;
  - Contact(kind, value, display);
  - Note(thanks | footnote, label);
  - Layout(space | font | colour);
  - Group(wrapper, events).
- **Layer B, per binding and syntax only:** which commands produce which events; the association policy (by mark, by
  run, to all, to last, by spelled name); the block shape (per call or one block).
- **Layer C:** a class-independent resolver that records the rule behind each association.
- **Layer D:** one emission, with no relocation pass.

**Lessons**
- 63y: a generic `annotate=new` gave IEEE blocks to every author.
- 64u r4: a kernel owner rule was a risk to every class.
- 64g r2: `held()` read text, and an `rdf` has none.
- 64d r4: the w6 regression.

**Evidence**
- shadow disagreement below 0.5% per family before migrating;
- byte-identical family A/Bs;
- REDs go green without new per-shape rules.

## 6. C3, C6, C7, C8

**C3 — placement of misnested structure (not seeded)**
- **Mass:** 11,012 `malformed:` first errors:
  - section inside section 562, item 554, quote 194, title 108;
  - TOC inside abstract 411;
  - paragraph inside item 513;
  - theorem inside quote 214, abstract 167.
- **Today:** six bridge rows at `latexml_core/src/document.rs:3569-3632`, lenient containers at `:3497-3503`, and three
  rulings under OD #189 (float, item, bibliography), each argued separately.
- **Design:** one table-driven placement rule, ruled once per container class.
- **First:** triage 6 papers per sub-cluster; the roots may be bindings.

**C6 — content-aware emptiness (not seeded)**
- **Today:** `alignment/normalize.rs:100-135` (Perl `Alignment.pm:458-470`) drops a cell measuring under 1sp.
- **What this cluster includes:**
  - the third per-binding QED sizer (64ra's K-row proposal, OD #490, KPE #581);
  - HF1 (OD #462), HF13 (OD #467), KPE #526;
  - 64x's prune deleting entries that hold only graphics (astro-ph0102416);
  - REDs store_default_that_typesets_nothing_leaves_no_element, deposit_of_only_punctuation_is_dropped,
    tblr_all_empty_table_keeps_its_rules, raisebox_empty_height_is_not_given, resizebox_zero_size_division,
    blank_footnote_mark_is_blank.
- **Design:**
  - whether an element "emits content" is computed from the constructor template;
  - measured size is used only for geometry;
  - `\label`, `\index` and anchors stay empty.

**C7 — string values (seed 5, narrowed)**
- **Design:** a `\pdfstringdef`-shaped reducer for attribute data only: mailto/href, RDFa content, hyperref pdfinfo,
  sort keys. Display fonts stay per binding, as 64zc's census showed.
- **Gaps:**
  - REDs url_contact_keeps_tilde, amsart_urladdr_tilde, aastex7_email_markup_recorded_as_text,
    hyperref_pdfinfo_expands_macros, contact_name_label_keeps_its_ligatures, attachfile_underscore_file_name,
    glossary_noidx_sort_expands_accents;
  - the one-character mailto splits from 64zc;
  - M8;
  - SYNC F2 / 64j.

**C8 — bibliography (seed 4)**
- **Mass split:** C2 owns the list group (2205.07302); C3 owns headings between `\bibitem`s (~750, ruling 3, 64z);
  Perl-origin binding guards own the rest (natbib ~230).
- **Residual:** an explicit implicit-bibliography state would replace these repair passes:
  - `\if@lx@inbibliography` being undone by group ends (KPE #593);
  - the phantom prune;
  - the emptied-list sweep;
  - heading absorption.

  Revisit after C2.

## 7. P0 — Workflow

**Evidence**
- 64zc r1: `cmp.py` hid one-character mailto links.
- 64x r4: a text-only phantom metric missed 12 lost graphics panels.
- 64x r1 and 64ra: A/Bs ran on stale binaries.
- 63w4: an orcid slot swallowed the next author, unflagged by the sample statistics.
- 64k5: a misdiagnosis left 41 empty contacts.
- 64u's EPL `{widetext}` regression was caught only by 64zb's A/B.
- 64b: a guard's whitespace normalisation hid a lost space.
- 122 ad-hoc `cmp*/compare*.py` scripts.
- Entry numbers renumbered 3 times (KPE #568→576, OD #484→487, 64x drafts #493→#503+).
- The NODUMP path is broken (64qb side finding).

**Design**: one Rust comparator, modelled on K17's `manual_net_compare.py` (it fails toward flagging). It is also the
seed of the A4 fidelity audit.
- It refuses to run unless both binaries' stamped revisions match the requested commits.
- Per paper it reports:
  - status and diagnostics by class;
  - element counts for every qname (graphics, picture, Math, creator, contact, bibitem …);
  - text by element class, including attribute text (href, content);
  - a canonical XML diff, with ids mapped rather than stripped and no wrapper stripping;
  - frontmatter tuples.
- Output: a TSV plus a summary.

Entry numbers are allocated only at integration, by a script that reads the current maximum.

## 8. Seeds not supported

- **Seed 1, overlay alone:** the largest legacy populations ship no file (axodraw v1 0/149, OSA 622/751), so emulation
  stays for them (C1(e)).
- **Seed 3, retiring per-class code:** the class bindings are already thin declarations; the debt is in the kernel
  splitter.
- **Seed 4, a separate state machine:** see C8.
- **Seed 5, a string type:** the display font is class-specific (REVTeX 4.1/4.2, AASTeX, acmart and svjour3 set roman;
  amsart, llncs and JHEP set tt).
- **Seed 6, loops:** these were bindings and locks; only X1 is a gullet algorithm.

## 9. Gaps not seeded

- **U1 — TeX Live vintage** (inside C1).
- **U2 — placement policy** (C3).
- **U3 — content-aware emptiness** (C6).
- **U4 — construct-then-repair DOM passes:** frontmatter relocation and coalescing, bibliography prune and absorb,
  alignment normalisation, algorithm2e after_close. Every 64x, 64g and 64k NO-SHIP round hit one of them. The cure is
  to decide at construction time (C2, C4).
- **U5 — untriaged clusters:** `misdefined:#` 772, `Stomach:Recursion` 418, `\noalign` 1488, `expected:$` 1910.

**Critical files**:
- `latexml_core/src/gullet.rs`
- `latexml_core/src/stomach.rs`
- `latexml_core/src/binding/def/dialect.rs`
- `latexml_engine/src/base_utilities.rs`
- `latexml_oxide/src/conformance/audit.rs`
