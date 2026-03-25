use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RawTeX!("\\newenvironment{restatable}[3][]{\\begin{#2}[#1]\\label{restatable:#3}\\expandafter\\gdef\\csname #3\\endcsname{\\lx@thm@restate{#3}\\@ifstar{}{}}}{\\end{\\@currenvir}}");
  DefMacro!("\\lx@thm@restate{}", "See \\ref{restatable:#1}");
});
