use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: amsxtra.sty.ltxml
  RequirePackage!("amsmath");

  // Perl: Tokens(T_SUPER, T_OTHER('^')) — superscript with caret char
  DefMacro!("\\sphat", "^{\\hat{}}");
  DefMacro!("\\spcheck", "^{\\vee}");
  // Perl: Tokens(T_SUPER, T_OTHER('~')) — superscript with tilde char
  DefMacro!("\\sptilde", "^{\\tilde{}}");
  DefMacro!("\\spdot", "^{\\dot}");
  DefMacro!("\\spddot", "^{\\dot\\dot}");
  DefMacro!("\\spdddot", "^{\\dot\\dot\\dot}");
  DefMacro!("\\spbreve", "^{\\smile}");

  DefMacro!("\\fracwithdelims[]{}{}{}{}",
    "\\left#2\\frac{#4}{#5}\\right#3");
  DefMacro!("\\accentedsymbol{}{}",
    "\\def\\#1{$#2}");
});
