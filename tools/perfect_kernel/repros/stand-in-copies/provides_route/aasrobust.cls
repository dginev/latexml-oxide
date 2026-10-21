%% aasrobust.cls -- a minimal stand-in written from scratch for a renamed AASTeX 5.2 class (`\ProvidesClass{aastex}`,
%% as 1402.7103's proto.cls:64) whose declared text names are more than text, through what the safety check must
%% follow: a robust command's inner macro (`\protect\X` → `\X `, a paragraph), a macro whose expansion is LaTeXML's
%% code (`\begin`), a name made at run time (`\csname`), a paragraph (`\prd`, which 78h r2 declined too); its two
%% class-end hooks run the check again (78h r2 logged `\prd`'s decline once more for each).
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{aastex}[2003/12/12 v5.2 stand-in]
\LoadClass{article}
\DeclareRobustCommand\myhead{\par\noindent HEADX}
\def\abstractname{\myhead}
\def\pra{\begin{center}PRAX\end{center}}
\def\prbname{PRBX}
\def\prb{\csname prbname\endcsname}
\def\prc{PRCX}
\def\prd{\par PRDX}
\AtEndOfClass{\def\aasrobusthook{}}
\AtEndOfClass{\def\aasrobusthooktoo{}}
\endinput
