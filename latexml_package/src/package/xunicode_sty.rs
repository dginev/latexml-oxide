use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: xunicode.sty.ltxml
  // Preliminary support for xelatex
  AssignValue!("PERL_INPUT_ENCODING" => "utf8");
  RequirePackage!("textcomp");
});
