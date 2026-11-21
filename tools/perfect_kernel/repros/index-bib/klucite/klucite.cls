% SolarPhysics.cls (1312.4435), solarphysics.cls (0705.1246), SSRv.cls: the `[namedreferences]` citation lines
% (:2048-2101) — the aux-writer `\citeauthoryear`, and a `\cite` whose `\klu@cite` redefines it at every call
\ProvidesClass{klucite}
\LoadClass{article}
\let\oldcite\cite
\let\@internalcite\oldcite
\def\cite{\@ifstar{\citeyear}{\klu@cite}}
\def\klu@cite{\def\@citeseppen{1000}%
  \def\@cite##1##2{(##1\if@tempswa , ##2\fi)}%
  \def\citeauthoryear##1##2{{\rm ##1, ##2}}\@internalcite }
\def\citeyear#1{\def\@citeseppen{1000}%
  \def\@cite##1##2{{##1\if@tempswa , ##2\fi}}%
  \def\citeauthoryear##1##2{{\rm ##2}}\@internalcite{#1}}
\def\citeauthoryear#1#2{\string\citeauthoryear{#1}{#2}}
