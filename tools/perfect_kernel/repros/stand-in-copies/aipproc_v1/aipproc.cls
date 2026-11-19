% The 1998 aipproc.cls (v1.0a, astro-ph0003214's copy), cut to its frontmatter interface: aipproc.cls:20-22, 85-89.
\NeedsTeXFormat{LaTeX2e}[1996/12/01]
\ProvidesClass{aipproc}
    [1998/09/14 v1.0a AIP Proceedings (FMi)]
\LoadClass{article}
\newcommand*\affiliation[1]{\def\AIP@affiliation{#1}}
\newcommand*\affiliationmark[1]{\textsuperscript{#1}}
\title{TITLE here}
\author{AUTHOR here}
\affiliation{AFFILIATION here}
\def\maketitle{\begin{center}\@title\par\@author\par\AIP@affiliation\end{center}}
