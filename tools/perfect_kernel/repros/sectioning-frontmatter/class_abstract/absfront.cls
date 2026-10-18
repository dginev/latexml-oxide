% The shape of frontiersFPHY.cls:516-524 (1508.06756's copy; FrontiersinHarvard, frontiersSCNS, bioinfo.cls:495-500):
% the class's `{abstract}` (`\newenvironment` there; `\renewenvironment` over article here) makes `\section` a run-in bold label inside its own group.
\ProvidesClass{absfront}
\LoadClass{article}
\newcommand{\absection}[1]{%
  \par\noindent{\bfseries #1}\space\ignorespaces}
\renewenvironment{abstract}{%
  \begingroup
  \let\section\absection
  \fontfamily{\sfdefault}\fontsize{12}{12}\sffamily\selectfont
  {\fontseries{b}\selectfont ABSTRACT}\par}
{\endgroup\bigskip\@afterheading\@afterindentfalse\vskip 12pt plus 3pt minus 1pt}
