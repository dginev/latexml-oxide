%% The paper's copy of sn-jnl.cls (2305.14168, 2408.00130, 2409.14383, 2503.20196, 2506.14776), cut down to its theorem
%% block (:1547-1640): its theorem styles exist only where amsthm is loaded before the class, and this version does not
%% load it; thmstylefour's head sets a flag no file declares.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{sn-jnl}[2023/06/05 v2.1]
\LoadClass{article}
\@ifpackageloaded{amsthm}{%
\newtheoremstyle{thmstylefour}%
{18pt plus2pt minus1pt}{18pt plus2pt minus1pt}{\small\normalfont}{0pt}{\small\itshape}{}{.5em}%
{\global\proofthmtrue\thmname{#1} \thmnote{#3}}%
}{}
\endinput
