use latexml_package::prelude::*;

LoadDefinitions!({
  // Source: https://arxiv.org/macros/emlines.sty — two `\put`s of the end points and the line `\special`, balanced (the
  // binding's body had two stray `}`, two "Attempt to close a group" Errors per `\emline`: 0911.1421 with 78,
  // cond-mat0106468, 0707.2749, 0906.2921). Guard: `perfect_kernel_batch66r::emline_in_picture_is_balanced`.
  DefMacro!(
    "\\emline{}{}{}{}{}{}",
    "\\put(#1,#2){\\special{em:point #3}}\\put(#4,#5){\\special{em:point #6}}\\special{em:line #3,#6}"
  );
  def_macro_noop("\\newpic{}")?;
});
