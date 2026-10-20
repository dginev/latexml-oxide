%% A renamed copy of an older jmlr.cls (myjmlr.cls, jmlr_mod.cls: `\ProvidesClass{jmlr}`), cut down to its theorem wrap
%% (jmlr.cls:753-779), which the jmlr binding's jmlrutils.sty makes too
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{jmlr}[2010/03/09 v1.08 (NLCT) JMLR Class]
\LoadClass{article}
\RequirePackage{etoolbox}
\newcommand*{\@theorembodyfont}{\normalfont\itshape}
\newcommand*{\@theoremheaderfont}{\normalfont\bfseries }
\let\jmlr@org@newtheorem\newtheorem
\renewcommand*{\newtheorem}{\@ifstar\jmlr@snewtheorem\jmlr@newtheorem}
\newcommand*{\jmlr@snewtheorem}[2]{\newenvironment{#1}{\par\textbf{#2.}\itshape}{\par}}
\newcommand{\jmlr@newtheorem}[1]{%
  \cslet{jmlr@thm@#1@body@font}{\@theorembodyfont}%
  \jmlr@org@newtheorem{#1}%
}
\newtheorem{example}{Example}
\endinput
