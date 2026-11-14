% The title-page shape store_beside_a_font_macro_keeps_its_order.tex reads (79a r3): the journal printed in the
% class's journal font (USG.cls:1590 `\smash{\@journalfont\@journal}`), the volume in a line of its own.
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{zzfmi}
\LoadClass{article}
\def\@journalfont{\bfseries\itshape}
\newcommand\journal[1]{\gdef\@journal{#1}}
\gdef\@journal{}
\newcommand\volume[1]{\gdef\@volume{#1}}
\gdef\@volume{}
\def\@maketitle{{\@journalfont\@journal\par}{\itshape\@volume\par}%
  \begin{center}{\LARGE\@title\par}{\large\@author\par}\end{center}}
