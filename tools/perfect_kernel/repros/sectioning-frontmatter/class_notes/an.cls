% A trimmed copy of an.cls (Astronomische Nachrichten, Wiley-VCH, 2006-01-13; not in TeX Live), the class
% astro-ph0611016 ships: its abstract setter (an.cls:414), the end-of-class hook that makes it `\abstract` (:98), its
% `\keywords` (:354-360) and the title page that reads both stores (`\@maketitle`, :715-718).
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{an}[2006/01/13 trimmed copy]
\LoadClass{article}
\AtEndOfClass{\let\abstract=\aaabstract}% -MWL- 2005-11-06
\def\keywordname{\textbf{Key words}}
\newcommand{\keywords}[1]{%
\if!#1!%
  \let\@keywords\relax%
\else%
  \def\@keywords{\textbf{\keywordname}\quad#1\ignorespaces\vskip6pt}%
\fi}
\keywords{}
\def\aaabstract#1{\gdef\@abstract{#1}}
\def\@maketitle{\newpage
 \normalfont
 {\fontsize{14.4}{17}\bfseries\selectfont\@title}\par
 \vskip 4.5mm\relax
 \noindent\ignorespaces{\normalsize\bfseries\selectfont{\@author}}%
 \vskip10pt\relax
 \noindent\@date
 \vskip 5mm
 \noindent
 \@keywords
 \noindent
 {\@abstract}
 \par\vskip 5mm}
