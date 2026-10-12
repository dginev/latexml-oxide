use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // thm-restate.sty:26 `\RequirePackage{thmtools}`: `\usepackage{thm-restate}` alone brings `\declaretheorem`,
  // `\declaretheoremstyle` and the rest of thmtools (witnesses 2412.02079, 2411.02602). Perl
  // thm-restate.sty.ltxml loads nothing (KNOWN_PERL_ERRORS #629). thmtools requires thm-restate in turn
  // (thmtools.sty:47-49); the cycle ends at the package already in progress, as `\ver@thmtools.sty` ends it in LaTeX.
  RequirePackage!("thmtools");
  RawTeX!("\\newenvironment{restatable}[3][]{\\begin{#2}[#1]\\label{restatable:#3}\\expandafter\\gdef\\csname #3\\endcsname{\\lx@thm@restate{#3}\\@ifstar{}{}}}{\\end{\\@currenvir}}");
  // thm-restate.sty:191 `restatable*` — same statement, the star only
  // suppresses the inline printing at the restating site (proof-at-the-end
  // demo). Perl thm-restate.sty.ltxml:18 omits it too.
  RawTeX!("\\newenvironment{restatable*}[3][]{\\begin{#2}[#1]\\label{restatable:#3}\\expandafter\\gdef\\csname #3\\endcsname{\\lx@thm@restate{#3}\\@ifstar{}{}}}{\\end{\\@currenvir}}");
  DefMacro!("\\lx@thm@restate{}", "See \\ref{restatable:#1}");
});
