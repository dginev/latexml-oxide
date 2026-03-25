use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: lscape.sty.ltxml
  // Conceivably could be useful to add "pagination markers"
  // (empty elements that are normally ignored) at beginning and end
  DefEnvironment!("{landscape}", "#body");
});
