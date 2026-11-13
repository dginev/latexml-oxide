use crate::prelude::*;

// shapepar.sty: paragraphs typeset in a shape. Raw, `\shapepar`'s automatic
// scaling (shapepar.sty:206-219) measures the shape's area from the text's
// line geometry (`\SH@measline`, :229) and `\loop`s `\multiply … 256` until
// the area is ≥ 1pt — an area LaTeXML's no-layout model leaves at 0, so the
// loop never ends (arsclassica/ArsClassica `\heartpar`, `MemoryBudget`;
// Perl loops too; pdflatex clean). A shaped paragraph is visual layout:
// the binding keeps the text as a plain paragraph and accepts the shape
// specifications. Ready-made wrappers per shapepar.sty:999-1133 (`\diamondpar`
// keeps its `$\diamondsuit$` ornaments). Guard:
// `perfect_kernel_batch54::shapepar_paragraphs_are_plain_paragraphs`.
#[rustfmt::skip]
LoadDefinitions!({
  // `\shapepar[<scale>]{<shape spec>} <text> \par` (:151), `\Shapepar` (:148)
  def_macro_noop("\\shapepar[]{}")?;
  def_macro_noop("\\Shapepar[]{}")?;
  def_macro_noop("\\cutout{}{}")?;
  DefMacro!("\\cutoutsepstretch", ".5");
  for shape in ["\\diamondshape", "\\squareshape", "\\heartshape", "\\circleshape",
                "\\nutshape", "\\hexagonshape", "\\CDlabshape", "\\starshape"] {
    def_macro_noop(shape)?;
  }
  def_macro_noop("\\rectangleshape{}{}")?;
  DefMacro!("\\diamondpar{}", "$\\diamondsuit$ #1 $\\diamondsuit$\\par");
  DefMacro!("\\squarepar{}", "#1\\par");
  DefMacro!("\\heartpar{}", "#1\\par");
  DefMacro!("\\circlepar{}", "#1\\par");
  DefMacro!("\\nutpar{}", "#1\\par");
  DefMacro!("\\hexagonpar{}", "#1\\par");
  DefMacro!("\\starpar{}", "#1\\par");
});
