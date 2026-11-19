use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // No Perl binding: the raw package runs, then its footnotes take `\verb` (OXIDIZED_DESIGN #192, #1085).
  InputDefinitions!("bigfoot", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // bigfoot.sty:31 "makes footnotes work": its \footnote/\footnotetext read the text as a box body, not an argument,
  // so `\verb` is legal there (pdflatex: 0 errors); the locked LaTeXML constructors take the kernel's live-body
  // variant instead. Witness 2310.18996 (saclaymocks.tex:45, three `\verb+…+` URLs in footnotes).
  RawTeX!(r"\lx@VerbatimFootnotes");
});
