% The title-page shapes address_after_a_line_break.tex reads (79a): the class's own line "Vol. 1, 2026" ended by
% `\\[2pt]` before the address, set in a font a setter stores (`\addressfont`, paper.cls:109-111's shape).
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{zzfmc}
\LoadClass{article}
\def\@jinfo{Vol.~1, 2026}
\newcommand\addressfont[1]{\gdef\@addrfont{#1}}
\def\@addrfont{\small\itshape}
\newcommand\address[1]{\gdef\@address{#1}}
\gdef\@address{}
\def\@maketitle{\begin{center}{\LARGE\@title\par}{\large\@author\par}{\@jinfo\\[2pt]\@addrfont\@address}\end{center}}
