//! revtex.sty — RevTeX 3 compatibility as a style package
//! Perl: revtex.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Preload natbib with numbers option
  RequirePackage!("natbib");
  // revtex3_support is not yet ported; skip for now
  // RequirePackage!("revtex3_support");
});
