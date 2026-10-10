\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{bibrefs}
\LoadClass{article}
% ajour.cls:2395-2427 (0711.2301's copy), abridged: its own {references} list, and {thebibliography} \let to a copy
\def\references{\section*{References}\list{\@biblabel{\arabic{enumiv}}}%
  {\settowidth\labelwidth{\@biblabel{10.}}\leftmargin\labelwidth\usecounter{enumiv}%
   \let\p@enumiv\@empty\def\theenumiv{\arabic{enumiv}}}}
\def\bibreferences#1{\references}
\def\endreferences{\endlist}
\let\thebibliography\bibreferences
\let\endthebibliography\endreferences
