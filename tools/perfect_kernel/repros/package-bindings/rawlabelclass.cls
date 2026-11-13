% A class with no binding, read raw, cut to the line that matters: SIAM's siamltex-derived classes (CMSLATEX.CLS:736,
% siuro210301.cls) redefine \label as the kernel's write of \newlabel to the .aux file.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{rawlabelclass}[2026/10/24 v1.0 test class]
\LoadClass{article}
\def\label#1{\@bsphack
  \protected@write\@auxout{}{\string\newlabel{#1}{{\@currentlabel}{\thepage}}}%
  \@esphack}
