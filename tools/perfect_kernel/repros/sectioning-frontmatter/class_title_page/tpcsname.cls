% A class `\author` that `\def`s a `\csname`-made macro with a parameter of its own (`##1`) beside the argument it was
% given (`#1`): the replay gate cannot know that name or parameter text before the body runs (review finding 1 of 75a).
\ProvidesClass{tpcsname}
\LoadClass{article}
\def\tp@au{}
\renewcommand{\author}[1]{\expandafter\def\csname tp@au#1\endcsname##1{##1}\gdef\tp@au{#1}}
\renewcommand{\maketitle}{\par\noindent Authors: \tp@au\par}
