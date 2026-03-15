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
└── latex_ch1_* .. latex_ch15_*, latex_other_*, latex_semi_*  (Lamport chapters)
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
| `latex_bootstrap.pool.ltxml` | *(not ported)* | `LoadFormat` machinery not yet in Rust |
| `latex_base.pool.ltxml` | *(not ported)* | `LoadFormat` machinery not yet in Rust |
| `latex_constructs.pool.ltxml` | *(not ported)* | `LoadFormat` machinery not yet in Rust |

### LaTeX chapters (inline in LaTeX.pool in Perl, split in Rust)

The Perl `LaTeX.pool.ltxml` is one large file (~5400 lines). In Rust it is
split by Lamport chapter into individual files, loaded from `latex.rs`.

| Rust file | Lamport chapter | Perl lines (approx) |
|---|---|---|
| `latex_ch1_documentclass.rs` | C.1.1 Document Class | 31–110 |
| `latex_ch1_environments.rs` | C.1.3 Environments | 110–180 |
| `latex_ch1_fragile_commands.rs` | C.1.4 Fragile Commands | 180–250 |
| `latex_ch1_break_command.rs` | C.1.5 \\ Command | 251–276 |
| `latex_ch2_document.rs` | C.2 The Document | 276–372 |
| `latex_ch3_sentences_and_paragraphs.rs` | C.3 Sentences and Paragraphs | 372–587 |
| `latex_ch4_sectioning_and_toc.rs` | C.4 Sectioning / ToC | 588–833 |
| `latex_ch5_packages.rs` | C.5.1 Packages | 833–1080 |
| `latex_ch5_page_styles.rs` | C.5.2 Page Styles | 1080–1140 |
| `latex_ch5_title_page_and_abstract.rs` | C.5.3 Title Page | 1102–1310 |
| `latex_ch6_displayed_paragraphs.rs` | C.6 Displayed Paragraphs | 1311–1376 |
| `latex_ch6_quotations_and_verse.rs` | C.6 Quotations and Verse | 1377–1395 |
| `latex_ch6_list_making_environments.rs` | C.6 List Environments | — |
| `latex_ch6_list_and_trivlist_environments.rs` | C.6 Lists and Trivlists | 1396–1550 |
| `latex_ch6_verbatim.rs` | C.6 Verbatim | 1551–1646 |
| `latex_ch7_math_mode_environments.rs` | C.7 Math Environments | 1646–2164 |
| `latex_ch7_math_common_structures.rs` | C.7 Math Structures | 2164–2180 |
| `latex_ch7_math_common_delimiters.rs` | C.7 Math Delimiters | 2180–2216 |
| `latex_ch7_math_mode_changing_style.rs` | C.7 Math Style Changes | 2216–2246 |
| `latex_ch8_defining_commands.rs` | C.8 Defining Commands | 2247–2511 |
| `latex_ch8_defining_environments.rs` | C.8 Defining Environments | 2512–2536 |
| `latex_ch8_theoremlike_environments.rs` | C.8 Theorem-like Envs | 2536–2712 |
| `latex_ch8_numbering.rs` | C.8 Numbering | 2712–2785 |
| `latex_ch9_figures_and_tables.rs` | C.9 Figures and Tables | 2786–2975 |
| `latex_ch9_marginal_notes.rs` | C.9 Marginal Notes | 2975–2985 |
| `latex_ch10_tabbing_environment.rs` | C.10 Tabbing | 2985–3086 |
| `latex_ch10_array_and_tabular.rs` | C.10 Array and Tabular | 3086–3229 |
| `latex_ch11_moving_information.rs` | C.11 Moving Information | 3230–3567 |
| `latex_ch11_splitting_the_input.rs` | C.11 Splitting Input | 3568–3626 |
| `latex_ch11_index_and_glossary.rs` | C.11 Index and Glossary | 3627–3821 |
| `latex_ch11_terminal_io.rs` | C.11 Terminal I/O | 3821–3832 |
| `latex_ch12_line_and_page_breaking.rs` | C.12 Line/Page Breaking | 3832–3866 |
| `latex_ch13_boxes.rs` | C.13 Boxes | 3866–4123 |
| `latex_ch14_pictures_and_color.rs` | C.14 Pictures and Color | 4124–4414 |
| `latex_ch15_font_selection.rs` | C.15 Font Selection | 4414–4568 |
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
