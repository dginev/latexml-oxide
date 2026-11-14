% A stand-in written from scratch for the shape of ajour.cls:2395-2427 (0711.2301's copy; Academic Press's class, not in
% TeX Live, no licence stated): the class's own numbered `{references}` list, and `{thebibliography}` `\let` to a copy of
% it. No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{bibrefs}
\LoadClass{article}
\def\references{\section*{References}%
  \list{\@biblabel{\arabic{enumiv}}}{\settowidth\labelwidth{\@biblabel{10.}}\setlength\leftmargin{\labelwidth}%
    \usecounter{enumiv}\let\p@enumiv\@empty\renewcommand\theenumiv{\arabic{enumiv}}}}
\def\endreferences{\endlist}
\def\bibrfs@thebibliography#1{\references}
\let\thebibliography\bibrfs@thebibliography
\let\endthebibliography\endreferences
