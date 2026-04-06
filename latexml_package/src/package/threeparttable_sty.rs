use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\TPTminimum",      "4em");
  DefMacro!("\\TPTrlap{}",       "#1");
  DefMacro!("\\TPTtagStyle{}",   "#1");
  DefMacro!("\\TPTnoteLabel{}",  "\\tnote{#1}\\hfil");
  DefMacro!("\\TPTnoteSettings", None);
  DefMacro!("\\TPToverlap",      None);

  DefMacro!("\\TPTdoTablenotes", None);

  // We SHOULD be playing games to link up the \tnote to the item...
  DefMacro!("\\tnote{}", "\\TPToverlap{\\textsuperscript{\\TPTtagStyle{#1}}}");
  DefEnvironment!("{threeparttable}", "#body");
  // optional keyvals: para,flushleft, online, normal
  // tablenotes env — maps to itemize list.
  // Note: DefMacro!("\\begin{tablenotes}"...) wrongly parses {tablenotes} as param spec.
  Let!("\\tablenotes", "\\itemize");
  Let!("\\endtablenotes", "\\enditemize");

  DefEnvironment!("{measuredfigure}", "#body");
});
