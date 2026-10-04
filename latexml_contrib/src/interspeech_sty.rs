//! INTERSPEECH2021.sty / INTERSPEECH2022.sty / INTERSPEECH2023.sty — the Interspeech package loaded on top of
//! `\documentclass{article}`: spconf.sty's frontmatter convention (bound in `spconf_sty`) plus the packages the
//! style itself requires (INTERSPEECH2021.sty:50-54, INTERSPEECH2022.sty alike). Bound as plain spconf, a paper's
//! `\includegraphics` was undefined (2103.14512, 2211.09381, 2106.13419; 15 papers of run 329).
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("graphicx");
  RequirePackage!("amssymb");
  RequirePackage!("amsmath");
  RequirePackage!("bm");
  RequirePackage!("textcomp");
  RequirePackage!("booktabs");
  RequirePackage!("caption", options => vec![s!("textfont=it"), s!("tableposition=top")]);
  RequirePackage!("spconf");
  // INTERSPEECH2021.sty:69-70: vectors and matrices are bold, not arrow-accented.
  DefMacro!("\\vec{}", "\\ensuremath{\\bm{{#1}}}");
  DefMacro!("\\mat{}", "\\vec{#1}");
});
