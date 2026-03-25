use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: pdfpages.sty.ltxml
  RequirePackage!("ifthen");
  RequirePackage!("calc");
  RequirePackage!("graphicx");

  // \includepdf: includes a pdf as a resource
  // Simplified: just link to the PDF file
  DefConstructor!("\\includepdf[]{}",
    "<ltx:resource src='#2' type='application/pdf'/>See <ltx:ref href='#2'>#2</ltx:ref>");
});
