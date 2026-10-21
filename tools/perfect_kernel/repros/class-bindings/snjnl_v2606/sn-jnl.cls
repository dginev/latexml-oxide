%%
%% Copyright 1993-2009 The LaTeX3 Project and any individual authors listed elsewhere in sn-jnl.cls (Springer
%% Nature's sn-jnl.cls, generated from files of the LaTeX base system)
%%
%% This work may be distributed and/or modified under the conditions of the LaTeX Project Public License, either
%% version 1.3c of this license or (at your option) any later version. The latest version of this license is in
%%   http://www.latex-project.org/lppl.txt
%% and version 1.3c or later is part of all distributions of LaTeX version 2005/12/01 or later.
%%
%% MODIFIED: this is a modified excerpt of two sn-jnl.cls copies (2504.21788's, 2606.24824's), made for
%% latexml-oxide's repros (tools/perfect_kernel/repros/class-bindings/snjnl_v2606/), not the original work: it keeps
%% only the lines named below.
%%
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
