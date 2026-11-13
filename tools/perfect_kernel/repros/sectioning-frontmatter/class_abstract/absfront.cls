% A stand-in written from scratch for the shape of frontiersFPHY.cls:516-524 (1508.06756's copy; Frontiers' classes,
% not in TeX Live, no licence stated; FrontiersinHarvard, frontiersSCNS, bioinfo.cls:495-500): the class's `{abstract}`
% (`\newenvironment` there; `\renewenvironment` over article here) makes `\section` a run-in bold label inside its own
% group. No code of the class is copied.
\ProvidesClass{absfront}
\LoadClass{article}
\newcommand\abf@runin[1]{\par\noindent\textbf{#1}\space\ignorespaces}
\renewenvironment{abstract}{\begingroup\let\section\abf@runin
  \fontfamily{\sfdefault}\fontsize{12}{12}\selectfont{\fontseries{b}\selectfont ABSTRACT}\par}%
  {\endgroup\bigskip\@afterheading\@afterindentfalse\vskip 12pt plus 3pt minus 1pt}
