% A minimal stand-in for tlp.cls (Cambridge University Press, Theory and Practice of Logic Programming, the class
% 1209.2617 ships — not in TeX Live; its notice allows use only for CUP publications), written from scratch for
% tlp_placeholders_and_date_line.tex: it has only the shapes the repro reads, with what the class does there — the
% title page's date line (:596-631, :627), the running head's placeholders and journal line (:697-728) printed only under `prodtf`
% (:760-764), a `\maketitle` (:515-531); line numbers are those of 1209.2617's copy. No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{tlp}[2003/01/01 repro stand-in]
\newif\ifprodtf
\DeclareOption{prodtf}{\prodtftrue}
\ProcessOptions
\LoadClass{article}
% the three dates of the title page's line, each with a default
\newcommand\submitted[1]{\gdef\@submitted{#1}}
\newcommand\revised[1]{\gdef\@revised{#1}}
\newcommand\accepted[1]{\gdef\@accepted{#1}}
\gdef\@submitted{0000}\gdef\@revised{0000}\gdef\@accepted{0000}
% the running head's stores, each with a placeholder; only the production head (`prodtf`) prints them
\newcommand\volume[1]{\gdef\@volume{#1}}\gdef\@volume{0}
\newcommand\pagerange[1]{\gdef\@pagerange{#1}}\gdef\@pagerange{0--0}
\newcommand\pubyear[1]{\gdef\@year{#1}}\gdef\@year{0000}
\newcommand\doi[1]{\gdef\@doi{#1}}\gdef\@doi{0000}
\newcommand\jdate[1]{\gdef\@jdate{#1}}\gdef\@jdate{January \@year}
% the journal line, a store with a default of its own made of the others (the class's `\journal`), which only the
% production head prints
\newcommand\journal[1]{\gdef\@journal{#1}}\gdef\@journal{TLP \@volume: \@pagerange, \@jdate. DOI \@doi}
\newcommand\tlps@prodhead{\small\itshape\@journal}
\newcommand\tlps@draftheadline{\small\itshape Under consideration for publication in Theory and Practice of Logic
  Programming}
\ifprodtf\let\tlps@head\tlps@prodhead\else\let\tlps@head\tlps@draftheadline\fi
\def\ps@titlepage{\let\@mkboth\@gobbletwo\def\@oddhead{\tlps@head\hfil}\let\@evenhead\@oddhead
  \let\@oddfoot\@empty\let\@evenfoot\@empty}
% the title page: the title, the author, the date line
\def\@maketitle{\newpage{\centering{\LARGE\itshape\@title\par}\vskip 16pt{\@author\par}\vskip 10pt
  {\small\itshape submitted \@submitted; revised \@revised; accepted \@accepted}\par}\vskip 18pt}
\renewcommand\maketitle{\par\begingroup\@maketitle\thispagestyle{titlepage}\endgroup}
\endinput
