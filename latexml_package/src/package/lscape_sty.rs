use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: lscape.sty.ltxml
  // lscape.sty:35 `\RequirePackage{graphics}`: a paper that loads lscape has `\includegraphics`, `\scalebox`,
  // `\resizebox` without loading graphics itself (Perl's binding lacks the load, KNOWN_PERL_ERRORS #692; witnesses
  // 0710.2354, 0802.0639, 1007.2821).
  RequirePackage!("graphics");
  // Conceivably could be useful to add "pagination markers"
  // (empty elements that are normally ignored) at beginning and end
  DefEnvironment!("{landscape}", "#body",
    // The begin code opens a `\begingroup` (a semi_simple_group) of its own: lscape.sty:38-42 `\begingroup`.
    tex_group => { TexGroup::SemiSimple });
});
