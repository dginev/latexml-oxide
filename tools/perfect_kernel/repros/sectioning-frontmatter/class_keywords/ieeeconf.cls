% The `abstract`/`keywords` begin codes of the ieeeconf.cls 1401.5657 ships (an IEEEtran.cls V1.6b derivative,
% :3800-3818), on article: under `conference` (two columns) `\keywords` is a label with no extent.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{ieeeconf}[2014/01/01 minimal ieeeconf keywords for a repro]
\DeclareOption{conference}{\OptionNotUsed}
\ProcessOptions
\LoadClass[twocolumn]{article}
\def\keywords{\normalfont\bfseries\textit{Index Terms}---\,\relax}
\def\endkeywords{\relax\vspace{0.67ex}\par\normalsize\normalfont}
