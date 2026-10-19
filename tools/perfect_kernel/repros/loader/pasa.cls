% pasa.cls:45-52 (2013/04/01 v1.0, shipped by 1307.3127 and its siblings): the LaTeX2e kernel's pre-2020
% `\@pushfilename`, called once bare, then a package loaded from the class (:1339).
\ProvidesClass{pasa}
\LoadClass{article}
\def\@pushfilename{%
\xdef\@currnamestack{%
	{\@currname}%
	{\@currext}%
	{\the\catcode`\@}%
	\@currnamestack}}%
	\@onlypreamble\@pushfilename%
	\@pushfilename%

\newlength\trimwidth
\usepackage{graphicx}
