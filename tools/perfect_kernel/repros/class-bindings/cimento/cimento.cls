%%
%% The page setup lines are minimal.cls's: Copyright (C) 1993-2025 The LaTeX Project and any individual authors
%% listed elsewhere in that file.
%%
%% This work may be distributed and/or modified under the conditions of the LaTeX Project Public License, either
%% version 1.3c of this license or (at your option) any later version. The latest version of this license is in
%%   http://www.latex-project.org/lppl.txt
%% and version 1.3c or later is part of all distributions of LaTeX version 2005/12/01 or later.
%%
%% MODIFIED: those lines are taken, modified, from minimal.cls into this stand-in for latexml-oxide's repros
%% (tools/perfect_kernel/repros/class-bindings/cimento/); the rest is a model of cimento.cls's author list.
%%
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{cimento}[2005/01/01 cimento.cls cut down]
% self-contained, as cimento.cls is (no \LoadClass); the page setup of minimal.cls
\renewcommand\normalsize{\fontsize{10pt}{12pt}\selectfont}
\setlength{\textwidth}{6.5in}
\setlength{\textheight}{8in}
\pagenumbering{arabic}
% cimento.cls:680, :704-706, :708-757: the author list ends with `|`; `\ETC`, defined in `\maketitle` only, keeps
% the rest of it on the title page (`\scrunch` drops it for the running head)
\catcode`\|=\active
\renewcommand{\author}[1]{\gdef\@author@{#1|}}
\gdef\unscrunch|#1|{#1}
\gdef\scrunch|#1|{}
\def\@maketitle{{\centering\scshape\@title\par\@author@\par}}
\newcommand{\maketitle}{\par\begingroup\def\ETC{\unskip\unscrunch|}\def|{}\@maketitle\endgroup}
\catcode`\|=12
