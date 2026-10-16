//! World Scientific proceedings and review-volume classes (ws-procs9x6, ws-procs961x669, ws-rv9x6): the ws journal
//! binding, and the `\body` these three classes add to it.
use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};
use latexml_package::{prelude::*, shipped_class::shipped_copy_text};

use crate::shipped_class::{class_code_contains, require_shipped_packages};

LoadDefinitions!({
  crate::ws_journal_cls::load_definitions()?;
  // ws-procs9x6.cls:586-591 (ws-procs961x669.cls:596-601, ws-rv9x6.cls:1011 the same; the journal classes define
  // neither, so a paper's own `\newcommand{\body}` stays theirs): `\body` (and its alias `\bodymatter`) opens the main
  // matter, restarting the footnotes and lettering them; the class's `\@makefnmark` there (a roman superscript) is the
  // printed mark's look only. Witnesses 0704.0883, 1008.0906, 1903.10424, 2301.12666, 1308.0373, 2202.11015
  // (ws-procs961x669), 0902.1904 (ws-rv9x6).
  RawTeX!(
    r"\def\body{\setcounter{footnote}{0}\def\thefootnote{\alph{footnote}}}
\def\bodymatter{\body}"
  );
  // ws-rv9x6.cls:27 the chapter-numbering switch `\if@addchapnum` (the class's `addchapnum` option, :44, sets it;
  // numbering is presentation here), which papers test in their own numbering macros (run 340: 113 papers; witnesses
  // 2103.07802, 2008.12305, 1609.07109).
  RawTeX!(r"\newif\if@addchapnum");
  // ws-rv9x6.cls:1524-1528 (0706.1148's copy) `\usepackage[figuresright]{rotating}`, `{amsfonts,amssymb,amsmath}`,
  // `{boxedminipage}`, `{makeidx}`, `{url}`: a review-volume chapter ends with `\printindex` without loading makeidx
  // (33 ws-rv9x6 papers of run 353; 0706.1148, 0810.2192, 0901.1361). The paper's copy decides, for these three classes
  // alike; without one, none.
  require_shipped_packages("cls", &["rotating", "boxedminipage", "makeidx", "url"])?;
  // The three classes' cross-reference shorthands (ws-procs9x6.cls v1.2u :984-993 as 0704.0369 ships it,
  // ws-procs961x669.cls :1139-1148, ws-rv9x6.cls :1234-1243): `\eref` "Eq. (n)", `\sref` "Sec. n", `\fref` "Fig. n",
  // `\tref` "Table n" and the capitalised `\Eref`…`\Tref` (run 347: 13 papers call `\fref`, 11 `\eref`; 0704.0369,
  // 1605.04791, 1110.2544); `\figurecaptionfont` (:79; 1911.07780); `\wstoc{title}{authors}` (:338-344), a proceedings
  // volume's table-of-contents line for the paper, kept out of the document as its `\\addtocontents` lines are (16
  // papers: 1605.04791, 2110.00344); `\mastertoc` (:218), the
  // volume's contents (2002.12333); `{unnumlist}` (:548), a list whose items carry no label (1402.0787, 1306.3110); the
  // theorem-like `{algorithm}` and `{assumption}` (:918-921, numbered within the section; 1109.0078, 2512.24879); and
  // `\aindx{name}`, an author-index entry — ws-procs `\index{author}{#1}` (:846, ws-multind's two-argument `\index`;
  // 0809.4982, 1607.03235), ws-rv9x6 `\index[aindx]{#1}` (:1155), as the paper's copy writes it. The cross-reference
  // names stay a document's own where its class lacks them (`DefinitionOrigin::Fallback`: cleveref's `\cref`, which the
  // classes also define, is left to cleveref).
  {
    let _class = OriginGuard::new(DefinitionOrigin::Fallback);
    RawTeX!(
      r"\newcommand{\eref}[1]{Eq.~(\ref{#1})}
\newcommand{\sref}[1]{Sec.~\ref{#1}}
\newcommand{\fref}[1]{Fig.~\ref{#1}}
\newcommand{\tref}[1]{Table~\ref{#1}}
\newcommand{\Eref}[1]{Equation (\ref{#1})}
\newcommand{\Sref}[1]{Section~\ref{#1}}
\newcommand{\Fref}[1]{Figure~\ref{#1}}
\newcommand{\Tref}[1]{Table~\ref{#1}}
\def\figurecaptionfont{\footnotesize}
\newcounter{ptitl}
\def\wstoc#1#2{\refstepcounter{ptitl}}
\newcommand\mastertoc{\tableofcontents}
\newenvironment{unnumlist}{\list{}{}}{\endlist}
\@ifundefined{algorithm}{\newtheorem{algorithm}{Algorithm}[section]}{}
\@ifundefined{assumption}{\newtheorem{assumption}{Assumption}[section]}{}"
    );
    if shipped_copy_text("cls")?.is_some_and(|text| class_code_contains(&text, "\\index[aindx]")) {
      RawTeX!(r"\def\aindx#1{\index[aindx]{#1}}");
    } else {
      RawTeX!(r"\def\aindx#1{\index{author}{#1}}");
    }
  }
});
