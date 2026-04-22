use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "mdframed.sty",
    "mdframed.sty is only minimally stubbed and will not be interpreted raw."
  );
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
  // Perl ar5iv-bindings/mdframed.sty.ltxml L31-34: wrap body in an
  // inline-block with framed="rectangle" and framecolor from the
  // current font. Rust port drops the framecolor properties closure —
  // font color plumbing is available but not exposed in DefEnvironment
  // properties closures yet — so the body renders as an
  // unbordered-color rectangle rather than a transparent inline block.
  DefEnvironment!(
    "{mdframed}[]",
    "<ltx:inline-block framed='rectangle' _noautoclose='1'>#body</ltx:inline-block>"
  );
});
