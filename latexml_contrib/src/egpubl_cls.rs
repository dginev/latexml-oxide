//! Stub for egpubl.cls (Eurographics conference proceedings).
use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");

  // Eurographics frontmatter — preserve author-supplied content into
  // ltx:note frontmatter entries with role markers.
  // egpubl.cls:699-703, 766-774: `\teaser{…}` stores its body, which `\@maketitle` sets under the title block with
  // `\def\@captype{figure}`: a figure where `\maketitle` stands (class `ltx_teaserfigure`, as acmart's {teaserfigure}),
  // whose `\caption` and `\label` number and anchor it (2609.00732 `\ref{fig:teaser}`). The PDF sets it before the
  // abstract; the schema keeps the abstract, top matter, ahead of every figure, so it follows the abstract here.
  DefMacro!("\\teaser{}", "\\gdef\\lx@egpubl@teaser{#1}");
  RawTeX!(r"\let\lx@egpubl@teaser\@empty");
  DefMacro!(
    "\\lx@egpubl@maketeaser",
    r"\ifx\lx@egpubl@teaser\@empty\else
\begin{figure}\lx@add@cssclass{ltx_teaserfigure}\lx@egpubl@teaser\end{figure}\global\let\lx@egpubl@teaser\@empty\fi"
  );
  AddToMacro!("\\lx@maketitle@body", "\\lx@egpubl@maketeaser");
  // ORCID id → kernel `\lx@add@orcid` → `ltx:contact[role=orcid]` → clickable
  // orcid.org link + logo icon (vs a bare dagger note). html_feedback#6571.
  DefMacro!("\\orcid{}", "\\lx@add@orcid{#1}");
  DefMacro!(
    "\\ccsdesc[]{}",
    "\\@add@frontmatter{ltx:classification}[scheme=ccs]{#2}"
  );
  def_macro_noop("\\printccsdesc")?;
  // egpubl.cls:986-987 (v4.0, 2609.32222's copy; every shipped copy alike): `\ConfYear` also stores the year in `\p@ConfYear` (`200x` until set),
  // which a conference style reads back (eurovis2014-short.sty:8 `\pdfSubject{\p@ConfName{} (\p@ConfYear)}`; run-347
  // inventory 90 papers; 1409.5623, 1708.07559, 1709.03450, 1806.09174, 2210.05365).
  RawTeX!(r"\gdef\p@ConfYear{200x}");
  DefMacro!(
    "\\ConfYear{}",
    "\\gdef\\p@ConfYear{#1}\\@add@frontmatter{ltx:note}[role=year]{#1}"
  );
  DefMacro!(
    "\\ConfEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors]{#1}"
  );
  // Editor sub-roles — gobble the format-style strings (\ConfEditorStrg
  // formatter) but preserve actual editor lists.
  def_macro_noop("\\ConfEditorStrg{}")?;
  DefMacro!(
    "\\EducationEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-education]{#1}"
  );
  DefMacro!(
    "\\TutorialEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-tutorial]{#1}"
  );
  DefMacro!(
    "\\STARPresEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-star]{#1}"
  );
  DefMacro!(
    "\\DCEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-dc]{#1}"
  );
  DefMacro!(
    "\\ShortPresEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-short]{#1}"
  );
  DefMacro!(
    "\\PosterEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-poster]{#1}"
  );
  DefMacro!(
    "\\EventNoEds{}",
    "\\@add@frontmatter{ltx:note}[role=event-no-eds]{#1}"
  );
  // Bibliography format and version selectors (egpubl.cls:2304-2311, 2515-2519), all without an argument: one read
  // as an argument the paper's next `\ifpdf`, leaving its `\else`/`\fi` unbalanced (2609.00732).
  def_macro_noop("\\biberVersion")?;
  def_macro_noop("\\BibtexOrBiblatex")?;
  def_macro_noop("\\PrintedOrElectronic")?;
  def_macro_noop("\\electronicVersion")?;
  def_macro_noop("\\printedVersion")?;
  def_macro_noop("\\bibtexVersion")?;
  // egpubl.cls:1014-1065: the licence selectors set the title page's copyright text — page furniture.
  for licence in [
    "\\CGFStandardLicense",
    "\\CGFccby",
    "\\CGFccbync",
    "\\CGFccbyncnd",
  ] {
    def_macro_noop(licence)?;
  }
  DefMacro!(
    "\\pdfSubject{}",
    "\\@add@frontmatter{ltx:note}[role=subject]{#1}"
  );
  // Internal counters — gobble (won't be user-content).
  def_macro_noop("\\j@volume{}")?;
  def_macro_noop("\\j@issue{}")?;
  // egpubl.cls:982-985: `\EGyear` stores the year in `\p@EGyear` (`'0x` until set), read back by a style's
  // `\pdfSubject{Pacific Graphics \p@EGyear, …}` (pg2026s.sty, 2609.00732); the year is kept once, from `\ConfYear`
  // ("the year of the conference"), which such a style sets too.
  RawTeX!(r"\gdef\p@EGyear{'0x}\def\EGyear#1{\gdef\p@EGyear{#1}}");
  // egpubl.cls:955-978: the remaining editor setters, kept as the ones above are (a style sets one; egpubl prints it
  // in the venue line, :1373).
  DefMacro!(
    "\\AreasEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-areas]{#1}"
  );
  DefMacro!(
    "\\MedicalPrizeEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-medicalprize]{#1}"
  );
  DefMacro!(
    "\\EuroVisShortPresEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-short]{#1}"
  );
  DefMacro!(
    "\\EuroVisSTARPresEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-star]{#1}"
  );
  DefMacro!(
    "\\EuroVisPosterEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-poster]{#1}"
  );
  DefMacro!(
    "\\EuroVisEducationEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-education]{#1}"
  );
  DefMacro!(
    "\\EuroVisVisGamesEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-visgames]{#1}"
  );
  // egpubl.cls:705-706, 987-993: the title-space length and the conference-name setters a conference style
  // (`pg2026s.sty`: `\ConfName{Pacific Graphics}`) calls; they store the running heads' text, page furniture here.
  RawTeX!(
    r"\newlength{\titlespace}\setlength{\titlespace}{-10pt}
\newcommand\ConfName[1]{\gdef\p@ConfName{#1}}\gdef\p@ConfName{EUROGRAPHICS Workshop on ...}
\newcommand\ConfNameJoint[2]{\gdef\p@ConfNameJoint{#1\\#2}}
\newcommand\ConfNameDH[2]{\gdef\p@ConfNameDH{#1\\#2}}"
  );
  // egpubl.cls:1126-1138, 1163-1852: the page-number and paper-type selectors set the running heads and the title
  // page's venue line — page furniture.
  for selector in [
    "\\noEGpagenumber",
    "\\EGpagenumber",
    "\\noWileypagenumber",
    "\\EGlocalpagenumber",
    "\\Wileypagenumber",
    "\\JournalSubmission",
    "\\JournalPaper",
    "\\ConferenceSubmission",
    "\\ConferencePaper",
    "\\Tutorial",
    "\\STAR",
    "\\Education",
    "\\Poster",
    "\\Areas",
    "\\MedicalPrize",
    "\\ShortPresentation",
    "\\SpecialIssueSubmission",
    "\\SpecialIssuePaper",
    "\\WsSubmission",
    "\\WsPaper",
    "\\WsShortPaper",
    "\\WsPoster",
    "\\WsWiP",
    "\\WsConferencePaper",
    "\\WsSubmissionJoint",
    "\\WsPaperJoint",
    "\\WsPaperJointposter",
    "\\WsPaperJointdemo",
    "\\Expressive",
    "\\DigitalHeritagePaper",
    "\\STAREurovis",
    "\\EuroVisPoster",
    "\\EuroVisShort",
    "\\EuroVisEducation",
    "\\EuroVisVisGames",
    "\\DC",
  ] {
    def_macro_noop(selector)?;
  }

  // The shipped copies' remaining commands papers call (run-347 inventory: 261 egpubl.cls papers with one undefined),
  // as the class defines them (line numbers of v4.0, 2609.32222's copy):
  // - :2063 `\let\shortcite=\cite` (70 papers; 1710.04867, 1311.0119, 1704.04456).
  // - :816 `\CCScat{<class>}{<number>}{<subject>}{<keywords>}`, a CCS category line set in place (42 papers; 1102.4258):
  //   "<number> [<class>]: <subject>---<keywords>" (295 of 299 copies; an older `[3]` form gives `#1 [#2]: #3`).
  // - the editor setters of the later venues, kept as the ones above are (`\EuroVisTutorialEditors`
  //   27 papers; 2604.14791 eurovis2026.sty:12; `\EuroVisPanelsTutorialsEditors` 20; `\EuroVisGDxDREditors`).
  // - :1000-1008 `\copyrightTextTitPag`, the title page's copyright line a paper gives (2404.02634 "\copyright\ 2024 The
  //   Authors. Proceedings published by Eurographics…"): the frontmatter copyright, and stored as the class stores it;
  //   the frontmatter copyright prints its own © (`name="©"`), so a leading `\copyright`/`\textcopyright` (and the
  //   space after it) is not given to it twice (OXIDIZED_DESIGN_DIVERGENCES #599);
  //   `\copyrightTextRunPag`, the running pages' copy, and :1107-1109 `\p@journalSubmissionText`, the submission line:
  //   page furniture, stored. `\pStartPage` (:914) stores the first page number.
  // - the paper-type and licence selectors of single copies (`\PaperL2P` :1744, `\ArxivPaper` 2509.17755,
  //   `\AuthorPreprint`, `\Paper`, `\Preprint`, `\SpecialIssuePreprint`, `\PpConferencePaper`, `\EuroVisGDxDR`,
  //   `\NoStandardLicense`, `\authorCGFStandardLicense`, `\NoLicense`, `\CGFpreprintccby`, `\arxiv`) set the running
  //   heads and copyright text — page furniture, as the selectors above; a copy's guesses that the document's own
  //   definition replaces (`DefinitionOrigin::Fallback`).
  RawTeX!(
    r"\let\shortcite=\cite
\newcommand\CCScat[4]{#2 [#1]: #3---#4}
\newcommand\copyrightTextTitPag[1]{\gdef\p@copyrightTextTitPag{#1}\lx@egpubl@copyright#1\lx@egpubl@cr@end}\gdef\p@copyrightTextTitPag{}
\def\lx@egpubl@copyright{\futurelet\lx@egpubl@cr@next\lx@egpubl@cr@sign}
\def\lx@egpubl@cr@sign{\let\lx@egpubl@cr@do\lx@egpubl@cr@keep
  \ifx\lx@egpubl@cr@next\copyright\let\lx@egpubl@cr@do\lx@egpubl@cr@drop\fi
  \ifx\lx@egpubl@cr@next\textcopyright\let\lx@egpubl@cr@do\lx@egpubl@cr@drop\fi
  \lx@egpubl@cr@do}
\def\lx@egpubl@cr@drop#1{\futurelet\lx@egpubl@cr@next\lx@egpubl@cr@space}
\def\lx@egpubl@cr@space{\let\lx@egpubl@cr@do\lx@egpubl@cr@keep
  \ifx\lx@egpubl@cr@next\ \let\lx@egpubl@cr@do\lx@egpubl@cr@drop@space\fi
  \ifx\lx@egpubl@cr@next~\let\lx@egpubl@cr@do\lx@egpubl@cr@drop@space\fi
  \lx@egpubl@cr@do}
\def\lx@egpubl@cr@drop@space#1{\lx@egpubl@cr@keep}
\def\lx@egpubl@cr@keep#1\lx@egpubl@cr@end{\lx@add@copyright{#1}}
\newcommand\copyrightTextRunPag[1]{\gdef\p@copyrightTextRunPag{#1}}\gdef\p@copyrightTextRunPag{}
\newcommand\p@journalSubmissionText[1]{submitted to COMPUTER GRAPHICS \textit{Forum} #1.}
\newcommand\pStartPage[1]{\gdef\j@pStartPage{#1}}"
  );
  DefMacro!(
    "\\EuroVisTutorialEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-tutorial]{#1}"
  );
  DefMacro!(
    "\\EuroVisPanelsTutorialsEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-panels-tutorials]{#1}"
  );
  DefMacro!(
    "\\EuroVisGDxDREditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors-gdxdr]{#1}"
  );
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    RawTeX!(r"\def\PaperL2P{}");
    for selector in [
      "\\ArxivPaper",
      "\\AuthorPreprint",
      "\\Paper",
      "\\Preprint",
      "\\SpecialIssuePreprint",
      "\\PpConferencePaper",
      "\\EuroVisGDxDR",
      "\\NoStandardLicense",
      "\\authorCGFStandardLicense",
      "\\NoLicense",
      "\\CGFpreprintccby",
      "\\arxiv",
    ] {
      def_macro_noop(selector)?;
    }
  }

  // {CCSXML} env — ACM-style XML metadata block, skipped as egpubl.cls:820 does, with the comment package's
  // `\excludecomment` (as acmart_cls.rs): an environment that digested its body read the XML's `_` in
  // `<concept_id>` as subscripts (2609.00994, 18 errors).
  RequirePackage!("comment");
  RawTeX!(r"\excludecomment{CCSXML}");
});
