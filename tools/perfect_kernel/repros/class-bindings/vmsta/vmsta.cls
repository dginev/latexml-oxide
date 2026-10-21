\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{vmsta}[2015/06/08 vmsta.cls v1.1 cut down]
\LoadClass{article}
% vmsta.cls:270-291, :307-322: the author list the title page prints; the name parts are defined only there
% (`\name@m` the identities, `\name@m@front` storing them and printing `\@fnm~\@snm`, the initials dropped)
\def\@@@authors{}
% (not in vmsta.cls, whose `\@fnm~\@snm` errors for a name without `\snm`: the cut-down class's names print without it)
\let\@fnm\@empty\let\@snm\@empty
\let\authorsep\@empty
\def\author{\@ifnextchar[{\@@author}{\@@author[]}}
\def\name@m{\def\inits##1{}\def\fnm##1{##1}\def\snm##1{##1}\def\suffix##1{##1}}
\def\name@m@front#1{\let\inits\@gobble\def\fnm##1{\def\@fnm{##1}}\def\snm##1{\def\@snm{##1}}\let\suffix\@gobble
  #1\@fnm~\@snm}
\def\@@author[#1]#2{\g@addto@macro\@@@authors{\bgroup\bfseries\name@m\authorsep\name@m@front{#2}\egroup
  \def\authorsep{\unskip,\space}}}
\def\maketitle{\par{\centering\@@@authors\par}}
