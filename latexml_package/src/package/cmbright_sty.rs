use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // cmbright.sty:26-28: Computer Modern Bright is the document's family — its sans (`cmbr`, font.rs) is the default,
  // as the PDF is set (CMBR/HFBR fonts); Perl's binding leaves the defaults alone. OXIDIZED_DESIGN_DIVERGENCES #779.
  RawTeX!(r"\renewcommand{\familydefault}{\sfdefault}\renewcommand{\sfdefault}{cmbr}\renewcommand{\ttdefault}{cmtl}");
  // Perl: cmbright.sty.ltxml
  Let!("\\upDelta", "\\Delta");
  Let!("\\upOmega", "\\Omega");
  DefMath!("\\mathsterling", "\u{00A3}"); // POUND SIGN
});
