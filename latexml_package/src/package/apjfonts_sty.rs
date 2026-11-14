use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // apjfonts.sty:342-352 (0704.2079's copy) `\DeclareMathSymbol\varGamma{\mathord}{cmletters}{"00}`…: the math italic's
  // capital Greek letters whatever the alphabet. The former `font => { shape => "italic" }` lost to a capital Greek
  // letter's default shape (`Font::specialize`), upright as Perl's (KNOWN_PERL_ERRORS #1147).
  declare_capital_greek("var", Some(LettersClass::Fixed))?;
  DefMath!("\\omicron", "\u{03BF}");
  DefMath!("\\barbar",  "\u{00AF}");
});
