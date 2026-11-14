% A minimal stand-in for an.cls v1.03 (Astronomische Nachrichten, 2002/07/04, the class astro-ph0306340 ships — not in
% TeX Live; it states no licence), written from scratch for an_v103_abstract_label_is_its_name.tex: the abstract setter
% `\aaabstract` (an.cls:422), which the class makes `\abstract` at its end (:76), and the title page, which prints the
% class's label before the store (`\@maketitle`, :662; v1.09a and later comment that line out), with what the class
% does there. No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{an}[2002/07/04 v1.03 repro stand-in]
\LoadClass{article}
\newcommand\aaabstract[1]{\gdef\@abstract{#1}}
\AtEndOfClass{\let\abstract\aaabstract}
\def\@maketitle{\newpage\normalfont
  {\Large\bf\@title\par}\vskip 3mm\relax
  \noindent{\large\sc\@author}\vskip 10pt
  \small\rm\vskip 22.85pt\noindent\@date\vskip 15mm
  \noindent{\bfseries\abstractname}\ \@abstract\par\vskip 5mm}
