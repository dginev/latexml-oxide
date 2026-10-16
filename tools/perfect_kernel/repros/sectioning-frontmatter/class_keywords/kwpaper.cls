% The shape of paper.cls:621-627 (TeX Live's paper class, read raw under the rawclasses lever): `{keywords}` is an
% environment whose begin code only typesets a run-in heading carrying the class's label.
\ProvidesClass{kwpaper}
\LoadClass{article}
\newcommand\keywordname{Keywords}
\def\keywords{\small\paragraph*{\keywordname:}}
\def\endkeywords{\par\bigskip}
