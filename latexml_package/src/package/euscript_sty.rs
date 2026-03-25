use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: euscript.sty.ltxml — obsolete name for AMS's eucal
  Let!("\\CMcal",    "\\mathcal");
  Let!("\\EuScript", "\\mathcal");
});
