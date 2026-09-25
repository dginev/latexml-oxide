//! underscore-ltx.sty — the LaTeX kernel's first aid for underscore.sty.
//!
//! The kernel loads it right after underscore.sty
//! (latex2e-first-aid-for-external-files.ltx:176-177); Perl has no binding, so
//! its active `_` stays underscore.sty's `\ifmmode\sb\else\textunderscore\fi`.
//! The first-aid `_` is `\protected` and a literal `_` inside a `\csname`
//! (underscore-ltx.sty:38-51), so a file name keeps its underscore:
//! `\input{sections/logic_T}` read `sections/logic\textunderscoreT` (arXiv
//! 2606.31852, once PDF output ran its `\ifpdf\usepackage{underscore}`; the
//! file name is expanded as `\set@curr@file` does it,
//! `gullet::expand_as_csname_text`). The `\lowercase` turns `~` into the active
//! `_` the body needs. Guard: `class_census::underscore_first_aid_file_name`.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RawTeX!(r"\begingroup\lccode`\~=`\_\lowercase{\endgroup
\protected\gdef~{\ifincsname\string~\else\ifx\protect\@typeset@protect\ifmmode\sb\else\BreakableUnderscore\fi\else\ifx\protect\@unexpandable@protect\noexpand~\else\protect~\fi\fi\fi}%
\global\let\ActiveUnderscore=~}");
});
