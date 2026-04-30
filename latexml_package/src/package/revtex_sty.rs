//! revtex.sty — RevTeX 3 compatibility as a style package
//! Perl: revtex.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl L22-23: natbib requires the `numbers` option (preload citation
  // style) and revtex3_support must be loaded. Prior Rust was dropping
  // the natbib option entirely and skipping revtex3_support outright
  // (with a stale comment — the support package is now ported).
  RequirePackage!("natbib", options => vec!["numbers".to_string()]);
  // Perl L23: RequirePackage('revtex3_support', withoptions => 1)
  require_package_with_options("revtex3_support")?;
});
