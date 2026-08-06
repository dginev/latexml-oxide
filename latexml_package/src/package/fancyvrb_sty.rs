use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: fancyvrb.sty.ltxml
  InputDefinitions!("fancyvrb", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Perl fancyvrb.sty.ltxml L22-25: hack internals to add css class ltx_verbatim
  // to every typeset source line (fancyvrb applies \FancyVerbFormatLine per line
  // for all Verbatim variants). NOTE: a package that redefines
  // \FancyVerbFormatLine after requiring fancyvrb overwrites this hook — fvextra
  // (L2249 `\def\FancyVerbFormatLine#1{#1}`) does exactly that, so fvextra_sty.rs
  // re-installs the hook over its redefinition (issue #502).
  Let!("\\lx@save@FancyVerbFormatLine", "\\FancyVerbFormatLine");
  DefMacro!("\\FancyVerbFormatLine{}",
    "\\lx@add@cssclass{ltx_verbatim}\\lx@save@FancyVerbFormatLine{#1}");
});
