\NeedsTeXFormat{LaTeX2e}\ProvidesClass{iucrjournals}[2024/01/01 stand-in for the IUCr class shipped by 2608.11466, 2503.09183]
\LoadClass{article}
\RequirePackage{hyperref}
\RequirePackage{authblk}
\RequirePackage{orcidlink}
% iucrjournals.cls:157
\newcommand{\IUCrOrcidlink}[1]{\orcidlink{#1}\,}
% iucrjournals.cls:201-206, the mail logo a tikz picture there
\newcommand\IUCr@cemaillogo{\texttt{@}}
\DeclareRobustCommand\IUCr@emaillinkX[1]{\href{mailto:#1}{\IUCr@cemaillogo}}
\DeclareRobustCommand\IUCr@cemaillinkX[1]{\href{mailto:#1}{\IUCr@cemaillogo}}
\newcommand{\IUCrEmaillink}[1]{\,\IUCr@emaillinkX{#1}\,}
\newcommand{\IUCrCemaillink}[1]{\,\IUCr@cemaillinkX{#1}\,}
% iucrjournals.cls:212-232, 293-296 (two symbols, the class's `\IUCr@fnsymbol` a table of five): a note stored with
% `\global\def\aufnX<n>`, each printed after the title
\newcounter{IUCr@aufnc}
\newcounter{IUCr@tmpc}
\newcommand{\IUCr@storeaufn}[1]{\stepcounter{IUCr@aufnc}\global\expandafter\def\csname aufnX\theIUCr@aufnc\endcsname{#1}}
\newcommand{\IUCr@fnsymbol}[1]{$^{\ifcase#1\or\dagger\or\ddagger\fi}$}
\newcommand{\IUCr@printaufn}[1]{\IUCr@fnsymbol{#1}\csname aufnX#1\endcsname\\}
\newcommand{\IUCrAufn}[2][0]{\if0#1\IUCr@storeaufn{#2}\IUCr@fnsymbol{\theIUCr@aufnc}\,\else
  \ifnum#1>\theIUCr@aufnc\IUCr@storeaufn{#2}\fi\IUCr@fnsymbol{\theIUCr@aufnc}\,\fi}
\newcommand{\IUCr@printauthornotes}{\ifnum\theIUCr@aufnc>0 \begin{center}\loop\stepcounter{IUCr@tmpc}%
  \IUCr@printaufn{\theIUCr@tmpc}\ifnum\value{IUCr@tmpc}<\value{IUCr@aufnc}\repeat\end{center}\fi}
\let\IUCr@maketitle=\maketitle
\def\maketitle{\IUCr@maketitle\IUCr@printauthornotes}
