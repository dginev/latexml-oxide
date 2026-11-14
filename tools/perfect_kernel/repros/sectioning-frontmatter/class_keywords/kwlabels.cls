% Label-only `{keywords}` begin codes of shipped classes, written from scratch for keywords_environment_label_shapes.tex
% (the classes are not in TeX Live and state no licence): pasa.cls:56/101-103 (1801.03506) — a class font macro, then a
% bold label — as `{keywords}`; biometrika.cls:178/525-527 (2412.19242) — an italic label in a group, then its colon —
% as `{keyword}`. No code of the classes is copied.
\ProvidesClass{kwlabels}
\LoadClass{article}
\newcommand\keywordfont{\small\rightskip=5.5pc\relax}
\newenvironment{keywords}{\noindent\keywordfont\textbf{Keywords: }}{\vskip 27pt}
\newcommand\kwfont{\fontsize{9}{10}\selectfont}
\newenvironment{keyword}{\par\addvspace{8pt}\kwfont\noindent{\itshape Some key words}:\ \ignorespaces}%
  {\par\addvspace{23pt}}
