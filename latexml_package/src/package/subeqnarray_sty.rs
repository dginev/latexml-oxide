use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: subeqnarray.sty.ltxml
  // see example use at arXiv:hep-th/0002165
  DefMacro!("\\subeqnarray",
    "\\lx@equationgroup@subnumbering@begin\\bgroup\\lx@begin@display@math");
  DefMacro!("\\endsubeqnarray",
    "\\lx@end@display@math\\egroup\\lx@equationgroup@subnumbering@end");

  InputDefinitions!("subeqnarray", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
