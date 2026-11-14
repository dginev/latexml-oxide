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
  // amsxtra.sty:32-34 superscripts text-font periods in a box (`^{\hbox{\raise\ex@\hbox{\normalfont .}}}`; the raise
  // is presentation). Perl's `^{\dot}` gave the accent the group's `}` as its argument: "Missing { inserted" at the next
  // `&`, the alignment unclosed (0907.3938's `\rbar1{(\T\lm)\spdot}` in {align*}).
  DefMacro!("\\spdot", "^{\\hbox{\\normalfont .}}");
  DefMacro!("\\spddot", "^{\\hbox{\\normalfont ..}}");
  DefMacro!("\\spdddot", "^{\\hbox{\\normalfont ...}}");
  DefMacro!("\\spbreve", "^{\\smile}");

  DefMacro!("\\fracwithdelims[]{}{}{}{}",
    "\\left#2\\frac{#4}{#5}\\right#3");
  DefMacro!("\\accentedsymbol{}{}",
    "\\def\\#1{$#2}");
});
