% Two class setters append to one list the class's `\maketitle` prints: the refused `\author` (replayed for its side
% effects) and a class-only `\workplace` (75a review finding 2).
\ProvidesClass{tpshared}
\LoadClass{article}
\def\tp@block{}
\renewcommand{\author}[1]{\g@addto@macro\tp@block{\par\textbf{#1}}}
\newcommand{\workplace}[1]{\g@addto@macro\tp@block{\par\textit{#1}}}
\renewcommand{\maketitle}{\par{\centering\Large\@title\par}\tp@block\par}
