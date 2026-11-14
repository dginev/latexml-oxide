%%
%% `\@pushfilename` below is the LaTeX2e kernel's (latex.ltx, before 2020), as pasa.cls copies it: Copyright (C)
%% 1993-2019 The LaTeX Project and any individual authors listed elsewhere in latex.ltx.
%%
%% This work may be distributed and/or modified under the conditions of the LaTeX Project Public License, either
%% version 1.3c of this license or (at your option) any later version. The latest version of this license is in
%%   http://www.latex-project.org/lppl.txt
%% and version 1.3c or later is part of all distributions of LaTeX version 2005/12/01 or later.
%%
%% MODIFIED: taken into this stand-in for latexml-oxide's repros (tools/perfect_kernel/repros/loader/), not the
%% original work.
%%
% pasa.cls:45-52 (2013/04/01 v1.0, shipped by 1307.3127 and its siblings): the LaTeX2e kernel's pre-2020
% `\@pushfilename`, called once bare, then a package loaded from the class (:1339).
\ProvidesClass{pasa}
\LoadClass{article}
\def\@pushfilename{%
\xdef\@currnamestack{%
	{\@currname}%
	{\@currext}%
	{\the\catcode`\@}%
	\@currnamestack}}%
	\@onlypreamble\@pushfilename%
	\@pushfilename%

\newlength\trimwidth
\usepackage{graphicx}
