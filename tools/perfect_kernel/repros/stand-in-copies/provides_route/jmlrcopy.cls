%%
%% Copyright 2010-2022 Nicola Talbot (jmlr.cls, jmlrutils.sty)
%%
%% This work may be distributed and/or modified under the conditions of the LaTeX Project Public License, either
%% version 1.3 of this license or (at your option) any later version. The latest version of this license is in
%%   http://www.latex-project.org/lppl.txt
%% and version 1.3 or later is part of all distributions of LaTeX version 2005/12/01 or later.
%%
%% MODIFIED: this is a modified and shortened copy of part of jmlr.cls (v1.08, 2010/03/09; the theorem wrap TeX
%% Live's jmlrutils.sty keeps), made for latexml-oxide's repros (tools/perfect_kernel/repros/stand-in-copies/
%% provides_route/), not the original work: it keeps only the `\newtheorem` wrap (jmlr.cls:753-779), on article.
%%
%% A renamed copy of an older jmlr.cls (myjmlr.cls, jmlr_mod.cls: `\ProvidesClass{jmlr}`), cut down to its theorem wrap
%% (jmlr.cls:753-779), which the jmlr binding's jmlrutils.sty makes too
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{jmlr}[2010/03/09 v1.08 (NLCT) JMLR Class]
\LoadClass{article}
\RequirePackage{etoolbox}
\newcommand*{\@theorembodyfont}{\normalfont\itshape}
\newcommand*{\@theoremheaderfont}{\normalfont\bfseries }
\let\jmlr@org@newtheorem\newtheorem
\renewcommand*{\newtheorem}{\@ifstar\jmlr@snewtheorem\jmlr@newtheorem}
\newcommand*{\jmlr@snewtheorem}[2]{\newenvironment{#1}{\par\textbf{#2.}\itshape}{\par}}
\newcommand{\jmlr@newtheorem}[1]{%
  \cslet{jmlr@thm@#1@body@font}{\@theorembodyfont}%
  \jmlr@org@newtheorem{#1}%
}
\newtheorem{example}{Example}
\endinput
