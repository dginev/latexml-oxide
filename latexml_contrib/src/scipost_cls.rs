//! Stub for SciPost.cls (SciPost journal class).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("fancyhdr");
  // SciPost ships review-mode line numbering via lineno; many papers
  // disable it with \nolinenumbers in the preamble. Witness 2407.00516.
  RequirePackage!("lineno");
  // SciPost.cls preloads caption (see `RequirePackage[width=...]{caption}`
  // at L13 of the bundled .cls). Authors use \captionsetup{...} without
  // explicit \usepackage{caption}. Witness 2308.16304.
  RequirePackage!("caption");
  RequirePackage!("cite");

  // SciPost.cls L52-53: deepblue / blue colours.
  RawTeX!(r"\definecolor{scipostdeepblue}{HTML}{002B49}");
  RawTeX!(r"\definecolor{scipostblue}{HTML}{0019A2}");

  // Common SciPost frontmatter — preserve author content.
  DefMacro!("\\preprint{}",
    "\\@add@frontmatter{ltx:note}[role=preprint]{#1}");
  DefMacro!("\\authorlist{}",
    "\\@add@frontmatter{ltx:note}[role=authorlist]{#1}");
  DefMacro!("\\inst{}", "\\textsuperscript{#1}");
  DefMacro!("\\affiliation{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}");
  DefMacro!("\\funder{}",
    "\\@add@frontmatter{ltx:note}[role=funder]{#1}");
  DefMacro!("\\doi{}",
    "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!("\\arxivlink{}",
    "\\@add@frontmatter{ltx:note}[role=arxiv]{#1}");
});
