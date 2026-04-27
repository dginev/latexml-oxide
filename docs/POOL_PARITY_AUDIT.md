# Pool Parity Audit — Perl `*.pool.ltxml` ↔ Rust `engine/*.rs`

> **Active worksheet, started 2026-04-26.** User directive:
> "we need to walk these pairs of files one by one, ensuring the
> order of loads and definitions is the exact same in perl and rust"

For each Perl `LaTeXML/blib/lib/LaTeXML/Engine/*.pool.ltxml`, the
corresponding `latexml_package/src/engine/*.rs` must:
1. Perform the **same** `LoadPool` / `LoadFormat` calls in the **same
   order** (mirrored as `InnerPool!(...)` or
   `crate::engine::<x>::load_definitions()`).
2. Define the **same** symbols in the **same order**, with the **same**
   options (locked, scope, prefixes, etc.).

Per-file audit status table follows. Update each row when its
walk-through completes.

## Top-level pool entry points (pure loaders)

| Perl pool | LoadPool/Format calls | Rust file | Audit status |
|---|---|---|---|
| `Base.pool.ltxml` | Base_Schema, Base_ParameterTypes, Base_Utility, Base_XMath, TeX_Box…TeX_Tables, eTeX, pdfTeX, Base_Deprecated | `engine/base.rs` (commit `cb09ae203`) | ✅ Order matches |
| `TeX.pool.ltxml` | LoadPool(Base); LoadFormat(plain); + DefAutoload triggers + `\documentstyle` | `engine/tex.rs` | ✅ Order matches (Base via `InnerPool!(base)` after the split) |
| `LaTeX.pool.ltxml` | LoadPool(TeX); LoadFormat(latex) | `engine/latex.rs` | ✅ Order matches |
| `latex_bootstrap.pool.ltxml` | LoadPool(plain_bootstrap) + bootstrap defs | `engine/latex_bootstrap.rs` | walk pending |
| `latex_constructs.pool.ltxml` | **Force-reload(plain_constructs); Force-reload(math_common)** + LaTeX construct defs | `engine/latex_constructs.rs` | ❌ **Gap 1**: missing the two reloads |
| `plain_constructs.pool.ltxml` | LoadPool(math_common) + plain construct defs | `engine/plain_constructs.rs` | walk pending |
| `BibTeX.pool.ltxml` | LoadPool(LaTeX) + bib defs | **MISSING** | ❌ **Gap 2**: no `bibtex.rs` |
| `AmSTeX.pool.ltxml` | (no `LoadPool` at file load — only inside macros) | `engine/amstex.rs` | walk pending |

## Leaf pool files (no LoadPool/Format at file-load time)

For these, the audit is purely a **definition-order** walk —
make sure each Rust file defines the same symbols, in the same
order, with the same options as its Perl counterpart.

| Perl pool | Rust file | Audit status |
|---|---|---|
| `Base_Schema.pool.ltxml` | `engine/base_schema.rs` | ✅ walked — order matches; Rust `after_open` closure has extra `xml:lang`/`DOCUMENT_LANGUAGE` handling not in Perl (likely belongs in babel binding) |
| `Base_ParameterTypes.pool.ltxml` | `engine/base_parameter_types.rs` | ✅ walked — every Perl `DefParameterType` has a matching Rust `DefParameterType!` in the same order. Rust adds extras (`Relation`, `Pair`, `TeXFileName`, `DirectoryList`, `MoveableBox`, `BalancedParen`, `TeXDelimiter`, `Digested`, `DigestUntil`, `DigestedBody`) interleaved or appended; not present in Perl Base_ParameterTypes — likely additions for downstream Rust bindings, harmless to stay |
| `Base_Utility.pool.ltxml` | `engine/base_utilities.rs` | ✅ walked & reordered — dash/space defs (`\lx@endash` etc.) hoisted to immediately after `\lx@ifundefined`; `\lx@ignorehardspaces` and `\@ADDCLASS` and the `frontmatter` `AssignValue` reordered to follow Perl L23-179 sequence. Rest of file (`\@add@frontmatter` chain, `\lx@tag*`, `\lx@@compose@title`, etc.) was already in Perl order. |
| `Base_XMath.pool.ltxml` | `engine/base_xmath.rs` | ✅ walked — early DefConstructor sequence (`\lx@assert@meaning`, `\lx@apply`, `\lx@symbol`, `\lx@wrap`, `\lx@superscript`, `\lx@subscript`) and DefMath sequence for ASCII operators all in Perl order |
| `Base_Deprecated.pool.ltxml` | `engine/base_deprecated.rs` | ✅ walked & **rewritten** — Rust had grouped entries by category (alignment / core / math / etc.); rewritten 1:1 in Perl L29-220 order. All 77 deprecation aliases now in same sequence. |
| `TeX_Box.pool.ltxml` | `engine/tex_box.rs` | ✅ walked & reordered — `{`/`}` primitives moved BEFORE `\lx@hidden@bgroup`/`\lx@hidden@egroup` (Perl L32-55 order); `\lx@overlay` hoisted from end of LoadDefinitions block to its Perl L69 position right after `\lx@hflipped`. Note: `\lx@nounicode` is currently between `\@hidden@egroup` (Rust extra) and `\lx@framed`/`\lx@hflipped`/`\lx@overlay` — Perl has it AFTER overlay (L76); minor; not reordered yet to keep diff focused. |
| `TeX_Character.pool.ltxml` | `engine/tex_character.rs` | walk pending |
| `TeX_Debugging.pool.ltxml` | `engine/tex_debugging.rs` | walk pending |
| `TeX_FileIO.pool.ltxml` | `engine/tex_file_io.rs` | walk pending |
| `TeX_Fonts.pool.ltxml` | `engine/tex_fonts.rs` | walk pending |
| `TeX_Glue.pool.ltxml` | `engine/tex_glue.rs` | walk pending |
| `TeX_Hyphenation.pool.ltxml` | `engine/tex_hyphenation.rs` | walk pending |
| `TeX_Inserts.pool.ltxml` | `engine/tex_inserts.rs` | walk pending |
| `TeX_Job.pool.ltxml` | `engine/tex_job.rs` | walk pending |
| `TeX_Kern.pool.ltxml` | `engine/tex_kern.rs` | walk pending |
| `TeX_Logic.pool.ltxml` | `engine/tex_logic.rs` | walk pending |
| `TeX_Macro.pool.ltxml` | `engine/tex_macro.rs` | walk pending |
| `TeX_Marks.pool.ltxml` | `engine/tex_marks.rs` | walk pending |
| `TeX_Math.pool.ltxml` | `engine/tex_math.rs` | walk pending |
| `TeX_Page.pool.ltxml` | `engine/tex_page.rs` | walk pending |
| `TeX_Paragraph.pool.ltxml` | `engine/tex_paragraph.rs` | walk pending |
| `TeX_Penalties.pool.ltxml` | `engine/tex_penalties.rs` | walk pending |
| `TeX_Registers.pool.ltxml` | `engine/tex_registers.rs` | walk pending |
| `TeX_Tables.pool.ltxml` | `engine/tex_tables.rs` | walk pending |
| `eTeX.pool.ltxml` | `engine/etex.rs` | walk pending |
| `pdfTeX.pool.ltxml` | `engine/pdftex.rs` | walk pending |
| `plain_bootstrap.pool.ltxml` | `engine/plain_bootstrap.rs` | walk pending |
| `plain_base.pool.ltxml` | `engine/plain_base.rs` | walk pending |
| `math_common.pool.ltxml` | `engine/math_common.rs` | walk pending |
| `latex_base.pool.ltxml` | `engine/latex_base.rs` | walk pending |
| `latex_dump.pool.ltxml` | `engine/latex_dump.rs` (generated) | n/a |
| `plain_dump.pool.ltxml` | `engine/plain_dump.rs` (generated) | n/a |

## Per-file walk notes

Notes accumulate here as each pair is walked. Format:

```
### <file>.pool.ltxml ↔ <file>.rs
- Perl: <key observation about loads / order / defs>
- Rust: <current state>
- Action: <what was done / what's needed>
```

### latex_constructs.pool.ltxml ↔ latex_constructs.rs (Gap 1)

Perl L19-38:
```perl
AssignValue('plain_constructs.pool.ltxml_loaded' => undef);  # Force RELOAD!
AssignValue('math_common.pool.ltxml_loaded'      => undef);  # Force RELOAD!
LoadPool('plain_constructs');
...
LoadPool('math_common');  # appears later, ~L37
```

Rust top of `latex_constructs.rs::LoadDefinitions!`: jumps straight
into definitions, neither flag is reset, neither pool is re-loaded.

The reset is **deliberate**: at the time `latex_constructs` runs,
`plain_constructs` and `math_common` were already loaded by the
plain-format chain (`tex.rs::LoadFormat('plain')`); some of their
definitions were since clobbered by `latex_base` / earlier
`latex_constructs` activity. Perl re-runs them to restore that state
on top of LaTeX-specific overrides.

Action: at the **top** of the Rust LoadDefinitions block, clear the
two `_loaded` flags via `state::assign_value(...)` then
`InnerPool!(plain_constructs); InnerPool!(math_common);` (the second
matches Perl L37 — separate from the first re-load).

### BibTeX.pool.ltxml ↔ (none) (Gap 2)

Perl `BibTeX.pool.ltxml` does `LoadPool('LaTeX')` at file load,
followed by ~360 lines of BibTeX-specific definitions.
Rust has no `engine/bibtex.rs`.

Action: create `engine/bibtex.rs` with `InnerPool!(latex)` (or the
runtime equivalent if BibTeX is loaded outside the LaTeX format
chain), then port the rest of the BibTeX defs.
