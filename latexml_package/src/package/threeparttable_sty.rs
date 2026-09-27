use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\TPTminimum",      "4em");
  DefMacro!("\\TPTrlap{}",       "#1");
  DefMacro!("\\TPTtagStyle{}",   "#1");
  DefMacro!("\\TPTnoteLabel{}",  "\\tnote{#1}\\hfil");
  DefMacro!("\\TPTnoteSettings", None);
  // threeparttable.sty:281 `\def\TPToverlap#1{}`: outside the environment a `\tnote` prints
  // nothing (pdflatex drops `4000\tnote{2}` in a plain tabular); `{threeparttable}` lets it to
  // `\relax` (:118) so the note shows. Perl's argument-less `''` (threeparttable.sty.ltxml:24)
  // printed every note (KPE #315; witnesses 2605.26854, 2605.23257).
  DefMacro!("\\TPToverlap{}",    None);

  DefMacro!("\\TPTdoTablenotes", None);

  // We SHOULD be playing games to link up the \tnote to the item...
  DefMacro!("\\tnote{}", "\\TPToverlap{\\textsuperscript{\\TPTtagStyle{#1}}}");
  // threeparttable.sty:110 `\@ifundefined{@captype}{\def\@captype{table}}{}`
  // (and :126 `figure` for `measuredfigure`): a `\caption` inside the
  // environment works outside a float, as the package documents; the bare
  // `#body` binding (Perl threeparttable.sty.ltxml:31,36 identical) dropped
  // it → "`\caption` outside any known float" (threeparttablex). Group-local
  // like the package's `\def`. Guard:
  // `perfect_kernel_batch54::threeparttable_sets_captype_outside_a_float`.
  // threeparttable.sty:107 `\newenvironment{threeparttable}[1][t]`: the optional vertical
  // placement (`\vtop`/`\vbox`/`$\vcenter`, :156-158) is never typeset; without the slot
  // `[b]` was left as a text panel (Perl :31 shares it, KPE #315; witness 2605.04144).
  DefEnvironment!("{threeparttable}[]", "#body",
    before_digest => {
      Digest!("\\@ifundefined{@captype}{\\def\\@captype{table}}{}\\let\\TPToverlap\\relax")?;
    });
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

  // threeparttable.sty:122 `\newenvironment{measuredfigure}[1][t]`, as `{threeparttable}`.
  DefEnvironment!("{measuredfigure}[]", "#body",
    before_digest => {
      Digest!("\\@ifundefined{@captype}{\\def\\@captype{figure}}{}")?;
    });
});
