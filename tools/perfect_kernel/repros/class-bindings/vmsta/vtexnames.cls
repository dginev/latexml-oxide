%%
%% Copyright 2014 by VTEX UAB (vmsta.cls)
%%
%% This work may be distributed and/or modified under the conditions of the LaTeX Project Public License, either
%% version 1.2 of this license or (at your option) any later version. The latest version of this license is in
%%   http://www.latex-project.org/lppl.txt
%%
%% MODIFIED: a modified excerpt of vmsta.cls (v1.1), renamed, made for latexml-oxide's repros
%% (tools/perfect_kernel/repros/class-bindings/vmsta/), not the original work.
%%
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{vtexnames}[2015/01/01 vmsta.cls cut down, under a name no binding answers]
\LoadClass{article}
\RequirePackage{etoolbox}
% vmsta.cls:271-283, :308-312: the author's name parts are defined only where the class sets the author list
\def\@@@authors{}
\def\author{\@ifnextchar[{\@@author}{\@@author[]}}
\csdef{name@m}{\def\inits##1{}\def\fnm##1{##1}\def\snm##1{##1}}
\def\@@author[#1]#2{\g@addto@macro\@@@authors{\name@m #2\par}}
\def\maketitle{{\bfseries\@@@authors}}

