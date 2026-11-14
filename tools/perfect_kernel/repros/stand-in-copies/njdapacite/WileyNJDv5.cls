\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{WileyNJDv5}[2023/03/09 v5]
\newif\if@APArefstyle
\DeclareOption{APA}{\global\@APArefstyletrue}
\ProcessOptions
\LoadClass{article}
\if@APArefstyle
  \usepackage[apaciteclassic]{NJDapacite}
\fi
