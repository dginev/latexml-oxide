use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: eucal.sty.ltxml
  // eucal basically redefines \mathcal to use an Euler script.
  // There's really nothing to do for LaTeXML.
  Let!("\\CMcal", "\\mathcal");
  // eucal.sty:32 `\DeclareMathAlphabet\EuScript{U}{eus}{m}{n}`, the Euler script alphabet eucal's `\mathcal` is
  // (as euscript_sty.rs has it); `\EuScript` was the first error of 229 run-340 papers (witnesses 2103.16394,
  // 2103.05078). eucal.sty:37-40: option `mathscr` makes `\mathscr` the same alphabet.
  Let!("\\EuScript", "\\mathcal");
  DeclareOption!("psamsfonts", None);
  DeclareOption!("mathcal", None);
  DeclareOption!("mathscr", "\\providecommand{\\mathscr}{}\\renewcommand{\\mathscr}{\\EuScript}");
  ProcessOptions!();
});
