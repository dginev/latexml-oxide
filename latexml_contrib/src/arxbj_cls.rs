use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("elsart_support");
  RequirePackage!("amssymb");
  RequirePackage!("bm");
  RequirePackage!("keyval");
  RequirePackage!("hyperref");
  // The VTeX IMS markup arxbj.cls shares with arximspdf.cls: `{barticle}`/`\b*` (:2599-2897), `{pf}` with `\qed`
  // (:1428-1445), `\tablewidth` (:1684) and `\bolds` (1003.1189, 1203.0186).
  RequirePackage!("ims_support");
  def_macro_noop("\\pdftitle {}")?;
  def_macro_noop("\\pdfauthor {}")?;
  def_macro_noop("\\pdfsubject {}")?;
  def_macro_noop("\\pdfkeywords {}")?;
  def_macro_noop("\\printhistory")?;
  // Motivated by arXiv:1102.2078
  DefMacro!("\\tfrac{}{}", "{\\textstyle\\frac{#1}{#2}}");
  DefMacro!("\\dfrac{}{}", "{\\displaystyle\\frac{#1}{#2}}");
  DefMacro!("\\dvt", "\\colon\\ ");
  DefMacro!(
    "\\dvtx",
    "\\mathchoice{\\nobreak\\,\\colon\\relax}%\n{\\nobreak\\,\\colon\\relax}%\n{\\nobreak\\,\\colon\\;\\relax}%\n{\\nobreak\\,\\colon\\;\\relax}%"
  );
  // `{longlist}` is the class's own list (arxbj.cls:930-958, labels re-set at :1070): items counted `longlist`, labelled
  // "(i)", "(ii)", … with an optional widest-label argument. A bare `\list` (the ar5iv-bindings Perl `\let`) took the
  // first `\item` as its label argument, so every item's label was `\item` again: an endless recursion (1203.0186,
  // 1003.1189; Perl deep_recursion too).
  RawTeX!(
    r"\newcounter{longlist}
\def\thelonglist{\@roman\c@longlist}
\def\labellonglist{(\thelonglist)}
\def\longlist{\@ifnextchar[{\@longlist}{\@longlist[]}}
\def\@longlist[#1]{\list{\labellonglist}{\usecounter{longlist}}}
\let\endlonglist\endlist"
  );
  DefMacro!(
    "\\MR{}",
    "\\href{http://www.ams.org/mathscinet-getitem?mr=#1}{MR#1}"
  );
  RawTeX!(r"\expandafter\def\csname remark*\endcsname{\begin{remark}}");
  RawTeX!(r"\expandafter\def\csname endremark*\endcsname{\end{remark}}");
});
