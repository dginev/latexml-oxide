use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: subeqnarray.sty.ltxml L21-22 — both \subeqnarray and
  // \endsubeqnarray carry `locked => 1`. The lock matters because the
  // immediately-following InputDefinitions pulls the raw subeqnarray.sty,
  // which redefines both names; without the lock the raw-TeX version
  // overwrites our subnumbering trampolines. See arXiv:hep-th/0002165.
  DefMacro!("\\subeqnarray",
    "\\lx@equationgroup@subnumbering@begin\\bgroup\\lx@begin@display@math",
    locked => true);
  DefMacro!("\\endsubeqnarray",
    "\\lx@end@display@math\\egroup\\lx@equationgroup@subnumbering@end",
    locked => true);

  InputDefinitions!("subeqnarray", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
