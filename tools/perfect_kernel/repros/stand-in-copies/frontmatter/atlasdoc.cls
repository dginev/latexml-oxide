\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{atlasdoc}[2019/01/10]
\newif\ifADOC@cernpreprint
\newif\ifADOC@datetop \ADOC@datetoptrue
\DeclareOption{cernpreprint}{\ADOC@cernpreprinttrue}
\DeclareOption{nodatetop}{\ADOC@datetopfalse}
\ProcessOptions
\LoadClass{article}
\author{The ATLAS Collaboration}
\date{DEFAULT DATE}
\newcommand{\ATLAS@TitleHeader}{\ifADOC@datetop\@date\par\fi}
\renewcommand{\maketitle}{\ifADOC@cernpreprint\else\ATLAS@TitleHeader\fi
  {\centering\@title\par}{\centering\@author\par}}
