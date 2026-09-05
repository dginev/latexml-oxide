use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RawTeX!("\\newenvironment{restatable}[3][]{\\begin{#2}[#1]\\label{restatable:#3}\\expandafter\\gdef\\csname #3\\endcsname{\\lx@thm@restate{#3}\\@ifstar{}{}}}{\\end{\\@currenvir}}");
  // thm-restate.sty:191 `restatable*` — same statement, the star only
  // suppresses the inline printing at the restating site (proof-at-the-end
  // demo). Perl thm-restate.sty.ltxml:18 omits it too.
  RawTeX!("\\newenvironment{restatable*}[3][]{\\begin{#2}[#1]\\label{restatable:#3}\\expandafter\\gdef\\csname #3\\endcsname{\\lx@thm@restate{#3}\\@ifstar{}{}}}{\\end{\\@currenvir}}");
  DefMacro!("\\lx@thm@restate{}", "See \\ref{restatable:#1}");
});
