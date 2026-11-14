//! Stub for ejpecp.cls (Electronic Journal of Probability / Communications in Probability).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  // ejpecp.cls:81-106 (TL 2025; 1607.07914's copy :68-91 the same set) `\RequirePackage{graphicx}`, `{mathtools}`,
  // `{microtype}`, `{latexsym}`, `{dsfont}` unless the `NODS` option, the AMS packages unless `NOAMS`, `{geometry}`:
  // papers write `\coloneqq`, `\DeclarePairedDelimiter`, `\mathds` without loading them (run 353: 22 with mathtools's,
  // 14 with dsfont's commands undefined; 1607.07914, 1611.05220, 1904.07135, 1111.6458). The class's
  // `\let\mathbb=\mathds` (:465-469) is not followed: both are double-struck here.
  RequirePackage!("graphicx");
  RequirePackage!("mathtools");
  RequirePackage!("microtype");
  RequirePackage!("latexsym");
  DeclareOption!("NODS", {
    AssignValue!("ejpecp@NODS" => true);
  });
  ProcessOptions!();
  if !lookup_bool("ejpecp@NODS") {
    RequirePackage!("dsfont");
  }
  RequirePackage!("geometry");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");

  // ejpecp frontmatter (L118-129) — preserve author content.
  DefMacro!("\\TITLE{}", "\\title{#1}");
  DefMacro!(
    "\\SHORTTITLE{}",
    "\\@add@frontmatter{ltx:note}[role=shorttitle]{#1}"
  );
  DefMacro!(
    "\\KEYWORDS{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}"
  );
  DefMacro!(
    "\\AMSSUBJ{}",
    "\\@add@frontmatter{ltx:classification}[scheme=AMS]{#1}"
  );
  // ejpecp.cls L137: optional secondary AMS subject classification.
  // Witness 2312.17406.
  DefMacro!(
    "\\AMSSUBJSECONDARY{}",
    "\\@add@frontmatter{ltx:classification}[scheme=AMS-secondary]{#1}"
  );
  DefMacro!(
    "\\AMSSUBJPRIMARY{}",
    "\\@add@frontmatter{ltx:classification}[scheme=AMS]{#1}"
  );
  DefMacro!("\\ABSTRACT{}", "\\begin{abstract}#1\\end{abstract}");
  DefMacro!("\\AUTHORS{}", "\\author{#1}");
  DefMacro!(
    "\\VOLUME{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}"
  );
  DefMacro!(
    "\\PAPERNUM{}",
    "\\@add@frontmatter{ltx:note}[role=papernumber]{#1}"
  );
  DefMacro!("\\YEAR{}", "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!(
    "\\SUBMITTED{}",
    "\\@add@frontmatter{ltx:note}[role=submitted]{#1}"
  );
  DefMacro!(
    "\\ACCEPTED{}",
    "\\@add@frontmatter{ltx:note}[role=accepted]{#1}"
  );
  DefMacro!("\\DOI{}", "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!("\\EMAIL{}", "\\@add@frontmatter{ltx:note}[role=email]{#1}");
  // ejpecp.cls:156 `\BEMAIL` — a further e-mail line (sample).
  DefMacro!("\\BEMAIL{}", "\\EMAIL{#1}");
  // ejpecp.cls:350-366 the supplement block and :372-375 `\MR`/`\ARXIV`
  // (verbatim; the binding shadows the class file). Guard:
  // `perfect_kernel_batch56::sweep46_single_name_gaps`.
  RawTeX!(
    r"\def\supplement@name{Supplementary Material}
\def\stitle#1{\def\@stitle{#1}}
\def\stitle@fmt#1{\textbf{#1.}\ }
\def\sdescription#1{\def\@sdescription{#1}}
\def\suppsection@fmt{\section*{\supplement@name}}
\long\def\supplement{\@ifnextchar[{\@supplement}{\@supplement[]}}
\long\def\@supplement[#1]{\suppsection@fmt\global\let\suppsection@fmt\smallskip}
\def\endsupplement{\@ifundefined{@stitle}{}{\stitle@fmt{\@stitle}}\@ifundefined{@sdescription}{}{\@sdescription}\par}
\def\@MRExtract#1 #2!{#1}
\newcommand{\MR}[1]{\xdef\@MRSTRIP{\@MRExtract#1 !}\href{https://mathscinet.ams.org/mathscinet-getitem?mr=\@MRSTRIP}{MR\@MRSTRIP}}
\newcommand{\ARXIV}[1]{\href{https://arXiv.org/abs/#1}{arXiv:#1}}"
  );
  // ejpecp.cls:467 `\let\realmathbb=\mathbb` (the class re-styles `\mathbb`).
  Let!("\\realmathbb", "\\mathbb");
  DefMacro!(
    "\\support{}",
    "\\@add@frontmatter{ltx:note}[role=support]{#1}"
  );

  // Standard envs commonly used in probability papers.
  DefEnvironment!("{acks}", "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  // ejpecp.cls:400-454 (TL 2025; every sampled copy numbers within sections, the older ones a subset): the class's
  // theorem set on `{theorem}`'s counter, numbered within sections, the statements in its `ejpecpbodyit` style (an
  // italic body), the examples and remarks in `ejpecpbodyrm` (`{notation}` 4 papers of the sample, `{hypothesis}` 3,
  // `{condition}` 3, `{assumptions}` 1: 1405.0815, 1406.6650, 2309.13899), and the equations numbered within sections
  // (:456-458) — under the AMS packages, which this binding loads whatever the `NOAMS` option.
  RawTeX!(
    r"\newtheoremstyle{ejpecpbodyit}{3pt}{3pt}{\itshape}{}{\bfseries\sffamily}{.}{ }{}
\newtheoremstyle{ejpecpbodyrm}{3pt}{3pt}{}{}{\bfseries\sffamily}{.}{ }{}
\theoremstyle{ejpecpbodyit}
\newtheorem{theorem}{Theorem}[section]
\newtheorem{assumptions}[theorem]{Assumptions}\newtheorem{assumption}[theorem]{Assumption}
\newtheorem{claim}[theorem]{Claim}\newtheorem{condition}[theorem]{Condition}\newtheorem{conjecture}[theorem]{Conjecture}
\newtheorem{corollary}[theorem]{Corollary}\newtheorem{definitions}[theorem]{Definitions}
\newtheorem{definition}[theorem]{Definition}\newtheorem{facts}[theorem]{Facts}\newtheorem{fact}[theorem]{Fact}
\newtheorem{heuristics}[theorem]{Heuristics}\newtheorem{hypothesis}[theorem]{Hypothesis}
\newtheorem{hypotheses}[theorem]{Hypotheses}\newtheorem{lemma}[theorem]{Lemma}
\newtheorem{notations}[theorem]{Notations}\newtheorem{notation}[theorem]{Notation}
\newtheorem{proposition}[theorem]{Proposition}
\theoremstyle{ejpecpbodyrm}
\newtheorem{example}[theorem]{Example}\newtheorem{exercise}[theorem]{Exercise}\newtheorem{problem}[theorem]{Problem}
\newtheorem{question}[theorem]{Question}\newtheorem{remark}[theorem]{Remark}
\theoremstyle{plain}
\numberwithin{equation}{section}"
  );
  // ejpecp.cls:150-152: `\ARXIVID{id}` and `\HALID{id}`, the earlier versions the first page says the paper supersedes
  // ("Supersedes arXiv:…", :316-328; 33 and 3 papers of the inventory: 1104.1060, 2103.00338), and `\ACKNO{text}`, the
  // acknowledgments (46 papers: 1103.0392); 1210.2883's copy :122 `\ARXIVPASSWORD{…}`, which prints nothing (3).
  DefMacro!(
    "\\ARXIVID{}",
    "\\@add@frontmatter{ltx:note}[role=supersedes]{Supersedes \\texttt{\\href{https://arXiv.org/abs/#1}{arXiv:#1}}.}"
  );
  DefMacro!(
    "\\HALID{}",
    "\\@add@frontmatter{ltx:note}[role=supersedes]{Supersedes \\texttt{\\href{https://hal.archives-ouvertes.fr/#1}{HAL:#1}}.}"
  );
  DefMacro!("\\ACKNO{}", "\\begin{acks}#1\\end{acks}");
  def_macro_noop("\\ARXIVPASSWORD{}")?;
});
