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

  //--- Schema-doc list environments -----------------------------------
  // The *description envs are aliases for `description`. We use the
  // \newenvironment desugaring (\X/\endX as a DefMacro pair) so the
  // body templates are expanded as TeX rather than emitted verbatim
  // as XML — DefEnvironment! is XML-emitting only.
  DefMacro!("\\moduledescription",       "\\begin{description}");
  DefMacro!("\\endmoduledescription",    "\\end{description}");
  DefMacro!("\\elementdescription",      "\\begin{description}");
  DefMacro!("\\endelementdescription",   "\\end{description}");
  DefMacro!("\\patterndescription",      "\\begin{description}");
  DefMacro!("\\endpatterndescription",   "\\end{description}");

  //--- Schema module section ------------------------------------------
  // Mirrors upstream `latexmlman.sty`'s `\section{Module {\perlfont #1}}`.
  // We use the bare \section{...} form (no optional [short] arg) since
  // oxide's \section is defined without optional-arg parsing; passing
  // `\section[X]{Y}` would be misparsed and produce no section element.
  DefMacro!("\\schemamodule{}",
    "\\section{Module \\texttt{#1}}\\label{schema.#1}\
     \\raggedright\
     \\begin{moduledescription}");
  DefMacro!("\\endschemamodule", "\\end{moduledescription}");

  //--- Schema definition macros ---------------------------------------
  // Each emits a `description`-list `\item` with a hypertarget.
  // Body wrapped in the relevant *description env when nonempty.
  DefMacro!("\\elementdef{}{}{}",
    "\\item[\\textit{Element }{\\bfseries\\schemafont #1}]\
     \\hypertarget{schema.#1}{#2}\
     \\begin{elementdescription}#3\\end{elementdescription}");

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

  //--- Cross-references -----------------------------------------------
  DefMacro!("\\moduleref{}",  "\\hyperref[schema.#1]{{\\ttfamily #1}}");
  DefMacro!("\\patternref{}", "\\hyperlink{schema.#1}{{\\sffamily\\slshape #1}}");
  DefMacro!("\\elementref{}", "\\hyperlink{schema.#1}{{\\sffamily #1}}");
});
