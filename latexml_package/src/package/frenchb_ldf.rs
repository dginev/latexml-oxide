//! frenchb.ldf — French (deprecated babel option name)
//!
//! Perl: `frenchb.ldf.ltxml` — calls `InputDefinitions('frenchb',
//! type => 'ldf', noltxml => 1)` to raw-load TeX Live's `frenchb.ldf`.
//!
//! NOTE: This module's `load_definitions` is currently NOT registered
//! in `latexml_package/src/lib.rs` for `("frenchb","ldf")` — that slot
//! is wired to `french_ldf::load_definitions`, where the babel-level
//! frenchb-language alias fix lives (TL2025 babel-french 3.7e turned
//! `frenchb.ldf` into a deprecation shim that doesn't chain
//! `french.ldf`). This module is retained for documentation parity
//! with Perl's `frenchb.ldf.ltxml`.
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
