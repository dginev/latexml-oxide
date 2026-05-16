//! Stub for Elsevier's autart.cls (Automatica journal).
//!
//! autart.cls is an article-derivative. The native binding fallback to
//! OmniBus misses class-defined helpers like {ack} environment and
//! several elsart-style frontmatter macros.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");

  // autart.cls L317-323: \def\ack{\section*{Acknowledgements}}
  // with \let\endack\par. Translate to a proper environment.
  DefMacro!(T_CS!("\\begin{ack}"), None, "\\section*{Acknowledgements}");
  DefMacro!(T_CS!("\\end{ack}"), None, "");
  DefMacro!(T_CS!("\\begin{ack*}"), None, "");
  DefMacro!(T_CS!("\\end{ack*}"), None, "");

  // Common elsart frontmatter macros (autart inherits elsart style) —
  // preserve author-supplied content as ltx:note frontmatter.
  DefMacro!("\\address[]{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#2}");
  DefMacro!("\\thanksref{}", "\\textsuperscript{#1}");
  DefMacro!("\\corauthref{}", "\\textsuperscript{*#1}");
  DefMacro!("\\corauth{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}");
  DefMacro!("\\thanks{}",
    "\\@add@frontmatter{ltx:note}[role=thanks]{#1}");
});
