//! mathptmx.sty — Times Roman math fonts
//! Perl: mathptmx.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMath!("\\omicron", "\u{03BF}");  // GREEK SMALL LETTER OMICRON
  // mathptmx.sty:93-103 `\DeclareMathSymbol{\upGamma}{\mathord}{operators}{0}`…: upright glyphs of their own under any
  // class, `tex=` reading the source's `\upGamma` — Perl's binding `\let`s them to `\Gamma`, which under
  // llncs.cls:379-389 (2606.09074) is the italic letter (KNOWN_PERL_ERRORS #1147; review rev79e finding 2). No census
  // paper writes them in math (0907.4547, 2105.08709, 2209.09470 only define their own): repro
  // fonts-nfss/mathptmx_up_capital_greek_keeps_its_name.
  declare_capital_greek_named("up", None, None)?;
  // mathptmx.sty:80-92 `slantedGreek`: the capital Greek letters from `letters`, italic as a Latin letter unless a math
  // alphabet is in force (KNOWN_PERL_ERRORS #1147); without it upright. Any other option is not the binding's concern.
  // No arXiv witness passes the option (79e A/B: 0 of 80 mathptmx/mathpple/ccfonts/cmbright papers); guard
  // `perfect_kernel_batch79e`.
  DeclareOption!("slantedGreek", { declare_capital_greek("", Some(LettersClass::Variable))?; });
  DeclareOption!(None, {});
  ProcessOptions!();
});
