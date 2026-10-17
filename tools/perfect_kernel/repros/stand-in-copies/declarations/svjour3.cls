\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{svjour3}[2007/05/08 v3.2]
\DeclareOption{copyoption}{\def\copyoptionseen{yes}}
\ProcessOptions
\LoadClass{article}
\newcommand\institute[1]{\gdef\@institute{#1}}
\renewcommand\maketitle{\par COPY TITLE PAGE\par}
\renewcommand\section{\@startsection{section}{1}{0pt}{1ex}{1ex}{\bfseries COPY}}
\renewenvironment{abstract}{COPY ABSTRACT}{}
\renewcommand\theequation{Z\arabic{equation}}
\RequirePackage{amsmath}
\numberwithin{equation}{section}
\newif\ifextraflag \extraflagtrue
\newcounter{extractr}\setcounter{extractr}{7}
\newtheorem{extrathm}{ExtraThm}
\newenvironment{extraenv}{[}{]}
\let\extralet\textbf
\AtEndOfClass{\renewcommand\maketitle{COPY AT END OF CLASS}\newcommand\aeocextra{AEOC}}
