//! frenchb.ldf — French (deprecated babel option name)
//!
//! Perl: `frenchb.ldf.ltxml` — calls `InputDefinitions('frenchb',
//! type => 'ldf', noltxml => 1)` to raw-load TeX Live's `frenchb.ldf`.
//! The TL shim (gated by `\ifx\CurrentOption\bbl@tempa{frenchb}`) does
//! `\chardef\l@frenchb\l@french` to alias the language counter so babel's
//! `\selectlanguage{frenchb}` resolves, then unconditionally `\input
//! french.ldf` for the actual French setup. Our dispatcher routes the
//! chained `\input french.ldf` back to `french_ldf::load_definitions`.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // frenchb doesn't normally require this package, but behaves better if it's loaded
  RequirePackage!("textcomp");

  // Raw-load TL's frenchb.ldf. Relies on `\CurrentOption` having been set
  // by babel's option pipeline (\ProcessOptions → \ds@frenchb →
  // \bbl@load@language{frenchb} → \input frenchb.ldf) with LETTER catcodes
  // so the shim's `\ifx\CurrentOption\bbl@tempa` succeeds.
  InputDefinitions!("frenchb", noltxml => true, extension => Some(Cow::Borrowed("ldf")));

  // Patches over the raw frenchb/french.ldf for known portability gaps:
  DefConstructor!("\\fup{}",        "<ltx:sup>#1</ltx:sup>", enter_horizontal => true);
  DefConstructor!("\\FB@up@fake{}", "<ltx:sup>#1</ltx:sup>", enter_horizontal => true);

  // Attempt to make it work with older & newer versions.
  Let!("\\ltx@orig@nombre", "\\nombre");
  DefMacro!("\\nombre{}",
    "\\@ifpackageloaded{numprint}{\\numprint{#1}}{\\ltx@orig@nombre{#1}}");

  // Perl: AtBeginDocument(sub { Let('\degre','\textdegree');
  // DefMacro('\degres','\hbox to 0.3em{\degre}'); });
  at_begin_document(TokenizeInternal!(
    r"\let\degre\textdegree\def\degres{\hbox to 0.3em{\degre}}"
  ))?;
});
