\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{aguauthors}[2026/11/10 repro stand-in]
% A minimal stand-in for the AGU `\authors` interface (agujournal2019.cls; not in TeX Live), written from scratch for
% the omnibus repros: no code of the class is copied. It is for pdflatex only, run from `omnibus/` with
% `TEXINPUTS=pdflatex//:`. It sits apart from the repros on purpose: beside a repro latexml_oxide would find it and read
% it raw (the repro's class then no unknown class read as OmniBus, 0 creators — review revZ18k S1).
\LoadClass{article}
% the numbered affiliation marks after a name: a superscript of the numbers
\newcommand\affil[1]{\textsuperscript{#1}}
% each numbered affiliation, stored and printed under the names with its number
\newcommand\@affils{}
\newcommand\affiliation[2]{\g@addto@macro\@affils{\par\textsuperscript{#1}#2}}
\newcommand\authors[1]{\gdef\@authors{#1}}
\renewcommand\maketitle{\par\begin{center}{\LARGE\@title\par}\vskip 1em{\large\@authors\par}
  {\small\@affils\par}\end{center}\par}
