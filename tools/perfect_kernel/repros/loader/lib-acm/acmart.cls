% A paper's copy of acmart.cls in a subdirectory (2605.00109 ships lib-acm/acmart.cls), cut to what pdflatex needs to
% compile subdir_class_is_its_binding.tex; read raw it would load xstring.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{acmart}[2026/01/01 v2.18 stand-in copy]
\LoadClass{article}
\RequirePackage{xstring}
\newcommand\affiliation[1]{\gdef\@affil{#1}}
\newcommand\institution[1]{#1}
\renewcommand\maketitle{\begin{center}\@title\par\@author\par\@affil\end{center}}
