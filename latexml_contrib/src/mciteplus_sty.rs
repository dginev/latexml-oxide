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
  // mciteplus.sty:140/775/780-782 — `\mciteSubRef[track]{key}` is a `\ref` to
  // the label the .bbl's sub-reference entries define (achemso-demo).
  RawTeX!(
    r"\def\mcitetrackID{main}\def\@mcitereflabelprefix{MciteSubReferenceLabel}
\def\mciteSubRef{\@ifnextchar[{\@mciteSubRef}{\@mciteSubRef[\mcitetrackID]}}
\def\@mciteSubRef[#1]#2{\ref{\@mcitereflabelprefix:#1:#2}}"
  );
  DefMacro!("\\endmcitethebibliography", "\\endthebibliography");
  DefConditional!("\\ifmciteBstWouldAddEndPunct");
  // mciteplus sends every citation through `\\mciteCiteA` (mciteplus.sty:1020; natbib's commands :1079-1084): a key
  // with a leading `*` (`\\@mciteCheckKey`, :757-762) joins the previous entry's group, written to the aux as a
  // `\\citation` (:764) in its place among the others, so bibtex lists it, but left out of the citation the text prints
  // (:567-568). So every key, the `*` stripped, is cited in order for the bibliography (`\\lx@mark@nocite` where it
  // stands, not `\\nocite`'s mark at `\\end{document}`: an unsorted style numbers them in citation order), and the
  // citation keeps the unstarred ones: `\cite{a,*b,c,*d}` prints [1, 3]. An empty or blank key (`\cite{a,}`, which
  // mciteplus rejects) adds nothing (the `\@empty` sentinel); a starred key before any unstarred one raises
  // mciteplus's error (:641-645). Repro index-bib/mciteplus_starred_keys; DIVERGENCES #427.
  RawTeX!(
    r"\let\lx@mciteplus@bibref\@@bibref
\def\@@bibref#1#2#3#4{\let\lx@mciteplus@heads\@empty\let\lx@mciteplus@all\@empty\let\lx@mciteplus@starred\@empty
  \@for\lx@mciteplus@key:=#2\do{\ifx\lx@mciteplus@key\@empty\else
    \expandafter\lx@mciteplus@sort\lx@mciteplus@key\@empty\@nil\fi}%
  \edef\lx@mciteplus@next{%
    \ifx\lx@mciteplus@starred\@empty\else\noexpand\lx@mark@nocite{\lx@mciteplus@all}\fi
    \noexpand\lx@mciteplus@bibref{\unexpanded{#1}}{\lx@mciteplus@heads}{\unexpanded{#3}}{\unexpanded{#4}}}%
  \lx@mciteplus@next}
\def\lx@mciteplus@sort#1#2\@nil{%
  \ifx*#1\def\lx@mciteplus@starred{*}\lx@mciteplus@add\lx@mciteplus@all{#2}%
    \ifx\lx@mciteplus@heads\@empty\PackageError{mciteplus}{Tail `#2' is declared without a valid head}{You have to
      declare a head citation before entering tail citations in cite list.}\fi
  \else\lx@mciteplus@add\lx@mciteplus@heads{#1#2}\lx@mciteplus@add\lx@mciteplus@all{#1#2}\fi}
\def\lx@mciteplus@add#1#2{\edef\lx@mciteplus@this{#2}\ifx\lx@mciteplus@this\@empty\else
  \ifx#1\@empty\let#1\lx@mciteplus@this\else\edef#1{#1,\lx@mciteplus@this}\fi\fi}"
  );
});
