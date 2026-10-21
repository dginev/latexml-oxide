% The shape of paper.cls:621-627 (TeX Live's paper class, GPL, read raw under the rawclasses lever), written from
% scratch: `{keywords}` is an environment whose begin code only typesets a run-in heading carrying the class's label,
% `\keywordname`. No code of the class is copied.
\ProvidesClass{kwpaper}
\LoadClass{article}
\newcommand\keywordname{Keywords}
\newenvironment{keywords}{\small\paragraph*{\keywordname:}}{\par\bigskip}
