use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: amstex.sty.ltxml
  // amstex.sty is obsolete; it is recommended to try to use amsmath instead.
  RequirePackage!("amsmath");
  RequirePackage!("amsfonts");
  RequirePackage!("amsxtra");

  DefMacro!("\\nolimits@", "\\nolimits");
  DefMacro!("\\nlimits@", "\\displaylimits");
  // Oddly, different defn than amsopn
  DefMacro!("\\qopname{}{}{}", "\\mathop{#3}\\csname #2limits@\\endcsname");

  DefMacro!("\\Sb", "\\lx@generalized@over{\\Sb}{meaning=substack}");
  DefMacro!("\\Sp", "\\lx@generalized@over{\\Sp}{meaning=superstack}");
  Let!("\\endSb", "\\relax");
  Let!("\\endSp", "\\relax");
});
