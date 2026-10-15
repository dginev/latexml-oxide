% The options of the v1.25 jfm.cls 2605.03843 ships (its :40-58), no `lineno` among them; the class is cut to what
% pdflatex needs for jfm_old_copy_has_no_lineno_option.tex.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{jfm}[2001/08/08 v1.25 Journal of Fluid Mechanics document class]
\newif\ifCUPmtlplainloaded
\DeclareOption{oneside}{\relax}
\DeclareOption{twoside}{\@twosidetrue \@mparswitchtrue}
\DeclareOption{prodtf}{\CUPmtlplainloadedtrue}
\ProcessOptions
\LoadClass{article}
