//! Stub for iccv.sty / iccvw.sty (ICCV conference style).
//!
//! Same \thetitle pattern as cvpr.sty.
use latexml_package::prelude::*;

LoadDefinitions!({
  def_macro_noop("\\thetitle")?;
  def_macro_noop("\\maketitlesupplementary")?;
  DefConditional!("\\ificcvfinal");
  DefConditional!("\\ificcvrebuttal");
  DefConditional!("\\ificcvpagenumbers");
  // \iccvfinalcopy / \iccvPaperID — page-numbering toggles in ICCV
  // templates. Affect print layout only; HTML rendering is invariant.
  // Witness 2 stage-2 papers.
  def_macro_noop("\\iccvfinalcopy")?;
  def_macro_noop("\\iccvPaperID{}")?;

  // CV-conference convention abbreviations (iccv.sty L254+). Same
  // rationale as in contrib/cvpr_sty.rs — when the path-stripping
  // fallback (`iccv2023AuthorKit/iccv.sty` → our binding) takes over,
  // the InputDefinitions raw-load below fails to find plain `iccv.sty`
  // on disk (bundled under a subdir), so \etal/\ie/\eg/\etc/\vs/\wrt/\dof
  // stay undefined. Stub them directly. Witness 2305.20091 (iccv2023
  // AuthorKit paper): 3 errors (\ie/\eg/\etal) → 0.
  DefMacro!("\\onedot", "\\@onedot");
  DefMacro!("\\@onedot", ".\\@");
  DefMacro!("\\etal", "\\emph{et al}\\onedot");
  DefMacro!("\\ie", "\\emph{i.e}\\onedot");
  DefMacro!("\\eg", "\\emph{e.g}\\onedot");
  DefMacro!("\\cf", "\\emph{c.f}\\onedot");
  DefMacro!("\\etc", "\\emph{etc}\\onedot");
  DefMacro!("\\vs", "\\emph{vs}\\onedot");
  DefMacro!("\\wrt", "\\emph{w.r.t}\\onedot");
  DefMacro!("\\dof", "d.o.f\\onedot");

  InputDefinitions!("iccv", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
