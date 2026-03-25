use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: doublespace.sty.ltxml
  // Mostly no-op for LaTeXML, except for two ignorable environments
  DefEnvironment!("{singlespace}", "#body");
  DefEnvironment!("{doublespace}", "#body");
  DefEnvironment!("{spacing}{}", "#body");
});
