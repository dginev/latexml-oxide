% The shape of pnastwo.cls (PNAS's class, not in TeX Live) the repros need: its `\affil#1#2` (:368-377), which numbers
% the affiliation with the footnote counter and stores its text for `\@maketitle`, printing only the mark; its
% `\maketitle` (:379-401, `\def\and{ and }` :383) running `\@maketitle` in a group, whose `\let\footnote\thanks` (:416) dies with it; and its
% `\contributor` line (:437-445), printed under the affiliations.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{cl6_pnas_affil}
\LoadClass{article}
\newcount\affilcount
\newcount\saveaffilcount
\def\affil#1#2{\normalsize\ \affilcount=#1
\ifnum\affilcount>\saveaffilcount\global\saveaffilcount\affilcount\fi
\expandafter\ifx\csname affil\the\affilcount\endcsname\relax
\global\advance\c@footnote by1
\expandafter\xdef\csname affilsymb\the\affilcount\endcsname{\the\c@footnote}{$^{\@fnsymbol\c@footnote}$}%
\expandafter\gdef\csname affil\the\affilcount\endcsname{#2}\else
$^{\expandafter\@fnsymbol\csname affilsymb\the\affilcount\endcsname}$%
\fi}
\renewcommand\maketitle{\par
  \begingroup
    \def\and{ and }%
    \renewcommand\thefootnote{\@fnsymbol\c@footnote}%
    \@maketitle
    \@thanks
  \endgroup
  \setcounter{footnote}{0}%
  \global\let\thanks\relax
  \global\let\maketitle\relax
  \global\let\@maketitle\relax
  \global\let\@thanks\@empty
  \global\let\@author\@empty
  \global\let\@title\@empty
  \global\let\@contributor\@empty}
\def\@maketitle{%
  \let\footnote\thanks
  {\raggedright\noindent\huge\@title\par}%
  \vskip6pt
  {\normalsize\@author\par
   {\vskip4pt\raggedright\affilcount=0\footnotesize
    \loop\ifnum\affilcount<\saveaffilcount
      \expandafter\ifx\csname affil\the\affilcount\endcsname\relax\else
        $^{\expandafter\@fnsymbol\csname affilsymb\the\affilcount\endcsname}$%
        \csname affil\the\affilcount\endcsname,\fi
      \global\advance\affilcount by1
    \repeat
    $^{\expandafter\@fnsymbol\csname affilsymb\the\affilcount\endcsname}$%
    \csname affil\the\affilcount\endcsname
   }}%
  \vskip4pt
  {\footnotesize\@contributor}%
  \par\vskip1.5em}
\def\contributor#1{\def\@contributor{#1}}
\contributor{Submitted to the Proceedings}
