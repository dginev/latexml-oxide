//! kluwer.cls (Kluwer Academic Publishers' LaTeX 2e class, TeX Live `tex/latex/kluwer`). A paper is
//! `\begin{article}…\end{article}` and its title page `\begin{opening}…\end{opening}`, whose end makes the title
//! (kluwer.cls:707-738; `\maketitle` is `\endopening`, :723). Each `\author{…}` names one author, and an
//! `\institute{…}` belongs to the authors given since the previous one (:766-769). OmniBus supplies the article-like
//! rest. Run 349: `{article}` 384, `{opening}` 344, `\surname` 335, `\newdisplay` 128 papers undefined under
//! OmniBus alone (witnesses astro-ph0001217, astro-ph0001398, astro-ph0002435).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  // kluwer.cls's own dependencies.
  RequirePackage!("amssymb");
  RequirePackage!("wasysym");
  // :124-125 `\if@thms`, set by the `thms` option only: the class's theorem and display environments (:2411-2432)
  // exist only under it, so a paper's own `\case`, `\rem` or `\Note` is its own otherwise. The class's `\rmtheorem`
  // (:2365, :2422-2426) sets {alg}, {defn} and {exmp} upright; here they are defined under an upright
  // `\thm@bodyfont`, which each theorem keeps from its definition. The public `\rmtheorem` stays unbound: a theorem
  // keeps its style from `\newtheorem`, so a later call has nothing to change.
  RawTeX!(r"\newif\if@thms");
  DeclareOption!("thms", "\\@thmstrue");
  DeclareOption!(None, {});
  ProcessOptions!();
  RawTeX!(
    r"\newcounter{article}
\newenvironment{article}{\stepcounter{article}}{}
\def\opening{}
\def\endopening{\maketitle}
\def\author#1{\lx@add@authors@append{#1}}
\def\institute#1{\lx@add@contact[role=affiliation,annotate=new]{#1}}
\def\surname#1{#1}
\def\orf#1{#1}
\def\nodagger{}
\def\received#1{\lx@add@date[role=received]{#1}}
\def\revised#1{\lx@add@date[role=revised]{#1}}
\def\accepted#1{\lx@add@date[role=accepted]{#1}}
\def\subtitle#1{\lx@add@subtitle{#1}}
\def\dedication#1{\lx@add@frontmatter{ltx:note}[role=dedication]{#1}}
\def\translation#1{\lx@add@frontmatter{ltx:note}[role=translation]{#1}}
\def\arttype#1{\lx@add@frontmatter{ltx:note}[role=articletype]{#1}}
\def\runningtitle#1{\gdef\@runningtitle{#1}}
\def\runningauthor#1{\gdef\@runningauthor{#1}}
\providecommand{\keywordsname}{Keywords}
\providecommand{\abbreviationsname}{Abbreviations}
\long\def\keywords#1{\lx@add@keywords[name={\keywordsname:~}]{#1}}
\long\def\abbreviations#1{\lx@add@frontmatter{ltx:note}[role=abbreviations]{#1}}
\def\abbrev#1#2{#1 -- #2}
\long\def\classification#1#2{\lx@add@classification[scheme={#1}]{#2}}
\newdimen\maxfloatwidth
\def\newdisplay{\newtheorem}
\if@thms
  \newtheorem{thm}{THEOREM}
  \newtheorem{cor}[thm]{COROLLARY}
  \newtheorem{lem}[thm]{LEMMA}
  \newtheorem{claim}[thm]{CLAIM}
  \newtheorem{conj}[thm]{CONJECTURE}
  \newtheorem{prop}[thm]{PROPOSITION}
  \newtheorem{exer}[thm]{EXERCISE}
  \newtheorem{REM}[thm]{REMARK}
  \newtheorem{prob}[thm]{PROBLEM}
  {\thm@bodyfont{\rmfamily\upshape}%
  \newtheorem{alg}{ALGORITHM}
  \newtheorem{defn}[thm]{DEFINITION}
  \newtheorem{exmp}[thm]{EXAMPLE}}
  \newdisplay{crit}{Criterion}
  \newdisplay{rem}{Remark}
  \newdisplay{Note}{Note}
  \newdisplay{summ}{Summary}
  \newdisplay{case}{Case}
\fi
\def\pr@@f#1{\par\noindent{\itshape #1.\/}\ \ignorespaces}
\def\endpr@@f{\par}
\def\newproof#1#2{\expandafter\def\csname #1\endcsname{\pr@@f{#2}}%
  \expandafter\def\csname end#1\endcsname{\endpr@@f}}
\newproof{pf}{Proof}
\newenvironment{pf*}[1]{\pr@@f{#1}}{\endpr@@f}"
  );
});
