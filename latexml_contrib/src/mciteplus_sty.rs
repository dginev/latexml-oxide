use latexml_package::prelude::*;

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
  DefMacro!("\\mciteSetBstMidEndSepPunct{}{}{}", "");
  DefMacro!("\\mciteSetMidEndSepPunct{}{}{}", "");
  DefMacro!("\\mciteSetBstSublistLabelBeginEnd{}{}{}", "");
  DefMacro!("\\mcitebstsublistbegin", "");
  DefMacro!("\\mcitebstsublistend", "");
  DefMacro!("\\mciteSetBstSublistMode{}", "");
  DefMacro!("\\mciteSetSublistMode{}", "");
  DefMacro!("\\mciteSetBstMaxWidthForm[]{}{}", "");
  DefMacro!("\\mciteSetMaxWidthForm[]{}{}", "");
  DefMacro!("\\mciteheadlist", "");
  DefMacro!("\\mciteCitePrehandlerArg", "");
  DefMacro!("\\mciteDoList{}{}{}", "");
  DefMacro!("\\mciteExtraDoLists", "");
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
