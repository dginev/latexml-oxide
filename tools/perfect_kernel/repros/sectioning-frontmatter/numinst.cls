\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{numinst}[2024/01/01 an article with llncs's numbered institutes, as preprint.cls (2405.16615) copies them]
\LoadClass{article}
% llncs.cls:870-885 (preprint.cls:233-256): `\institutename` prints the `\institute` list, its `\and` stepping the
% institute counter and printing the number before the next entry, the first numbered 1
\newcounter{inst}
\def\institutename{\par\begingroup\setcounter{inst}{1}%
  \def\and{\par\stepcounter{inst}\noindent$^{\theinst}$\enspace\ignorespaces}%
  \noindent$^{\theinst}$\enspace\@institute\par\endgroup}
\def\institute#1{\gdef\@institute{#1}}
\def\@maketitle{\begin{center}{\Large\bfseries\@title\par}\vskip1em{\large\bfseries\@author\par}\end{center}%
  \institutename}
