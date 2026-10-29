% A stand-in written from scratch (no code of zaa.cls): a title page whose condition compares a control sequence
% that is never defined, and addresses kept under numbered names that only \maketitle prints.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{tpifx}[2026/10/28 stand-in for zaa.cls's title page]
\LoadClass{article}
\newcount\tpifx@count
\def\address#1{\global\advance\tpifx@count by 1
  \expandafter\gdef\csname tpifx@addr\number\tpifx@count\endcsname{#1}}
\def\tpifx@addresses{\ifnum\tpifx@count>0 \csname tpifx@addr1\endcsname\fi}
\newcommand*{\dedication}[1]{\gdef\@dedication{#1}}
\def\@dedication{}
\renewcommand{\maketitle}{\begingroup
  {\Large\bfseries\@title\par}
  {\itshape\@author\par}
  \ifx\@dedication\undefined\else{\itshape\@dedication\par}\fi
  \par\noindent\tpifx@addresses\par
  \endgroup}
