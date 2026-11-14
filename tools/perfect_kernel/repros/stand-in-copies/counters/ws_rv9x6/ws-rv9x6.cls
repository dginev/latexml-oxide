% A minimal stand-in for ws-rv9x6.cls (World Scientific's review-volume class, v2r 2015/04/20, the copy 2301.03442 ships
% — not in TeX Live; "All rights are reserved"), written from scratch for the repros beside it: the class's declaration,
% its counters — section within chapter, subsection within section, the section numbered with its chapter only under
% `\if@addchapnum` (:397-411) — and its `\chapter` (:497-501), with what the class does there; the class has no
% `\LoadClass`. No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{ws-rv9x6}[2015/04/20 v2r repro stand-in]
\newif\if@addchapnum
\setcounter{secnumdepth}{3}
\newcounter{part}\newcounter{chapter}
\newcounter{section}[chapter]\newcounter{subsection}[section]
\def\thechapter{\arabic{chapter}}
\if@addchapnum\def\thesection{\thechapter.\arabic{section}}\else\def\thesection{\arabic{section}}\fi
\def\thesubsection{\thesection.\arabic{subsection}}
\def\chapter{\clearpage\global\@topnum\z@\@afterindentfalse\secdef\@chapter\@schapter}
