% Minimal stand-in for svmult.cls v1.1 (astro-ph0108509 ships it): the line the svmult binding reads to choose the
% version's running heads (:37-38, `\DeclareOption{runningheads}{\@runheadtrue}`). pdflatex validated with the real copy.
\ProvidesClass{svmult}[2000/01/01 v1.1]
\newif\if@runhead \@runheadfalse
\DeclareOption{runningheads}{\@runheadtrue}
