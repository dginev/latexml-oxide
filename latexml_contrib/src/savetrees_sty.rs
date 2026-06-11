use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("ifluatex");
  // No effect from ifpdf.sty
  RequirePackage!("xkeyval");
  RequirePackage!("microtype");
  DefMacro!("\\bibfont", "\\normalfont\\small");
  def_macro_noop("\\bibsetup")?;
  def_macro_noop("\\markeverypar")?;
  DefMacro!("\\savetreesbibnote{}", "#1");
});
