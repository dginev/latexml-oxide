\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{mdpi}[21/06/2021 MDPI paper class]
\LoadClass[10pt,a4paper]{article}
\RequirePackage{hyperref}
\RequirePackage{paracol}
\globalcounter{figure}
\RequirePackage{newfloat}
\DeclareFloatingEnvironment[]{scheme}
\RequirePackage{caption}
\captionsetup[figure]{position=bottom, labelsep=period}
\AtBeginDocument{\begin{paracol}{2}\switchcolumn}
