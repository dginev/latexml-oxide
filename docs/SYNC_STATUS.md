# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-04. Only lists open gaps & TODOs; completed items live in git history.

**Test inventory:** 407 tests pass (338 integration + 1 post + 39+7+6+15 latexml_post unit tests + 1 post integration). All integration tests zero-diff against Rust reference XMLs. Perl reference parity: 221/298 zero-diff (74.2%), ~31K diff lines across 77 non-zero tests (xml:id renumbering + math parser structural diffs + SVG differences).

**arxiv sandbox:** 36/47 papers produce meaningful HTML output. 6 fail (cascading errors from missing packages), 5 timeout (>60s, mostly tikz-heavy).

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
| base_parameter_types.rs | GAPS | `DirectoryList`, `CommaList`, `DigestUntil` unported; `Variable` reversion `todo!()` |
| base_xmath.rs | MINOR | Missing: `MathWhatsit()` |
| tex_box.rs | MINOR | Minor box dimension edge cases |
| tex_fonts.rs | MINOR | Missing: `\fontdimen` full array semantics, `getFontDimen()` helper |
| tex_tables.rs | MINOR | Minor: padding CSS classes |
| latex_ch4_sectioning_and_toc.rs | MINOR | Missing: `LABEL_MAPPING_HOOK` |
| latex_ch14_pictures_and_color.rs | GAPS | 30% — picture environment not implemented |

---

## Missing Tag() Calls

| Tag | Perl Source |
|-----|-------------|
| `Tag('ltx:picture', autoOpen => 0.5, autoClose => 1, ...)` | latex_constructs L4994 |

---

## Cross-Cutting Infrastructure Gaps

1. **`FontDef` parameter type** — Simplified to `FontToken`. Blocks `\fontdimen`, `\hyphenchar` per-font tracking.

---

## Unported Perl Files

| File | Defs | Priority | Notes |
|------|------|----------|-------|
| `latex_constructs.pool.ltxml` | ~843 | Low | ~93% ported. Missing: picture env |
| `math_common.pool.ltxml` | 312 | OK | Fully ported (DefMathLigature, all symbols) |
| `Base_Deprecated.pool.ltxml` | 77 | Low | ~16% — deprecated compat shims |
| `AmSTeX.pool.ltxml` | 112 | Low | ~30% |
| `BibTeX.pool.ltxml` | 150 | Low | ~9% |

---

## Core Modules (MINOR+ only)

| Module | Status | Open Gaps |
|--------|--------|-----------|
| gullet.rs | MINOR | `readArg` isolation (type ergonomics) |
| document.rs | MINOR | XML comment creation (needs libxml2 FFI) |
| rewrite.rs | MINOR | ~98% ported. `domToXPath` ported (L686-850). Missing: `digest_rewrite` helper |
| pathname.rs | MINOR | Missing: `pathname_make`, `pathname_relative`, `pathname_findall` |

---

## Package.pm — DefFoo Sync Status (dialect.rs)

| DefFoo | Status | Gaps |
|--------|--------|------|
| `DefMacroI` | MINOR | `outer`/`long` not mapped |
| `DefPrimitiveI` | MINOR | Missing `outer`/`long` |
| `DefConstructorI` | MINOR | Missing `outer`/`long`/`attributeForm`; robust alias fallback |

---

## Package Bindings (open gaps only)

| Package | Status | Notes |
|---------|--------|-------|
| amsmath.sty | MINOR | ~95% ported. Minor: cfrac mathstyle tracking |
| listings.sty | MINOR | ~95% ported. Missing: literate `*` (protected) flag enforcement |

All other packages OK. 411 packages (433+ dispatch entries) + 91 ar5iv contrib bindings.

---

## Tikz Test References

XML files in `LaTeXML/t/tikz/` are OUTDATED. Always regenerate fresh Perl output.

### Priority FIX items (shared across tikz tests)

1. **foreignObject transform Y=16.6** — Perl uses fixed 12pt maxy; Rust uses actual height
2. **foreignObject width/height** — `fo_get_size` differs from Perl
3. **Nested minipage/SVG sizing** — `appendNodeBox` vs Perl's `pushContent`
4. **Arrow tip shape** — Different arrowhead path data
5. **`<pagination role="newpage"/>`** — Missing `\newpage` handling
6. **SVG viewBox/width** — Total dimensions differ slightly
7. **Listings escapechar + color** — `escapechar=@` with `\color{red}` inline
8. **Missing `\vspace{2mm}` output** — `\vspace` in vertical mode

---

## Work Plan — Ordered TODO List

Follow this list in order. Work on the first unchecked `[ ]` item. Skip items marked BLOCKED.

**Status (2026-04-04):** 407 pass, 0 fail, 0 ignored.

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

- [ ] **B1. Port IEEEtran.cls binding** — 458-line Perl binding. Used by IEEE conference papers (arxiv 2511.11713). Defines class options, `\IEEEauthorblockN/A`, `{IEEEkeywords}`, section numbering (Roman), `{IEEEproof}`, `\IEEEurl`, `\IEEEPARstart`. LoadClass('article') as parent.
- [ ] **B2. Port JHEP.cls binding** — 314-line Perl binding. Used by JHEP physics journal (arxiv 2511.03798). Defines `\JHEP@preprint`, `\procemark`, author/affiliation, section formatting, bibliography style.
- [ ] **B3. Port pstricks.sty binding** — 44-line Perl binding (mostly stubs). Used by 2 arxiv papers. Defines `\psset`, `\pscircle`, `\psline`, `\rput` as no-ops since PSTricks requires DVI backend.

### Open TODO items — Engine Parity

- [ ] **E1. Precompile kernel dumps on `cargo build`** — Design in `docs/DUMP_DESIGN.md`. build.rs updated with TeX Live version check. Manual generation still required.
- [ ] **E2. `\newpage` in SVG/tikz context** — `\lx@newpage` is defined but `<pagination>` elements are missing from 10 tikz test outputs. The `^` prefix float-up may not work inside tikzpicture's SVG mode. Affects 10 tests (~22 diff lines).
- [ ] **E3. Reduce TooManyErrors aborts** — 6 arxiv papers abort due to cascading errors from missing packages. Need: (a) increase MAX_ERRORS default for real-world papers, (b) better error recovery so `\end{document}` still produces partial output after Fatal, (c) match Perl's error tolerance.
- [ ] **E4. Fix `\@@eqnarray` recursion** — Paper 2511.03798 hits infinite recursion: `\@@eqnarray` → `$` → `\lx@begin@inline@math` → `\@@eqnarray` cycle. Root cause: eqnarray triggered inside inline math mode by jheppub.sty.

### Open TODO items — Math Parser & Post-Processing

- [ ] **S1. siunitx unit tree builder** — `six_convert_units_from_tokens` handles simple unit chains. Missing: non-unit content passthrough (`\pi`, `\frac{}`), literal notation (`m^2.s`), complex number formatting (`I_dual`). Currently ~6900 normalized diffs vs Perl.
- [ ] **M14. Reduce `\lxDeclare` rewrite diffs** — `declare.xml` has 859 diffs. Key issues: subscript content not math-parsed inside `\hat{x}` patterns, `decl_id` not propagated to all matching tokens, wildcard pattern `\WildCard` matching incomplete.

### Open TODO items — Library & Infrastructure

- [ ] **L1. Deep clone for `rust-libxml`** — Add `xmlCopyNode` FFI wrapper for `node.deep_copy()`. Required for Scan storing XML node values (currently stores text, losing inline markup).
- [ ] **L2. `get_attribute("xml:id")` for `rust-libxml`** — Returns None on some builds. Workaround: `get_property("id")`.
- [ ] **L4. Default namespace handling in `rust-libxml`** — Creates `<ltx:ref>` instead of `<ref>` when default xmlns matches. Workaround in place.
- [ ] **X1. arxiv batch comparison catalog** — Systematic comparison of all 47 papers. Current: 36 OK, 6 fail, 5 timeout. Track diff counts per paper, identify top-5 blockers.

### ar5iv conversion sandbox (47 papers)

`arxiv-examples/` contains 47 arXiv papers for parity testing between latexml-oxide and latexmlc. Run `arxiv-examples/compare.sh [id]` to generate and compare HTML output.

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
