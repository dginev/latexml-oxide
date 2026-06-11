use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("elsart_support");
  RequirePackage!("amssymb");
  RequirePackage!("bm");
  RequirePackage!("keyval");
  RequirePackage!("hyperref");
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
  Let!("\\longlist", "\\list");
  Let!("\\endlonglist", "\\endlist");
  DefMacro!(
    "\\MR{}",
    "\\href{http://www.ams.org/mathscinet-getitem?mr=#1}{MR#1}"
  );
  RawTeX!(r"\expandafter\def\csname remark*\endcsname{\begin{remark}}");
  RawTeX!(r"\expandafter\def\csname endremark*\endcsname{\end{remark}}");
});
