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

Produces `output-dir/` with one HTML page per source `.rnc` module,
plus an `index.html` cover page. Each page carries the module's
narrative aside (sourced from `## comments` at the head of the
`.rnc`), all of its `\patterndef` / `\elementdef` / `\attrdef` items
as decorated description-list cards, and a per-page sidebar index
grouped by kind.

`--catalog FILE` is needed only for schemas whose `<include
href="..."/>` directives use `urn:` prefixes (LaTeXML's own `.rnc`
set is the example; the catalog lives at
`my-LaTeXML/lib/LaTeXML/LaTeXML.catalog`).

### Required external tools

| Tool | What it does | Where it comes from |
|---|---|---|
| `trang` | RNC → RNG, preserving `## comments` as `<a:documentation>`. | <https://relaxng.org/jclark/trang.html> |
| `latexml_oxide` | TeX → HTML5 with `--split --splitat=section` and the `--schemadocs` post-pass. | this workspace, `latexml_oxide/bin/latexml_oxide.rs` |
| `genschema_oxide` | RNG → `schema.tex` (`\schemamodule{}` blocks of `\patterndef` / `\elementdef` / `\attrdef`). | this workspace, `latexml_oxide/bin/genschema_oxide.rs` |

`trang` you install once. The two oxide binaries are produced by
`cargo build`; either install them globally or prepend
`target/{debug,release}` to `PATH` for the session.

## Page layout

`--splitat=section` (one page per `\section{Module …}`), with each
def expressed as a `<dt class="schema-def">` / `<dd class="ltx_item">`
pair inside a single `<dl class="ltx_description">`. No per-kind-bucket
subsections — patterns and elements interleave in source order so
cross-refs between siblings stay on one page.

```
output/
├── index.html                              ← title page + chapter list
├── schema.<module-1>.html                  ← all defs in module 1
├── schema.<module-2>.html
└── …
```

Each definition card carries:
- a kind chip (`Pattern` / `Element` / `Attribute` / `Add to`),
- the schema name + `§` permalink,
- the doc-arg as the lead paragraph,
- description rows (`Attributes:`, `Content:`, `Used by:`, …),
- an `id="schema.<name>"` so cross-refs from any page resolve.

A per-module sidebar index (kind-grouped, alphabetised) sits at the
top of the navbar; long pages get a JS-driven filter input.

## Rendering decisions

These shape what the reader sees inside a card. Each is a deliberate
trade-off between fidelity to the source RNC and visual clarity.

| Source shape | Rendered as | Why |
|---|---|---|
| `attribute foo {text}?` (and similar `xsd:string` / `xsd:integer` / …) | `Text attributes: a, b, c, …` (one line per type, names sorted, monospaced) | A long run of identical `ATTRIBUTE foo / = text` rows compresses 90+ rows into one line. Non-trivial bodies (enums, pattern refs, attached docs) stay as individual `\attrdef` cards. |
| `element (*) { … }` / `attribute (*) { … }` (`<anyName/>`) | `element *:* { … }` inline, single occurrence | Wildcards aren't real names, so they get text-shape rendering instead of a nested `\elementdef` card. The `*` / `*:*` pair the scanner emits collapses to one. |
| `X = element a {…} \| element b {…} \| …` (Choice/Group/Interleave of named elements) | Pattern Content: alphabetised `(a \| b \| c \| …)` of `\elementref` links + sibling `\elementdef` cards (one per unique name) | Embedding `\elementdef{…}` cards inside another card's body produced orphan `(`, `\|`, `)` text fragments because LaTeXML promotes `\item` macros out of paragraphs. The link-list keeps the structure visible; per-name siblings carry the actual content. |
| Singleton `X = element Y {B}` with leading `## doc` (which blocks the simplify shortcut) | Same as above — Pattern body links to `Y`, sibling Element card carries `B` | Without this, the empty `<dd>` under "Content:" was the most common artifact. |
| Any other pattern body containing nested `Pattern::Element` (mixed with refs, text, etc.) | Each Element renders inline as `\elementref{NAME}`; sibling `\elementdef` extracted | Same `\item`-promotion problem applied to mixed Choices like `(text & (element a {…} \| ref \| element b {…})*)`. Inline links are safe text. |
| Cross-ref href (`\elementref{xhtml:foo}` → `#schema.xhtml..foo`) | Decorator-side `id="schema.xhtml..foo"` on the matching `<dt>` | LaTeXML's `\cleanhypername` rewrites `:` → `..` in fragment ids; the post-pass mirrors that substitution so the dt-id and href agree. Nested `<dt>`s (e.g. when a pattern wraps a single element) are also promoted, with a per-page `seen_ids` HashSet preventing duplicate-id collisions. |

The artifact hunter at `tests/audit/hunt_artifacts.py` (and the
companion `audit_links.py`) currently report **0 broken links and 0
visible artifacts** across the three reference sites (mathml-core,
scholarly-ltx, LaTeXML).

## Step-by-step (what `generate-scholarly-schema-docs` runs internally)

```bash
# 1. Stage RNC files; trang resolves `include "…"` relative to the
#    master file's directory. URN-prefixed includes need an OASIS
#    XML catalog passed via -C.
mkdir -p work/
cp schema-dir/*.rnc work/
trang work/master.rnc work/master.rng
# Or, with a catalog:
#   export XML_CATALOG_FILES=path/to/foo.catalog
#   trang -C path/to/foo.catalog work/master.rnc work/master.rng
#   # trang strips `urn:x-LaTeXML:RelaxNG:` from <include> hrefs;
#   # restore so genschema_oxide's URN resolver finds satellite .rngs
#   # and module names retain the `:svg:` substring SKIP_SVG keys on:
#   sed -i \
#     -e 's|include href="LaTeXML-|include href="urn:x-LaTeXML:RelaxNG:LaTeXML-|g' \
#     -e 's|include href="svg/|include href="urn:x-LaTeXML:RelaxNG:svg:|g' \
#     work/*.rng

# 2. RNG → LaTeX manual.tex.
#    --module-abstract lifts the first-patterndef doc-arg of each
#    schemamodule into a top-level \moduleabstract so it renders as
#    the module's narrative aside. LaTeXML namespace conventions
#    (xml/ltx/svg/xlink/m/xhtml) are pre-registered; for non-LaTeXML
#    schemas pass `--no-latexml-defaults` plus `--ns prefix=URI`
#    (repeatable) so the primary namespace doesn't fall back to
#    `namespace1:foo`.
genschema_oxide work/master.rng --module-abstract -o work/schema.tex

# 3. Wrap in a small driver document.
cat > work/schema-doc.tex <<'TEX'
\documentclass{article}
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
\input{schema}
\end{document}
TEX

# 4. Stage the stylesheet AND its sister runtime script so per-page
#    <link>/<script> references resolve. Both files share a basename
#    (the `<script src>` is injected by the schemadocs post-pass).
mkdir -p output
cp resources/CSS/relaxng-schema-rustdoc-theme.css \
   output/relaxng-schema-rustdoc-theme.css
cp resources/javascript/relaxng-schema-rustdoc-theme.js \
   output/relaxng-schema-rustdoc-theme.js

# 5. TeX → split HTML5 site, with the schemadocs post-pass on each page.
latexml_oxide --format=html5                  \
  --split --splitnaming=labelrelative         \
  --splitat=section                           \
  --navigationtoc=context                     \
  --schemadocs                                \
  --sourcedirectory=work                      \
  --dest=output/index.html                    \
  --nodefaultresources                        \
  --css=relaxng-schema-rustdoc-theme.css      \
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
          │  simplify::simplify_top  │   resolves Ref/ParentRef qnames,
          │                          │     records uses_name graph,
          │                          │     collects defs/elementdefs
          │  tex::document_modules   │   walks AST + state, emits:
          │   + lift_module_abstract │     \schemamodule  → \section
          │                          │     \patterndef    → \item
          │                          │     \elementdef    → \item
          │                          │     \attrdef       → \item (or grouped)
          │                          │     \moduleabstract
          │                          │   detect_element_choice +
          │                          │     collect_element_descendants:
          │                          │       extract nested elements as
          │                          │       sibling cards, keep parent
          │                          │       body as \elementref links
          └──────────┬───────────────┘
                     │  schema.tex
                     ▼
          ┌──────────────────────────┐  ── Rust: latexml_oxide ──
          │       latexml_oxide      │
          ├──────────────────────────┤
          │  TeX engine              │  via latexmlman_sty.rs:
          │   (\schemamodule         │   \section{Module …}
          │    \patterndef …)        │   \item[Pattern …] in description
          │                          │
          │  Split (--splitat=       │  one PostDocument per
          │     section)             │    \section in body
          │                          │
          │  Scan + ObjectDB         │  per-doc Pattern entries
          │  CrossRef                │  prev/next/up/start refs
          │  Graphics, MathML, XSLT  │  ltx XML → HTML5
          │                          │
          │  schema_docs post-pass   │  per-page string transforms:
          │   • lift_module_narrative│   <p schema_module_narrative>
          │                          │     → <aside …>
          │   • render_content_models│   one-line operator walls
          │                          │     → multi-line block
          │   • decorate_definitions │   kind chip + permalink +
          │                          │     id="schema.X" on <dt>
          │                          │     (clean_anchor_name applies
          │                          │     `:` → `..`; seen_ids HashSet
          │                          │     prevents duplicate ids when
          │                          │     a name appears nested in
          │                          │     multiple parent patterns)
          │   • inject_sidebar_index │   per-kind list per module,
          │                          │     injected into navbar
          │   • inject_filter_script │   sticky filter input on long
          │                          │     pages
          └──────────┬───────────────┘
                     │
                     ▼
              output/  (see "Page layout" above)
```

## What lives where

| Concern | Module |
|---|---|
| RelaxNG AST + scanning | `latexml_core::common::relaxng::scan` |
| AST normalization, defs/elements/uses tables | `latexml_core::common::relaxng::simplify` |
| TeX manual.tex emission (incl. attribute grouping, wildcard inline, element-choice extraction) | `latexml_core::common::relaxng::tex` |
| Schema-doc TeX macros (`\schemamodule`, `\elementdef`, `\patterndef`, `\moduleabstract`, …) | `latexml_contrib::latexmlman_sty` |
| Visual post-pass (kind chips, content models, sidebar, narrative lift, anchor-id cleanup) | `latexml_post::schema_docs` |
| CSS shipped at site (rustdoc-styled theme, reusable) | `resources/CSS/relaxng-schema-rustdoc-theme.css` |
| JS shipped at site (theme-switcher wiring + in-page filter) | `resources/javascript/relaxng-schema-rustdoc-theme.js` |
| RNG → schema.tex CLI | `latexml_oxide/bin/genschema_oxide.rs` |
| Pipeline orchestration shell | `tools/generate-scholarly-schema-docs` |

## Notes for callers

- **Per-module narratives** come from `## comments` at the head of
  each `.rnc` file. trang preserves them as `<a:documentation>`;
  `genschema_oxide --module-abstract` lifts the first-patterndef
  doc-arg of each module to `\moduleabstract{…}`; the post-pass
  renders that as a left-bordered aside above the section heading.
- **Module preamble** (`Includes: …`, `Start symbol: …`) is emitted
  as paragraph text under the module heading.
- **Stable URLs**: every definition's `<dt>` carries
  `id="schema.<cleaned-name>"` where `clean_anchor_name` rewrites
  `:` → `..` to match LaTeXML's `\cleanhypername`. With
  `--splitnaming=labelrelative`, the per-page filename is
  `schema.<module>.html`, so a full URL is e.g.
  `schema.scholarly-ltx-scaffold.html#schema.xhtml..header`.
- **Patternadds** (`\patternadd{name}{…}`) get a synthetic
  `schema.add.<name>` id so they don't collide with the canonical
  `schema.<name>` def.
- **Cross-page linking** uses LaTeXML's CrossRef pass; `\elementref`
  / `\patternref` resolve to the page where the referenced def
  lives, even if the link is in a different module's page.
