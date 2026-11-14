\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{raw_store_affiliation}[2026/10/30 a raw class printing its affiliation store under the names]
\LoadClass{article}
\def\affiliation#1{\gdef\@affiliation{#1}}
\gdef\@affiliation{}
\def\@maketitle{%
  \newpage\null\vskip 1pc%
  \begin{center}%
    {\bfseries\LARGE \@title \par}%
    \vskip 12pt%
    {\scshape \@author \par}%
    \vskip 4pt%
    {\itshape \@affiliation \par}%
  \end{center}\par}
\endinput
