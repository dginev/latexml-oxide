use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: german.sty.ltxml
  // This should be essentially the same, right?
  // (considering we don't do hyphenation, etc)
  RequirePackage!("babel", options => vec!["german".to_string()]);
});
