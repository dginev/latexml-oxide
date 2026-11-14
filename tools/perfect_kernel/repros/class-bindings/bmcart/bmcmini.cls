%%
%% bmcart.cls (BioMed Central's article class): "It may be distributed under the terms of the LaTeX Project Public
%% License, as described in lppl.txt in the base LaTeX distribution. Either version 1.0 or, at your option, any later
%% version." — and, if changed, renamed.
%%
%% MODIFIED and RENAMED: bmcmini.cls is a modified, shortened copy of part of bmcart.cls (as 2305.08284 ships it), made
%% for latexml-oxide's repros (tools/perfect_kernel/repros/class-bindings/bmcart/), not the original work.
%%
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{bmcmini}[2023/01/01 bmcart cut down]
\LoadClass{article}
% bmcart.cls:712-717 and :1503-1510
\def\figure{\@ifnextchar[{\@figure}{\@figure[t]}}
\def\@figure[#1]{\def\@captype{figure}\def\csentence##1{{\mathversion{bold}\bfseries ##1}}\@float{figure}[#1]}
\gdef\bmc@abstract{\@ifnextchar[{\abstract@fmt}{\abstract@fmt[]}}
\def\abstract@fmt[#1]{\def\parttitle##1{\vskip3\p@\noindent{\bfseries ##1:}}\noindent{\abstractname}\par}
\def\set@frontmatter@cmd{\let\abstract\bmc@abstract}
\AtBeginDocument{\set@frontmatter@cmd}
