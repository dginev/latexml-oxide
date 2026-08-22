//! tabto.sty (tabto-ltx) — tab to a horizontal position.
//!
//! Perl LaTeXML has NO binding and raw-loads this package. The raw `\tabto`
//! implements positioning via a `$$…$$` display-math measurement hack (it reads
//! `\predisplaysize` inside a one-row `\halign`). Our engine turns that hack into a
//! spurious empty display equation AND breaks the following content onto a new line
//! — so a right-justified `algpseudocodex` `\Comment` (`\tabto{\dimexpr\linewidth-…}`,
//! `\RequirePackage{tabto}` at algpseudocodex.sty L29) stacks BELOW its statement
//! instead of flushing right (witness arXiv 2511.21969 Algorithm 1).
//!
//! LaTeXML has no positional layout model, so we approximate `\tabto` as `\hfill`
//! (fill to the tab stop ≈ push the following content to the right) — the dominant
//! use is exactly the right-justified algorithm comment, which `\hfill` renders as a
//! `float:right` on the same line, matching the pdflatex golden. Surpass over Perl,
//! which raw-loads the `$$` hack and stacks the comment. OXIDIZED_DESIGN #150.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Length registers documents read through tabto (algpseudocodex uses both).
  DefRegister!("\\CurrentLineWidth" => Dimension::new(0));
  DefRegister!("\\TabPrevPos" => Dimension::new(0));
  // \tabto{pos} and \tabto*{pos} (the overlap `\tab*` variant) → right-flush.
  DefMacro!("\\tabto OptionalMatch:* {}", "\\hfill");
  // \tab advances to the next tab stop; same approximation.
  DefMacro!("\\tab", "\\hfill");
  DefMacro!("\\NextTabStop", "\\linewidth");
  // Remaining tabto API a document may touch — harmless no-ops / stubs.
  def_macro_noop("\\TabPositions{}")?;
  def_macro_noop("\\NumTabs OptionalMatch:* {}{}")?;
  def_macro_noop("\\TabsBadStop")?;
});
