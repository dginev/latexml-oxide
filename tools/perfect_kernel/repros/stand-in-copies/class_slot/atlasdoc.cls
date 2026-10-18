% A reduced model of atlasdoc.cls (2509.01219): its declaration, its main class by macro, its own `\LoadClass` and a
% KOMA call after it (v13 :382-384).
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{atlasdoc}[2019/01/10]
\newcommand*{\ADOC@mainclass}{scrartcl}
\ProcessOptions
\LoadClass[titlepage, 11pt]{\ADOC@mainclass}
\KOMAoptions{paper=a4}
\setkomafont{title}{\bfseries}
\newcommand{\ATLASextra}{ATLAS}
