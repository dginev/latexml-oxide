%% aascopy.cls -- a shortened stand-in for 1402.7103's proto.cls, a renamed AASTeX 5.2 class, with a few of its lines
%% (cited): it declares itself aastex (proto.cls:64 `\ProvidesClass{aastex}`), loads article (:130), redefines the
%% caption labels (:1694-1695, verbatim but for the table's word), the abstract's name (:798, here "ABSTRACT" as the
%% AASTeX 6+ copies' :884) and the journal names (:1729-1731, `\ref@jnl` and `\apj` verbatim); the figure's name,
%% the theorem counter separator, the figure's id (`\thefigure@ID`), the first item label and a journal name that is
%% not text (`\aj`, a paragraph) are written for this repro; its own `\appendix` and `\acknowledgments` are
%% structure, which the aastex binding keeps.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{aastex}[2003/12/12 v5.2 stand-in]
\LoadClass{article}
\def\fnum@figure{{\rmfamily Fig.\space\thefigure.---}}
\def\fnum@table{{\rmfamily TABLE \thetable}}
\def\figurename{Fig.}
\def\abstractname{ABSTRACT}
\def\@thmcountersep{}
\def\thefigure@ID{ZZ\arabic{figure}}
\renewcommand\labelitemi{$\star$}
\let\jnl@style=\rmfamily
\def\ref@jnl#1{{\jnl@style#1}}
\newcommand\apj{\ref@jnl{\it Astrophys.\ J.}}
\newcommand\ao{\ref@jnl{Applied~Optics}}
\def\aj{\par\ref@jnl{Astron.\ J.}}
\def\appendix{\par\setcounter{section}{0}\def\thesection{\Alph{section}}}
\def\acknowledgments{\section*{Acknowledgments}}
\newcommand\aasextra{EXTRA}
\endinput
