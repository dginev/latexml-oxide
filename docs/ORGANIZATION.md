# Engine Organization: Perl → Rust Mapping

This document describes how the Rust files in `latexml_package/src/engine/`
relate to the Perl files in `LaTeXML/lib/LaTeXML/Engine/`.

## Loading hierarchy

### Perl
```
TeX.pool
├── Base.pool          (loads all TeX_* + eTeX + pdfTeX + Base_Deprecated)
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
    └── plain_constructs
        └── math_common

LaTeX.pool
├── LoadPool('TeX')        (everything above)
├── LoadFormat('latex')
│   ├── latex_bootstrap
│   ├── latex_base
│   └── latex_constructs
│       ├── plain_constructs  (won't reload)
│       └── math_common       (won't reload)
└── (C.1 through C.14 chapters inline in LaTeX.pool)
```

### Rust
```
tex.rs                     (≈ TeX.pool + Base.pool combined)
├── base_schema
├── base_parameter_types
├── base_utilities
├── base_xmath
├── tex_box .. tex_tables  (18 files, same as Perl)
├── tex_scripts            (≈ subscript/superscript part of TeX_Math)
├── etex
├── pdftex
└── plain                  (≈ plain_bootstrap + plain_base + plain_constructs + math_common)

latex.rs                   (≈ LaTeX.pool)
├── LoadPool!("TeX")       (loads tex.rs above)
├── latex_bootstrap.rs     (≈ latex_bootstrap.pool.ltxml)
├── latex_base.rs          (≈ latex_base.pool.ltxml)
├── latex_dump             (≈ LoadFormat('latex'))
└── latex_constructs.rs    (≈ latex_constructs.pool.ltxml, 7800 lines, sections C.1–C.15)
```

## File-by-file mapping

### TeX primitives (Base.pool children)

| Perl file | Rust file | Notes |
|---|---|---|
| `Base_Schema.pool.ltxml` | `base_schema.rs` | |
| `Base_ParameterTypes.pool.ltxml` | `base_parameter_types.rs` | |
| `Base_Utility.pool.ltxml` | `base_utilities.rs` | |
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
| `TeX_Math.pool.ltxml` | `tex_math.rs` + `tex_scripts.rs` | Scripts split out in Rust |
| `TeX_Page.pool.ltxml` | `tex_page.rs` | |
| `TeX_Paragraph.pool.ltxml` | `tex_paragraph.rs` | |
| `TeX_Penalties.pool.ltxml` | `tex_penalties.rs` | |
| `TeX_Registers.pool.ltxml` | `tex_registers.rs` | |
| `TeX_Tables.pool.ltxml` | `tex_tables.rs` | |
| `eTeX.pool.ltxml` | `etex.rs` | |
| `pdfTeX.pool.ltxml` | `pdftex.rs` | |
| `Base_Deprecated.pool.ltxml` | *(not ported)* | TODO: low priority |

### Plain TeX format (LoadFormat 'plain')

| Perl file | Rust file | Notes |
|---|---|---|
| `plain_bootstrap.pool.ltxml` | `plain.rs` (top) | `\TeX`, `\newif`, `\leavevmode`, `\alloc@` |
| `plain_base.pool.ltxml` | `plain.rs` (middle) | Appendix B: registers, allocation, spacing, formatting |
| `plain_constructs.pool.ltxml` | `plain.rs` (middle-lower) | Accents, ligatures, non-English symbols, constructors |
| `math_common.pool.ltxml` | `plain.rs` (lower half) | Greek, operators, relations, arrows, delimiters, log functions, matrices |

All four Perl files are merged into a single `plain.rs` (1899 lines).
The `MATH_CHAR_NEGATIONS` static at the top is used by the `\not` DefRewrite.

### LaTeX format (LoadFormat 'latex')

| Perl file | Rust file | Notes |
|---|---|---|
| `latex_bootstrap.pool.ltxml` | `latex_bootstrap.rs` | Stubs for font/counter internals |
| `latex_base.pool.ltxml` | `latex_base.rs` | Infrastructure: DefMacro, Let, DefRegister, RawTeX |
| `latex_constructs.pool.ltxml` | `latex_constructs.rs` | All constructors/environments (7800 lines, sections C.1–C.15) |

### LaTeX constructs — section mapping

`latex_constructs.rs` (7800 lines) is a single file matching Perl's
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
| C.15 | Font Selection and Special Symbols | 4414–4568 |
| `latex_ch15_special_symbol.rs` | C.15 Special Symbols | 4568–4665 |
| `latex_other_in_appendices.rs` | Other / Appendices | 4666–5200 |
| `latex_semi_undocumented.rs` | Semi-documented | 5200–5366 |

### Other Perl files without Rust counterparts

| Perl file | Status |
|---|---|
| `AmSTeX.pool.ltxml` | Not ported (low priority) |
| `BibTeX.pool.ltxml` | Not ported |
| `Base_Deprecated.pool.ltxml` | Not ported (low priority) |

### Rust files without direct Perl counterparts

| Rust file | Purpose |
|---|---|
| `base_functions.rs` | Shared Rust helper functions (e.g. `reenter_text_mode`, `writable_tokens`) |
| `latex_functions.rs` | Shared LaTeX Rust helper functions (e.g. `start_appendices`) |
| `tex_scripts.rs` | Sub/superscript handling, split out from `TeX_Math.pool.ltxml` |
| `latex_hook.rs` | LaTeX hook system |
| `latex_tables_3.rs` | Empty placeholder (1 line) |

## LoadFormat gap

In Perl, `LoadFormat('plain')` and `LoadFormat('latex')` trigger
`plain_{bootstrap,base,constructs}` and `latex_{bootstrap,base,constructs}`
respectively, via the Format loading machinery in `LaTeXML::Core::State`.

This `LoadFormat` system is **not yet ported** to Rust. Currently:
- The plain format files are merged into `plain.rs` and loaded directly via `InnerPool!`.
- The latex format files (`latex_bootstrap`, `latex_base`, `latex_constructs`)
  are **not ported at all** — their content is either in `latex.rs` chapter files
  or missing entirely.
- `math_common.pool.ltxml` content is folded into `plain.rs`.
