use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("ifluatex");
  // No effect from ifpdf.sty
  RequirePackage!("xkeyval");
  RequirePackage!("microtype");
  // savetrees.sty:193-194 loads calc under `\if@st@tight@lists`, which is on
  // by default (:47; off only for `lists=normal` or the `subtle` style, which
  // this binding does not read): a document can rely on it (OXIDIZED_DESIGN
  // #317).
  RequirePackage!("calc");
  DefMacro!("\\bibfont", "\\normalfont\\small");
  def_macro_noop("\\bibsetup")?;
  def_macro_noop("\\markeverypar")?;
  DefMacro!("\\savetreesbibnote{}", "#1");
});
