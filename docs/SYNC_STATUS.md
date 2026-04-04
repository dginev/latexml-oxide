# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-05. Only lists open gaps & TODOs; completed items live in git history.

**Test inventory:** 407 tests pass (338 integration + 1 post + 39+7+6+15 latexml_post unit tests + 1 post integration). All integration tests zero-diff against Rust reference XMLs. Perl reference parity: 246/314 effective zero-diff (78%), ~18K meaningful diff lines across 68 non-zero tests. Top diff sources: siunitx (3.5K), SVG/tikz (4.3K), beamer (1.2K), physics (1.2K).

**arxiv sandbox:** 42/48 papers produce output (88%). 6 fail (4 timeout, 1 pgf arrows, 1 wrong main file).

**Production-ready:** Full CorTeX ZIP-to-ZIP pipeline operational. All legacy production options supported:
```
latexml_oxide --whatsin=archive --format=html5 --pmml --mathtex --noinvisibletimes \
  --nodefaultresources --nobibtex --preload=ar5iv.sty --timeout=2700 --log=log.txt \
  --dest=output.zip input.zip
```

**High-level roadmap:** See [`mini_3_plan.md`](mini_3_plan.md) for the 4-phase strategic plan
(Engine Parity → Package Bindings → Post-Processing → Production).

## Legend
- **OK** = fully synced | **MINOR** = small gaps | **GAPS** = significant missing | **EMPTY** = not ported

**See also:** [`KNOWN_PERL_ERRORS.md`](KNOWN_PERL_ERRORS.md) — upstream Perl issues (not Rust bugs)

---

## Engine Files — Open Gaps Only

Only files with GAPS or significant MINOR issues listed. OK files omitted (see git history).

| File | Status | Open Gaps |
|------|--------|-----------|
| base_parameter_types.rs | MINOR | `DirectoryList`, `CommaList`, `DigestUntil` stubbed (low usage); `Variable` reversion safe fallback |
| base_xmath.rs | MINOR | Missing: `MathWhatsit()` |
| tex_box.rs | MINOR | Minor box dimension edge cases |
| tex_fonts.rs | MINOR | Missing: `\fontdimen` full array semantics, `getFontDimen()` helper |
| tex_tables.rs | MINOR | Minor: padding CSS classes |
| latex_ch4_sectioning_and_toc.rs | MINOR | Missing: `LABEL_MAPPING_HOOK` |
| latex_ch14_pictures_and_color.rs | OK | ~95% — picture env fully ported (put, line, vector, circle, oval, qbezier, multiput, dashbox, shortstack, pic@makebox) |

---

## Missing Tag() Calls

None — all critical Tag() calls ported.

---

## Cross-Cutting Infrastructure Gaps

1. **`FontDef` parameter type** — Simplified to `FontToken`. Blocks `\fontdimen`, `\hyphenchar` per-font tracking.

---

## Unported Perl Engine Files

| File | Defs | Status | Notes |
|------|------|--------|-------|
| `latex_constructs.pool.ltxml` | ~843 | ~98% | Picture env ported. Remaining: \ensuremathfollows (internal) |
| `math_common.pool.ltxml` | 312 | OK | Fully ported |
| `Base_Deprecated.pool.ltxml` | 77 | OK | Fully ported |
| `AmSTeX.pool.ltxml` | 112 | ~30% | Plain TeX format (rare) |
| `BibTeX.pool.ltxml` | 956 | 0% | Skipped via `--nobibtex` in production |

---

## Core Modules (MINOR+ only)

| Module | Status | Open Gaps |
|--------|--------|-----------|
| gullet.rs | MINOR | `readArg` isolation (type ergonomics) |
| document.rs | MINOR | XML comment creation (needs libxml2 FFI) |
| rewrite.rs | OK | ~98% ported. `digest_rewrite` not needed (different Rust approach) |
| pathname.rs | OK | Fully ported (`make`, `relative`, `findall` added) |

---

## Package.pm — DefFoo Sync Status (dialect.rs)

| DefFoo | Status | Gaps |
|--------|--------|------|
| `DefMacroI` | MINOR | `outer`/`long` not mapped |
| `DefPrimitiveI` | MINOR | Missing `outer`/`long` |
| `DefConstructorI` | MINOR | Missing `outer`/`long`/`attributeForm`; robust alias fallback |

---

## Package Bindings — Exhaustive Translation Audit

**Goal: translate every `.sty.ltxml` and `.cls.ltxml` in Perl to Rust, exhaustively.**

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
5. ~~`<pagination role="newpage"/>`~~ — Resolved: stale Perl refs
6. **SVG viewBox/width** — Total dimensions differ slightly
7. **Listings escapechar + color** — `escapechar=@` with `\color{red}` inline
8. **Missing `\vspace{2mm}` output** — `\vspace` in vertical mode

---

## Work Plan — Ordered TODO List

Follow this list in order. Work on the first unchecked `[ ]` item. Skip items marked BLOCKED.

**Status (2026-04-05):** 407 pass, 0 fail, 0 ignored. Zero `todo!()` panics, 3 engine `panic!()` hardened.

### Completed (summary — see git history for details)

- [x] F–I: Post-processing pipeline, codegen Until, pgfsys patterns, unified CLI
- [x] P1–P8: Scan, CrossRef, MakeBibliography, Split, Writer, Graphics, MathML intent, Plane 1 Unicode
- [x] A1–A16: All ar5iv CLI options
- [x] M1–M12: Math parser ambiguity reduction (diffop filtering, formulae split, online dedup, convergence budget, script arg dedup)
- [x] D1–D3: Diff reduction (header guessing, equation numbering, listings)
- [x] SVG color groups, math parser dedup, CLI directory creation, preload+option handling
- [x] ar5iv 2502.04134 comparison (structural parity achieved)
- [x] L3: rust-libxslt migration
- [x] DefMathLigature: all 8 ligatures ported (double-factorial, assign, letter combining, number combining, cdots, ldots)
- [x] siunitx `\lx@six@unitobject@collapsible` macro (Perl L1227-1249)
- [x] M12/M13: Script argument 2^N ambiguity eliminated (removed `expression` from postsubarg/postsuperarg/bigopsub/bigopsup — 8-333x raw tree reduction)

### Open TODO items — Package Bindings

- [x] **B1. Port IEEEtran.cls binding** — 166-line Rust binding. Working for arxiv 2511.11713 (94KB output).
- [x] **B2. Port JHEP.cls binding** — 77-line Rust binding. Frontmatter, acknowledgements, journal abbreviations.
- [x] **B3. Port pstricks.sty binding** — 55-line Rust stub (DVI-only, all commands no-op or passthrough).
- [x] **B4. Expand 9 PARTIAL bindings** — revtex4_support, aas_support, braket (pipe-splitting), algorithm2e, subfig, inst_support, html, titling, authblk (+423 lines total).
- [x] **B5. Expand 7 more bindings** — jheppub (full port), iopart_support (journal abbrevs, bibliography), JHEP.cls (journal refs, arXiv links), elsart_support (theorems, isotopes), elsart_support_core (keyword env), mn2e_support (bold Greek, math relations), texvc (Greek, aliases). +630 lines total.
- [x] **B6. Expand IEEEtran, OmniBus, aa_support, cfrac** — IEEEtran (options, biography, eqnarray), OmniBus (requires, altaffilmark, references, metadata), aa_support (14 requires, abstract, theorems), amsmath cfrac mathstyle.
- [x] **B7. Expand sv_support, subfloat, psfrag, llncs, geometry** — sv_support (15 theorem envs), subfloat (container envs), psfrag (rescan macros), llncs (15 theorems), geometry (requires fix).

### Open TODO items — Engine Parity

- [x] **E1. Precompile kernel dumps on `cargo build`** — Complete: build.rs checks for dumps, generates loaders/stubs, validates TeX Live version. `--init` and `--codegen` CLI flags work. Manual dump generation is by design (bootstrap problem).
- [x] **E2. `\newpage` in SVG/tikz context** — Resolved: Perl tikz reference XMLs are outdated. Fresh Perl output matches Rust (no pagination). Not a code bug.
- [x] **E3. FindFile_fallback for versioned packages** — Ported. 2306.00809: 39B→141KB, 2402.03300: 53KB→322KB.
- [x] **E4. Reduce TooManyErrors aborts** — MAX_ERRORS default raised to 10000 (was 100). digest_internal error recovery improved to catch Fatals during salvage.
- [x] **E5. Fix `\@@eqnarray` recursion** — Fixed: eqnarray/inline math cycle broken.

### Open TODO items — Math Parser & Post-Processing

- [x] **S1. siunitx mixed-content unit passthrough** — Fixed: `six_resolve_unit_objects()` resolves `\lx@six@unitobject` tokens to presentation text within the active unit context, before falling to `six_parse_literalunits`. `\pi . \mm . \mrad` now produces `pi * mm * mrad`. 6th attempt succeeded after 5 failed approaches.
- [ ] **M14. Reduce `\lxDeclare` rewrite diffs** — `declare.xml` has ~400 diffs. decl_id now propagated (0→49 of 84 Perl). Remaining: subscript content not matched (35 missing decl_id), structural XMDual/XMApp wrapping differences.

### Open TODO items — Library & Infrastructure

- [x] **L1. Deep clone for `rust-libxml`** — `append_clone` fully implemented in Document (100 lines with id remapping). ObjectDB stores text values (Perl stores nodes); functional for current usage.
- [x] **L2. `get_attribute("xml:id")` for `rust-libxml`** — Workaround stable: `get_property("id")` used in 3 places. rust-libxml upstream issue.
- [x] **L4. Default namespace handling in `rust-libxml`** — Workaround stable in document.rs serializer. rust-libxml upstream issue.
- [x] **X1. arxiv batch comparison catalog** — Done: 38/47 OK (81%), 3 EMPTY, 6 FAIL. 5 zero-error papers. See `arxiv-examples/CATALOG.md`.

### ar5iv conversion sandbox (48 papers)

`arxiv-examples/` contains 48 arXiv papers for parity testing between latexml-oxide and latexmlc. Run `arxiv-examples/compare.sh [id]` to generate and compare HTML output.

Papers span 2007–2026, covering diverse LaTeX packages (natbib, hyperref, amsmath, tikz, beamer, siunitx, physics, listings, etc.). Key test targets:
- Citation resolution (natbib `\citep`/`\citet`)
- Custom class files (nips, iclr, acl, aastex, IEEE)
- Complex math (physics, quantum mechanics)
- Tables and figures (booktabs, subfigure, graphicx)
- Bibliographies (BibTeX, biblatex)

### Permanent ignores (5)
- **ns1–ns5** (52_namespace) — DTD not supported in Rust port.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
