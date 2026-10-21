%%
%% Copyright 2014 by VTEX UAB (vmsta.cls)
%%
%% This work may be distributed and/or modified under the conditions of the LaTeX Project Public License, either
%% version 1.2 of this license or (at your option) any later version. The latest version of this license is in
%%   http://www.latex-project.org/lppl.txt
%%
%% MODIFIED: a reduced model of part of vmsta.cls (1506.02940's copy), made for latexml-oxide's repros
%% (tools/perfect_kernel/repros/sectioning-frontmatter/class_title_page/), not the original work.
%%
% The shape of vmsta.cls:84-93, :296-315, :534-545 (1506.02940): the class's `\author` appends each name to its own
% list, and its `\maketitle` prints that list and the dates it stores.
\ProvidesClass{tplist}
\LoadClass{article}
\def\tp@authors{}\let\tp@sep\@empty
\renewcommand{\author}[1]{\g@addto@macro\tp@authors{\tp@sep#1\def\tp@sep{, }}}
\let\@received\@empty
\def\received#1{\gdef\@received{{Received:~\hbox{#1}}}}
\renewcommand{\maketitle}{\par
  {\centering\Large\@title\par}%
  {\centering\bfseries\tp@authors\par}%
  {\centering\small\@received\par}}
