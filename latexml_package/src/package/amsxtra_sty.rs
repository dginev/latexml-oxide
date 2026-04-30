use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: amsxtra.sty.ltxml
  RequirePackage!("amsmath");

  // Perl L23: Tokens(T_SUPER, T_OTHER('^')) — two raw tokens: superscript
  // marker followed by a literal `^` (catcode OTHER). Expressed directly
  // rather than the prior `^{\hat{}}` transcription, which introduced an
  // unintended \hat-accent node semantically distinct from the Perl output.
  DefMacro!("\\sphat", None, Tokens!(T_SUPER!(), T_OTHER!("^")));
  DefMacro!("\\spcheck", "^{\\vee}");
  // Perl L25: Tokens(T_SUPER, T_OTHER('~')) — superscript + literal tilde.
  DefMacro!("\\sptilde", None, Tokens!(T_SUPER!(), T_OTHER!("~")));
  DefMacro!("\\spdot", "^{\\dot}");
  DefMacro!("\\spddot", "^{\\dot\\dot}");
  DefMacro!("\\spdddot", "^{\\dot\\dot\\dot}");
  DefMacro!("\\spbreve", "^{\\smile}");

  DefMacro!("\\fracwithdelims[]{}{}{}{}",
    "\\left#2\\frac{#4}{#5}\\right#3");
  DefMacro!("\\accentedsymbol{}{}",
    "\\def\\#1{$#2}");
});
