% The guard fixture of emit_a_symbol_inside_an_entrys_brackets_is_its_text: a raw class storing `\institute` (cernall.sty:452).
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{emit_institute_store}
\LoadClass{article}
\def\institute#1{\gdef\@institute{#1}}
\def\@maketitle{\begin{center}{\LARGE\@title\par}\vskip 1em{\large\@author\par}\vskip 1em{\small\@institute\par}\end{center}}
