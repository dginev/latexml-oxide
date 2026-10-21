% A minimal stand-in for JINST.cls v1.0.1 (Journal of Instrumentation, the class 2601.17161 ships — not in TeX Live;
% "Copyright JINST 2006: it can be freely distributed but" not modified), written from scratch for
% jhep_family_title_page.tex: it defines only the interface the repro and the JHEP family's binding
% (latexml_contrib/src/jinst_cls.rs) use — the stores, their flags, `\receivedname`/`\keywordsname`, the title page
% `\auto@maketitle` typesets at `\begin{document}` and the checks before it — with what the class does there; line
% numbers are those of 2601.17161's copy. No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{JINST}[2006/01/01 repro stand-in]
\LoadClass{article}
% :455-470 a flag for each store given, checked at \begin{document}
\newif\if@author \newif\if@abstract \newif\if@keywords
\newcommand\receivedname{Received:}
\newcommand\keywordsname{Keywords:}
% :455-560 the stores
\newcommand\jst@authors{}
\renewcommand\author[1]{\global\@authortrue\g@addto@macro\jst@authors{\par #1}}
\newcommand\jst@abstract{}
\renewcommand\abstract[1]{\global\@abstracttrue\gdef\jst@abstract{\textsc{\abstractname}\ #1}}
\newcommand\jst@keywords{}
\newcommand\keywords[1]{\global\@keywordstrue\gdef\jst@keywords{\textsc{\keywordsname} #1.}}
\newcommand\jst@received{}
\newcommand\received[1]{\gdef\jst@received{\textsc{\receivedname} \textit{#1}}}
\newcommand\email[1]{\texttt{#1}}
% :463-506 `\maketitle` is ignored: the title page is typeset at \begin{document}, and `\email` is gone after it
\renewcommand\maketitle{\ClassWarning{JINST}{\protect\maketitle\space IGNORED}}
\renewcommand\@maketitle{\begin{center}{\Large\bfseries\@title\par}\jst@authors\par\jst@abstract\par\jst@keywords
  \par\jst@received\end{center}}
\newcommand\auto@maketitle{{\@maketitle}\global\let\email\@gobble}
% :1588-1607 the checks at \begin{document}, then the title page
\AtBeginDocument{%
  \if@author\else\ClassError{JINST}{Some \protect\author{...} should appear before \protect\begin{document}}{}\fi
  \if@abstract\else\ClassError{JINST}{\protect\abstract{...} should appear before \protect\begin{document}}{}\fi
  \auto@maketitle}
\endinput
