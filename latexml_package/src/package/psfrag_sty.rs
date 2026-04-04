//! psfrag.sty — PostScript fragment overlays on EPS images
//! Perl: psfrag.sty.ltxml — 166 lines
//! Stores psfrag commands for later use when including EPS graphics.
//! The actual overlay is done by LaTeX (we just preserve the fragments).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Options — Perl L28-32
  DeclareOption!("209mode", {});
  DeclareOption!("2emode",  {});
  DeclareOption!("scanall", {});
  ProcessOptions!();

  // \psfrag — stores fragment for later overlay — Perl L46-55
  // NOT a constructor since args should not be digested yet
  DefPrimitive!("\\psfrag OptionalMatch:* Semiverbatim [][][][]{}", None);
  DefConstructor!("\\lx@delayed@psfrag OptionalMatch:* Semiverbatim [][][][]{}", "");

  // Scan control — Perl L56-60
  DefMacro!("\\psfragscanon", "");
  DefMacro!("\\psfragscanoff", "");

  // The Perl version hooks into \includegraphics and \epsfbox to check
  // if the image is an EPS that needs psfrag processing, and if so,
  // wraps it in a <ltx:picture> with the TeX overlay.
  // This requires image type detection (psfrag_requirepicture) which
  // we don't have. For now, includegraphics works normally without overlay.

  // Rescan macros — Perl L78-85
  DefMacro!("\\tex Semiverbatim", "#1");
  DefMacro!("\\psfragrescan", "");
  DefMacro!("\\psfragrescanoff", "");
  DefMacro!("\\psfragrescanon", "");
  DefMacro!("\\psfragdebugon", "");
  DefMacro!("\\psfragdebugoff", "");
});
