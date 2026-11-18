%%
%% Copyright 1993-2009 The LaTeX3 Project and any individual authors listed elsewhere in sn-jnl.cls (Springer
%% Nature's sn-jnl.cls, generated from files of the LaTeX base system)
%%
%% This work may be distributed and/or modified under the conditions of the LaTeX Project Public License, either
%% version 1.3c of this license or (at your option) any later version. The latest version of this license is in
%%   http://www.latex-project.org/lppl.txt
%% and version 1.3c or later is part of all distributions of LaTeX version 2005/12/01 or later.
%%
%% MODIFIED: this is a modified excerpt of 2209.11450's sn-jnl.cls, made for latexml-oxide's repros
%% (tools/perfect_kernel/repros/class-bindings/snjnl_v2209/), not the original work: it keeps only the lines named below.
%%
% The lines of 2209.11450's sn-jnl.cls that the sn-jnl binding reads (an excerpt, not a working class):
% :506-516, a starred name's superscript with no `*` after its labels (`{}`),
\newcommand{\@@corrauthor}[2][]{\def\@authfrstarg{#1}\@corauemailtrue\advance\corraucount by 1
\g@addto@macro\artauthors{
    \global\@auemailtrue
    \Authorfont
    \def\baselinestretch{1}
    \authorsep{#2}\unskip\ifx\@authfrstarg\empty\else\textsuperscript{\smash{{
      \@for\@@affmark:=#1\do{\edef\affnum{\@ifundefined{X@\@@affmark}{\@@affmark}{\jmkRef{\@@affmark}}}
     \unskip\sep\affnum\let\sep=,}}}{}\hskip-1pt}\fi\unskip
    \def\authorsep{\au@and~}
    \global\let\sep\@empty\global\let\@corref\@empty
}}
% :585-587, `\equalcont` appending nothing to the names,
\newif\ifequalcont
\def\@equalconttxt{}
\def\equalcontxt#1{\gdef\@equalconttxt{#1}}\equalcontxt{}
\newcommand{\equalcont}[1]{\gdef\@equalconttext{#1}\g@addto@macro\artauthors{}\global\equalconttrue}
% and :763 its print, no corresponding line before it (commented out in the copy).
%        \ifequalcont{\par\textsuperscript{*}\@equalconttext\par}\fi
