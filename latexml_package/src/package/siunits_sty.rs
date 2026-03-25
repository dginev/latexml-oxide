use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: SIunits.sty.ltxml
  // Apparently siunitx is a revision and extension of SIunits
  // Not quite backwards compatible, but worth a try...
  RequirePackage!("siunitx");

  // Apparently similar, but expects the numbers to be already formatted?
  // (things like \times, ^, etc appear)
  Let!("\\unit", "\\SI");

  RawTeX!("\\sisetup{parse-numbers = false, input-product = \\times,}");

  DefMacro!("\\squaren{}", "{#1}^{2}");

  // Apparently, all unit macros are available, all the time !!!
  // TODO: six_enableUnitMacros(1) — stub; siunitx not fully ported
});
