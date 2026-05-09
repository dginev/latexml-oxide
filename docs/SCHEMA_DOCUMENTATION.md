# Schema Documentation Pipeline

How to turn a RelaxNG Compact (`.rnc`) schema into a rustdoc-styled
HTML5 documentation site, end-to-end. The pipeline runs entirely
through latexml-oxide; the only external dep is `trang` for the
RNC → RNG conversion.

## One-shot generation

```bash
$ tools/generate-scholarly-schema-docs \
    --schema   path/to/schema.rnc      \
    --output   path/to/output-dir      \
    --title    "My Schema Reference"   \
    --author   "Your Name"
```

That single invocation produces a complete `output-dir/` containing
`index.html`, `Ch1/index.html`, and one `Ch1/<module>.html` per
schema module — with rustdoc-style sidebar navigation, kind chips on
every Pattern / Element / Attribute, pretty-printed structural content
models, and a per-module narrative aside (sourced from the `##
comments` at the head of each `.rnc` file via trang).

### Required external tools

| Tool | What it does | Where it comes from |
|---|---|---|
| `trang` | Converts RNC to RNG, preserving `## comments` as `<a:documentation>` annotations. | <https://relaxng.org/jclark/trang.html> (Java) |
| `latexml_oxide` | TeX → HTML5 with `--split --splitat=section`, `--schemadocs` post-pass for kind chips / content models / sidebar / narrative. | this workspace, `latexml_oxide/bin/latexml_oxide.rs` |
| `genschema_oxide` | RNG → `schema.tex` (`\schemamodule{}` blocks of `\patterndef` / `\elementdef` / `\attrdef`). | this workspace, `latexml_oxide/bin/genschema_oxide.rs` |

Both `latexml_oxide` and `genschema_oxide` come out of `cargo build`;
`trang` you install once.

## Step-by-step (what `generate-scholarly-schema-docs` runs internally)

If you want to drive the pipeline yourself, the orchestration shell
runs roughly:

```bash
# 1. Stage RNC files in a working directory; trang resolves
#    `include "..."` relative to the master file's directory.
mkdir -p work/
cp schema-dir/*.rnc work/
trang work/master.rnc work/master.rng

# 2. RNG → LaTeX manual.tex (\schemamodule{}/\patterndef{}/\elementdef{}/...).
#    --module-abstract lifts the first-patterndef doc-arg of each
#    schemamodule into a top-level \moduleabstract{...} so the
#    rendered docs read it as a per-module narrative aside rather
#    than as documentation attached to one specific pattern.
genschema_oxide work/master.rng --module-abstract -o work/schema.tex

# 3. Wrap the schema.tex in a small driver document with title/author
#    and `\input{schema}`.
cat > work/schema-doc.tex <<TEX
\documentclass{book}
\usepackage{latexml}
\usepackage{latexmlman}
\usepackage{makeidx}
\makeindex
\title{My Schema Reference}
\author{Your Name}
\date{\today}
\begin{document}
\maketitle
\tableofcontents
\chapter{My Schema Reference}
\input{schema}
\end{document}
TEX

# 4. TeX → split HTML5 site, with the schemadocs post-pass on each page.
latexml_oxide --format=html5                  \
  --split --splitnaming=labelrelative         \
  --splitat=section                           \
  --navigationtoc=context                     \
  --schemadocs                                \
  --sourcedirectory=work                      \
  --dest=output/index.html                    \
  --nodefaultresources                        \
  --css=scholarly-schema-docs.css             \
  work/schema-doc.tex
```

## Internal pipeline diagram

```
   .rnc files (one per module, header `## comments`)
                  │
                  ▼
          ┌──────────────┐         ── external Java ──
          │    trang     │
          └──────┬───────┘
                 │  flattened .rng
                 │  with <a:documentation>
                 ▼
          ┌──────────────────────────┐  ── Rust: latexml_core::common::relaxng ──
          │     genschema_oxide      │
          ├──────────────────────────┤
          │  scan::scan_external     │   walks RNG XML via libxml
          │   ▶ Pattern AST          │
          │  simplify::simplify_top  │   resolves Ref/ParentRef qnames,
          │   ▶ populated Relaxng    │     records uses_name graph,
          │     state                │     collects defs / elementdefs,
          │                          │     orders modules
          │  tex::document_modules   │   walks AST + state, emits
          │   + lift_module_abstract │     \schemamodule / \patterndef /
          │   ▶ schema.tex           │     \elementdef / \attrdef /
          │                          │     \moduleabstract / refs
          └──────────┬───────────────┘
                     │  schema.tex
                     ▼
          ┌──────────────────────────┐  ── Rust: latexml_oxide ──
          │       latexml_oxide      │
          ├──────────────────────────┤
          │  TeX engine              │  via latexmlman_sty.rs:
          │   (\schemamodule         │   \section{Module …},
          │    \patterndef           │   description list,
          │    \elementdef           │   \hypertarget{schema.X},
          │    \attrdef              │   \moduleabstract → <ltx:p>
          │    \moduleabstract …)    │
          │   ▶ ltx XML              │
          │                          │
          │  Split (--splitat=       │  one PostDocument per
          │     section)             │    \section in body
          │   ▶ Vec<PostDocument>    │
          │                          │
          │  Scan + ObjectDB         │  per-doc Pattern entries:
          │                          │    location, pageid, …
          │  MakeBibliography        │
          │  CrossRef                │  prev/next/up/start refs,
          │                          │   global TOC under <ltx:nav>
          │  Graphics, MathML        │
          │  XSLT (LaTeXML-html5)    │  ltx XML → HTML5
          │                          │
          │  schema_docs post-pass   │  per-page string transforms:
          │   • lift_module_narrative│   <p class=schema_module_narrative>
          │                          │     → <aside …>
          │   • render_content_models│   one-line operator walls
          │                          │     → multi-line block
          │   • decorate_definitions │   kind chip + permalink +
          │                          │     promote schema.X anchor onto <dt>
          │   • inject_sidebar_index │   per-module item index in nav
          └──────────┬───────────────┘
                     │
                     ▼
              output/
                index.html                                ← chapter root
                Ch1/index.html                            ← module list
                Ch1/<module>.html        (one per module) ← definitions
                scholarly-schema-docs.css                 ← stylesheet
```

## What lives where

| Concern | Module |
|---|---|
| RelaxNG AST + scanning | `latexml_core::common::relaxng::scan` |
| AST normalization, defs/elements/uses tables | `latexml_core::common::relaxng::simplify` |
| TeX manual.tex emission | `latexml_core::common::relaxng::tex` |
| Schema-doc TeX macros (`\schemamodule`, `\patterndef`, `\moduleabstract`, …) | `latexml_contrib::latexmlman_sty` |
| Visual post-pass (kind chips, content models, sidebar, narrative lift) | `latexml_post::schema_docs` |
| CSS shipped at site | `resources/CSS/scholarly-schema-docs.css` |
| RNG → schema.tex CLI | `latexml_oxide/bin/genschema_oxide.rs` |
| Pipeline orchestration shell | `tools/generate-scholarly-schema-docs` |

## Notes for callers

- **Per-module narratives** flow from `## comments` at the head of
  each `.rnc` file (no separate metadata file). trang preserves them
  as `<a:documentation>`; `genschema_oxide --module-abstract` lifts
  them into a `\moduleabstract{}` macro at module level; the visual
  post-pass renders them as a left-bordered aside under each module's
  section heading.
- **`--splitat=section`** treats every `\begin{schemamodule}{...}` as
  its own page (because `\schemamodule` expands to `\section{…}`).
  Each page gets its own sidebar item index plus the global module
  navigation.
- **Stable URLs**: each definition's `<dt>` carries an
  `id="schema.<name>"` anchor (rather than the auto-generated
  `I1.ix1`-shape). Cross-references resolve to those.
- **Tests**: `latexml_core::common::relaxng` has 32 unit tests +
  3 integration tests against `LaTeXML.rng` (SKIP-on-missing). The
  validator's `scholarly-ltx.rng` is exercised in the validator
  repo against `latexml_core` as a dependency.
