use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ccfonts.sty.ltxml
  // Nothing much to do here.
  Let!("\\upDelta", "\\Delta");
  Let!("\\upOmega", "\\Omega");
  // ccfonts.sty:62-74 `slantedGreek`: the capital Greek letters from `letters`, italic as a Latin letter unless a math
  // alphabet is in force (KNOWN_PERL_ERRORS #1147); without it upright. No arXiv witness passes the option (79e A/B: 0
  // of 80 mathptmx/mathpple/ccfonts/cmbright papers); guard `perfect_kernel_batch79e`.
  DeclareOption!("slantedGreek", { declare_capital_greek("", Some(LettersClass::Variable))?; });
  DeclareOption!(None, {});
  ProcessOptions!();
});
