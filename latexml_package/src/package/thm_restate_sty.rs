use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // thm-restate.sty:26 `\RequirePackage{thmtools}`: `\usepackage{thm-restate}` alone brings `\declaretheorem`,
  // `\declaretheoremstyle` and the rest of thmtools (witnesses 2412.02079, 2411.02602). Perl
  // thm-restate.sty.ltxml loads nothing (KNOWN_PERL_ERRORS #629). thmtools requires thm-restate in turn
  // (thmtools.sty:47-49); the cycle ends at the package already in progress, as `\ver@thmtools.sty` ends it in LaTeX.
  RequirePackage!("thmtools");
  // A theorem stored for `\<name>`: labelled for the reference the restated copy is here, and `\<name>` (or
  // `\<name>*`) that reference. `{restatable}` stores its theorem so, and so does a theorem's `restate=<name>` key
  // (thm-restate.sty:198-215; latex_constructs `thmtools_use_keys`; 2108.11527, 2602.09462).
  RawTeX!("\\def\\lx@thm@restate@store#1{\\label{restatable:#1}\\expandafter\\gdef\\csname #1\\endcsname{\\lx@thm@restate{#1}\\@ifstar{}{}}}");
  RawTeX!("\\newenvironment{restatable}[3][]{\\begin{#2}[#1]\\lx@thm@restate@store{#3}}{\\end{\\@currenvir}}");
  // thm-restate.sty:191 `restatable*` — same statement, the star only
  // suppresses the inline printing at the restating site (proof-at-the-end
  // demo). Perl thm-restate.sty.ltxml:18 omits it too.
  RawTeX!("\\newenvironment{restatable*}[3][]{\\begin{#2}[#1]\\lx@thm@restate@store{#3}}{\\end{\\@currenvir}}");
  DefMacro!("\\lx@thm@restate{}", "See \\ref{restatable:#1}");
});
