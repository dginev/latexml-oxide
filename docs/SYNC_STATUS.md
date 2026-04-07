# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-07. Only lists open gaps & TODOs; completed items live in git history.

**Test inventory:** 313+ non-tikz tests pass (90 workspace + 223 integration); 7/10 tikz tests pass (3 pre-existing loops). MakeBibliography pipeline fully operational.

**arxiv sandbox:** See [`arxiv-examples/CATALOG.md`](../arxiv-examples/CATALOG.md) for the full 47-paper test catalog. **Session 96: 43/47 OK (91%), 0 FAIL.** All Phase B tasks completed. Token limit raised 30M→100M (recovers 2209.14198: 0→1.3MB, 0 errors). Remaining 3 EMPTY — Perl also fails on all 3 (2402.03300 pgfkeys, 2410.10068 quantikz, 2511.03798 eqnarray).

**Production-ready:** Full CorTeX ZIP-to-ZIP pipeline operational. All legacy production options supported:
```
latexml_oxide --whatsin=archive --format=html5 --pmml --mathtex --noinvisibletimes \
  --nodefaultresources --nobibtex --preload=ar5iv.sty --timeout=2700 --log=log.txt \
  --dest=output.zip input.zip
```

**High-level roadmap:** See [`mini_3_plan.md`](mini_3_plan.md) for the 4-phase strategic plan
(Engine Parity → Package Bindings → Post-Processing → Production).

**Performance:** See [`PERFORMANCE.md`](PERFORMANCE.md) — repeatable optimization checklist for release milestones.

## Legend
- **OK** = fully synced | **MINOR** = small gaps | **GAPS** = significant missing | **EMPTY** = not ported

**See also:** [`KNOWN_PERL_ERRORS.md`](KNOWN_PERL_ERRORS.md) — upstream Perl issues (not Rust bugs)

---

## Engine Files — Open Gaps Only

Only files with GAPS or significant MINOR issues listed. OK files omitted (see git history).

| File | Status | Open Gaps |
|------|--------|-----------|
| base_parameter_types.rs | MINOR | `DirectoryList`, `CommaList`, `DigestUntil` stubbed; `Variable` reversion safe fallback |
| tex_box.rs | MINOR | Minor box dimension edge cases |
| tex_fonts.rs | MINOR | Missing: `\fontdimen` full array semantics, `getFontDimen()` helper |
| tex_tables.rs | MINOR | Minor: padding CSS classes |

---

## Cross-Cutting Infrastructure Gaps

1. **`FontDef` parameter type** — Simplified to `FontToken`. Blocks `\fontdimen`, `\hyphenchar` per-font tracking.

---

## Unported Perl Engine Files

| File | Defs | Status | Notes |
|------|------|--------|-------|
| `AmSTeX.pool.ltxml` | 112 | ~30% | Plain TeX format (rare) |
| `BibTeX.pool.ltxml` | 956 | 0% | Skipped via `--nobibtex` in production |

---

## Core Modules (MINOR+ only)

| Module | Status | Open Gaps |
|--------|--------|-----------|
| gullet.rs | MINOR | `readArg` isolation (type ergonomics) |
| document.rs | MINOR | XML comment creation (needs libxml2 FFI) |

---

## Package.pm — DefFoo Sync Status (dialect.rs)

| DefFoo | Status | Gaps |
|--------|--------|------|
| `DefMacroI` | MINOR | `outer`/`long` not mapped |
| `DefPrimitiveI` | MINOR | Missing `outer`/`long` |
| `DefConstructorI` | MINOR | Missing `outer`/`long`/`attributeForm`; robust alias fallback |

---

## Package Bindings

**100% coverage: all 406 Perl bindings ported to Rust.** Zero `todo!()` panics. Zero MISSING.

### Remaining gaps in ported bindings

| Binding | Gap | Notes |
|---|---|---|
| beamer.cls | 88% | Overlay specs, themes — largest gap (unused by arxiv test papers) |
| authblk/inst_support | callbacks | `relocateInstitute`/`authblkRelocateAffil` DOM surgery (no test regression) |

---

## Tikz Test References

XML files in `LaTeXML/t/tikz/` are OUTDATED. Always regenerate fresh Perl output.

### Priority FIX items (shared across tikz tests)

1. **foreignObject transform Y=16.6** — Perl uses fixed 12pt maxy; Rust uses actual height
2. **foreignObject width/height** — `fo_get_size` differs from Perl
3. **Nested minipage/SVG sizing** — `appendNodeBox` vs Perl's `pushContent`
4. **Arrow tip shape** — Different arrowhead path data
5. **SVG viewBox/width** — Total dimensions differ slightly
6. **Listings escapechar + color** — `escapechar=@` with `\color{red}` inline
7. **Missing `\vspace{2mm}` output** — `\vspace` in vertical mode

---

## Work Plan — Ordered TODO List

Follow the [`arxiv-examples/CATALOG.md`](../arxiv-examples/CATALOG.md) for per-paper status.

**Current status (2026-04-05):** 37/47 OK (79%), 27/37 >=90% (73%), 30/37 >=80% (81%). Bibliography: BBL preferred via bibconfig=bbl,bib.
**Visual comparison (session 93 final):** 20/37 IDENTICAL (54%), 10 near-identical/cosmetic, 2 Rust-better, 2 critical (tikz truncation).

**Remaining actionable bugs (session 93):**
- [x] **2405.19425**: Missing images — Fixed: multi-page PDF page extraction in graphics post-processor. `page=N` option now parsed, passed to ImageMagick with `[N-1]` selector, unique filenames generated per page (x1.png, x2.png, ...). All 10 images render with dimensions.
- [x] **2405.19425**: Listing color/background/line-numbers — Already working: Rust has syntax highlighting (ltx_lst_string, ltx_lst_comments), background color (--ltx-bg-color), line numbers (ltx_lst_numbers_left), and language classes. Verified in session 94 output.
- [x] **2602.18719**: tikz-cd body truncation — FIXED. Was 34KB/1001 errors, now 677KB/17 errors (119% of Perl size). Paired `\bgroup`/`\egroup` `defined_as` fix in `parse_halign_template` + `digest_alignment_body` + `digest_alignment_column` resolved the root cause. Remaining 17 errors are post-processing cross-ref ID misses for one math expression (S4.Ex63.m1).
- [x] **2603.15617**: l3file quarks fixed — `\q__file_nil`, `\s__file_stop`, `\__file_quark_if_nil:nTF` etc. now defined via post-load `\cs_if_exist:NF` + native expl3 functions (`\quark_new:N`, `\scan_new:N`, `\__kernel_quark_new_conditional:Nn`). tcolorbox with `[most]` + `\tcbuselibrary{listings}` loads fully. Paper's remaining loop is content-specific (not l3file infrastructure), and each section converts individually (1.1MB combined without ar5iv preload). The specific loop trigger in the full paper needs further investigation.
- [x] **pgf arrow tips**: 2402.10301 (previously FAIL/timeout "pgf Computer Modern Rightarrow") now renders 1.8MB (124% of Perl) with 0 errors — resolved by \bgroup/\egroup fix. Arrow tips render as inline SVG paths via existing pgfsys primitives. Remaining EMPTY papers (2209.14198, 2410.10068) blocked by expl3 undefined macros, not arrow tips.

**Session 94 fixes (2026-04-06) — Graphics page extraction + tikz halign \bgroup/\egroup:**
1. `graphics.rs`: Multi-page PDF `page=N` extraction. Parse page option from graphicx options, pass `[N-1]` page selector to ImageMagick `convert`, generate unique per-page filenames (x1.png, x2.png). Added `-define pdf:use-cropbox=true`. 2405.19425: all 10 images now render with dimensions.
2. `graphicx_sty.rs`: Added `DefKeyVal!("Gin", "page", "")` — page option now properly parsed in keyval context.
3. `tex_tables.rs`: Paired `\bgroup`/`\egroup` fix in alignment — `defined_as(T_BEGIN/T_END)` at all three Perl-matched locations: `parse_halign_template` entry (L190), `digest_alignment_body` termination (L319), `digest_alignment_column` scan end (L395). Early return on error with token unread. Recovered 3 tikz tests (ac_drive_components, atoms_and_orbitals, consort_flowchart now produce real SVG content).
4. `pgfsys_latexml_def.rs`: Added `\lxSVG@halign` DefConstructor + `tikz_alignment_bindings()` — SVG matrix layout using `svg:g` elements with transform matrices. Bails out gracefully when template has no columns.
5. Tikz status: 7/10 pass (was 5/10); consort_flowchart produces 364-line SVG (was 23-line stub), has known tikz SVG parity diffs vs Perl; 3 still loop (dominoes, various_colors, unit_tests_by_silviu — pre-existing).

**Session 93 fixes (2026-04-05) — Algorithm2e + bibconfig + elsart:**
1. `algorithm2e_sty.rs`: `\BlankLine` (`\vskip 1ex`) leaked "1ex" as literal text in listing lines. Fixed: override to `\lx@algo@par` inside algorithm env.
2. `algorithm2e_sty.rs`: `\lx@algo@pop@indentation` implemented (was missing, log showed undefined macro). Pops last token from indentation register matching Perl L159-163.
3. `algorithm2e_sty.rs`: `\lx@algo@startline` now emits `\the\lx@algo@indentation` — produces `<ltx:rule>` vertical bar elements (18 rules generated for 2508.18544, was 0).
4. `latexml_sty.rs`: `bibconfig=bbl,bib` keyval from ar5iv.sty now processed — was silently ignored, causing BBL files to be skipped. Bibliography ordering now matches Perl.
5. `elsart_support_core_sty.rs`: `\affiliation[]{key=val,...}` parser — extracts organization, addressline, city, postcode, state, country and produces clean comma-separated affiliation text. 2508.18544 affiliations now match Perl.
6. 2508.18544: bibliography entries match Perl (56 entries, BBL ordering), algo indentation bars present, "1ex" eliminated, affiliations clean.

**Session 92 fixes (2026-04-05) — Visual comparison bug fixes:**
1. `authblk_sty.rs`: `\lx@authormark` constructor had mark text as content (should be empty element). Fixed: removed `#1` from content, matching Perl L56-58. Papers affected: 2603.15617 and any authblk user.
2. `elsart_support_core_sty.rs`: Added `\affiliation[]{}`→`\@@@affiliation` definition. elsarticle uses `\affiliation[inst]{organization={...}}` but this was undefined in both Perl and Rust. Papers affected: 2508.18544.

**Session 91 fixes (2026-04-05) — Bibliography revolution:**
1. Removed `--nobibtex` flag — bibliography processing now enabled for all papers.
2. `\lx@ifusebbl` fallback chain — iterates `BIB_CONFIG` phases (bbl→bib) instead of only checking first.
3. Pure Rust BibTeX parser — `parse_bibtex()` + `convert_bib_file_to_xml()` in `make_bibliography.rs`. Handles `@type{key, field={value}}` with nested braces, string concat, author parsing.
4. MakeBibliography ObjectDB registration — after formatting bibitems, registers `BIBLABEL:*` and `ID:*` entries with `id`, `location`, `fragid`, `number` for CrossRef resolution.
5. Fixed bibitem ID generation — empty `xml:id` on BibTeX-parsed entries now falls back to `bib.bibN`.
6. Fixed libxml2 use-after-free — `bib_docs` kept alive during entry formatting via tuple return.
7. 2511.14458: **Rust now ahead of Perl** — 57 bibitems resolved from raw `.bib`, Perl has 37 missing citations.
8. 20+ papers gain resolved bibliographies from `.bbl` files (previously skipped with `--nobibtex`).

**Session 90 fixes (2026-04-05):**
1. `input_definitions()` search order — versioned package fallback before raw TeX. Banners eliminated in 2306.00809 + 2506.03074. 2405.19425 now uses neurips.sty binding.
2. `\pgfmathsetlength` override — prevents `\pgfmath@` delimiter cascade (A2 papers: 1001→45 errors). Zigzag decoration path now matches Perl.
3. `\pgfmath@smuggleone` override — proper scope smuggling for expandable definitions.
4. Visual comparison: 5/6 issues were FALSE POSITIVES (stale CSS, upstream Perl bug). Only real bug: `authblkRelocateAffil` DOM surgery not ported (2511.14458).
5. Stale Perl HTML regeneration with correct flags (`--nodefaultresources` + ar5iv CSS).

---

### Phase A: Get 10 EMPTY/FAIL papers to produce HTML (37→47 OK)

Papers grouped by shared root cause, ordered by impact (most papers fixed per task):

#### [x] A1. PGF arrows.meta library — PARTIALLY DONE (session 94)
**Papers:** 2209.14198, 2402.10301, 2410.10068
**What was fixed:** `input_definitions()` returned `Ok(())` even when file not found with `noerror=true`, which broke tikz library fallback loading. The loader tried `tikzlibraryarrows.meta.code.tex` (nonexistent), got `Ok`, and never tried `pgflibraryarrows.meta.code.tex` (the real file). Fix: return `Err` on not-found when `noerror=true` (matches Perl `InputDefinitions` which returns undef). All arrow tip errors eliminated.
**Remaining blockers (papers still EMPTY):**
- **2209.14198**: Token limit (30M) hit during tikz decoration processing → empty output. Perl produces full HTML with commutative diagrams.
- **2402.10301**: OOM (4GB+ allocation) during tikz processing → crash. Perl produces full output.
- **2410.10068**: 1001 errors from tikz-cd matrix (\halign, \pgf@matrix@last@nextcell@options) → empty output.

#### [x] A2. PGF pgfscope nesting hang — FIXED (session 95)
**Papers:** 2005.13625, 2103.01205
**Root cause:** The raw TeX `\pgfsetdash` uses `\pgf@strip` — a recursive macro loop with `\ifx\pgf@@temp\pgf@stop` as sentinel. When newlines between pgfscope commands create space tokens, these corrupt the `\pgf@strip` token stream during expansion, causing an infinite loop that consumes all subsequent tokens. The bug is in the interaction between `\pgfsysprotocol@literal`'s `\edef` expansion (which contains nested `\ifx` conditionals) and the line-ending token processing.
**Fix:** Override `\pgfsetdash` with a native Rust `DefPrimitive` that parses dash pattern brace groups natively (extracting dimensions via `pgfmathparse_eval_with_units`), builds the comma-separated dash string, and calls `\pgfsys@setdash{result}{\the\pgf@x}`. This bypasses the `\pgf@strip` loop entirely, following the same pattern as our `\pgfmathsetlength` override. Locked to prevent raw TeX override.
**Result:** 2005.13625: 0KB→987KB (39 errors). 2103.01205: 0KB→471KB (0 errors).

#### [x] A3. PGF keys sentinel recursion — PARTIALLY FIXED (session 95)
**Papers:** 2402.03300
**Root cause:** PGF defines self-referential sentinel macros (`\def\pgfkeys@mainstop{\pgfkeys@mainstop}`) for `\ifx` comparison and delimiter matching. Our expansion engine expands these, triggers recursion detection (returns empty), which breaks the sentinel pattern causing pgfkeys to read past the sentinel in an infinite loop.
**Fix:** Override `\pgfkeys@mainstop` and `\pgfkeysvaluerequired` as locked non-expandable `DefPrimitive` in `pgfsys_latexml_def.rs`. Non-expandable primitives preserve `\ifx` semantics (`\let`-copies have equal meaning) and work as delimiters (matched by CS name, not meaning).
**Result:** Recursion errors eliminated (1586→0). But paper remains EMPTY due to a SEPARATE issue: babel's `\AtBeginDocument` hook destroys PGF arrow definitions (1002 undefined errors → TooManyErrors). This babel+PGF interaction needs deeper investigation — babel's raw TeX hooks run before PGF's hooks and corrupt PGF state.
**Remaining:** babel+PGF `\AtBeginDocument` interaction (also needs `datetime.sty` stub).

#### [x] A4. smfart.cls + expl3 autoload — FIXED (session 95)
**Papers:** 2507.23241
**Root cause:** Two issues: (1) smfart.cls raw TeX loading fails at line 369 (`\ifx\relax\mathfrak` parameter consumption crash), cascading into expl3 failure. (2) `animate.sty` uses `\ExplSyntaxOn` without explicit `\usepackage{expl3}` (LaTeX 2022+ kernel feature).
**Fix:** (1) Created `smfart_cls.rs` binding that loads amsart as base (smfart is an AMS-derived French math journal class). (2) Enabled expl3 autoload triggers in `latex_hook.rs` — `\ExplSyntaxOn`, `\ProvidesExplClass`, `\ProvidesExplPackage` auto-load expl3.sty on first use, matching Perl's `DefAutoload` in `TeX.pool.ltxml` L42-48. (3) Created `animate_sty.rs` (raw load with autoload) and `media9_sty.rs` (stub) bindings.
**Result:** 2507.23241: 0KB→876KB (97 errors, mostly math parser warnings).

#### [x] A5. Stale Perl HTML regeneration — DONE (session 90)
Perl HTML regenerated with correct `--nodefaultresources` + ar5iv CSS flags.

#### Permanent ignores (4 papers — both Perl and Rust fail)
- **2508.15260** (tcolorbox/minted): minted.sty "listing" parameter type not implemented. Perl: 101 errors + fatal. Only 1KB output.
- **2511.03798** (eqnarray): `\@@eqnarray` recursion in jheppub.sty. Perl: 101 errors + fatal.
- **2603.14602** (minted): Same minted parameter type. Perl: LaTeXML dies.
- Note: 2402.03300 Perl also crashes with different error but produces some output.

---

### Phase B: Improve 37 OK papers toward full parity

Ordered by number of papers affected:

#### [x] B1. convertBibliography() — DONE (session 91)
**Result:** Pure Rust BibTeX parser implemented. Raw `.bib` → XML conversion, `\lx@ifusebbl` fallback chain, ObjectDB registration. 20+ papers gain resolved bibliographies. 2511.14458 is Rust-ahead (57 bibitems vs Perl's 37 missing).
**Remaining:** 13 papers still have `missing_citation` — these use inline `\thebibliography` or have no `.bib`/`.bbl` files. Not a convertBibliography issue.

#### [x] B2. authblkRelocateAffil — DONE (session 90)
DOM surgery ported in `authblk_sty.rs`: `Tag!("ltx:document", after_close => ...)` + `authblk_relocate_affil()`. 2511.14458 affiliations now match Perl.

#### [x] B3. Listing per-token syntax highlighting — ALREADY DONE (session 93)
**Result:** Per-token styling was already working. Session 93 fixed `lstdefinestyle` type mismatch, which activated listing styles. Rust output now matches Perl: 146 `--ltx-fg-color` occurrences, 28 styled tokens with `ltx_lst_string`/`ltx_lst_keyword`/`ltx_lst_comment` classes. Colors match: `#9400D1` (strings), `#FF00FF` (keywords), `#009900` (comments). Background `#F2F2EB` and line numbers also correct.

#### [x] B4. \shortstack/\vtop mode cascade — DONE (session 96)
**Root cause:** `\shortstack` rebinding `\lx@hidden@cr` to `\@shortstack@cr` caused `is_column_end()` to match `\\` as a table column separator inside alignment contexts (because `\lx@hidden@cr` is a COLUMN_END sentinel, and `is_column_end` compares meanings). When `align_group_count` reached 0 (from the `"before-column"` MARKER in the alignment template), `read_x_token`'s column-end check intercepted `\\` tokens inside `\shortstack`'s `{}` argument, injecting column-end tokens into the content stream and breaking mode nesting.
**Fix:** Removed `Let!("\\lx@hidden@cr", "\\@shortstack@cr")` from `\shortstack`'s beforeDigest — matches Perl, which only rebinds `\\` (not `\lx@hidden@cr`). Kept `Let!("\\lx@newline", "\\@shortstack@cr")` because `\\` is Let to `\lx@newline` at the top level. Updated diagbox expected XML for minor dimensional changes.
**Result:** 2508.18544 goes from 22 errors (11 shortstack + 11 vtop cascading) to 0 errors.

#### [x] B5. tikzpicture mode corruption — RESOLVED (session 96, via A2)
**Result:** The mode corruption was caused by pgfscope nesting issues fixed in A2 (session 95). After A2, 2603.15617 produces 1.2MB output with 22 sections (previously only 3% content). Only 3 remaining errors: `verbatim` inside `_CaptureBlock_` during building phase — a minor construction issue, not mode corruption.

#### [x] B6. tikz-cd for 2602.18719 — LARGELY RESOLVED (session 96)
**Result:** After A1/A2 fixes, 2602.18719 produces 1.7MB output with only 17 errors (all `Cannot find node with xml:id='S4.Ex63.m1.*'` — id reference issues in one equation, not tikz-cd). The tikz-cd content renders.

#### [x] B7. 1502.04955: missing sections 6–7 and bibliography — DONE (session 94)
**Root cause:** Two bugs:
1. `DefMacro!("\\begin{keyword}", ...)` wrongly parsed `{keyword}` as a parameter spec instead of as part of the compound CS name. Fix: use `DefMacro!(T_CS!("\\begin{keyword}"), None, ...)`.
2. `\@keyword` used `Until:` (non-expanding) instead of `XUntil:` (expanding), so `\end{keyword}` → `\@keyword@cut` sentinel was never found.
**Result:** All 7 sections + bibliography (95 references) now appear. Also fixed same bug pattern in `minted_sty`, `breqn_sty`, `siamltex_cls` (all `DefMacro!("\\begin{...}")` → `DefMacro!(T_CS!("\\begin{...}"), None, ...)`).

#### [x] B8. 2101.00726: images failing to render — RESOLVED (session 96)
**Result:** Only 1 error (`\PackageNoteNoLine` undefined in utf8x.def). 45 graphics references present in XML output. Image rendering is a post-processing issue (B10), not a digestion bug.

#### [x] B9. 2310.18318: missing table of contents — RESOLVED (session 96)
**Result:** TOC is present in output (2 `ltx:TOC` elements). Only 2 minor xy-pic errors (`\xyshape@thicker@`, `\xylinewidth@i` undefined).

#### [x] B10. Graphics post-processing — ALREADY WORKING (session 96 verified)
**Result:** The Rust graphics post-processor (`latexml_post::graphics::Graphics`) is fully functional when `--post` is enabled. It resolves graphic files via search paths, converts PDF/EPS→PNG via ImageMagick, detects image dimensions via `imagesize` crate, assigns aspect-ratio classes (`ltx_img_landscape`/`ltx_img_square`/`ltx_img_portrait`), and renames to `x1.png`/`x2.png` etc. Session 96 fixed `find_graphic_file` to resolve candidate paths relative to search paths (not just CWD).
**Verified on 2405.19425:** `--post` produces `imagesrc="x1.png" imagewidth="601" imageheight="319" class="ltx_centering ltx_img_landscape"` — matching Perl's output structure.

---

### Permanent ignores (regression tests)
- **ns1–ns5** (52_namespace) — DTD not supported in Rust port.

### Permanent ignores (arxiv papers — Perl also fails)
- **2508.15260** — tcolorbox + minted cascading. Perl output: 1KB.
- **2511.03798** — jheppub eqnarray recursion. Perl: 101 errors + fatal.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
