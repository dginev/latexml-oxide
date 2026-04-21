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

  // Perl SIunits.sty.ltxml: unlike siunitx (which auto-enables per-\sisetup
  // or per-\DeclareSIUnit), SIunits makes every unit macro unconditionally
  // available. Port calls six_enable_unit_macros(1) at load — matching
  // Perl's "all unit macros are available, all the time".
  crate::package::siunitx_sty::six_enable_unit_macros(true);
});
