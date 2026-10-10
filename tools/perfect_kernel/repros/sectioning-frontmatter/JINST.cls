% An excerpt of JINST.cls v1.0.1 (the copy 2601.17161 ships), its frontmatter (:455-560, :750-834, :1588-1607),
% the title page trimmed to the fields it typesets, for jhep_family_title_page.tex.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{JINST}
\LoadClass{article}
\newif\if@author\@authorfalse \newif\if@abstract\@abstractfalse \newif\if@keywords\@keywordsfalse
\newif\if@rece\@recefalse \newif\if@preprint\@preprinttrue
\def\receivedname{Received:} \def\keywordsname{Keywords:}
\def\maketitle{\ClassWarning{JINST}{\protect\maketitle\space IGNORED}}
\def\@maketitle{\begin{center}{\Large\bfseries\@title}\par\@author\par
  \abstract@cs\par\@keywords\par\Jrece@cs\end{center}}
% :463-506: the title page, then the commands it typeset killed (:499)
\newcommand\auto@maketitle{\begingroup\@maketitle\endgroup\global\let\email\@gobble}
\newtoks\prev@t \newtoks\cur@t
\def\@author{}
\renewcommand\author[1]{\global\@authortrue\prev@t=\expandafter{\@author}%
  \cur@t={\vskip 6pt\vtop\bgroup #1\egroup\par}\long\xdef\@author{\the\prev@t\the\cur@t}}
\def\Jrece@cs{}
\newcommand\received[1]{\gdef\Jrece@cs{{\scshape\receivedname} {\itshape #1}}\@recetrue\let\received\gobble}
\newcommand\email[1]{{\tt #1}}
\def\abstract@cs{}
\renewcommand{\abstract}[1]{\gdef\abstract@cs{{\noindent{\scshape\abstractname}\ \ignorespaces #1}}%
  \global\@abstracttrue\let\abstract\gobble}
\def\@keywords{}
\newcommand\keywords[1]{\gdef\@keywords{\noindent{\scshape\keywordsname} #1.}\global\@keywordstrue}
\AtBeginDocument{\bgroup
  \def\@APPerr#1{\ClassError{JINST}{\protect#1 should appear before \protect\begin{document}}{}}%
  \if@author\else\@APPerr{Some \protect\author{...}}\fi
  \if@abstract\else\@APPerr{\abstract{...}}\fi
  \egroup
  \auto@maketitle}
\endinput
