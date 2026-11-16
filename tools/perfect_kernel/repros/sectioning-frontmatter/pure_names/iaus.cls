% A stand-in for the IAU symposium class (iaus.cls, shipped by 0707.3527), written from scratch: its author line and
% its \affiliation, each a store \maketitle sets in a paragraph of its own (iaus.cls:821, :868-877).
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{iaus}[2026/11/16 stand-in]
\LoadClass{article}
\renewcommand\author[2][]{\gdef\@author{#2}}
\newcommand\affiliation[1]{\gdef\@affiliation{#1}}
\gdef\@affiliation{}
\renewcommand\maketitle{\par
  \begingroup\centering
  {\LARGE\bfseries \@title\par}\vskip 10pt
  {\large\bfseries \@author\par}\vskip 4pt
  {\small \@affiliation\par}\par
  \endgroup\vskip 13pt}
