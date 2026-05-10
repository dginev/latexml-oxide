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
    --author   "Your Name"             \
    [--catalog path/to/foo.catalog]
```

Produces `output-dir/` with one HTML page per definition (rustdoc
style — every `\elementdef` / `\patterndef` / `\attrdef` is its own
page), plus per-module overview pages and a chapter index. Each
definition page carries a kind chip (Pattern / Element / Attribute),
permalink, pretty-printed structural content model, and (on module
overview pages) a narrative aside sourced from the `## comments` at
the head of each `.rnc`.

`--catalog FILE` is needed only for schemas whose `<include
href="..."/>` directives use `urn:` prefixes (LaTeXML's own `.rnc`
set is the example; the catalog lives at
`my-LaTeXML/lib/LaTeXML/LaTeXML.catalog`).

### Required external tools

| Tool | What it does | Where it comes from |
|---|---|---|
| `trang` | Converts RNC to RNG, preserving `## comments` as `<a:documentation>` annotations. | <https://relaxng.org/jclark/trang.html> (Java) |
| `latexml_oxide` | TeX → HTML5 with `--split --splitat=subsection`, `--schemadocs` post-pass for kind chips / content models / sidebar / narrative. | this workspace, `latexml_oxide/bin/latexml_oxide.rs` |
| `genschema_oxide` | RNG → `schema.tex` (`\schemamodule{}` blocks of `\patterndef` / `\elementdef` / `\attrdef` subsections). | this workspace, `latexml_oxide/bin/genschema_oxide.rs` |

`trang` you install once. The two oxide binaries are produced by
`cargo build`; either install them globally:

```bash
$ cargo install --path latexml_oxide --bin latexml_oxide --bin genschema_oxide
```

…or prepend the build target to `PATH` for the session:

```bash
$ export PATH="$(pwd)/target/debug:$PATH"     # debug builds for dev
$ export PATH="$(pwd)/target/release:$PATH"   # release builds when needed
```

The orchestration shell looks them up via `command -v` and bails if
either is missing.

## Layout: per-kind-bucket pages, per-def subsubsections inline

The pipeline uses a 4-level TeX hierarchy and splits at the second
level (`--splitat=subsection`):

| Level | TeX | Role |
|---|---|---|
| chapter | `\chapter{Title}` | top of site |
| section | `\schemamodule{name}` → `\section{Module …}` | one per `.rnc` submodule |
| **subsection** (split here) | `\subsection{Elements}` / `\subsection{Patterns}` | kind bucket — one page each |
| subsubsection | `\elementdef{name}…` → `\subsubsection{Element …}` | individual definition, inlined on the kind page |

That gives every schema a useful page count regardless of how the
source `.rnc` is organised: monolithic (mathml4-core, one `.rnc`)
gets the same per-kind page split as modular (LaTeXML, 12 `.rnc`s).

```
output/
├── index.html                                ← title page
└── Ch1/
    ├── index.html                            ← chapter / module list
    ├── schema.<module>/
    │   ├── index.html                        ← module overview + narrative
    │   ├── elements.html                     ← all element defs in this module
    │   └── patterns.html                     ← all pattern defs in this module
    ├── schema.<module-2>/…
    └── …
```

Module overview pages carry the narrative aside (sourced from the
`.rnc` `## comments`) and the module preamble — `Includes:
<modref>, …` and `Start symbol: <ref>` — as paragraph text.

Definition pages list every def of one kind under the parent module.
Each def is a `<section class="ltx_subsubsection schema-def">` with a
decorated heading: kind chip + schema-name + § permalink. The body is
paragraph-form (`Attributes:`, `Content:`, `Used by:`, `Expansion:`)
rather than a description list — items inside a description list
don't auto-close on a sibling section command, which would trip
LaTeXML's content-model validator on multi-def pages.

## Step-by-step (what `generate-scholarly-schema-docs` runs internally)

If you want to drive the pipeline yourself, the orchestration shell
runs roughly:

```bash
# 1. Stage RNC files in a working directory; trang resolves bare
#    `include "..."` relative to the master file's directory. Schemas
#    that use `urn:`-prefixed includes (e.g. LaTeXML's own .rnc set)
#    need an OASIS XML catalog passed to trang via -C and the matching
#    XML_CATALOG_FILES env var.
mkdir -p work/
cp schema-dir/*.rnc work/
trang work/master.rnc work/master.rng
# Or, with a catalog:
#   export XML_CATALOG_FILES=path/to/foo.catalog
#   trang -C path/to/foo.catalog work/master.rnc work/master.rng
#   # trang strips the `urn:x-LaTeXML:RelaxNG:` prefix from <include>
#   # hrefs in its output; restore it so genschema_oxide's URN resolver
#   # finds the satellite .rng files and so module names keep the
#   # `:svg:` / `:LaTeXML-` substring that SKIP_SVG keys on:
#   sed -i \
#     -e 's|include href="LaTeXML-|include href="urn:x-LaTeXML:RelaxNG:LaTeXML-|g' \
#     -e 's|include href="svg/|include href="urn:x-LaTeXML:RelaxNG:svg:|g' \
#     work/*.rng

# 2. RNG → LaTeX manual.tex (\schemamodule{}/\patterndef{}/\elementdef{}/...).
#    --module-abstract lifts the first-patterndef doc-arg of each
#    schemamodule into a top-level \moduleabstract{...} so the
#    rendered docs read it as a per-module narrative aside rather
#    than as documentation attached to one specific pattern.
#
#    By default the LaTeXML conventions (`xml`, `ltx`, `svg`, `xlink`,
#    `m`, `xhtml`) are pre-registered. For non-LaTeXML schemas whose
#    primary namespace would otherwise render as `namespace1:foo`,
#    pass `--ns prefix=URI` (repeatable). For schemas whose authors
#    declared every namespace with a prefix in the .rnc, no flag is
#    needed — trang preserves those bindings on the flattened grammar
#    and the scanner picks them up automatically.
genschema_oxide work/master.rng --module-abstract -o work/schema.tex
#   # Non-LaTeXML example:
#   genschema_oxide work/tei.rng --module-abstract --no-latexml-defaults \
#     --ns tei=http://www.tei-c.org/ns/1.0 -o work/schema.tex

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

# 4. Copy the stylesheet next to the output so the per-page
#    `<link rel="stylesheet" href="../scholarly-schema-docs.css">`
#    references resolve.
mkdir -p output
cp resources/CSS/scholarly-schema-docs.css output/scholarly-schema-docs.css

# 5. TeX → split HTML5 site, with the schemadocs post-pass on each page.
#    --splitat=subsection makes every kind-bucket subsection
#    (`\subsection{Elements}` / `\subsection{Patterns}`) its own page;
#    --splitnaming=labelrelative routes them under
#    `schema.<module>/elements.html` and `schema.<module>/patterns.html`.
#    Definition subsubsections stay inlined on those pages.
latexml_oxide --format=html5                  \
  --split --splitnaming=labelrelative         \
  --splitat=subsection                        \
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
          │   + lift_module_abstract │     \schemamodule    → \section
          │   ▶ schema.tex           │     \subsection{Elements/Patterns}
          │                          │       (kind buckets, split-points)
          │                          │     \elementdef      → \subsubsection
          │                          │     \patterndef      → \subsubsection
          │                          │     \attrdef         → paragraph
          │                          │     module preamble  → paragraph
          └──────────┬───────────────┘
                     │  schema.tex
                     ▼
          ┌──────────────────────────┐  ── Rust: latexml_oxide ──
          │       latexml_oxide      │
          ├──────────────────────────┤
          │  TeX engine              │  via latexmlman_sty.rs:
          │   (\schemamodule         │   \section{Module …}
          │    \patterndef           │   \subsubsection{Pattern …}
          │    \elementdef           │   \subsubsection{Element …}
          │    \attrdef              │   \par + bold name = type
          │    \moduleabstract …)    │   \moduleabstract → <ltx:p>
          │   ▶ ltx XML              │
          │                          │
          │  Split (--splitat=       │  one PostDocument per
          │     subsection)          │    \subsection in body
          │   ▶ Vec<PostDocument>    │    (Elements / Patterns kind
          │                          │     buckets per module)
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
          │                          │     → <aside …> on overview pages
          │   • render_content_models│   one-line operator walls
          │                          │     → multi-line block
          │   • decorate_definitions │   kind chip + permalink +
          │                          │     schema.X anchor on subsection h1
          │   • inject_sidebar_index │   cross-page sibling list per
          │                          │     module, injected into navbar
          └──────────┬───────────────┘
                     │
                     ▼
              output/  (see "Layout" above)
```

## What lives where

| Concern | Module |
|---|---|
| RelaxNG AST + scanning | `latexml_core::common::relaxng::scan` |
| AST normalization, defs/elements/uses tables | `latexml_core::common::relaxng::simplify` |
| TeX manual.tex emission | `latexml_core::common::relaxng::tex` |
| Schema-doc TeX macros (`\schemamodule`, `\elementdef`, `\patterndef`, `\moduleabstract`, …) | `latexml_contrib::latexmlman_sty` |
| Visual post-pass (kind chips, content models, sidebar, narrative lift) | `latexml_post::schema_docs` |
| CSS shipped at site | `resources/CSS/scholarly-schema-docs.css` |
| RNG → schema.tex CLI | `latexml_oxide/bin/genschema_oxide.rs` |
| Pipeline orchestration shell | `tools/generate-scholarly-schema-docs` |

## Notes for callers

- **Per-module narratives** come from `## comments` at the head of
  each `.rnc` file. trang preserves them as `<a:documentation>`;
  `genschema_oxide --module-abstract` lifts the first-patterndef
  doc-arg of each module to `\moduleabstract{…}`; the post-pass
  renders that as a left-bordered aside on the module overview page.
- **Module preamble** (`Includes: …`, `Start symbol: …`,
  `Module … included.`) is emitted as paragraph text under the
  module heading, not as description-list items. That keeps the
  module overview page valid markup whether the schema is monolithic
  or split into many `.rnc` files.
- **Stable URLs**: every definition's surrounding `<section>` carries
  an `id="schema.<name>"` (set via `\label{schema.<name>}` in each
  def macro). With `--splitnaming=labelrelative`, that label also
  determines the URL path —
  `schema.<module>/schema.<def>/index.html`.
- **`\patternadd`** uses a synthetic `schema.add.<name>` label so its
  page lives at a distinct URL from the canonical `schema.<name>`
  definition; the rendered subsection links back to the canonical
  definition via `\hyperref`.
