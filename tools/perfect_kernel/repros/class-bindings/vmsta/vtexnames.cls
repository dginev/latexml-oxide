\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{vtexnames}[2015/01/01 vmsta.cls cut down, under a name no binding answers]
\LoadClass{article}
\RequirePackage{etoolbox}
% vmsta.cls:271-283, :308-312: the author's name parts are defined only where the class sets the author list
\def\@@@authors{}
\def\author{\@ifnextchar[{\@@author}{\@@author[]}}
\csdef{name@m}{\def\inits##1{}\def\fnm##1{##1}\def\snm##1{##1}}
\def\@@author[#1]#2{\g@addto@macro\@@@authors{\name@m #2\par}}
\def\maketitle{{\bfseries\@@@authors}}

