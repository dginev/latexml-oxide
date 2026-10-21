% A minimal stand-in for elife.cls (Overleaf's template for eLife submissions, v1.11 2022/06/06, the copy 2311.00616
% ships — not in TeX Live; it states no licence), written from scratch for figure_supplements_are_printed.tex: the
% figure-supplement interface (elife.cls:380-461) — `\figsupp[short]{caption}{graphic}`, its heading "Figure N—figure
% supplement M." in the figure and the supplement itself printed after the paper; the supplement's source data and
% code, `\figsuppdata` and `\figsuppsrccode`, numbered within it — with what the class does there (the binding,
% latexml_contrib/src/elife_cls.rs, owns `\figsupp`; the copy's `\figsuppdata` prints). No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{elife}[2022/06/06, v1.11]
\LoadClass{extarticle}
\RequirePackage{etoolbox}
\RequirePackage{graphicx}
\newcounter{figsupp}\newcounter{fsdata}\newcounter{fssrccode}
\AtBeginEnvironment{figure}{\setcounter{figsupp}{0}}
% the heading of a supplement and of a part of it ("—source data 1"), bold
\newcommand\efs@name{\figurename~\thefigure---figure supplement \arabic{figsupp}}
\DeclareRobustCommand\figsuppdata[1]{\protect\refstepcounter{fsdata}%
  \par\small\textbf{\efs@name---source data~\arabic{fsdata}. } #1\par}
\DeclareRobustCommand\figsuppsrccode[1]{\protect\refstepcounter{fssrccode}%
  \par\small\textbf{\efs@name---source code~\arabic{fssrccode}. } #1\par}
% in the figure the heading and the short caption (or the caption, its data and code left out; `none`: nothing); the
% graphic and the caption collected for the end of the paper
\newcommand\efs@all{}
\newcommand\figsupp[3][]{\refstepcounter{figsupp}%
  {\ifstrequal{#1}{none}{}{\small\textbf{\efs@name.} \ifstrempty{#1}{{\def\figsuppdata##1{}\def\figsuppsrccode##1{}#2}}{#1}}}%
  \par
  \edef\efs@nums{{\the\c@figure}{\the\c@figsupp}}\expandafter\efs@add\efs@nums{#2}{#3}}
\newcommand\efs@add[4]{\g@addto@macro\efs@all{\efs@print{#1}{#2}{#3}{#4}}}
\newcommand\efs@print[4]{\setcounter{figure}{#1}\setcounter{figsupp}{#2}\setcounter{fsdata}{0}\setcounter{fssrccode}{0}%
  \par\noindent\begin{minipage}{\linewidth}#4\par\textbf{\efs@name.} #3\par\vskip 8pt\end{minipage}\par\vskip 16pt}
\AtEndDocument{\efs@all}
