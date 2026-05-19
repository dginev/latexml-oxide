use latexml_package::prelude::*;


LoadDefinitions!({
  // Source: https://arxiv.org/macros/emlines.sty
  DefMacro!(
    "\\emline{}{}{}{}{}{}",
    "\\put(#1,#2){\\special{em:point #3}}\\put(#4,#5){\\special{em:point #6}}\\special{em:line #3,#6}}}"
  );
  def_macro_noop("\\newpic{}")?;
});
