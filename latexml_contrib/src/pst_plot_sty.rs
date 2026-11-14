//! `pst-plot.sty` — PSTricks function/data plots.
//!
//! The drawing is PostScript (out of scope, like every `\ps…` object in the
//! pstricks binding), but the commands must CONSUME their arguments so no
//! plot expression or coordinate leaks as text: the former refusing stub
//! left `\psplot` & co. undefined (`\psaxes{->}(0,0)(4,3)\psplot{0}{4}{x 2
//! div}` typeset `-¿(0,0)(4,3)…04x 2 div`; 17 corpus manuals, K1 step 3 pass
//! two, batch 56an). ar5iv's `pst-plot.sty.ltxml` is the same refusing stub.
//! Signatures follow pst-plot.tex: `\readdata[opts]{\macro}{file}` (:107-108),
//! `\savedata{\macro}[data]` (:156), `\dataplot`/`\fileplot`/`\listplot
//! [opts]{data}` (:766-808), `\psplot[opts]{min}{max}[algebra]{fn}` (:949-951),
//! `\parametricplot[opts]{min}{max}[opt][opt]{fn}` (:1096-1099), `\psaxes
//! [opts]{arrows}(x0,y0)(x1,y1)(x2,y2)` (the pstricks open-shape shape:
//! optional arrows + coordinate pairs, `\lx@psgobble@shape`).
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // pst-plot.sty:29-31 (its current branch): `\RequirePackage{pstricks}`, `{pst-xkey}`, `{multido}` — a paper's
  // `\multido` loops over the plot's points with no `\usepackage{multido}` of its own (0711.2399: `Error:undefined:
  // \multido` where Perl, reading pst-plot.sty raw, is clean). Guard `perfect_kernel_batch78m::pst_plot_loads_multido`.
  RequirePackage!("pstricks");
  RequirePackage!("pst-xkey");
  RequirePackage!("multido");
  // pst-plot.tex:16-17 marks itself loaded; pstricks-add.tex:26 (and any other
  // `\ifx\PSTplotLoaded\endinput\else\input pst-plot\fi`) must not raw-input
  // the real file over this binding — raw `\psaxes` typesets its tick labels.
  RawTeX!(r"\let\PSTplotLoaded\endinput");
  DefMacro!("\\readdata [] {} {}", "");
  DefMacro!("\\savedata {} []", "");
  DefMacro!("\\dataplot OptionalMatch:* [] {}", "");
  DefMacro!("\\fileplot OptionalMatch:* [] {}", "");
  DefMacro!("\\listplot OptionalMatch:* [] {}", "");
  DefMacro!("\\psplot OptionalMatch:* [] {} {} [] {}", "");
  DefMacro!("\\parametricplot OptionalMatch:* [] {} {} [] [] {}", "");
  DefMacro!("\\psaxes OptionalMatch:* []", "\\lx@psgobble@shape");
});
