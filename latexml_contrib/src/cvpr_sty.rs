//! Stub for cvpr.sty / iccv.sty / iccvw.sty (computer vision conference style).
//!
//! cvpr.sty redefines \title to save the argument in \thetitle so it can
//! be reused (typically by \maketitlesupplementary). Our raw load of
//! cvpr.sty appears not to wire this up reliably; bind cvpr defensively
//! to define \thetitle as a no-op default, plus stub the rebuttal/
//! supplementary frontmatter.
use latexml_package::prelude::*;

LoadDefinitions!({
  // Eager dependency loads — cvpr2025.sty L30-37 lists these as
  // RequirePackage. The raw-load of cvpr*.sty doesn't always execute
  // them via our system, so load them here so user macros like
  // \toprule/\midrule/\bottomrule (booktabs), \includegraphics
  // (graphicx) resolve. Witness 2503.24026 (cvpr2025, \toprule
  // undefined).
  //
  // Pre-load xcolor WITH [dvipsnames, table] options: CVPR papers
  // overwhelmingly use \color{Maroon}/{ForestGreen}/{MidnightBlue}
  // (dvipsnames named palette) AND \cellcolor (colortbl, via the
  // `table` option). If we pre-load xcolor WITHOUT options, the
  // user's later `\usepackage[dvipsnames, table]{xcolor}` becomes
  // an option-clash no-op and dvipsnam.def never gets loaded →
  // 60+ undefined-color errors per CVPR-style paper. Witness 2305.13500.
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);
  RequirePackage!("graphicx");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("booktabs");
  RequirePackage!("natbib");
  RequirePackage!("etoolbox");
  RequirePackage!("hyperref");
  // caption.sty for \captionof — many CVPR templates use this for
  // figure/table sub-captions. Witness 2503.24026 (cvpr2025).
  RequirePackage!("caption");

  // \thetitle: default-empty, gets overridden when user calls \title{...}.
  DefMacro!("\\thetitle", "");
  DefMacro!("\\maketitlesupplementary", "");

  // cvpr.sty supplies these toggles via etoolbox — provide as fallback.
  DefConditional!("\\ifcvprfinal");
  DefConditional!("\\ifcvprrebuttal");
  DefConditional!("\\ifcvprpagenumbers");

  // CV-conference convention abbreviations (cvpr.sty L640-645, iccv.sty
  // similar). Real cvpr.sty defines these via `\def\etal{\emph{et al}\onedot}`
  // and friends. Our path-stripping fallback (`iccv_template/cvpr.sty` →
  // `cvpr.sty.ltxml` registry → our binding) means the InputDefinitions
  // raw-load below is asked to find plain `cvpr.sty` on disk — which is
  // NOT there (the paper bundles it under `iccv_template/`). So \etal etc
  // stay undefined unless we stub them here. Witness 2305.13460: was 5
  // errors (\etal/\ie/\etc/\wrt/\vs), now 0.
  DefMacro!("\\onedot",  "\\@onedot");
  DefMacro!("\\@onedot", ".\\@");  // approximates the spacing trick
  DefMacro!("\\etal",    "\\emph{et al}\\onedot");
  DefMacro!("\\ie",      "\\emph{i.e}\\onedot");
  DefMacro!("\\eg",      "\\emph{e.g}\\onedot");
  DefMacro!("\\cf",      "\\emph{c.f}\\onedot");
  DefMacro!("\\etc",     "\\emph{etc}\\onedot");
  DefMacro!("\\vs",      "\\emph{vs}\\onedot");
  DefMacro!("\\wrt",     "\\emph{w.r.t}\\onedot");
  DefMacro!("\\dof",     "d.o.f\\onedot");

  InputDefinitions!("cvpr", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
