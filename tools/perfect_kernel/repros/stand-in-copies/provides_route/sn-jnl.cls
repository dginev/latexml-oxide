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
