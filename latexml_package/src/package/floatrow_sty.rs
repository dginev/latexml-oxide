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
