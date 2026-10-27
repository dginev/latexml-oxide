\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{eptcs}[2022/05/20 v1.7 EPTCS (repro stand-in)]
% A minimal stand-in for eptcs.cls (Electronic Proceedings in Theoretical Computer Science; the papers ship it, it is
% not in TeX Live), written from scratch for the repros beside it: it defines only the interface they use, with what
% the class does there, the class's line numbers those of 2406.16409's copy (v1.7, 2022/05/20). No code of the class
% is copied.
% :26-35 the options, of the first page's footer only
\DeclareOption*{}
\ProcessOptions\relax
% :37 the base class
\LoadClass[11pt]{article}
% :160-180 the title page, reduced to its parts: the title, then `\@author` in one centred column, where `\and` (the
% article class's) starts a column beside it
\renewcommand\maketitle{\par
  \begin{center}
    {\LARGE\bfseries\@title\par}\vskip 2em
    {\large\begin{tabular}[t]{c}\@author\end{tabular}\par}
  \end{center}\par\vskip 1em}
% :186-190 `\institute`: a new line of the author column, holding the institute set as a small one-column table
\newcommand\institute[1]{\\{\footnotesize\begin{tabular}[t]{c}#1\end{tabular}}}
% :191 `\email`: a new line of the author column, the address small and in typewriter type
\newcommand\email[1]{\\{\footnotesize\ttfamily #1}}
