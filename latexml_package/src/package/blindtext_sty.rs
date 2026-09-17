use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: blindtext.sty.ltxml:21 `RequirePackage('babel', options => ['english'])`
  // — NOT carried. blindtext.sty:342-354 `\blind@addtext` registers its
  // per-language texts through `\addto\extras<lang>` only when babel is
  // loaded; babel then runs `\extrasenglish` at begin-document and the
  // English "Hello, here is some text without a meaning" REPLACES the Latin
  // `\blindtext@text` default (blindtext.sty:356-369). pdflatex without babel
  // keeps the Latin default, which is what the shipped PDFs show (assoccnt /
  // xassoccnt examples, 18-43% recall with the lorem-ipsum words missing).
  // The require's only stated purpose was `\languagename`, which the engine
  // defines (tex_hyphenation.rs). A document that loads babel itself is
  // unchanged. OXIDIZED_DESIGN_DIVERGENCES #228.
  InputDefinitions!("blindtext", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
