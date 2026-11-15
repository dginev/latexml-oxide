# Schema extensions: our RelaxNG against Perl's

Every difference between our schema (`latexml_core/resources/RelaxNG/`, embedded in the binary) and Perl's
(`LaTeXML/lib/LaTeXML/resources/RelaxNG/` in the main checkout, upstream master through `0db6f47e`), each with the rule
an upstream patch would state. This is the basis of an upstream proposal (Bruce Miller Q&A, QA19: Q2.1, Q7.3). Bruce
found the permissive-nesting changes fine. A new attribute or element needs a proposal.

**Basis (2026-11-15).** A line diff of every `.rnc` (7 of the 12 `LaTeXML*.rnc` differ; `svg/*.rnc` are identical).
Then a semantic diff of the two compiled `LaTeXML.model` files: for each element, its attributes and its children as
sets. The compiled model is what the document builder consults (`latexml_core/src/common/model.rs`, and
`latexml_codegen/src/modelable.rs` at compile time). It shows exactly the changes below and nothing more. So no other
OD entry changes the schema. Every change widens the schema: nothing is removed, and no element is added. A change
edits all three forms, `.rnc` → `.rng` → `LaTeXML.model` (`tools/compileschema.sh`).

**Counts.** There are 12 differences:

| Kind | Count | Rows |
|---|---|---|
| Permissive nesting | 6 | S1–S6 |
| Changed model (a parent reference replaced by a wider one) | 1 | S7 |
| New attribute: an existing attribute group on more elements | 2 | S8, S9 |
| New attribute: a new attribute name | 3 groups, 7 names | S10–S12 |
| New element | 0 | — |

## The differences

| # | Rule (our `.rnc`) | Perl | Ours | Kind | Why (doc) | Emitted by | Generalization: the upstream rule |
|---|---|---|---|---|---|---|---|
| S1 | `quote_model` (`LaTeXML-block.rnc:141`) | `Block.model` | `+ para, logical-block, sectional-block` | nesting | OD #271 | a minipage in a quote: a `logical-block` around `\tableofcontents` (webquiz), a `sectional-block` around `\section*` (aguplus) | Ours adds exactly `para`, `logical-block` (two `Para.class` members; `pagination` and `rule` were already in) and `sectional-block`. The upstream rule: a quote admits `Para.class` minus the floats (figure, table, float, which float to the page), plus `sectional-block`. That rule is wider than our schema: it also admits `theorem`, `proof` and `TOC`. Perl's own comment asks for it (Perl `LaTeXML-block.rnc:143`: "should be Block.model, perhaps even Para.model?"). |
| S2 | `figure_model`, `table_model`, `float_model` (`LaTeXML-para.rnc:157,180,205`) | `(tags? \| figure \| table \| float \| Block.model \| Caption.class)*` | `+ theorem, proof` | nesting | OD #38 | any theorem or proof environment inside a float (fixture `latexml_oxide/tests/complex/figure_mixed_content.tex`) | Ours adds exactly `theorem` and `proof`. The upstream rule: a float admits `theorem` and `proof`, two `Para.class` members a `logical-block` already admits. |
| S3 | `document.body.class` (`LaTeXML-structure.rnc:23`) | `Para.model \| paragraph \| subsubsection \| …` | `+ subparagraph`; flattened into `ltx:document`, `ltx:sectional-block`, `ltx:inline-sectional-block` | nesting | OD #252 | `\@startsection` at level ≥ 5 with no enclosing unit (smfart.cls:873, the `\tableofcontents` heading) | The document body admits every sectional unit, as the bodies of `part`, `chapter`, `section`, `subsection`, `subsubsection` and `paragraph` do (Perl `LaTeXML-structure.rnc:62-141`: `subparagraph` beside `paragraph`). |
| S4 | `creator_model` (`LaTeXML-structure.rnc:449`) | `(Person.class \| Misc.class)*` | `+ note` | nesting | OD #156 | an author's `\thanks` → `<ltx:note role="thanks">` (`frontmatter_fields.rs` `\lx@add@thanks`) | A creator admits `Meta.class` (note, declare, indexmark, glossarydefinition, rdf, resource, navigation), as `caption`, `quote` and `equation` do. S4 and S5 are two of its seven members. |
| S5 | `creator_model` | (as S4) | `+ rdf` | nesting | OD #479 | AASTeX 7's `\email` without `[show]` → `<ltx:rdf property="schema:email">` (`\lx@add@creator@rdf`, `frontmatter_api.rs`; `aastex_cls.rs`) | Same rule as S4. |
| S6 | `picture_model` (`LaTeXML-picture.rnc:95`) | `(Picture.class \| Inline.class \| Misc.class \| Meta.class)*` | a leading `tags?`, then the same | nesting | OD #813 | a raw `\refstepcounter` inside a box drawn as a picture (tcolorbox's `\newtcbtheorem`, `\newtcolorbox[auto counter]`; a `\put`) | A picture that a counter step numbers carries its number as a float does: a leading `tags?`, with S8. |
| S7 | `SVG.foreignObject.content` (`LaTeXML-misc.rnc:114`) | `parent Inline.model` | `parent Flow.model`: adds block, description, enumerate, equation, equationgroup, itemize, listing, p, pagination, quote | changed model (a widening) | OD #186 | block content in a TikZ node (`pgfsys_latexml_def.rs`; `base_utilities/block_insertion.rs` auto-opens the foreignObject) | `svg:foreignObject` takes `Flow.model`, as Perl's own `SVG.desc.content` does (Perl `LaTeXML-misc.rnc:115`). |
| S8 | `picture` attributes (`LaTeXML-picture.rnc:81`) | `ID.attributes` | `Labelled.attributes` (adds `labels`, `inlist`) | existing group, one more element | OD #813 | as S6 | Every element a `\label` can name carries `Labelled.attributes`. |
| S9 | `abstract_attributes`, `acknowledgements_attributes` (`LaTeXML-structure.rnc:515,527`) | `Common.attributes, FrontMatter.attributes` | `+ Labelled.attributes` (adds `xml:id`, `fragid`, `labels`, `inlist`) | existing group, two more elements | no OD entry: the arXiv-fork TOC cluster (`docs/archive/ARXIV_FORK_AUDIT_2026-07-03.md`) | `Tag` on `ltx:abstract` and `ltx:acknowledgements` sets `inlist="toc"` and an id (`latex_constructs/sect04.rs`), so the navigation TOC lists them | Front- and back-matter units a TOC lists carry `Labelled.attributes`, as sectional units do. |
| S10 | `XMath.attributes` (`LaTeXML-math.rnc:76`) | — | `role_side` (text) on 7 elements: XMApp, XMArg, XMArray, XMDual, XMText, XMTok, XMWrap (the `.rnc` puts it in `XMath.attributes`, which XMHint and XMRef take too, but the compiled model lacks it on those two) | new attribute | no OD entry: `docs/math/MATH_PARSE_RESIDUALS.md` ("`role_side` reaches the output XML") | `\left`, `\right` (`tex_math.rs`: `left`, `right`), `\middle` (`etex.rs`: `middle`) on a stretchy delimiter | None. This is a parser hint, not markup (see below). |
| S11 | `Panel.attributes` in `Common.attributes` (`LaTeXML-common.rnc:87,185`) | — | `panelwidth` (decimal), `panelspan` (nonNegativeInteger) on every element | new attribute | OD #887 | `annotate_panel_grid` (`latex_constructs/mod.rs`), on each panel of a float of two or more | Needs a proposal (below). |
| S12 | `PanelGrid.attributes` on figure, table, float (`LaTeXML-para.rnc:146,172,195`; `LaTeXML-common.rnc:194`) | — | `panelrows`, `panelcols` (nonNegativeInteger), `panelline`, `panelfill` (decimal) | new attribute | OD #887 | as S11, on the float | Needs a proposal (below). |

## The new attributes

**S8, `ltx:picture` gets `Labelled.attributes`.** tcolorbox numbers its boxes with a raw `\refstepcounter` inside the
frame picture (tcolorbox.sty:2311, :998, :1154-1161, :1925). LaTeX's `\label` then names that box. Perl's label climbs
past the picture to the enclosing section, so `\ref` prints the section's number (KNOWN_PERL_ERRORS #1038). Ours stops
at the picture, which takes the step's `tags` (S6) and `labels`. No new names: a proposal only adds `picture` to the
elements that are labelled. Guard: `perfect_kernel_batch78g::tcb_box_label_takes_the_box_number`.

**S9, `ltx:abstract` and `ltx:acknowledgements` get `Labelled.attributes`.** They join the navigation TOC, as
`ltx:bibliography` already does. This needs `inlist="toc"` and an `xml:id` that the TOC entry links to. A `\label` in
an abstract can then name it. No new names. The change came from the ar5iv fork, so it has no OD entry (doc gap,
reported for the main session). 10 goldens carry it (for example `latexml_oxide/tests/structure/svabstract.xml`).

**S10, `role_side`.** `\left`, `\right` and `\middle` mark their stretchy delimiter `left`, `right` or `middle`. The
math grammar uses the mark to pair vertical bars (`\left|…\right|`, the `\vertii`/`\vertiii` kerned stacks). Perl
emits no such attribute, and 8 goldens carry it. `docs/math/MATH_PARSE_RESIDUALS.md` already plans to make it internal
(Perl's convention: an `_`-prefixed attribute removed at finalization). Once that is done, this schema change goes too.
**Not for upstream.**

**S11 and S12, the panel grid** (OD #887; FIDELITY_CHARTER §2 "Float panels"). Perl's `arrange_panels_and_breaks`
(latex_constructs.pool.ltxml L3236-3356) sets a float's panels in rows with `ltx:break`s, and the XML records nothing
more. A panel's stated width is lost (Perl subcaption.sty.ltxml:60-66 stores a `{subfigure}`'s width but never emits
it). Nothing tells a gallery of independent panels from a matrix whose rows and columns mean something. The attributes
record what the PDF shows:
- on the float, `panelrows` and `panelcols` (the rows, and the most columns a row takes; with the class
  `ltx_panels_matrix` or `ltx_panels_gallery`);
- where every panel states its width, `panelline` (the float's line as a fraction of the running-text column) and
  `panelfill` (the widest row's sum of `panelwidth`);
- on each panel, `panelwidth` (its stated width as a fraction of its row's line) and `panelspan` (the columns it spans
  in a matrix, when 2 or more).

The CSS renders a panel at `panelwidth × panelline` of the theme's column. `Panel.attributes` sits in
`Common.attributes` because any element a float arranges can be a panel. A proposal could narrow it to the children
the float model admits. These seven names are the only new vocabulary in our schema. Guards:
`perfect_kernel_batch79c::*`.

## The proposal, ranked

1. **The panel grid** (S11, S12): new vocabulary, with a rendering rule, from a charter ruling.
2. **A creator admits `Meta.class`** (S4, S5). This is one rule for both of our additions. It also lets a `\thanks`
   render as a marked note and not as an inline contact (the XSLT half: `LaTeXML-structure-xhtml.xsl`, OD #156).
3. **A quote admits `Para.class` minus the floats, plus `sectional-block`; a float admits `theorem` and `proof`**
   (S1, S2). Perl errs on valid LaTeX here (a theorem in a figure; a minipage holding a `\section*` in a quote). The
   quote rule answers Perl's own "perhaps even Para.model?" and is wider than our schema, which adds only `para`,
   `logical-block` and `sectional-block` to a quote.
4. **`svg:foreignObject` takes `Flow.model`** (S7). It matches Perl's `SVG.desc.content`. Without it a paragraph or a
   `\verb` in a TikZ node is a schema error (`<ltx:p> isn't allowed in <svg:g>`).
5. **Labelled front matter and pictures** (S8, S9, S6). These are existing groups on more elements.

S3 is an asymmetry fix that needs no argument. S10 is not proposed: it is removed when it becomes internal.

## Not schema changes

Some OD entries mention the content model but leave the schema as Perl's. They rebuild the tree to fit it. The
`.model` diff above shows these entries change no schema: #189, #240, #241, #242, #244, #245, #250, #263, #689.
