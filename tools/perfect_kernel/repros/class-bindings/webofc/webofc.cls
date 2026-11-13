\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{webofc}[2024/04/25 v2.1 Web of Conferences (repro stand-in)]
% A minimal stand-in for EDP Sciences' webofc.cls (Web of Conferences; the papers ship it, it is not in TeX Live),
% written from scratch for the repros beside it: it defines only the interface they use, with what the class does
% there, the class's line numbers those of 2610.23498's copy (v2.1, 2024/04/25). No code of the class is copied.
\LoadClass[fleqn]{article}
\RequirePackage{url}
% :217 `\fnsep`: a comma raised as a footnote mark is, between two marks
\newcommand\fnsep{\unskip\textsuperscript{,}}
% :429-435 the title-page stores (:465 no institute); :438 `\inst{n}`: the n-th institute's number, raised; :440
% `\email`: "e-mail: " and the address set as a URL
\renewcommand\title[1]{\gdef\@title{#1}}
\newcommand\institute[1]{\gdef\@institute{#1}}
\renewcommand\abstract[1]{\gdef\@abstract{#1}}
\newcommand\inst[1]{\unskip\textsuperscript{#1}}
\newcommand\email[1]{e-mail: \url{#1}}
\gdef\@institute{}
% :477-478 the name parts, the given name in italics
\newcommand\firstname[1]{\textit{#1}}
\newcommand\lastname[1]{#1}
% :605-656 the title page, reduced to its parts: the title, the authors (an `\and` a comma; a `\thanks` a footnote),
% the institutes one per line (each numbered by the class, :566-589), the abstract
\renewcommand\maketitle{\par
  {\Large\bfseries\@title\par}\medskip
  {\let\thanks\footnote\renewcommand\and{\unskip, }\@author\par}\medskip
  {\small\renewcommand\and{\par}\@institute\par}\medskip
  {\small\textbf{Abstract.} \@abstract\par}\bigskip
  \setcounter{footnote}{0}}
% :788-812 the caption: the label, its period and the space after it bold (`\@CaptionTitleFont`, :334, before
% `\@FloatCounterSep`, :786 ". ")
\newif\if@woc@side
\renewcommand\@makecaption[2]{%
  \if@woc@side
    \parbox[b]{0.4\linewidth}{\small\textbf{#1. }#2}%
  \else
    \par\vskip\abovecaptionskip{\small\textbf{#1. }#2\par}\vskip\belowcaptionskip
  \fi}
% :824-857 `\sidecaption`: the float's body, then its caption set beside it
\long\def\sidecaption#1\caption{\leavevmode\hbox{#1}\hspace{10pt}\@woc@sidetrue\caption}
\endinput
