use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: amstex.sty.ltxml
  // amstex.sty is obsolete; it is recommended to try to use amsmath instead.
  RequirePackage!("amsmath");
  RequirePackage!("amsfonts");
  RequirePackage!("amsxtra");
  // amstex.sty:877-928 `\matrix`, `\pmatrix`, `\cases`, … open their arrays with no `\matrix@check`: under
  // `\documentstyle{amsart}` (amsart.cls:353-362 loads amstex.sty in compatibility mode) `\pmatrix … \endpmatrix` is
  // no "Old form" error, as amstex.rs has it for AMS-TeX (math0101059, hep-th9309006, math0003076).
  RawTeX!(r"\long\def\lx@ams@matrix@check#1#2#3{#2}");

  DefMacro!("\\nolimits@", "\\nolimits");
  DefMacro!("\\nlimits@", "\\displaylimits");
  // Oddly, different defn than amsopn
  DefMacro!("\\qopname{}{}{}", "\\mathop{#3}\\csname #2limits@\\endcsname");

  DefMacro!("\\Sb", "\\lx@generalized@over{\\Sb}{meaning=substack}");
  DefMacro!("\\Sp", "\\lx@generalized@over{\\Sp}{meaning=superstack}");
  Let!("\\endSb", "\\relax");
  Let!("\\endSp", "\\relax");
});
