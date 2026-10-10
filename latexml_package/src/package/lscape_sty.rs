use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: lscape.sty.ltxml
  // Conceivably could be useful to add "pagination markers"
  // (empty elements that are normally ignored) at beginning and end
  DefEnvironment!("{landscape}", "#body",
    // The begin code opens a `\begingroup` (a semi_simple_group) of its own: lscape.sty:38-42 `\begingroup`.
    tex_group => { TexGroup::SemiSimple });
});
