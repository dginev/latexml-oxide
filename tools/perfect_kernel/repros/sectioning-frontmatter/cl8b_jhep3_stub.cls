% A stand-in for JHEP3.cls (not in TeX Live) for pdflatex: LaTeXML reads its own JHEP3 binding (`jhep3_cls.rs`, `\email`
% = the frontmatter API's `\lx@add@email`), never this file. JHEP3 prints the address in typewriter.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{cl8b_jhep3_stub}
\LoadClass{article}
\def\email#1{\texttt{#1}}
\def\abstract#1{\gdef\@abstract{#1}}
\AtBeginDocument{\maketitle}
