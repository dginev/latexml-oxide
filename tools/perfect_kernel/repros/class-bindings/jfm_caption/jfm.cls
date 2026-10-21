% A minimal stand-in for jfm.cls v1.18 (Cambridge University Press's Journal of Fluid Mechanics class, the copy 0712.2739
% ships — not in TeX Live; its notice allows use only for CUP publications), written from scratch for the repros beside
% it: the class's own caption (:943-965) — "Figure 1." with the label in small caps, the period dropped once after
% `\removefullpoint` — with what the class does there. No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{jfm}[2007/01/01 v1.18 Journal of Fluid Mechanics (repro stand-in)]
\LoadClass{article}
\newif\ifrem@fullpt
\newcommand\removefullpoint{\global\rem@fullpttrue}
\long\def\@makecaption#1#2{\vskip 5pt
  {\small\sbox\@tempboxa{{\normalfont\scshape#1\ifrem@fullpt\else.\fi} #2}%
   \ifdim\wd\@tempboxa>\hsize\unhbox\@tempboxa\par\else{\centering\unhbox\@tempboxa\par}\fi}%
  \global\rem@fullptfalse}
