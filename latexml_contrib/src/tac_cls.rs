//! Stub for tac.cls (Theory and Applications of Categories journal).
//!
//! tac.cls is a paper-bundled class from TAC (theory + applications of
//! categories). It extends article with custom AMS subject-classification
//! commands (`\amsclass`, `\subjclass`) and journal frontmatter.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("article");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");

  // tac.cls L339: \amsclass{<codes>} — preamble AMS subject classification.
  // L341: \let\subjclass\amsclass. Witness 2312.12356.
  DefMacro!(
    "\\amsclass{}",
    "\\@add@frontmatter{ltx:classification}[scheme=AMS]{#1}"
  );
  Let!("\\subjclass", "\\amsclass");
  // tac.cls custom frontmatter helpers — preserve author content as
  // ltx:note where possible, no-op otherwise.
  DefMacro!(
    "\\keywords{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}"
  );
  DefMacro!(
    "\\runninghead{}",
    "\\@add@frontmatter{ltx:note}[role=runninghead]{#1}"
  );
  DefMacro!(
    "\\volyear{}",
    "\\@add@frontmatter{ltx:note}[role=volume-year]{#1}"
  );
  DefMacro!(
    "\\volnum{}",
    "\\@add@frontmatter{ltx:note}[role=volume-number]{#1}"
  );
  // tac.cls L317: \address{...} — author postal address note.
  DefMacro!(
    "\\address{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#1}"
  );
  // tac.cls L322: \eaddress{...} — email/electronic address note.
  DefMacro!(
    "\\eaddress{}",
    "\\@add@frontmatter{ltx:note}[role=email]{#1}"
  );
  // tac.cls L337: \copyrightyear{...} — journal frontmatter copyright year.
  DefMacro!(
    "\\copyrightyear{}",
    "\\@add@frontmatter{ltx:note}[role=copyright-year]{#1}"
  );
  // tac.cls dedicatedto / submitted / received / revised — frontmatter
  // pubdate annotations.
  DefMacro!(
    "\\dedicatedto{}",
    "\\@add@frontmatter{ltx:note}[role=dedication]{#1}"
  );
  DefMacro!(
    "\\submitted{}",
    "\\@add@frontmatter{ltx:note}[role=submitted]{#1}"
  );
  DefMacro!(
    "\\received{}",
    "\\@add@frontmatter{ltx:note}[role=received]{#1}"
  );
  DefMacro!(
    "\\revised{}",
    "\\@add@frontmatter{ltx:note}[role=revised]{#1}"
  );
  DefMacro!("\\Vol{}", "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\No{}", "\\@add@frontmatter{ltx:note}[role=number]{#1}");
  DefMacro!("\\pages{}", "\\@add@frontmatter{ltx:note}[role=pages]{#1}");

  // tac.cls L315: \def\CR{\\\phantom{\rm Email: }} — line-break helper
  // in author block. Use a no-op \\ since phantom is layout-only.
  DefMacro!("\\CR", "\\\\");

  // tac.cls L192-200: theorem-likes. Pre-register so they don't fire
  // undefined errors. \newtheoremrm{remark} is just like \newtheorem
  // (the rm marker is roman-text styling, irrelevant for HTML).
  RawTeX!(r"\newtheorem{theorem}{Theorem}");
  RawTeX!(r"\newtheorem{proposition}[theorem]{Proposition}");
  RawTeX!(r"\newtheorem{corollary}[theorem]{Corollary}");
  RawTeX!(r"\newtheorem{lemma}[theorem]{Lemma}");
  RawTeX!(r"\newtheorem{definition}[theorem]{Definition}");
  RawTeX!(r"\newtheorem{scholium}[theorem]{Scholium}");
  RawTeX!(r"\newtheorem{assumption}[theorem]{Assumption}");
  RawTeX!(r"\newtheorem{remark}[theorem]{Remark}");
  RawTeX!(r"\newtheorem{example}[theorem]{Example}");
  // tac.cls (every copy: 0708.1921's :466-470, 2401.05974's) `\newtheoremrm{remarks}{Remarks}`,
  // `\newtheoremrm{examples}{Examples}` (8 witnesses: 0907.2460, 1003.5552, 1202.1004, 1203.4318, 1309.4821),
  // defined at the document's start where the paper has not: tac.cls's own `\newtheorem` is a `\def` (0708.1921's
  // :409-418), so a paper's `\newtheoremrm{examples}{Examples}[section]` replaces the class's (2407.21129), which a
  // theorem declared here first would not cleanly (its counter stays the shared one: "Examples 2.0").
  RawTeX!(
    r"\AtBeginDocument{\@ifundefined{remarks}{\newtheorem{remarks}[theorem]{Remarks}}{}%
\@ifundefined{examples}{\newtheorem{examples}[theorem]{Examples}}{}}"
  );
  RawTeX!(r"\newtheorem{notation}[theorem]{Notation}");
  Let!("\\newtheoremrm", "\\newtheorem");

  // What every tac.cls copy papers ship defines (83 of 83: 2015/03/16 and 2023/03/06 versions; line numbers from
  // 2401.05974's 2023 copy, then 0708.1921's 2015 copy) and Perl has no binding for:
  // - `\mathrmdef[cs]{Word}`, `\mathbfdef`, `\mathopdef`, `\mathsfdef`, `\mathcaldef`, … (:651-678; 0708.1921 :690-713),
  //   which define `\cs` (named `\Word` without the option) as the word in that face: 45 papers (0709.0837,
  //   0711.0674, 0907.2460, 1003.5552), `\mathbfdef` 36. The 2023 version follows each with `\xspace` (it loads
  //   xspace, :31); the paper's own copy decides, the newer version without one.
  // - `\refs`…`\endrefs` / `{refs}` (:572-578; 0709.0837 :703-709) and `{references}`, `{references*}` (:545-564),
  //   the class's reference lists of `\bibitem`s under "References": the bibliography (23 papers: 0812.1674,
  //   0902.2554, math0610772; 1202.6034, 2012.03797).
  // - `\dedication{…}` (:311-312), printed under the title (12 papers: 0812.1674, 1002.1372); `\cauthor{…}` (:485), the
  //   copyright line's author; `\endproofbox` (:236-238), the end-of-proof square; `\proofof[…]` (0708.1921 :428).
  // - The class's own statement headings `\state`/`\staterm` (:148-157) and `\thirdleveltheorems` (:159-163), which
  //   moves them down a sectioning level (13 papers: 1106.1983, 1112.0560). The class's theorems are such headings; this
  //   binding's are theorem environments, which the switch leaves as they are.
  let xspace = crate::shipped_class::shipped_class_text("tac")?
    .is_none_or(|text| crate::shipped_class::class_text_loads(&text, "xspace"));
  if xspace {
    RequirePackage!("xspace");
    RawTeX!(
      r"\def\@mathopdef[#1]#2{\expandafter\def\csname#1\endcsname{\mathop{\rm{}#2}\nolimits\xspace}}
\def\@mathoplsdef[#1]#2{\expandafter\def\csname#1\endcsname{\mathop{{}#2}\xspace}}
\def\@mathbfdef[#1]#2{\expandafter\def\csname#1\endcsname{{\rm\bf#2}\xspace}}
\def\@mathrmdef[#1]#2{\expandafter\def\csname#1\endcsname{{\ensuremath{\rm#2}}\xspace}}
\def\@mathsfdef[#1]#2{\expandafter\def\csname#1\endcsname{{\ensuremath{\sf#2}}\xspace}}
\def\@mathfkdef[#1]#2{\expandafter\def\csname#1\endcsname{\ensuremath{\mathfrak{#2}}\xspace}}
\def\@mathcaldef[#1]#2{\expandafter\def\csname#1\endcsname{{\ensuremath{\mathcal{#2}}}\xspace}}"
    );
  } else {
    RawTeX!(
      r"\def\@mathopdef[#1]#2{\expandafter\def\csname#1\endcsname{\mathop{\rm#2}\nolimits}}
\def\@mathoplsdef[#1]#2{\expandafter\def\csname#1\endcsname{\mathop{\rm#2}}}
\def\@mathbfdef[#1]#2{\expandafter\def\csname#1\endcsname{{\rm\bf#2}}}
\def\@mathrmdef[#1]#2{\expandafter\def\csname#1\endcsname{{\rm#2}}}
\def\@mathsfdef[#1]#2{\expandafter\def\csname#1\endcsname{{\ensuremath{\sf#2}}}}
\def\@mathfkdef[#1]#2{\expandafter\def\csname#1\endcsname{\mathfrak{#2}}}
\def\@mathcaldef[#1]#2{\expandafter\def\csname#1\endcsname{{\ensuremath{\mathcal{#2}}}}}"
    );
  }
  RawTeX!(
    r"\def\mathopdef{\@dblarg\@mathopdef}\def\mathoplsdef{\@dblarg\@mathoplsdef}\def\mathbfdef{\@dblarg\@mathbfdef}
\def\mathrmdef{\@dblarg\@mathrmdef}\def\mathsfdef{\@dblarg\@mathsfdef}\def\mathfkdef{\@dblarg\@mathfkdef}
\def\mathcaldef{\@dblarg\@mathcaldef}
\def\refs{\begin{thebibliography}{}}\def\endrefs{\end{thebibliography}}
\let\references\refs\let\endreferences\endrefs
\expandafter\let\csname references*\endcsname\refs\expandafter\let\csname endreferences*\endcsname\endrefs
\def\cauthor#1{\def\@cauthor{#1}}
\newbox\@endproofbox\setbox\@endproofbox\hbox{\vrule width5pt height 5pt depth 0pt}\def\endproofbox{\copy\@endproofbox}
\def\proofof[#1]{\subsubsection*{{\textsc{Proof of #1}}}}
\def\@credit[#1]{\expandafter\@@credit\@firstofone#1\@@nil}
\def\@@credit{\@ifnextchar\cite{\@@@credit}{\@@@@credit}}
\def\@@@credit#1\@@nil{{\rm#1}}\def\@@@@credit#1\@@nil{{\rm[#1]}}
\def\state#1{\subsection{\sc#1}\begingroup\em\@ifnextchar[{\@credit}{}}
\def\staterm#1{\subsection{\sc#1}\begingroup\@ifnextchar[{\@credit}{}}
\def\endstate{\endgroup\par\addvspace{\medskipamount}}
\def\thirdleveltheorems{\def\state##1{\subsubsection{\sc##1}\begingroup\em\@ifnextchar[{\@credit}{}}%
\def\staterm##1{\subsubsection{\sc##1}\begingroup\@ifnextchar[{\@credit}{}}}"
  );
  DefMacro!(
    "\\dedication{}",
    "\\@add@frontmatter{ltx:note}[role=dedication]{#1}"
  );
});
