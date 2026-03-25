use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: eucal.sty.ltxml
  // eucal basically redefines \mathcal to use an Euler script.
  // There's really nothing to do for LaTeXML.
  Let!("\\CMcal", "\\mathcal");
});
