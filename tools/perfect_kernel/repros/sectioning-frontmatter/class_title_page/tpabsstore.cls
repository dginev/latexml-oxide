% A stand-in written from scratch (no code of zaa.cls): a class `\abstract` that only stores, printed under a label by
% the class's own \maketitle.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{tpabsstore}[2026/10/28 stand-in for zaa.cls's abstract]
\LoadClass{article}
\renewcommand{\abstract}[1]{\gdef\@abstract{#1}}
\def\@abstract{}
\renewcommand{\maketitle}{\begingroup
  {\Large\bfseries\@title\par}
  {\itshape\@author\par}
  \vbox{\noindent\textbf{Abstract.}\ \@abstract\par}
  \endgroup}
