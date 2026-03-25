//! pdftexcmds.sty — pdfTeX utility commands
//! Perl: pdftexcmds.sty.ltxml
//! Everything is in pdfTeX.pool already; just require iftex.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("iftex");
});
