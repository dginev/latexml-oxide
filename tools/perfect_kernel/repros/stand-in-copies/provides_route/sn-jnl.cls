%%
%% Copyright 1993-2009 The LaTeX3 Project and any individual authors listed elsewhere in sn-jnl.cls (Springer
%% Nature's sn-jnl.cls, generated from files of the LaTeX base system)
%%
%% This work may be distributed and/or modified under the conditions of the LaTeX Project Public License, either
%% version 1.3c of this license or (at your option) any later version. The latest version of this license is in
%%   http://www.latex-project.org/lppl.txt
%% and version 1.3c or later is part of all distributions of LaTeX version 2005/12/01 or later.
%%
%% MODIFIED: this is a modified and shortened copy of sn-jnl.cls (2207.07492's copy), made for latexml-oxide's repros
%% (tools/perfect_kernel/repros/stand-in-copies/provides_route/), not the original work: it keeps only what is named
%% below.
%%
%% The paper's own sn-jnl.cls, reduced to what its read runs through pgf: its tikz load (2207.07492's copy,
%% sn-jnl.cls:302-303) and a recursion over pgf's scratch registers defined in the copy's own text (the loop idiom
%% 2101.08181's lmcs.cls copy reached through pgf).
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{sn-jnl}[2021/10/26 v1.0 the paper's copy]
\LoadClass{article}
\usepackage{tikz}
\usetikzlibrary{snakes}
\def\pgfutil@reserved@b#1{\ifx#1\relax\else[#1]\expandafter\pgfutil@reserved@c\fi}
\let\pgfutil@reserved@c\pgfutil@reserved@b
\edef\snloopresult{\pgfutil@reserved@c abc\relax}
\endinput
