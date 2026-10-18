% Two class setters insert into one list the class's `\maketitle` prints, each just before its end sentinel: the
% refused `\author` (replayed for its side effects) and a class-only `\workplace` (75a review r2 finding 1).
\ProvidesClass{tpsentinel}
\LoadClass{article}
\let\tp@end\relax
\def\tp@block{\tp@end}
\long\def\tp@ins#1{\long\def\tp@tmp##1\tp@end{\gdef\tp@block{##1#1\tp@end}}\expandafter\tp@tmp\tp@block}
\renewcommand{\author}[1]{\tp@ins{\par\textbf{#1}}}
\newcommand{\workplace}[1]{\tp@ins{\par\textit{#1}}}
\renewcommand{\maketitle}{\par{\centering\Large\@title\par}\tp@block\par}
