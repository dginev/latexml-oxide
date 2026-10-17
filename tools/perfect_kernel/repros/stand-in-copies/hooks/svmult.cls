\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{svmult}[2007/06/25 v5.4]
\LoadClass{book}
\newcommand\svextra{SV-EXTRA}
\AtBeginDocument{\def\bibsection{\section*{COPY REFERENCES}}}
\renewenvironment{thebibliography}[1]{\bibsection\list{[\arabic{enumiv}]}{\usecounter{enumiv}}}{\endlist}
