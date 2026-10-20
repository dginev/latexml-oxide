%% A derivative of the standard article class that loads a style a binding stands in for (jheppub) and then
%% `\providecommand`s that style's journal name: the style's definition is the first, as in LaTeX.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{article}[2020/01/01 v1 a derivative]
\renewcommand\normalsize{\fontsize{10}{12}\selectfont}
\setlength\textwidth{6.5in}
\setlength\textheight{8in}
\pagenumbering{arabic}
\RequirePackage{jheppub}
\providecommand\jname{J. High Energy Phys.}
\endinput
