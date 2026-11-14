%% A renamed copy of a REVTeX 4 class (qm2008_abs.cls, snp.cls: `\ProvidesClass{revtex4}`), cut down
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{revtex4}[2001/08/02 v4.0 (http://publish.aps.org/revtex4/ for documentation)]
\LoadClass{article}
\newcommand\affiliation[1]{\gdef\qm@aff{#1}}
\newcommand\qmextra{QM-EXTRA}
\def\@maketitle{\begin{center}{\Large\@title}\par\@author\par\textit{\qm@aff}\end{center}}
\endinput
