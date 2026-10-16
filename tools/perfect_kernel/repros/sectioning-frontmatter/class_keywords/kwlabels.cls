% Label-only begin codes from shipped classes: pasa.cls:56/101-103 (1801.03506) as `{keywords}`, a class font macro and
% a styled label; biometrika.cls:178/525-527 (2412.19242) as `{keyword}`, a group label followed by its colon.
\ProvidesClass{kwlabels}
\LoadClass{article}
\def\keywordfont{\small\rightskip5.5pc}
\newenvironment{keywords}{%
  \noindent\keywordfont\textbf{Keywords: }%
}{\vskip27pt}
\def\kwfont{\fontsize{9}{10}\selectfont}
\newenvironment{keyword}{\par\addvspace{8pt}%
  \kwfont\noindent{\itshape Some key words}:\ \ignorespaces%
}{\par\addvspace{23pt}}
