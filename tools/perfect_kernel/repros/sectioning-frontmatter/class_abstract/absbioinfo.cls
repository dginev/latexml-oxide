% The shape of bioinfo.cls:495-500 (1406.7114's copy; frontiersFPHY.cls:516-524, 1508.06756): the class defines
% `{abstract}` with `\newenvironment` (here after `\let`ting article's away, as LaTeX's `\@ifundefined` allows), its
% begin code making `\section` a run-in bold label inside its own group.
\ProvidesClass{absbioinfo}
\LoadClass{article}
\newcommand{\absection}[1]{%
  \par\noindent{\bfseries #1}\space\ignorespaces}
\let\abstract\relax
\let\endabstract\relax
\newenvironment{abstract}{%
  \begingroup
  \let\section\absection
  \fontfamily{\sfdefault}\fontsize{8}{11}\sffamily\selectfont
  {\fontseries{b}\selectfont ABSTRACT}\par}
{\endgroup\bigskip\@afterheading\@afterindentfalse\vskip 12pt plus 3pt minus 1pt}
