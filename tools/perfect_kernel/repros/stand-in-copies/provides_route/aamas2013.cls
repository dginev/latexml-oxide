%% An acmproc-era aamas class (1401.1086's confSt.cls, `\ProvidesClass{aamas2013}`, "based on acmproc.cls V1.3"),
%% reduced to its author block (confSt.cls:333-344, :403-405).
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{aamas2013}[2009/24/07 - V1.9 - based on acmproc.cls V1.3]
\LoadClass{article}
\newcount\aucount
\newdimen\auwidth \auwidth=\textwidth
\gdef\numberofauthors#1{\global\aucount=#1\global\divide\auwidth by \aucount}
\def\alignauthor{\end{tabular}\begin{tabular}[t]{p{\auwidth}}\centering}
\def\affaddr#1{#1}
\def\email#1{{\par #1}}
\endinput
