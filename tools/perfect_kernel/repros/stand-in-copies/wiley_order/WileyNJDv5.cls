\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{WileyNJDv5}
\LoadClass{article}
\RequirePackage{caption}
\usepackage{lettersp}
\newenvironment{boxtext}[1]{}{}
\def\received#1{\gdef\@received{Received #1;}}\gdef\@received{Added at production}
\usepackage{algorithm,algorithmicx,algpseudocode}
