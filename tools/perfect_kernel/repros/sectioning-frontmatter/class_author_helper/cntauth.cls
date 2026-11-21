% A stand-in for bullsrsl.cls:325-358,958-961 (2312.08217, 2607.02675): `\author` hands its call to the class's own
% `\@author[#1]#2#3`, which counts the authors; the class's environment errs when the count is 1.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{cntauth}
\LoadClass{article}
\newcounter{nbauth}
\def\author{\@ifnextchar[\@author{\@author[]}}
\def\@author[#1]#2#3{\addtocounter{nbauth}{1}}
\newenvironment{contrib}{\ifnum\value{nbauth}=1 \ClassError{cntauth}{Unwanted contrib}{}\fi}{}
