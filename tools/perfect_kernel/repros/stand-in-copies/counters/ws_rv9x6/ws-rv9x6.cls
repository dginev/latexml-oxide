% A reduced ws-rv9x6.cls (2301.03442's copy, v2r 2015/04/20): its declaration, its counters (:397-411) and its
% `\chapter` (:497-501); the class has no `\LoadClass`.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{ws-rv9x6}[2015/04/20 v2r reduced]
\newif\if@addchapnum
\setcounter{secnumdepth}{3}
\newcounter{part}
\newcounter{chapter}
\newcounter{section}[chapter]
\newcounter{subsection}[section]
\renewcommand\thechapter{\arabic{chapter}}
\if@addchapnum
  \renewcommand\thesection{\thechapter.\arabic{section}}
\else
  \renewcommand\thesection{\arabic{section}}
\fi
\renewcommand\thesubsection{\thesection.\arabic{subsection}}
\newcommand\chapter{\clearpage\global\@topnum\z@\@afterindentfalse\secdef\@chapter\@schapter}
