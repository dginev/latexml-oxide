% The title-page shape short_title_in_a_head_definition.tex reads (79a r3): the class's `\@maketitle` defines the
% first page's heads, `\def\@oddhead{\@shortauthor: \@shorttitle\hfil}` and `\gdef\@evenhead{Vol.\@volume\hfil}`
% (ptptex.cls's `\maketitle` head), the short author a store of a setter no table names, the volume glued to "Vol.".
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{zzfmh}
\LoadClass{article}
\newcommand\shorttitle[1]{\gdef\@shorttitle{#1}}
\let\@shorttitle\empty
\newcommand\shortauthor[1]{\gdef\@shortauthor{#1}}
\let\@shortauthor\empty
\newcommand\volume[1]{\gdef\@volume{#1}}
\gdef\@volume{}
\def\@maketitle{{\def\@oddhead{\@shortauthor: \@shorttitle\hfil}\gdef\@evenhead{Vol.\@volume\hfil}}%
  \begin{center}{\LARGE\@title\par}{\large\@author\par}\end{center}}
