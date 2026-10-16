% The lines of two sn-jnl.cls copies that the sn-jnl binding reads (an excerpt, not a working class):
% 2504.21788's :613-617, a `\equalcont` whose text is optional, the class's default in its place,
\newif\ifequalcont%
\def\@equalconttxt{}% 
\def\equalcontxt#1{\gdef\@equalconttxt{#1}}\equalcontxt{These authors contributed equally to this work.}%
\newcommand{\equalcont}[1][\@equalconttxt]{\gdef\@equalconttext{#1}\g@addto@macro\artauthors{$^{\dagger}$}\global\equalconttrue}%
% and 2606.24824's :596-601, the same for `\intern`.
\newif\ifintern%
\def\@interntxt{}% 
\def\interntxt#1{\gdef\@interntxt{#1}}\interntxt{Work done as an intern at Westlake University.}%
\newcommand{\intern}[1][\@interntxt]{\gdef\@interntext{#1}\g@addto@macro\artauthors{$^{\ddagger}$}\global\interntrue}%
