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
  // Perl L30: DefMacroI('\begin{tablenotes}', '[]', '\begin{itemize}');
  // ie the {tablenotes} env optionally takes [keyvals] (para/flushleft/online/normal)
  // and discards them — the body is just an itemize list. Previously Rust used
  // Let (which couldn't absorb the optional arg); switch to DefMacro with an
  // explicit [] parameter slot so `\begin{tablenotes}[para]` no longer leaks
  // `[para]` into the itemize input stream.
  DefMacro!("\\tablenotes[]", "\\itemize");
  DefMacro!("\\endtablenotes", "\\enditemize");

  DefEnvironment!("{measuredfigure}", "#body");
});
