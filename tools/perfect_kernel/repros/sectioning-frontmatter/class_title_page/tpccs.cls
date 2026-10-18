% The shape of CSML.cls:227-229, :285-296 (LMCS; 1608.04592, 1602.00476): the class stores its ACM CCS terms and its
% `\@maketitle` prints them in an unmarked footnote of the title page (amsart.cls:655's `\let\@thefnmark\relax`).
\ProvidesClass{tpccs}
\LoadClass{article}
\def\ACMCCS#1{\def\@ACMCCS{#1}}
\let\@ACMCCS=\@empty
\def\ACMCCSname{2012 ACM CCS}
\def\@setACMCCS{{\itshape\ACMCCSname:}\enspace\@ACMCCS.}
\def\@maketitle{%
  \newpage
  \let\@makefnmark\relax\let\@thefnmark\relax
  \ifx\@empty\@ACMCCS\else \@footnotetext{\@setACMCCS}\fi
  {\Large\@title\par}%
  \noindent\@author\par}
