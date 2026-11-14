% A `{keywords}` whose begin code sets a flag its title page reads (tADR2e.cls:450 `\global\keywordstrue`, 1911.00238):
% not label-only, so it stays the class's.
\ProvidesClass{kwflag}
\LoadClass{article}
\newif\ifkeywords
\newenvironment{keywords}{\global\keywordstrue\par\noindent{\bfseries Keywords:} }{\par}
