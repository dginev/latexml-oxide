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
  // subcaption — many CVPR papers use \begin{subfigure}{...} for
  // multi-panel figures without explicit \usepackage{subcaption}.
  // cvpr2025.sty L30-37 doesn't \RequirePackage{subcaption}, but the
  // template's example file does — so authors copy the example and
  // omit the use-package. Witness 2312.03526.
  RequirePackage!("subcaption");

  // \thetitle: default-empty, gets overridden when user calls \title{...}.
  def_macro_noop("\\thetitle")?;
  def_macro_noop("\\maketitlesupplementary")?;

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

  // Raw-load whichever cvpr*.sty the user actually `\usepackage`d. The
  // binding registry routes cvpr / cvpr2023 / cvpr2024 / cvpr2025 ALL to
  // this same load_definitions; each paper's bundled .sty file differs in
  // small ways (e.g. cvpr2024_conference.sty configures cleveref via
  // \AtEndPreamble — which our stub doesn't reproduce). Falling back to a
  // hard-coded "cvpr" name means the paper-bundled `cvpr2024_conference.sty`
  // is never raw-loaded — its `\AtEndPreamble{\usepackage{cleveref}...}` is
  // skipped, leaving `\cref` undefined and triggering "_ in math mode"
  // cascades on every `\cref{fig:foo_bar}`. Witness 2312.06720.
  //
  // \@currname expands to the actual package name being processed (e.g.
  // "cvpr2024_conference"). Use it so the raw-load targets the user's
  // own file. Fall back to "cvpr" if \@currname is somehow empty.
  let currname = gullet::do_expand(T_CS!("\\@currname"))?.to_string();
  let raw_name = if currname.is_empty() { "cvpr".to_string() } else { currname };
  let _ = input_definitions(&raw_name, InputDefinitionOptions {
    extension: Some(Cow::Borrowed("sty")),
    noltxml: true,
    noerror: true,
    ..Default::default()
  });
});
