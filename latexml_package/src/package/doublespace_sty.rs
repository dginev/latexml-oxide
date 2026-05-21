use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: doublespace.sty.ltxml
  // Mostly no-op for LaTeXML, except for two ignorable environments
  // Same vertical-mode fix as setspace_sty.rs — paragraph-container envs
  // must keep BOUND_MODE vertical so `$$` inside enters display math.
  DefEnvironment!("{singlespace}",  "#body", mode => "internal_vertical");
  DefEnvironment!("{doublespace}",  "#body", mode => "internal_vertical");
  DefEnvironment!("{spacing}{}",    "#body", mode => "internal_vertical");
});
