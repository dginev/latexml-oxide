# Engine Organization: Perl → Rust Mapping

This document describes how the Rust files in `latexml_engine/src/`
relate to the Perl files in `LaTeXML/lib/LaTeXML/Engine/`.

**All 36 ported Perl engine files have 1:1 matching Rust files.**

## Loading hierarchy

### Perl
```
TeX.pool
├── Base.pool          (loads all Base_* + TeX_* + eTeX + pdfTeX + Base_Deprecated)
│   ├── Base_Schema
│   ├── Base_ParameterTypes
│   ├── Base_Utility
│   ├── Base_XMath
│   ├── TeX_Box .. TeX_Tables  (18 files)
│   ├── eTeX
│   ├── pdfTeX
│   └── Base_Deprecated
└── LoadFormat('plain')
    ├── plain_bootstrap
    ├── plain_base
    ├── plain_dump
    └── plain_constructs
        └── math_common

LaTeX.pool
├── LoadPool('TeX')        (everything above)
└── LoadFormat('latex')
    ├── latex_bootstrap
    ├── latex_base
    ├── latex_dump
    └── latex_constructs
```

### Rust

Strict-Perl `LoadFormat` mutual exclusivity (commit `0c4d609ad` and
follow-ups). Either dump XOR base, never both:

```
tex.rs                     (≈ TeX.pool + Base.pool combined)
├── base_schema
├── base_parameter_types
├── base_utilities         (≈ Base_Utility)
├── base_xmath
├── tex_box .. tex_tables  (18 files, same as Perl)
├── etex
├── pdftex
├── base_deprecated
├── plain_bootstrap        (≈ plain_bootstrap.pool.ltxml)
├── if dump available && !LATEXML_NODUMP:
│     plain_dump           (runtime loader for plain.dump.txt)
│   else:
│     plain_base           (≈ plain_base.pool.ltxml)
└── plain_constructs       (≈ plain_constructs.pool.ltxml)
    └── math_common        (≈ math_common.pool.ltxml)

latex.rs                   (≈ LaTeX.pool)
├── LoadPool!("TeX")       (loads tex.rs above)
├── latex_bootstrap.rs     (≈ latex_bootstrap.pool.ltxml)
├── if dump available && !LATEXML_NODUMP:
│     latex_dump           (runtime loader for latex.dump.txt)
│   else:
│     latex_base.rs        (≈ latex_base.pool.ltxml)
└── latex_constructs.rs    (≈ latex_constructs.pool.ltxml, ~8675 lines, sections C.1–C.15)
```

Mirrors Perl `Package.pm:LoadFormat` L2734-2752 exactly. See
[`PERL_LOADFORMAT_AUDIT.md`](PERL_LOADFORMAT_AUDIT.md) for the
parity audit.

Dump-build snapshots are taken by the init/dump-generation path
(`ini_tex::dump_format`), not by the normal runtime `tex.rs` /
`latex.rs` load chain.

## File-by-file mapping

### TeX primitives (Base.pool children)

| Perl file | Rust file | Notes |
|---|---|---|
| `Base.pool.ltxml` | *(inlined in `tex.rs`)* | Pure loader — zero definitions, just `LoadPool(...)` calls |
| `Base_Schema.pool.ltxml` | `base_schema.rs` | |
| `Base_ParameterTypes.pool.ltxml` | `base_parameter_types.rs` | |
| `Base_Utility.pool.ltxml` | `base_utilities.rs` | Rust name uses plural convention |
| `Base_XMath.pool.ltxml` | `base_xmath.rs` | |
| `TeX_Box.pool.ltxml` | `tex_box.rs` | |
| `TeX_Character.pool.ltxml` | `tex_character.rs` | |
| `TeX_Debugging.pool.ltxml` | `tex_debugging.rs` | |
| `TeX_FileIO.pool.ltxml` | `tex_file_io.rs` | |
| `TeX_Fonts.pool.ltxml` | `tex_fonts.rs` | |
| `TeX_Glue.pool.ltxml` | `tex_glue.rs` | |
| `TeX_Hyphenation.pool.ltxml` | `tex_hyphenation.rs` | |
| `TeX_Inserts.pool.ltxml` | `tex_inserts.rs` | |
| `TeX_Job.pool.ltxml` | `tex_job.rs` | |
| `TeX_Kern.pool.ltxml` | `tex_kern.rs` | |
| `TeX_Logic.pool.ltxml` | `tex_logic.rs` | |
| `TeX_Macro.pool.ltxml` | `tex_macro.rs` | |
| `TeX_Marks.pool.ltxml` | `tex_marks.rs` | |
| `TeX_Math.pool.ltxml` | `tex_math.rs` | Includes subscript/superscript (was `tex_scripts.rs`) |
| `TeX_Page.pool.ltxml` | `tex_page.rs` | |
| `TeX_Paragraph.pool.ltxml` | `tex_paragraph.rs` | |
| `TeX_Penalties.pool.ltxml` | `tex_penalties.rs` | |
| `TeX_Registers.pool.ltxml` | `tex_registers.rs` | |
| `TeX_Tables.pool.ltxml` | `tex_tables.rs` | |
| `eTeX.pool.ltxml` | `etex.rs` | |
| `pdfTeX.pool.ltxml` | `pdftex.rs` | |
| `Base_Deprecated.pool.ltxml` | `base_deprecated.rs` | |

### Plain TeX format (LoadFormat 'plain')

| Perl file | Rust file | Notes |
|---|---|---|
| `plain_bootstrap.pool.ltxml` | `plain_bootstrap.rs` | `\TeX`, `\newif`, `\leavevmode`, `\alloc@` |
| `plain_base.pool.ltxml` | `plain_base.rs` | Appendix B: registers, allocation, spacing, formatting |
| `plain_constructs.pool.ltxml` | `plain_constructs.rs` | Font commands, accents, alignment, footnotes |
| `math_common.pool.ltxml` | `math_common.rs` | Greek, operators, relations, arrows, delimiters, log functions |

Loading chain in `tex.rs` (strict-Perl `LoadFormat`):
`InnerPool!(plain_bootstrap)` → EITHER `InnerPool!(plain_dump)` OR
`InnerPool!(plain_base)`
(mutually exclusive) → `InnerPool!(plain_constructs)` (which loads
`InnerPool!(math_common)`).

### LaTeX format (LoadFormat 'latex')

| Perl file | Rust file | Notes |
|---|---|---|
| `latex_bootstrap.pool.ltxml` | `latex_bootstrap.rs` | Stubs for font/counter internals |
| `latex_base.pool.ltxml` | `latex_base.rs` | Infrastructure: DefMacro, Let, DefRegister, RawTeX |
| `latex_constructs.pool.ltxml` | `latex_constructs.rs` | All constructors/environments (~8675 lines, C.1–C.15) |

Loading chain in `latex.rs` (strict-Perl `LoadFormat`):
`LoadPool!("TeX")` → `InnerPool!(latex_bootstrap)` →
EITHER `InnerPool!(latex_dump)` OR `InnerPool!(latex_base)`
(mutually exclusive) →
`InnerPool!(latex_constructs)`.

### LaTeX constructs — section mapping

`latex_constructs.rs` (~8675 lines) is a single file matching Perl's
`latex_constructs.pool.ltxml` (6014 lines). It contains all LaTeX semantic
definitions organized by Lamport chapter with section comment headers.

| Section | Lamport chapter | Perl lines (approx) |
|---|---|---|
| C.1 | Commands and Environments | 31–276 |
| C.2 | The Structure of the Document | 276–372 |
| C.3 | Sentences and Paragraphs | 372–587 |
| C.4 | Sectioning and Table of Contents | 588–833 |
| C.5 | Classes, Packages and Page Styles | 833–1310 |
| C.6 | Displayed Paragraphs | 1311–1646 |
| C.7 | Mathematical Formulas | 1646–2246 |
| C.8 | Definitions, Numbering and Programming | 2247–2785 |
| C.9 | Figures and Other Floating Bodies | 2786–2985 |
| C.10 | Lining It Up in Columns | 2985–3229 |
| C.11 | Moving Information Around | 3230–3832 |
| C.12 | Line and Page Breaking | 3832–3866 |
| C.13 | Lengths, Spaces and Boxes | 3866–4123 |
| C.14 | Pictures and Color | 4124–4414 |
| C.15 | Font Selection and Special Symbols | 4414–5366 |
| (auxiliary) | Auxiliary file stubs, language declarations | 5366–6014 |

### Unported Perl engine files

| Perl file | Status | Notes |
|---|---|---|
| `AmSTeX.pool.ltxml` | ~30% ported | Plain TeX format, rare |
| `BibTeX.pool.ltxml` | Not ported | Skipped via `--nobibtex` in production |

### Rust-only files (no Perl .pool.ltxml equivalent)

| Rust file | Purpose |
|---|---|
| `engine.rs` | Module declarations (Rust boilerplate) |
| `plain_dump.rs` | Runtime loader for `resources/dumps/plain.dump.txt` (delegates to `dump_reader`) |
| `latex_dump.rs` | Runtime loader for `resources/dumps/latex.dump.txt` (delegates to `dump_reader`) |
