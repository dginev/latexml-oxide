\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{check_dangling_keywords}[2026/11/05 a raw class whose keywords setter leaves its conditional open]
\LoadClass{article}
% CUP-JNL-DCE.cls:500-506 (2003.03814): the `\fi` closing `\keywords`'s `\ifx` sits inside `\@keywords`, and
% `\printkeywords`'s `\ifx...\else\@keywords\fi` balances it when the title page prints.
\def\printkeywords{\ifx\@keywords\empty\else\@keywords\fi\par}%
\def\keywords#1{\ifx#1\empty\else\def\@keywords{\par{\bfseries Keywords: #1\par}\fi}}
\def\@keywords{}
\def\@maketitle{%
  \newpage\null\vskip 1pc%
  \begin{center}%
    {\LARGE \@title \par}\vskip 1em%
    {\large \@author \par}%
  \end{center}%
  {\printkeywords\par}%
  \par\vskip 1pc}
