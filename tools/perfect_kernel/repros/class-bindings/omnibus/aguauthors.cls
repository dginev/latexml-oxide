\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{aguauthors}[2026/11/10 repro stand-in]
% A minimal stand-in for the AGU `\authors` interface (agujournal2019.cls; not in TeX Live), written from scratch for
% the repros beside it: no code of the class is copied. latexml_oxide reads an unknown class as OmniBus and never this
% file; it is for pdflatex.
\LoadClass{article}
% the numbered affiliation marks after a name: a superscript of the numbers
\newcommand\affil[1]{\textsuperscript{#1}}
% each numbered affiliation, stored and printed under the names with its number
\newcommand\@affils{}
\newcommand\affiliation[2]{\g@addto@macro\@affils{\par\textsuperscript{#1}#2}}
\newcommand\authors[1]{\gdef\@authors{#1}}
\renewcommand\maketitle{\par\begin{center}{\LARGE\@title\par}\vskip 1em{\large\@authors\par}
  {\small\@affils\par}\end{center}\par}
