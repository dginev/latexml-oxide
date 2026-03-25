use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: hypcap.sty.ltxml
  // This package fixes up caption links for pdf using hyperref.
  // LaTeXML shouldn't have an issue with those, so we don't need to do anything.
  DefMacro!("\\capstart", None);
  DefMacro!("\\hyecapspace", "0.5\\baselineskip");
  DefMacro!("\\hypcapredef{}", None);
  DefConditional!("\\ifcapstart");
});
