//! Port of LaTeXML's `latexmlman.sty` schema-doc support.
//!
//! Used by the scholarly-schema docs pipeline to render
//! `\begin{schemamodule}...\end{schemamodule}` blocks containing
//! `\elementdef` / `\attrdef` / `\patterndef` / `\moduleref` /
//! `\patternref` / `\elementref` / `\attrval` / `\typename` etc.
//!
//! Mirrors `doc/sty/latexmlman.sty` from Perl LaTeXML for the
//! schema-doc-relevant subset; the title-page customizations and the
//! `\pod` / `\ltxpod` family are intentionally omitted (they target
//! Perl module documentation, not schema docs).
//!
//! `\cleanhypername` is intentionally NOT ported. Identifiers in the
//! emitted schema docs may contain `:` (e.g. `schema.namespace1:div`);
//! HTML5 permits that and cross-references resolve as long as
//! `\hypertarget` and `\hyperlink` use the same form, which they do.

use latexml_package::prelude::*;

LoadDefinitions!({
  // hyperref provides \hypertarget / \hyperlink / \hyperref used below.
  RequirePackage!("hyperref");

  //--- Font-switching aliases (mirror upstream \def commands) ---------
  DefMacro!("\\perlfont",   "\\ttfamily");
  DefMacro!("\\shellfont",  "\\ttfamily");
  DefMacro!("\\latexfont",  "\\ttfamily");
  DefMacro!("\\schemafont", "\\sffamily");
  DefMacro!("\\patternfont","\\sffamily\\slshape");

  //--- Phrase-level inline markup -------------------------------------
  DefMacro!("\\code{}",     "{\\ttfamily #1}");
  DefMacro!("\\codevar{}",  "{\\ttfamily\\itshape #1}");
  DefMacro!("\\method{}",   "{\\ttfamily ->#1}");
  DefMacro!("\\attr{}",     "{\\sffamily #1}");
  DefMacro!("\\attrval{}",  "{\\ttfamily #1}");
  DefMacro!("\\cmd{}",      "{\\ttfamily #1}");
  DefMacro!("\\cs{}",       "{\\ttfamily $\\backslash$#1}");
  DefMacro!("\\typename{}", "\\textit{#1}");

  //--- Schema module section ------------------------------------------
  // `\schemamodule` is a labeled section. Definitions inside it are
  // \newtheorem-defined named statements, NOT subsections — so they
  // appear as `<ltx:theorem>` blocks at module scope, side-by-side
  // with the module preamble paragraphs.
  DefMacro!("\\schemamodule{}",
    "\\section{Module \\texttt{#1}}\\label{schema.#1}");
  DefMacro!("\\endschemamodule", "");

  //--- Schema-doc list environments -----------------------------------
  // Each kind bucket (`\subsection{Patterns}` / `\subsection{Elements}`)
  // wraps its defs in a description list. Defs are list items; their
  // bodies open further description lists for facts (Attributes /
  // Content / Used by). Mirrors upstream `latexmlman.sty` exactly —
  // nested `<ltx:item>`s under another item are valid (an inline
  // `\elementdef` inside a pattern's content becomes a nested `<dl>`),
  // which avoids the structural conflict that bare-`\subsubsection`
  // defs hit when nested inside another def's content model.
  DefMacro!("\\elementdescription",      "\\begin{description}");
  DefMacro!("\\endelementdescription",   "\\end{description}");
  DefMacro!("\\patterndescription",      "\\begin{description}");
  DefMacro!("\\endpatterndescription",   "\\end{description}");

  //--- Schema definition macros ---------------------------------------
  // Definitions are description-list items, matching the upstream
  // `RelaxNG.pm` shape (Perl `\elementdef`/`\patterndef` always
  // emitted `\item[Element/Pattern …]`, not a section command). The
  // anchor is `\hypertarget{schema.<name>}{…}` (paired with
  // `\hyperlink` in the cross-references below).
  DefMacro!("\\elementdef{}{}{}",
    "\\item[\\textit{Element }{\\bfseries\\schemafont #1}]\
     \\hypertarget{schema.#1}{#2}\
     \\begin{elementdescription}#3\\end{elementdescription}");

  // \attrdef stays an `\item` — it only ever appears inside an
  // \elementdef / \patterndef body's description list.
  DefMacro!("\\attrdef{}{}{}",
    "\\item[\\textit{Attribute }{\\bfseries\\schemafont #1}] = #3\
     \\par\\noindent #2");

  DefMacro!("\\patterndef{}{}{}",
    "\\item[\\textit{Pattern }{\\bfseries\\patternfont #1}]\
     \\hypertarget{schema.#1}{#2}\
     \\begin{patterndescription}#3\\end{patterndescription}");

  DefMacro!("\\patternadd{}{}{}",
    "\\item[\\textit{Add to }{\\bfseries\\patternfont #1}] \\hspace{1em} #2\
     \\begin{patterndescription}#3\\end{patterndescription}");

  DefMacro!("\\patterndefadd{}{}{}",
    "\\item[\\textit{Add to }{\\bfseries\\patternfont #1}]\
     \\hypertarget{schema.#1}{#2}\
     \\begin{patterndescription}#3\\end{patterndescription}");

  //--- Module-level prose abstract -------------------------------------
  // Synthesized by `tools/genschema` from the `## comments` at the
  // head of each RNC module file (which trang preserves as
  // `<a:documentation>` annotations). Emits a paragraph carrying the
  // `schema_module_narrative` class token; CSS in
  // `resources/CSS/relaxng-schema-rustdoc-theme.css` styles that
  // class as the left-bordered narrative aside above each module's
  // definitions.
  // `<ltx:para>` (a paragraph-block, not a single paragraph) so
  // multi-paragraph abstracts — produced when trang emits more than
  // one `<a:documentation>` per define — keep all their `<p>`s
  // wrapped under one element with the `schema_module_narrative`
  // marker class. With `<ltx:p>`, LaTeXML closes the paragraph at
  // the first `\par` and starts new `<p>`s without the class, which
  // leaves the trailing prose stranded outside the post-pass aside.
  DefConstructor!(
    "\\moduleabstract{}",
    "<ltx:para class='schema_module_narrative'>#1</ltx:para>"
  );

  //--- Cross-references -----------------------------------------------
  DefMacro!("\\moduleref{}",  "\\hyperref[schema.#1]{{\\ttfamily #1}}");
  DefMacro!("\\patternref{}", "\\hyperlink{schema.#1}{{\\sffamily\\slshape #1}}");
  DefMacro!("\\elementref{}", "\\hyperlink{schema.#1}{{\\sffamily #1}}");

  //--- Source-link footer for the cover page --------------------------
  // `\schemasource{label}{url}` renders a small "Source:" line that
  // links back to the upstream schema file. The orchestration shell
  // (`tools/generate-scholarly-schema-docs`) invokes it with
  // git-derived values when the schema lives in a checkout — typically
  // a SHA-pinned GitHub/GitLab `blob/SHA/path` URL — and skips the
  // call entirely when the schema isn't in any git repo.
  //
  // Emitted as a `<ltx:para class='schema_source'>` so the post-pass
  // / theme stylesheet can style it (small, muted) without touching
  // the document body. Uses `\href` so the link target is the URL
  // and the label remains a friendly path/SHA fragment.
  DefConstructor!(
    "\\schemasource{}{}",
    "<ltx:para class='schema_source'><ltx:text>Source: </ltx:text><ltx:ref href='#2' class='ltx_url'><ltx:text class='ltx_font_typewriter'>#1</ltx:text></ltx:ref></ltx:para>"
  );
});
