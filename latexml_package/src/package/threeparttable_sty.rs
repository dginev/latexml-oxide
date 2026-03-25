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
  DefMacro!("\\begin{tablenotes}[]",  "\\begin{itemize}");
  DefMacro!("\\end{tablenotes}",      "\\end{itemize}");

  DefEnvironment!("{measuredfigure}", "#body");
});
