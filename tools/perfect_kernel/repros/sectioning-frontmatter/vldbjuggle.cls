% A vldb.cls-shaped class (1511.05911): \alignauthor closes the tabular \@maketitle opens around \@author and opens one
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{vldbjuggle}
\LoadClass{article}
\newdimen\auwidth \auwidth=0.3\textwidth
\def\alignauthor{\end{tabular}\begin{tabular}[t]{p{\auwidth}}\centering}
\def\affaddr{\normalsize}
\def\email#1{{{\normalsize{\vskip 4pt#1}}}}
\def\numberofauthors#1{}
\def\@maketitle{\newpage\null
  \begin{center}{\LARGE\@title\par}\vskip 1.5em
    {\large\begin{tabular}[t]{c}\@author\end{tabular}\par}\vskip 1.5em
  \end{center}}
