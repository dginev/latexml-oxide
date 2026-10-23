use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // cmbright.sty:26-28: Computer Modern Bright is the document's family — its sans (`cmbr`, font.rs) is the default,
  // as the PDF is set (CMBR/HFBR fonts); Perl's binding leaves the defaults alone. OXIDIZED_DESIGN_DIVERGENCES #779.
  RawTeX!(r"\renewcommand{\familydefault}{\sfdefault}\renewcommand{\sfdefault}{cmbr}\renewcommand{\ttdefault}{cmtl}");
  // Perl: cmbright.sty.ltxml
  Let!("\\upDelta", "\\Delta");
  Let!("\\upOmega", "\\Omega");
  Let!("\\upGamma", "\\Gamma"); // cmbright.sty:81
  DefMath!("\\mathsterling", "\u{00A3}"); // POUND SIGN
  // cmbright.sty:66-78 `slantedGreek` (processed at :143, after the `\up…` names): the capital Greek letters from
  // `letters`, italic as a Latin letter unless a math alphabet is in force (KNOWN_PERL_ERRORS #1147); without it
  // upright. No arXiv witness passes the option (79e A/B: 0 of 80 mathptmx/mathpple/ccfonts/cmbright papers); guard
  // `perfect_kernel_batch79e`.
  DeclareOption!("slantedGreek", { declare_capital_greek("", Some(LettersClass::Variable))?; });
  DeclareOption!(None, {});
  ProcessOptions!();
});
