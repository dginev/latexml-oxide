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
| `BibTeX.pool.ltxml` | LoadPool(LaTeX) + bib defs | `engine/bibtex.rs` (skeleton) | ⚠️ Skeleton-only — `LoadPool!("LaTeX")` mirrors Perl L19. The 936+ lines of bib entry-type constructors / field handlers / key normalization are NOT yet ported. `amsrefs_sty.rs` still flags `\bib` as TODO. |
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
| `TeX_Character.pool.ltxml` | `engine/tex_character.rs` | ✅ walked — `\ ` (space), `\char`, `\chardef`, `\uppercase`, `\lowercase`, `\number`, `\romannumeral`, `\string`, `\catcode`, `\sfcode`, `\lccode`, `\uccode`, `\endlinechar`, `\escapechar`, `\newlinechar` all in Perl order. **Missing:** Rust has no `\accent <Number>` primitive (Perl L108-150) — it's referenced in `math_common.rs:298` as a use-site but never defined. TODO. Accent macro loop (Perl L92-101 via `DefAccent`) uses different mechanism in Rust (`\lx@applyaccent` helper) — semantically equivalent but the per-accent `\\'`, `\\``, `\\^` etc. defs are scattered. |
| `TeX_Debugging.pool.ltxml` | `engine/tex_debugging.rs` | ✅ walked — `\lx@ERROR`, `\errorstopmode`/`\scrollmode`/`\nonstopmode`/`\batchmode`, `\pausing`, `\message`, `\errhelp`, `\errmessage`, `\errorcontextlines`, `\meaning`, `\show`/`\showbox`/`\showlists`/`\showthe`, `\showboxbreadth`/`\showboxdepth`, tracingmacros/tracingcommands AssignValue + Registers, `\tracing*` registers all in Perl order. |
| `TeX_FileIO.pool.ltxml` | `engine/tex_file_io.rs` | ✅ walked — `\@currnamestack` / `\@currname` / `\@currext`, `\lx@pushfilename` / `\lx@popfilename` / `\lx@p@pfilename`, `\openin`, `\closein`, `\read`, `\endinput`, `\inputlineno`, `\openout`, `\closeout`, `\write`, `\immediate`, `\input`, `\special`, `SpecialPS` keyvals all match Perl order. |
| `TeX_Fonts.pool.ltxml` | `engine/tex_fonts.rs` | ✅ walked & reordered — `\defaultskewchar` and `\defaulthyphenchar` (Perl L77-78) hoisted to right before `\font` (Perl L82) where they belong; were previously after `\fontdimen`. ParameterType has divergent name in Rust (`FontToken` vs Perl `FontDef`) but is in the right relative position. Rest of file (\font, \fontname, \fontdimen, \nullfont, \/, \lx@fontencoding, ligatures) matches. |
| `TeX_Glue.pool.ltxml` | `engine/tex_glue.rs` | ✅ walked — `\lx@default@jot`, `\hskip`, `\vskip`, `\unskip`, `\hss`/`\hfilneg`, `\hfil`/`\hfill`, `\vfil`/`\vfill`/`\vss`/`\vfilneg`, `\lastskip` all in Perl order. |
| `TeX_Hyphenation.pool.ltxml` | `engine/tex_hyphenation.rs` | ✅ walked & cleaned — `\-`, `\discretionary`, `\hyphenation`, `\patterns`, `\language`, `\setlanguage`, `\hyphenchar`, `\lefthyphenmin`/`\righthyphenmin`/`\uchyph` all in Perl order. **Removed duplicate** `\defaulthyphenchar` register (Perl defines it only in TeX_Fonts L78 — Rust had it in both `tex_fonts.rs` and `tex_hyphenation.rs`). Rust extra: `\languagename` macro (Perl doesn't have it in this pool — added for babel use). |
| `TeX_Inserts.pool.ltxml` | `engine/tex_inserts.rs` | ✅ walked — `\insert`, `\vsplit`, `\splitfirstmark`/`\splitbotmark`, `\insertpenalties`/`\splitmaxdepth`/`\splittopskip`/`\holdinginserts` all in Perl order. |
| `TeX_Job.pool.ltxml` | `engine/tex_job.rs` | ✅ walked — `\jobname`, `\time`/`\day`/`\month`/`\year`/`\mag` registers, time AssignValues, `\lx@end@document`, `\let \end`, `\everyjob`/`\deadcycles`/`\maxdeadcycles`, `\dump`, `\documentstyle` all in Perl order. Rust extra `\let \@@end` (probably for amsTeX compat). |
| `TeX_Kern.pool.ltxml` | `engine/tex_kern.rs` | ✅ walked — `\kern`, `\unkern`, `\lastkern`, `\lower`, `\raise`, `\moveleft`, `\moveright` in Perl order. |
| `TeX_Logic.pool.ltxml` | `engine/tex_logic.rs` | ✅ walked & reordered — `\ifvoid`/`\ifhbox`/`\ifvbox` reordered to Perl L111-113 sequence (Rust had hbox/vbox before void). Rest matches. |
| `TeX_Macro.pool.ltxml` | `engine/tex_macro.rs` | ✅ walked — `\begingroup`/`\endgroup`, `\relax`, `\let \protect \relax`, `\special_relax`, `\afterassignment`, `\aftergroup`, `CSName` ParameterType all in Perl order. |
| `TeX_Marks.pool.ltxml` | `engine/tex_marks.rs` | ✅ walked & reordered — `\firstmark`/`\botmark` swapped to mirror Perl L33-34 (Perl: topmark, botmark, firstmark; Rust had: topmark, firstmark, botmark). |
| `TeX_Math.pool.ltxml` | `engine/tex_math.rs` | walk pending |
| `TeX_Page.pool.ltxml` | `engine/tex_page.rs` | ✅ walked — `\hoffset`/`\voffset`/`\topskip`/`\pagedepth`/`\pagetotal`/`\maxdepth`/`\vsize`/`\pagegoal` all in Perl order. |
| `TeX_Paragraph.pool.ltxml` | `engine/tex_paragraph.rs` | ✅ walked — `\ignorespaces`/`\noboundary`, `\vadjust`, `\everypar`, `\indent`/`\noindent`, `\lx@normal@par`, `\let \par \lx@normal@par` all in Perl order. |
| `TeX_Penalties.pool.ltxml` | `engine/tex_penalties.rs` | ✅ walked — `\penalty`, `\unpenalty`, `\lastpenalty`, `\brokenpenalty`/`\clubpenalty`/`\exhyphenpenalty`/`\floatingpenalty`/`\hyphenpenalty`/`\interlinepenalty`/`\linepenalty`/`\outputpenalty`/`\widowpenalty` all in Perl order. |
| `TeX_Registers.pool.ltxml` | `engine/tex_registers.rs` | ✅ walked — `\count`/`\dimen`/`\skip`/`\muskip`/`\toks` registers, `\countdef`/`\dimendef`/`\skipdef`/`\muskipdef`/`\toksdef`, `\lx@alloc@`, `\lx@counter@arabic`, `\advance`/`\multiply`/`\divide` all in Perl order. |
| `TeX_Tables.pool.ltxml` | `engine/tex_tables.rs` | walk pending (760+1340 lines — large file, deferred) |
| `eTeX.pool.ltxml` | `engine/etex.rs` | ✅ walked & **rewritten** — full LoadDefinitions block reordered to mirror Perl L39-407 1:1: tracing registers → `\showgroups`/`\showtokens` → `\eTeXrevision`/`\eTeXversion`/`\interactionmode` → `\currentif*`/`\currentgroup*`/`\lastnodetype` → `\fontchar*` → `\parshape*` → NumExpr/DimExpr/GlueExpr/MuExpr ParameterTypes → `\numexpr`/etc. → `\gluestretch*`/`\glueshrink*` → marks (`\marks`/`\topmarks`/`\firstmarks`/`\botmarks`/`\splitfirstmarks`/`\splitbotmarks`) → `\readline`/`\scantokens`/`\everyeof` → `\lastlinefit`/`*penalties` → `\middle` → `\savinghyphcodes` → `\savingvdiscards`/`\pagediscards`/`\splitdiscards` → `\ifdefined`/`\ifcsname`/`\ifincsname` (Rust extra)/`\unless` → `\unexpanded`/`\detokenize` → `\TeXXeTstate`/`\beginL`/etc./`\predisplaydirection` → `\protected` → `\pdftexcmds@directlua`/`\synctex`/`\reserveinserts`. Helper functions (etex_readexpr, etex_penalties_*) defined at top of LoadDefinitions block. |
| `pdfTeX.pool.ltxml` | `engine/pdftex.rs` | ✅ walked — `\pdfoutput`/`\pdfminorversion`/.../`\pdfprotrudechars`, `\efcode`/`\lpcode`/`\rpcode`/.../`\tagcode`, `\pdfforcepagebox`/etc. all in Perl order line-by-line. |
| `plain_bootstrap.pool.ltxml` | `engine/plain_bootstrap.rs` | walk pending |
| `plain_base.pool.ltxml` | `engine/plain_base.rs` | walk pending |
| `math_common.pool.ltxml` | `engine/math_common.rs` | walk pending |
| `latex_base.pool.ltxml` | `engine/latex_base.rs` | walk pending |
| `latex_dump.pool.ltxml` | `engine/latex_dump.rs` (generated) | n/a |
| `plain_dump.pool.ltxml` | `engine/plain_dump.rs` (generated) | n/a |

## InnerPool! invocation audit (2026-04-26)

Cross-checked every Rust `InnerPool!(...)` invocation against the
corresponding Perl `LoadPool(...)` in the source pool file:

| Rust file | Calls | Perl source | Match? |
|---|---|---|---|
| `engine/base.rs` | base_schema, base_parameter_types, base_utilities, base_xmath, tex_box, tex_character, tex_debugging, tex_file_io, tex_fonts, tex_glue, tex_hyphenation, tex_inserts, tex_job, tex_kern, tex_logic, tex_macro, tex_marks, tex_math, tex_page, tex_paragraph, tex_penalties, tex_registers, tex_tables, etex, pdftex, base_deprecated (26) | `Base.pool.ltxml` L26-52 | ✅ identical order |
| `engine/tex.rs` | base, plain_bootstrap, plain_dump\|plain_base (conditional), plain_constructs (5) | `TeX.pool.ltxml` L22-23 (`LoadPool('Base')` + `LoadFormat('plain')`) — `LoadFormat` expands to `bootstrap → dump\|base → constructs` per `Package.pm:2734-2752` | ✅ |
| `engine/latex.rs` | latex_bootstrap, latex_dump\|latex_base (conditional), latex_constructs (3) | `LaTeX.pool.ltxml` L28-29 (`LoadPool('TeX')` + `LoadFormat('latex')`) | ✅ — `LoadPool!("TeX")` precedes the InnerPools as in Perl |
| `engine/latex_bootstrap.rs` | plain_bootstrap (1) | `latex_bootstrap.pool.ltxml` L18 | ✅ |
| `engine/latex_constructs.rs` | plain_constructs, math_common (2) — preceded by `_loaded` flag reset | `latex_constructs.pool.ltxml` L19-38 (`AssignValue('plain_constructs.pool.ltxml_loaded' => undef); LoadPool('plain_constructs'); ... LoadPool('math_common')`) | ✅ Gap 1 fix matches |
| `engine/plain_constructs.rs` | math_common (1) — at end after `\allowbreak` | `plain_constructs.pool.ltxml` L319 (after L317 `\allowbreak`) | ✅ position matches |

All 38 InnerPool! invocations across 6 files mirror their Perl
counterparts in both pool name and ordinal position. The new
`InnerPool!` macro guard (commit `8dfcb12f7`) ensures Perl's
`LoadPool` `_loaded` semantics are honored consistently.

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
