\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{jfm}[2007/01/01 v1.18 Journal of Fluid Mechanics (repro copy)]
\LoadClass{article}
% 0712.2739's jfm.cls:943-965 (shortened): the class's own caption, "Figure 1." with the label in small caps
\newif\ifrem@fullpt
\newcommand\removefullpoint{\global\rem@fullpttrue}
\long\def\@makecaption#1#2{%
  \vskip 5\p@
  \begingroup
    \small\setbox\@tempboxa\hbox{{\normalfont\scshape
     #1\ifrem@fullpt\else .\fi} #2}%
    \ifdim \wd\@tempboxa >\hsize
      \unhbox\@tempboxa\par
    \else
      {\centering \unhbox\@tempboxa\par}%
    \fi
  \endgroup
  \global\rem@fullptfalse}
