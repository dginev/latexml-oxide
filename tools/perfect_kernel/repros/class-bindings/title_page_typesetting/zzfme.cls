% The title-page shape doi_glued_to_class_text.tex reads (79a r2): piparticle-final.cls:272's DOI line, the class's
% printed prefix glued to the store, `{DOI: 10.4279/PIP.\@doi}`, and its `\if\@doi\relax` test; and an address after a
% skip's dimension, `\hskip.5em\@address` (jps-cp.cls:`\hskip.5em\@kword`'s shape), its letters no printed text.
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{zzfme}
\LoadClass{article}
\def\@doi{\relax}
\newcommand{\doi}[1]{\gdef\@doi{#1}}
\newcommand\address[1]{\gdef\@address{#1}}
\gdef\@address{}
\def\@maketitle{\begin{center}{\LARGE\@title\par}{\large\@author\par}{\itshape\hskip.5em\@address\par}\small
  \if\@doi\relax\else{DOI: 10.4279/PIP.\@doi}\fi\par\end{center}}
