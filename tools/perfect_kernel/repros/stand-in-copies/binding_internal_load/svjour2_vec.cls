% Springer's svjour code renamed svjour2.cls, with the copy's own bold \vec.
\ProvidesClass{svjour}[2003/04/15 v2.8 Springer journal class]
\LoadClass{article}
\newif\if@mathematic \@mathematictrue
\def\figurename{Fig.}
\def\vec#1{\ensuremath{\mathbf{#1}}}
