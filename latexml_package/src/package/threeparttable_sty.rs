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
  //
  // Expand to the FULL `\begin{itemize}`/`\end{itemize}` environment, NOT the raw
  // `\itemize`/`\enditemize` list macros — matching Perl exactly. `\begin{itemize}`
  // performs the list's vertical-mode setup (the env's `\par`/leavevmode); the bare
  // `\itemize` does not, so when `\begin{tablenotes}` is reached in HORIZONTAL mode
  // — e.g. right after `\end{tabular}` under a journal style like `spr-astr-addons`
  // that leaves the table body in horizontal mode — the raw `\itemize` started the
  // list in mode `horizontal`, and its close then cascaded into "Attempt to close a
  // group that switched to mode horizontal due to \itemize" + `\end{table}` can't
  // close (witness 1910.05543: 12 errors, Perl 0). `\begin{itemize}` forces vmode.
  DefMacro!("\\tablenotes[]", "\\begin{itemize}");
  DefMacro!("\\endtablenotes", "\\end{itemize}");

  DefEnvironment!("{measuredfigure}", "#body");
});
