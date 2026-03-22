//! ifpdf.sty — PDF mode detection (always false in LaTeXML)
//! Perl: ifpdf.sty.ltxml
use crate::prelude::*;

LoadDefinitions!({
  // Perl: RawTeX \newif\ifpdf\pdffalse
  DefConditional!("\\ifpdf");
});
