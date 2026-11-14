% A minimal stand-in for an.cls (Astronomische Nachrichten, Wiley-VCH, 2006-01-13, the class astro-ph0611016 ships —
% not in TeX Live; it states no licence), written from scratch for the repros beside it: the abstract setter
% `\aaabstract` (an.cls:414), which the class makes `\abstract` at its end (:98), its `\keywords` (:354-360: an empty
% argument stores nothing) and the title page that prints both stores (`\@maketitle`, :715-718), with what the class
% does there. No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{an}[2006/01/13 repro stand-in]
\LoadClass{article}
\newcommand\aaabstract[1]{\gdef\@abstract{#1}}
\AtEndOfClass{\let\abstract\aaabstract}
\newcommand\keywordname{\textbf{Key words}}
\newcommand\keywords[1]{\ifx\relax#1\relax\let\@keywords\relax
  \else\def\@keywords{\textbf{\keywordname}\quad#1\vskip 6pt}\fi}
\let\@keywords\relax
\def\@maketitle{\newpage\normalfont
  {\fontsize{14.4}{17}\bfseries\selectfont\@title}\par\vskip 4.5mm\relax
  \noindent{\normalsize\bfseries\selectfont{\@author}}\vskip 10pt\relax
  \noindent\@date\vskip 5mm
  \noindent\@keywords
  \noindent{\@abstract}\par\vskip 5mm}
