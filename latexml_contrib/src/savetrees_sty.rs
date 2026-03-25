use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("ifluatex");
  // No effect from ifpdf.sty
  RequirePackage!("xkeyval");
  RequirePackage!("microtype");
  DefMacro!("\\bibfont", "\\normalfont\\small");
  DefMacro!("\\bibsetup", "");
  DefMacro!("\\markeverypar", "");
  DefMacro!("\\savetreesbibnote{}", "#1");
});
