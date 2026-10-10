\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{biblet}
\LoadClass{article}
% a class putting its own list in the kernel's bibliography by \let (ajour, cjour, IEEE-Con-Sys-mag)
\def\bibreferences#1{\section*{\refname}\list{\@biblabel{\@arabic\c@enumiv}}{\usecounter{enumiv}}}
\let\endbibreferences\endlist
\let\thebibliography\bibreferences
\let\endthebibliography\endbibreferences
