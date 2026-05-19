use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

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
