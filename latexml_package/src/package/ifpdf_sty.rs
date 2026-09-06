//! ifpdf.sty — PDF mode detection.
//! Perl: ifpdf.sty.ltxml (`\newif\ifpdf\pdffalse`, a static FALSE).
use crate::prelude::*;

LoadDefinitions!({
  // ifpdf.sty v3.4 is literally `\RequirePackage{iftex}` (plus the legacy
  // `\pdftrue`/`\pdffalse` aliases iftex.sty:269-270 itself defines): one
  // source of truth for `\ifpdf`. Perl's static `\pdffalse` contradicted the
  // profile-aware `iftex` value — tufte-common.def:266 `\RequirePackage{ifpdf}`
  // loaded this binding first, so tikzrput.sty:66 `\ifpdf…\def\rput…\fi`
  // never defined `\rput` under the luatex profile (pgfornament ornaments,
  // tikzrput). Guard: `perfect_kernel_batch56::ifpdf_delegates_to_iftex`.
  RequirePackage!("iftex");
});
