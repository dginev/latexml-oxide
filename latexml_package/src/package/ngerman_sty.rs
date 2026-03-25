use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ngerman.sty.ltxml
  // This should be essentially the same, right?
  // (considering we don't do hyphenation, etc)
  RequirePackage!("babel", options => vec!["ngerman".to_string()]);
});
