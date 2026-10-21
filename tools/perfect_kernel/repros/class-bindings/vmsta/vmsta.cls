%%
%% Copyright 2014 by VTEX UAB (vmsta.cls)
%%
%% This work may be distributed and/or modified under the conditions of the LaTeX Project Public License, either
%% version 1.2 of this license or (at your option) any later version. The latest version of this license is in
%%   http://www.latex-project.org/lppl.txt
%%
%% MODIFIED: this is a modified and shortened copy of vmsta.cls (v1.1, 2015/06/08), made for latexml-oxide's repros
%% (tools/perfect_kernel/repros/class-bindings/vmsta/), not the original work: it keeps only the frontmatter named
%% below, on article. This modified copy, kept under the original file name, is distributed under LPPL 1.3c (the
%% original's "any later version"); it says it is modified here and in its `\ProvidesClass` line (LPPL 1.3c clause 6).
%%
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{vmsta}[2015/06/08 vmsta.cls v1.1 cut down]
\LoadClass{article}
% vmsta.cls:270-291, :307-322: the author list the title page prints; the name parts are defined only there
% (`\name@m` the identities, `\name@m@front` storing them and printing `\@fnm~\@snm`, the initials dropped)
\def\@@@authors{}
% (not in vmsta.cls, whose `\@fnm~\@snm` errors for a name without `\snm`: the cut-down class's names print without it)
\let\@fnm\@empty\let\@snm\@empty
\let\authorsep\@empty
\def\author{\@ifnextchar[{\@@author}{\@@author[]}}
\def\name@m{\def\inits##1{}\def\fnm##1{##1}\def\snm##1{##1}\def\suffix##1{##1}}
\def\name@m@front#1{\let\inits\@gobble\def\fnm##1{\def\@fnm{##1}}\def\snm##1{\def\@snm{##1}}\let\suffix\@gobble
  #1\@fnm~\@snm}
\def\@@author[#1]#2{\g@addto@macro\@@@authors{\bgroup\bfseries\name@m\authorsep\name@m@front{#2}\egroup
  \def\authorsep{\unskip,\space}}}
% vmsta.cls:234-256, :347-390 (cut down): the e-mail written to the .aux for a later run's title page, the addresses
% appended to the title page's list
\let\@@@@eads\@empty
\def\email{\@ifnextchar[{\@uemail}{\@email}}
\gdef\@email#1{\immediate\write\@auxout{\string\emailauthor{#1}{}}}
\gdef\emailauthor#1#2{\g@addto@macro\@@@@eads{#1 }}
\def\@@@address{}
\def\address{\@ifnextchar[{\@@address}{\@address}}
\long\def\@@address[#1]#2{\g@addto@macro\@@@address{\textsuperscript{#1}#2\par}}
\long\def\@address#1{\g@addto@macro\@@@address{#1\par}}
\def\maketitle{\par{\centering\@@@authors\par{\itshape\@@@address}\@@@@eads\par}}
