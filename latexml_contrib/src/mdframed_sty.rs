use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!("missing_file", "mdframed.sty",
    "mdframed.sty is only minimally stubbed and will not be interpreted raw.");
  RequirePackage!("kvoptions");
  RequirePackage!("xparse");
  RequirePackage!("etoolbox");
  RequirePackage!("xcolor");
  DefMacro!("\\newmdtheoremenv[]{}{}[]", "");
  DefMacro!("\\newmdenv[]{}", "");
  DefMacro!("\\renewmdenv[]{}", "");
  DefMacro!("\\surroundwithmdframed[]{}", "");
  DefMacro!("\\mdfsubtitle[]{}", "");
  DefMacro!("\\mdfapptodefinestyle{}{}", "");
  DefMacro!("\\mdfsetup{}", "");
  DefMacro!("\\mdfdefinestyle{}{}", "");
  DefRegister!("\\mdflength" => Dimension::new(0));
  // TODO: Perl has DefEnvironment for {mdframed}[] with inline-block framed="rectangle"
  // and framecolor from current font color. Stubbed as empty for now.
  DefEnvironment!("{mdframed}[]", "#body");
});
