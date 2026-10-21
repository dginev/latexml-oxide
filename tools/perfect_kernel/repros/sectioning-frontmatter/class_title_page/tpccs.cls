% A stand-in written from scratch for the shape of CSML.cls:227-229, :285-296 (Logical Methods in Computer Science;
% 1608.04592, 1602.00476 — the class states no licence of its own): the class stores its ACM CCS terms and its
% `\@maketitle` prints them in an unmarked footnote of the title page (as amsart.cls:655 `\let\@thefnmark\relax`). No
% code of the class is copied.
\ProvidesClass{tpccs}
\LoadClass{article}
\newcommand\ACMCCS[1]{\gdef\@ACMCCS{#1}}
\let\@ACMCCS\@empty
\newcommand\ACMCCSname{2012 ACM CCS}
\def\@maketitle{\newpage
  \let\@makefnmark\relax\let\@thefnmark\relax
  \ifx\@ACMCCS\@empty\else\@footnotetext{\textit{\ACMCCSname:}\enspace\@ACMCCS.}\fi
  {\Large\@title\par}\noindent\@author\par}
