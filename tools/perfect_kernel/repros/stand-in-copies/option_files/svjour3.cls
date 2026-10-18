\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{svjour3}[2007/05/08 v3.2]
\DeclareOption*{\InputIfFileExists{sv\CurrentOption.clo}{}{}}
\ProcessOptions
\LoadClass{article}
\newcommand\institute[1]{\gdef\@institute{#1}}
\renewcommand\maketitle{\par\@title\par\@author\par\@institute\par}
