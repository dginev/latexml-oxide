% Synthetic stand-in for EDP Sciences' edpsmath.cls (the shapes of 1110.0287's and 1901.04804's copies): a raw class
% over amsart whose `\keywords` sets a box that its `\@maketitle` prints in a footnote, and whose journal option file
% (edpsproc.clo) empties `\@setkeywords` at the end of the class.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{edpsmath}[2024/01/01 synthetic]
\DeclareOption*{\PassOptionsToClass{\CurrentOption}{amsart}}
\ProcessOptions\relax
\LoadClass[reqno]{amsart}
\let\journalopt\@empty
\DeclareOption*{\InputIfFileExists{edps\CurrentOption.clo}{\let\journalopt\CurrentOption}{\OptionNotUsed}}
\ProcessOptions\relax
\def\@maketitle{\normalfont\normalsize
  \vbox{\hbox to\hsize{\bfseries Synthetic Journal\hfill}}%
  \@setkeywords\@settitle\@setauthors\@setabstract}
\newbox\@b@kwrds
\def\keywords#1{\global\setbox\@b@kwrds\vtop{\advance\hsize by-12pt
    \noindent\footnotesize\textit{\keysname.~}#1}}
\def\@setkeywords{\ifvoid\@b@kwrds\footnote{\@MSSG@KWRD}\else\footnote{\box\@b@kwrds}\fi}
\providecommand{\keysname}{Keywords and phrases}
\def\@MSSG@KWRD{{WARNING: give at least one key word}}
