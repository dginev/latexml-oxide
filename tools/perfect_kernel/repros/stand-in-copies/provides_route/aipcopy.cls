%% An AIP Conference Proceedings class cut down (aip-cp.cls:24, :228-282, :501-516): a standard-class derivative that
%% declares itself article and patches the kernel's frontmatter: `\author[aff]{…}` and `\affil[aff]{…}` accumulate
%% into lists its own `\@maketitle` prints
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{article}[2014/12/10 v1.0 AIP Conference Proceedings]
\LoadClass{article}
\def\aipauthors{}
\let\authorsep\@empty
\def\author{\@ifnextchar[{\@@author}{\@author}}
\def\@@author[#1]#2{\g@addto@macro\aipauthors{\authorsep#2\textsuperscript{#1}\def\authorsep{, }}}
\def\@author#1{\g@addto@macro\aipauthors{\authorsep#1\def\authorsep{, }}}
\def\aipaffil{}
\def\affil{\@ifnextchar[{\@@affil}{\@affil}}
\long\def\@@affil[#1]#2{\g@addto@macro\aipaffil{\textsuperscript{#1}\textit{#2}\par}}
\long\def\@affil#1{\g@addto@macro\aipaffil{\textit{#1}\par}}
\def\@maketitle{\begin{center}{\Large\@title\par}\aipauthors\par\aipaffil\end{center}}
\endinput
