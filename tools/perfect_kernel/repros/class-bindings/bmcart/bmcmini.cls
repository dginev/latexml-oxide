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
