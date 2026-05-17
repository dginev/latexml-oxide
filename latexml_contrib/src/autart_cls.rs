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
  // with \let\endack\par. Bind {ack}/{ack*} as structural
  // ltx:acknowledgements (post-processors map to canonical
  // role/styling).
  DefEnvironment!("{ack}",  "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  DefEnvironment!("{ack*}", "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");

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
