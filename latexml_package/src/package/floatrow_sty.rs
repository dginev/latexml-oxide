use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // floatrow.sty itself lays out each floatbox: its caption and its `\floatfoot` text go to its
  // own boxes, `\@floatcapt` (filled by `\flrow@makecaption`, floatrow.sty:85-102) and
  // `\flrow@foot`, and the layout (`\flrow@FB@`, `\flrow@FC@`, :363-418) places the foot only
  // where the caption box is not void. LaTeXML's locked `\@caption` never calls `\@makecaption`,
  // so the box stayed void and every foot was lost; the kernel's caption material goes through
  // `\lx@setfloatcapt` (sect09.rs), which a floatbox points at `\@floatcapt`.
  InputDefinitions!("floatrow", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // floatrow.sty:34-37 marks float.sty and rotfloat.sty loaded (`\@namedef{ver@float.sty}`: it carries its own excerpt
  // of both), so LaTeX skips a later `\usepackage{float}` (latex.ltx `\@onefilewithoptions`). Our loader keys on its
  // own record, not on `\ver@`, so the binding records the same; otherwise float's binding replaced floatrow's
  // `\float@end` and every restyled float stayed open (2010.00977, 2003.11778, 2011.10793, 2105.04869;
  // KNOWN_PERL_ERRORS #668; user ruling 2026-10-14: floatrow's marks, no general `\ver@` rule). Guard:
  // `perfect_kernel_batch66v::floatrow_marks_float_loaded`.
  AssignValue!("float.sty_loaded" => true, Some(Scope::Global));
  AssignValue!("rotfloat.sty_loaded" => true, Some(Scope::Global));

  // `\FBget@box` runs once per floatbox (`\@@@floatbox`, :776), in its group, before every
  // measuring pass and the real one (`\FBsetbox@obj`); each pass refills `\@floatcapt`, as
  // `\flrow@makecaption` does. Sub-floats reset the capture at their begin (`begin_float`).
  // `\RawCaption` keeps the plain caption (:252). Guards
  // `perfect_kernel_batch58::{floatrow_floatfoot_keeps_its_text, floatrow_subcaption_stays_in_its_panel}`.
  // The capture is `\flrow@makecaption`'s box (floatrow.sty:88-102): the caption box starts from
  // the paragraph and font defaults, so a size switch in the object does not reach it.
  RawTeX!(r"\def\lx@flrow@setfloatcapt#1{\global\setbox\@floatcapt\vbox\bgroup\@parboxrestore\reset@font
  \if@@FS\ifdim\FBc@wd>\z@\hsize\FBc@wd\else\adj@dim\hsize+\FBo@wadj=\hsize\fi\fi
  \linewidth\hsize\ifdim\hsize<70mm\sloppy\fi\normalsize\abovecaptionskip\z@\belowcaptionskip\z@#1\egroup}
\let\lx@flrow@FBget@box\FBget@box
\def\FBget@box{\let\lx@setfloatcapt\lx@flrow@setfloatcapt\lx@flrow@FBget@box}
\def\RawCaption#1{{\let\@makecaption\FR@makecaption\let\lx@setfloatcapt\lx@kernel@setfloatcapt#1}}");
});
