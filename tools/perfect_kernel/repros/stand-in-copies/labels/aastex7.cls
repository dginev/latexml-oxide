% The paper's copy of aastex7.cls, cut to the line that matters: aastex7.cls:635 (2025/02/06; aastex701.cls,
% siamltex.cls, emulateapj.cls, jasatex.cls alike) redefines \label to write the label to the .aux file.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{aastex7}[2025/02/06 Version 7/AAS markup document class]
\LoadClass{article}
\def\label#1{%
 \@bsphack
  \protected@write\@auxout{}{%
   \string\newlabel{#1}{{\@currentlabel}{\thepage}{}{}{}}%
  }%
 \@esphack
}%
