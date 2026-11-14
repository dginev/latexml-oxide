% A stand-in written from scratch for the shape of bioinfo.cls:495-500 (1406.7114's copy; Oxford University Press's
% Bioinformatics class, not in TeX Live, no licence stated; frontiersFPHY.cls:516-524, 1508.06756): the class defines
% `{abstract}` with `\newenvironment` (here after `\let`ting article's away, as LaTeX's `\@ifundefined` allows), its
% begin code making `\section` a run-in bold label inside its own group. No code of the class is copied.
\ProvidesClass{absbioinfo}
\LoadClass{article}
\newcommand\abb@runin[1]{\par\noindent\textbf{#1}\space\ignorespaces}
\let\abstract\relax
\let\endabstract\relax
\newenvironment{abstract}{\begingroup\let\section\abb@runin
  \fontfamily{\sfdefault}\fontsize{8}{11}\selectfont{\fontseries{b}\selectfont ABSTRACT}\par}%
  {\endgroup\bigskip\@afterheading\@afterindentfalse\vskip 12pt plus 3pt minus 1pt}
