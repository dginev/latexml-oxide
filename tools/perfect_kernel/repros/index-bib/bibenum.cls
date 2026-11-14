\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{bibenum}
\LoadClass{article}
\def\bibreferences#1{\section*{\refname}\begin{enumerate}}
\def\endbibreferences{\end{enumerate}}
\let\thebibliography\bibreferences
\let\endthebibliography\endbibreferences
