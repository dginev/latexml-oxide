use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: fancyvrb.sty.ltxml
  InputDefinitions!("fancyvrb", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Perl fancyvrb.sty.ltxml L22-25: hack internals to add css class ltx_verbatim
  // to every typeset source line (fancyvrb applies \FancyVerbFormatLine per line
  // for all Verbatim variants; fvextra/minted raw-load the real .sty whose
  // \RequirePackage{fancyvrb} routes through this binding, so they inherit it).
  Let!("\\lx@save@FancyVerbFormatLine", "\\FancyVerbFormatLine");
  DefMacro!("\\FancyVerbFormatLine{}",
    "\\lx@add@cssclass{ltx_verbatim}\\lx@save@FancyVerbFormatLine{#1}");
});
