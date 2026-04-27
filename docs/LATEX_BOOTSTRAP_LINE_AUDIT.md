# `latex_bootstrap.pool.ltxml` ↔ `latex_bootstrap.rs` line audit

Strict line-by-line walk of the Perl bootstrap pool against the Rust
sibling. The Perl source is 65 lines; this is a single-phase audit.

**Status legend**:
* ✅ PARITY — Perl entry has Rust counterpart in expected location.
* ↻ ORDER — entry exists in Rust but in a different sibling file.
* 📁 FILE — entry placed correctly relative to file structure.
* ⚠ DIVERGE — entry differs in semantics or shape.
* ❌ MISSING — Perl entry has no Rust counterpart.
* 🔵 RUST_ONLY — Rust entry without Perl source.

| Perl L | Symbol | Rust file:line | Status |
|--------|--------|----------------|--------|
| 18 | `LoadPool('plain_bootstrap')` | latex_bootstrap.rs:11 (`InnerPool!(plain_bootstrap)`) | ✅ |
| 22-32 | `\LaTeX` Constructor (CSS-styled L<a>T<e>X) | latex_bootstrap.rs:17-24 | ✅ |
| 33-44 | `\LaTeXe` Constructor (LaTeX2ε) | latex_bootstrap.rs:26-34 | ✅ |
| 49 | `\e@alloc{}{}{}{}{}{}` (locked) | latex_bootstrap.rs:38 | ✅ |
| 50 | `\e@ch@ck{}{}{}{}` (locked) | latex_bootstrap.rs:39 | ✅ |
| 51-53 | `\newcounter{}[]` DefPrimitive (locked) | latex_constructs.rs:5772 | ↻ MISPLACED |
| 54 | `\@definecounter` (DefMacro→`\newcounter`, locked) | latex_bootstrap.rs:42 | ✅ ⚠ shape |
| 58 | `Let '\@@input' '\input'` | latex_bootstrap.rs:48 | ✅ |
| 62 | `\try@load@fontshape` (locked, empty) | latex_bootstrap.rs:43 | ✅ |
| 63 | `\define@newfont` (locked, empty) | latex_bootstrap.rs:44 | ✅ |

## Findings

* **Strong PARITY** for 7/9 substantive entries. Both are direct
  translations with matching semantics (`\LaTeX`/`\LaTeXe` Constructor
  shapes, `\e@alloc` macro, `\@@input` Let, `\try@load@fontshape` and
  `\define@newfont` lock-stubs).
* **↻ MISPLACED**: `\newcounter` (Perl L51-53) is in `latex_constructs.rs:5772`
  in Rust instead of `latex_bootstrap.rs`. Per CLAUDE.md "exact same
  definitions in the exact same order" mandate, this should relocate
  to `latex_bootstrap.rs`. Has identical semantics to Perl: 2-arg
  primitive that calls `NewCounter` after expanding the args. Also
  picked up the `\setcounter`/`\addtocounter`/`\stepcounter`/
  `\refstepcounter` siblings at the same Rust location — those are
  Perl latex.ltx primitives consumed via raw-load, so they may legit-
  imately belong in latex_constructs.rs (kernel-derived, not bootstrap).
* **⚠ Shape divergence**: `\@definecounter` (Perl L54) is `Let
  \@definecounter \newcounter`-style (per Perl `locked => 1`) but
  Rust uses `DefMacro!("\\@definecounter", "\\newcounter", locked
  => true)` which is a token-list macro, not a Let. Functional
  equivalent in normal usage; flagging for completeness.
* **Pre-defined fallback at L15-16**: Rust `latex_bootstrap.rs` adds
  `DefMacro!("\\LaTeX", "LaTeX")` and `DefMacro!("\\LaTeXe", "LaTeX2e")`
  *before* the Constructor definitions at L17-34. These are dead-code
  fallbacks since Constructors override them in the same load. Could
  remove, but harmless. Not in Perl; flag as 🔵 RUST_ONLY but cosmetic.

## Cumulative parity health

`latex_bootstrap.pool.ltxml` is **mostly parity** with one
substantive misplacement (`\newcounter`). The bootstrap layer is
small and well-aligned.
