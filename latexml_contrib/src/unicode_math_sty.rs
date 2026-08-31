//! unicode-math.sty — Unicode/OpenType math for Xe/LuaLaTeX (no Perl
//! binding). Part of the opt-in `luatex` profile family (user decision
//! 2026-08-31): math-FONT selection is presentation — LaTeXML's math
//! pipeline is Unicode-native already, so the configuration surface absorbs
//! silently and the standard math machinery carries the content.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("amsmath");
  RequirePackage!("fontspec");
  def_macro_noop("\\setmathfont[]{}[]")?;
  def_macro_noop("\\setmathfontface DefToken []{}[]")?;
  def_macro_noop("\\unimathsetup{}")?;
  def_macro_noop("\\NewNegationCommand{}{}")?;
  def_macro_noop("\\NewNegatedSymbol{}{}")?;
  // \symbf/\symit/… — Unicode-math's semantic alphabet switches; map to the
  // classical math alphabets so the CONTENT keeps its lettering.
  DefMacro!("\\symbf{}", "\\mathbf{#1}");
  DefMacro!("\\symit{}", "\\mathit{#1}");
  DefMacro!("\\symsf{}", "\\mathsf{#1}");
  DefMacro!("\\symtt{}", "\\mathtt{#1}");
  DefMacro!("\\symcal{}", "\\mathcal{#1}");
  DefMacro!("\\symbb{}", "\\mathbb{#1}");
  DefMacro!("\\symfrak{}", "\\mathfrak{#1}");
  DefMacro!("\\symrm{}", "\\mathrm{#1}");
  DefMacro!("\\symnormal{}", "#1");
  DefMacro!("\\symup{}", "\\mathrm{#1}");
  // Bold-italic: prefer \boldsymbol when a loaded package provides it
  // (keeps the italic), else plain bold. Witnesses: numbersets-doc,
  // physics2-legacy (\symbfit); shtthesis-user-guide (\symbfsf);
  // rec-thy (\symbffrak); intexgral/xfakebold/yquant docs (\symup).
  DefMacro!(
    "\\symbfit{}",
    "\\ifdefined\\boldsymbol\\boldsymbol{#1}\\else\\mathbf{#1}\\fi"
  );
  DefMacro!("\\symbfup{}", "\\mathbf{#1}");
  DefMacro!("\\symbfsf{}", "\\mathsf{#1}");
  DefMacro!("\\symbffrak{}", "\\mathfrak{#1}");
});
