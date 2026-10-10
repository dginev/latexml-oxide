% An excerpt of jcm.cls (the copy quant-ph0412067 ships): its two-argument \homepage (:157-162), for
% compositio_family_old_variant.tex.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{jcm}
\LoadClass{article}
\renewcommand*\author[2][]{\g@addto@macro\@curr@author{\\*#2}}
\def\@curr@author{\@gobbletwo}
\def\JCM@gobble{\@gobbletwo}
\def\homepage#1#2{%
  \ifx\@curr@author\JCM@gobble
    \ClassError{jcm}{\noexpand\homepage with no \string\author}\@ehd
  \else
    \g@addto@macro\@curr@author{\\ \href{{#1}}{\tt #2}}%
  \fi}
\endinput
