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
  // `\thm@bodyfont`, which each theorem keeps from its definition. A paper's own later `\rmtheorem{x}` is accepted and
  // changes nothing: a theorem keeps its style from `\newtheorem`. Divergence kept: kluwer's appended `\rmfamily`
  // hides the optional `[Note]` of these three (printed as text, "DEFINITION 2. [Foo]"); ours reads it as the title.
  RawTeX!(r"\newif\if@thms");
  DeclareOption!("thms", "\\@thmstrue");
  // :110-114: author-year references unless `numreferences`.
  RawTeX!(r"\newif\if@numreferences");
  DeclareOption!("namedreferences", "\\@numreferencesfalse");
  DeclareOption!("numreferences", "\\@numreferencestrue");
  // :135-136 `\if@chapterdef`, set by the `chapter` option: the class's chapters (:2464-2503 `\newcommand{\chapter}`,
  // :2704-2708 `\newcounter{chapter}`, `\newcounter{section}[chapter]`, `\thesection` `\thechapter.\arabic{section}`),
  // so a package's `\@ifundefined{chapter}` and a `\chapter` call see them (OXIDIZED_DESIGN #179 (b): the counter
  // keeps the kernel's `\chapter`).
  RawTeX!(r"\newif\if@chapterdef");
  DeclareOption!("chapter", "\\@chapterdeftrue");
  DeclareOption!(None, {});
  ProcessOptions!();
  if if_condition(&T_CS!("\\if@chapterdef"))?.unwrap_or(false) {
    NewCounter!("chapter", "document", idprefix => "Ch");
    RawTeX!(r"\@addtoreset{section}{chapter}\def\thesection{\thechapter.\arabic{section}}");
  }
  RawTeX!(
    r"\newcounter{article}
\newenvironment{article}{\stepcounter{article}}{}
\def\opening{}
\def\endopening{\maketitle}
\def\author#1{\lx@add@authors@append{#1}}
\def\institute#1{\lx@add@contact[role=affiliation,annotate=new]{#1}}
\def\surname#1{#1}
\def\rmtheorem#1{}
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
\def\abbrevnamefont{\bf}
\long\def\abbreviations#1{\lx@add@frontmatter{ltx:note}[role=abbreviations]{{\abbrevnamefont \abbreviationsname:~}#1}}
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
  // The class's own commands papers call (run-347 inventory: 140 kluwer.cls papers with one undefined), its lines
  // verbatim:
  // - :2007-2065, :2128-2145 the citation forms: under `numreferences` all are `\cite`; under the default named
  //   references `\cite{x}` is "(Author, Year)", `\inlinecite{x}` "Author (Year)", `\opencite{x}` "Author, Year",
  //   `\shortcite{x}` "(Year)", `\cite*{x}` "Year" (:2125 `\def\cite{\@ifstar{\citeyear}{\klu@cite}}`), over the
  //   `\citeauthoryear` labels kluwer.bst writes (`\inlinecite` 64, `\opencite` 48 papers; 1306.4881, 1408.6262,
  //   astro-ph0001448, 0802.0687, astro-ph0307528). kluwer.cls has its own author-year code, natbib's in all but name,
  //   and never loads natbib: natbib stands for it here, borrowed (`\@ifpackageloaded{natbib}` stays false, as the
  //   endnotes below), as `\citep`, `\citet`, `\citealp`, `\citeyear` and `\citeyearpar`, before the class's forms are
  //   set (natbib's own `\shortcite` is `\cite`, natbib_sty.rs).
  // - :1909-1975 the endnotes, endnotes.sty's interface (klunote.sty) without loading it: borrowed (`\theendnotes`
  //   31 papers; 0910.2393, 0711.4475, 0904.2540).
  // - :147 `\I` (bold I); :162-176 the math symbols, which the class composes from overprinted glyphs (`\dR` is
  //   `I \! \! R`, `\lsim` a raised `<` over a `\sim`): here the symbols they draw, as the astronomy bindings set
  //   `\la`/`\ga` (aa_support_sty.rs) — ≲ ≳ ≢ and the double-struck ℂ ℙ ℕ ℝ ℤ ℚ (`\gsim`/`\lsim` 14/13 papers;
  //   astro-ph0309194, astro-ph0510296, astro-ph0401471; `\dZ`, `\dR`, `\dC`, `\dN`, `\dQ`, `\dP`, `\nequiv`;
  //   math0308074, math0212055, astro-ph0410557). Their boxes made two letters of `\dZ` in the formula
  //   (OXIDIZED_DESIGN_DIVERGENCES #598).
  // - :1508, :1571, :1758 the caption fonts and dot (`\tabcapfont` 13 papers).
  // - :2292-2307 `{subequation}` (9 papers), :2343-2348 `\qed` (8), :1862-1864 `{kaprotate}`.
  // - :955-960 `\nomenclature` (a title-page note, as `\abbreviations`, :950-953) and `\nomen`: each note keeps the
  //   heading the class prints before it, bold (`\nomennamefont`/`\abbrevnamefont`, :929-930): "Abbreviations:~" run
  //   in; "Nomenclature:" is on a line of its own in the PDF, run in here, as a `\par` in a frontmatter note pulls the
  //   formula after it into the heading's text (RED repro sectioning-frontmatter/frontmatter_note_par_then_math).
  // - :966-983 `{motto}`, set as a quote where it stands (its box went to the title block).
  // - :1097 `\crline` (the copyright line: page furniture), :1487-1488 `\rcline`, `\lcline`, `\lrcline`, `\rlcline`
  //   (rules over part of a row, as `\cline`), :48 `\filedate`.
  require_package_borrowed("endnotes")?;
  DefMath!("\\lsim", "\u{2272}", role => "RELOP", meaning => "less-than-or-similar-to");
  DefMath!("\\gsim", "\u{2273}", role => "RELOP", meaning => "greater-than-or-similar-to");
  DefMath!("\\nequiv", "\u{2262}", role => "RELOP", meaning => "not-equivalent-to");
  DefMath!("\\dC", "\u{2102}", role => "ID", meaning => "complexes");
  DefMath!("\\dP", "\u{2119}", role => "ID");
  DefMath!("\\dN", "\u{2115}", role => "ID", meaning => "natural-numbers");
  DefMath!("\\dR", "\u{211D}", role => "ID", meaning => "reals");
  DefMath!("\\dZ", "\u{2124}", role => "ID", meaning => "integers");
  DefMath!("\\dQ", "\u{211A}", role => "ID", meaning => "rationals");
  if !if_condition(&T_CS!("\\if@numreferences"))?.unwrap_or(false) {
    require_package_borrowed("natbib")?;
  }
  RawTeX!(
    r"\if@numreferences
  \let\shortcite\cite
  \let\inlinecite\cite
  \let\opencite\cite
\else
  \def\cite{\@ifstar{\citeyear}{\citep}}%
  \def\shortcite{\citeyearpar}%
  \def\opencite{\citealp}%
  \def\inlinecite{\citet}%
\fi
\def\filedate{2000/04/29}
\def\I{{\bf I}}
\def\figtabdot{.}
\def\tabcapfont{\footnotesize }
\def\figcapfont{\footnotesize}
\def\subequation{\@ifnextchar[{\@subequation}{\@subequation[alph]}}
\def\@subequation[#1]{\refstepcounter{equation}%
  \def\@testoption{arabic}%
  \def\@testparam{#1}%
  \edef\old@equation{\the\c@equation}%
  \edef\old@theequation{\theequation}%
  \setcounter{equation}{0}%
  \ifx\@testoption\@testparam
    \def\theequation{\old@theequation.\csname #1\endcsname{equation}}%
  \else
    \def\theequation{\old@theequation\csname #1\endcsname{equation}}%
  \fi}
\def\endsubequation{%
  \setcounter{equation}{\old@equation}%
  \global\@ignoretrue
}
\def\qed{\relax
     \ifmmode
       ~\hfill\Box
     \else
        \unskip\nobreak ~\hfill$\Box$%
      \fi \par}
\newif\if@kaprotate \@kaprotatefalse
\def\kaprotate{\global\@kaprotatetrue}
\def\endkaprotate{\global\@kaprotatefalse}
\providecommand{\nomenclaturename}{Nomenclature}
\def\nomen#1#2{#1 -- #2}
\def\nomennamefont{\bf}
\long\def\nomenclature#1{\lx@add@frontmatter{ltx:note}[role=nomenclature]{{\nomennamefont\nomenclaturename:} #1}}
\newenvironment{motto}[1][]{\begin{quote}\footnotesize}{\end{quote}}
\def\crline#1{\gdef\@crline{#1}}
\let\rcline\cline \let\lcline\cline
\let\lrcline\cline \let\rlcline\cline"
  );
});
