% The title-page shape short_title_in_a_running_head.tex reads (79a r2): the class's `\@maketitle` sets the running
% head to `\@shortauthor: \@shorttitle` (preprint.cls's `\ihead{\@shortauthor:~\@shorttitle}`, 2512.04983), the
% short author a store of a setter no table names.
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{zzfmg}
\LoadClass{article}
\newcommand\shorttitle[1]{\gdef\@shorttitle{#1}}
\let\@shorttitle\empty
\newcommand\shortauthor[1]{\gdef\@shortauthor{#1}}
\let\@shortauthor\empty
\def\@maketitle{\markboth{\@shortauthor: \@shorttitle}{\@shortauthor: \@shorttitle}%
  \begin{center}{\LARGE\@title\par}{\large\@author\par}\end{center}}
\pagestyle{myheadings}
