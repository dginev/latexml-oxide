/// Perl: iftex.sty.ltxml — TeX engine detection conditionals
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // eTeX is always "true" for LaTeXML; the pdfTeX/LuaTeX pair follows the
  // opt-in `luatex` latexml.sty profile (user decision 2026-08-31): a
  // LuaLaTeX-authored document converted with
  // `--preload=[…,luatex]latexml.sty` identifies as LuaTeX so its
  // engine-detection branches run; everything else stays the pdfTeX model.
  // Consulting the state value HERE (not a load-time constant) matters
  // because this binding may load after latexml.sty processed its options.
  DefConditional!("\\ifetex", { true });
  DefConditional!("\\ifeTeX", { true });
  DefConditional!("\\ifpdftex", { !lookup_bool("LUATEX_PROFILE") });
  DefConditional!("\\ifPDFTeX", { !lookup_bool("LUATEX_PROFILE") });
  DefConditional!("\\ifluatex", { lookup_bool("LUATEX_PROFILE") });
  DefConditional!("\\ifLuaTeX", { lookup_bool("LUATEX_PROFILE") });
  // All others are false
  DefConditional!("\\ifpdf");
  DefConditional!("\\ifxetex");
  DefConditional!("\\ifXeTeX");
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
  DefConditional!("\\iftutex", { lookup_bool("LUATEX_PROFILE") });
  DefConditional!("\\ifTUTeX", { lookup_bool("LUATEX_PROFILE") });
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
