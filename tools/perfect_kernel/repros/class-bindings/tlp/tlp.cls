% The shapes of tlp.cls (Theory and Practice of Logic Programming, shipped by 1209.2617; not in TeX Live) the repro
% tlp_placeholders_and_date_line.tex reads: its title-page date line (:596-631), the running head's placeholders
% (:697-728), the switch that prints them only under `prodtf` (:760-764) and its `\maketitle` (:515-531, in short).
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{tlp}
\newif\ifprodtf
\DeclareOption{prodtf}{\prodtftrue}
\ProcessOptions
\LoadClass{article}
\renewcommand\maketitle{\par\begingroup\@maketitle\thispagestyle{titlepage}\endgroup}
\newcommand\submitted[1]{\gdef\@submitted{#1}}
  \gdef\@submitted{1 January 2003}
\newcommand\revised[1]{\gdef\@revised{#1}}
  \gdef\@revised{1 January 2003}
\newcommand\accepted[1]{\gdef\@accepted{#1}}
  \gdef\@accepted{1 January 2003}
\def\@maketitle{%
  \newpage
  {\centering \sloppy
   {\normalfont\LARGE\itshape \@title\par}%
   \vskip 16\p@
   {\normalfont\normalsize\rmfamily \@author\par}%
     \vskip 10pt%
     {{\small\it submitted \@submitted; revised \@revised; accepted \@accepted}}\par
  }%
  \vskip 18\p@ \@plus 2\p@ \@minus 1\p@
}
\newcommand\volume[1]{\gdef\@volume{#1}}
\gdef\@volume{{\normalfont\bfseries 1} (1)}
\newcommand\pagerange[1]{\gdef\@pagerange{#1}}
\gdef\@pagerange{1--000}
\newcommand\pubyear[1]{\gdef\@year{#1}}
\gdef\@year{20XX}
\newcommand\jdate[1]{\gdef\@jdate{#1}}
\gdef\@jdate{January \@year}
\newcommand\doi[1]{\gdef\@doi{10.1017/#1}}
\gdef\@doi{10.1017/S147106840100xxxx}
\newcommand\journal[1]{\gdef\@journal{#1}}
\def\@journal{%
  \vbox to 5.6\p@{\noindent\parbox[t]{4.8in}{\normalfont\small\rmfamily
    {\itshape TLP\/}\ \@volume: \@pagerange, \@jdate.\quad
    \copyright\ \@year\ Cambridge University Press\\[2.5\p@]%
    DOI: \@doi\quad Printed in the United Kingdom}%
  \vss}%
}
\def\@underjournal{%
  \vbox to 5.6\p@{\noindent\parbox[t]{4.8in}{\normalfont\small\rmfamily
    {\itshape Under consideration for publication in
     Theory and Practice of Logic Programming\/}\\[2.5\p@]
     \ }%
  \vss}%
}
\ifprodtf
  \let\@j@urnal\@journal
\else
  \let\@j@urnal\@underjournal
\fi
\def\ps@titlepage{\let\@mkboth\@gobbletwo
  \def\@oddhead{\@j@urnal\hfil}\let\@evenhead\@oddhead
  \let\@oddfoot\@empty\let\@evenfoot\@empty}
