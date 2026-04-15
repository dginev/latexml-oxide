//! pstricks.sty — PSTricks graphics package (stubs)
//! PSTricks requires DVI backend; we just stub commands to prevent errors.
//! Perl: pstricks.sty.ltxml (44L) + pstricks_support.sty.ltxml (1057L)
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("xcolor");

  // Core PSTricks parameter setting
  DefMacro!("\\psset{}", "");
  DefMacro!("\\newpsobject{}{}{}", "");
  DefMacro!("\\newpsstyle{}{}", "");

  // Drawing commands — all no-ops for HTML
  DefMacro!("\\psline OptionalMatch:* []{}", "");
  DefMacro!("\\psframe OptionalMatch:* []{}", "");
  DefMacro!("\\pscircle OptionalMatch:* []{}{}", "");
  DefMacro!("\\psarc OptionalMatch:* []{}{}{}{}", "");
  DefMacro!("\\psbezier OptionalMatch:* []{}", "");
  DefMacro!("\\pscurve OptionalMatch:* []{}", "");
  DefMacro!("\\psecurve OptionalMatch:* []{}", "");
  DefMacro!("\\psccurve OptionalMatch:* []{}", "");
  DefMacro!("\\parabola OptionalMatch:* []{}{}", "");
  DefMacro!("\\pspolygon OptionalMatch:* []{}", "");
  DefMacro!("\\psdots OptionalMatch:* []{}", "");
  DefMacro!("\\psdot OptionalMatch:* []{}", "");
  DefMacro!("\\qline{}{}", "");
  DefMacro!("\\qdisk{}{}", "");

  // Text placement — pass through the text content
  DefMacro!("\\rput OptionalMatch:* []{}{}", "#3");
  DefMacro!("\\uput OptionalMatch:* []{}{}{}", "#4");
  DefMacro!("\\cput OptionalMatch:* []{}{}", "#3");

  // Box commands
  DefMacro!("\\psframebox OptionalMatch:* []{}", "#2");
  DefMacro!("\\psshadowbox OptionalMatch:* []{}", "#2");
  DefMacro!("\\pscirclebox OptionalMatch:* []{}", "#2");
  DefMacro!("\\psovalbox OptionalMatch:* []{}", "#2");
  DefMacro!("\\psdblframebox OptionalMatch:* []{}", "#2");

  // Environment
  DefEnvironment!("{pspicture} OptionalMatch:* []{}", "#body");
  DefEnvironment!("{pspicture*} OptionalMatch:* []{}", "#body");

  // Grid
  DefMacro!("\\psgrid OptionalMatch:* []{}", "");

  // Misc
  DefMacro!("\\pscustom OptionalMatch:* []{}", "");
  DefMacro!("\\psclip{}", "");
  DefMacro!("\\endpsclip", "");
  DefMacro!("\\SpecialCoor", "");
  DefMacro!("\\NormalCoor", "");
  DefMacro!("\\degrees[]", "");
  DefMacro!("\\radians", "");
});
