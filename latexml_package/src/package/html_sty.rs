use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("hyperref");
  DefMacro!("\\latextohtml",                              "\\LaTeX2\\texttt{HTML}");
  DefMacro!("\\htmladdnormallinkfoot{}{}",                "\\href{#2}{#1}");
  DefMacro!("\\htmladdnormallink{}{}",                    "\\href{#2}{#1}");
  DefMacro!("\\htmladdimg{}",                             "\\hyperimage{#1}");
  DefMacro!("\\externallabels Semiverbatim Semiverbatim", "");
  DefMacro!("\\externalref{}",                            "");
  DefMacro!("\\externalcite",                             "\\nocite");
  DefMacro!("\\htmladdTOClink[]{}{}{}",                   "");
  DefConstructor!("\\htmlrule OptionalMatch:*", "<ltx:rule/>");
  DefConstructor!("\\HTMLrule OptionalMatch:*", "<ltx:rule/>");
  DefConstructor!("\\htmlclear",                "<ltx:br/>");
  DefMacro!("\\bodytext{}", "");
  DefMacro!("\\htmlbody",   "");
  DefMacro!("\\htmlcite{}{}", "");
  DefMacro!("\\htmlimage{}", "");
  DefMacro!("\\htmlborder{}", "");
  DefMacro!("\\htmladdtonavigation{}", "");
  DefMacro!("\\html{}", "");
  DefMacro!("\\latex{}",          "#1");
  DefMacro!("\\latexhtml{}{}",    "#1");
  DefMacro!("\\strikeout{}",      "#1");
  DefMacro!("\\htmlurl Semiverbatim", "\\url{#1}");
  DefMacro!("\\HTMLset{}{}",              "");
  DefMacro!("\\htmlinfo OptionalMatch:*", "");
});
