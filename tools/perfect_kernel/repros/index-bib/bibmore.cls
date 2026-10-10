\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{bibmore}
\LoadClass{article}
\def\bibreferences#1{\section*{\refname}\list{\@biblabel{\@arabic\c@enumiv}}{\usecounter{enumiv}}}
\def\endbibreferences{\endlist\par\noindent\textit{End of references.}}
\let\thebibliography\bibreferences
\let\endthebibliography\endbibreferences
