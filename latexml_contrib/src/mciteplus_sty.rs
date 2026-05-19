use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  Warn!(
    "missing_file",
    "mciteplus.sty",
    "mciteplus.sty is only minimally stubbed and will not be interpreted raw."
  );
  RawTeX!(
    r"\providecommand{\mcitedefaultmidpunct}{;\space}
\providecommand{\mcitedefaultendpunct}{.}
\providecommand{\mcitedefaultseppunct}{\relax}
\providecommand{\mcitedefaultsublistlabel}{\alph{mcitesubitemcount})\space}
\providecommand{\mcitedefaultsublistbegin}{\relax}
\providecommand{\mcitedefaultsublistend}{\relax}
\providecommand{\mcitedefaultmaxwidthbibitemform}{\arabic{mcitebibitemcount}}
\providecommand{\mcitedefaultmaxwidthbibitemforminit}{\mciteorgbibsamplelabel}
\providecommand{\mcitedefaultmaxwidthsubitemform}{\alph{mcitesubitemcount})}
\providecommand{\mcitedefaultmaxwidthsubitemforminit}{a)}
\def\mcitebibsamplelabel{\rule{\mcitemaxwidthbibitem sp}{0.2pt}}
\def\@mciteMacrod{d}
\def\@mciteMacron{n}
\def\@mciteMacros{s}
\def\@mciteMacrob{b}
\def\@mciteMacrof{f}
\def\@mciteMacroh{h}
\def\@mciteMacrobibitem{bibitem}
\def\@mciteMacrosubitem{subitem}"
  );
  def_macro_noop("\\mciteSetBstMidEndSepPunct{}{}{}")?;
  def_macro_noop("\\mciteSetMidEndSepPunct{}{}{}")?;
  def_macro_noop("\\mciteSetBstSublistLabelBeginEnd{}{}{}")?;
  def_macro_noop("\\mcitebstsublistbegin")?;
  def_macro_noop("\\mcitebstsublistend")?;
  def_macro_noop("\\mciteSetBstSublistMode{}")?;
  def_macro_noop("\\mciteSetSublistMode{}")?;
  def_macro_noop("\\mciteSetBstMaxWidthForm[]{}{}")?;
  def_macro_noop("\\mciteSetMaxWidthForm[]{}{}")?;
  def_macro_noop("\\mciteheadlist")?;
  def_macro_noop("\\mciteCitePrehandlerArg")?;
  def_macro_noop("\\mciteDoList{}{}{}")?;
  def_macro_noop("\\mciteExtraDoLists")?;
  DefMacro!("\\EndOfBibitem", "\\relax");
  DefMacro!("\\mciteEndOfBibGroupPresubcloseHook", "\\relax");
  DefMacro!("\\mciteEndOfBibGroupPostsubcloseHook", "\\relax");
  DefMacro!("\\mcitethebibliographyHook", "\\relax");
  DefMacro!("\\mciteBIBdecl", "\\relax");
  DefMacro!("\\mciteBIBenddecl", "\\relax");
  DefMacro!("\\mcitefwdBIBdecl", "\\relax");
  DefMacro!("\\mcitebibitem", "\\bibitem");
  DefMacro!("\\mcitethebibliography", "\\thebibliography");
  DefMacro!("\\endmcitethebibliography", "\\endthebibliography");
  DefConditional!("\\ifmciteBstWouldAddEndPunct");
});
