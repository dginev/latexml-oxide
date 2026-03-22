/// Perl: iftex.sty.ltxml — TeX engine detection conditionals
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // eTeX and pdfTeX are considered "true" for LaTeXML
  DefConditional!("\\ifetex", { true });
  DefConditional!("\\ifeTeX", { true });
  DefConditional!("\\ifpdftex", { true });
  DefConditional!("\\ifPDFTeX", { true });
  // All others are false
  DefConditional!("\\ifpdf");
  DefConditional!("\\ifxetex");
  DefConditional!("\\ifXeTeX");
  DefConditional!("\\ifluatex");
  DefConditional!("\\ifLuaTeX");
  DefConditional!("\\ifluahbtex");
  DefConditional!("\\ifLuaHBTeX");
  DefConditional!("\\ifptex");
  DefConditional!("\\ifpTeX");
  DefConditional!("\\ifuptex");
  DefConditional!("\\ifupTeX");
  DefConditional!("\\ifptexng");
  DefConditional!("\\ifpTeXng");
  DefConditional!("\\ifvtex");
  DefConditional!("\\ifVTeX");
  DefConditional!("\\ifalephtex");
  DefConditional!("\\ifAlephTeX");
  DefConditional!("\\iftutex");
  DefConditional!("\\ifTUTeX");
  DefConditional!("\\iftexpadtex");
  DefConditional!("\\ifTexpadTeX");
  DefConditional!("\\ifhint");
  DefConditional!("\\ifHINT");

  // \Require* macros — all no-ops
  for cs in [
    "\\RequireeTeX", "\\RequirePDFTeX", "\\RequireXeTeX",
    "\\RequireLuaTeX", "\\RequireLuaHBTeX", "\\RequirepTeX",
    "\\RequireupTeX", "\\RequirepTeXng", "\\RequireVTeX",
    "\\RequireAlephTeX", "\\RequireTUTeX", "\\RequireTexpadTeX",
    "\\RequireHINT",
  ] {
    DefMacro!(T_CS!(cs), None, None);
  }
});
