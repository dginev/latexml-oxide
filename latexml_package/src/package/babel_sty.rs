//! babel.sty — multilingual support
//! Perl: babel.sty.ltxml — loads the raw sty with noltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("babel", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // After babel loads, clear \@fontenc@load@list to prevent stray commas
  // from babel's AtBeginDocument font encoding iteration. Babel uses
  // \def\@elt#1{,#1,}\edef\bbl@tempa{\expandafter\@gobbletwo\@fontenc@load@list}
  // which produces commas that leak into the document when the TeX engine
  // doesn't fully support the token manipulation involved.
  // This is safe because LaTeXML doesn't need font encoding switching.
  RawTeX!(r"\def\@fontenc@load@list{}");
});
