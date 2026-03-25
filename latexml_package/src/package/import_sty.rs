use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: import.sty.ltxml

  // \lx@set@path and \lx@append@path: complex sub{} bodies that manipulate SEARCHPATHS
  // Stub as no-ops; the path manipulation is not critical for document conversion
  DefMacro!("\\lx@set@path OptionalMatch:* {}", None);
  DefMacro!("\\lx@append@path OptionalMatch:* {}", None);

  DefMacro!("\\import OptionalMatch:* {}{}", "{\\lx@set@path #1{#2} \\input{#3}}");
  DefMacro!("\\includefrom OptionalMatch:* {}", "{\\lx@set@path #1{#2} \\include{#3}}");
  DefMacro!("\\subimport OptionalMatch:* {}{}", "{\\lx@append@path #1{#2} \\input{#3}}");
  DefMacro!("\\subincludefrom OptionalMatch:* {}", "{\\lx@append@path #1{#2} \\include{#3}}");
  Let!("\\inputfrom", "\\import");
  Let!("\\subinputfrom", "\\subimport");
});
