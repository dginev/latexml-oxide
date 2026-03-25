use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  Let!("\\pcal", "\\@undefined");
  Let!("\\pmit", "\\@undefined");
  Let!("\\cal",  "\\mathcal");
  Let!("\\mit",  "\\mathnormal");
  RequirePackage!("latexsym");
});
