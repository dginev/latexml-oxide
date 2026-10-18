% A trimmed copy of an.cls v1.03 (Astronomische Nachrichten, 2002/07/04; not in TeX Live), the class astro-ph0306340
% ships: its abstract setter (an.cls:422), the end-of-class hook that makes it `\abstract` (:76) and the title page
% that prints the class's label before the store (`\@maketitle` :662 `\noindent{\bfseries\abstractname}\ \@abstract`;
% v1.09a and later comment that line out).
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{an}[2002/07/04 v1.03 trimmed copy]
\LoadClass{article}
\AtEndOfClass{\let\abstract=\aaabstract}% :76 (its `\mathindent\z@` needs the fleqn option the class loads with)
\def\aaabstract#1{\gdef\@abstract{#1}}
\def\@maketitle{\newpage
 \normalfont
 {\Large\bf \@title \par}\vskip 3mm\relax
 \noindent\ignorespaces{\large \sc \@author}\vskip10pt
 \small\rm
 \vskip 22.85pt \noindent\@date
 \vskip 15mm % -ADS 111001
% \noindent{\sffamily\bfseries\abstractname}\ \@abstract
 \noindent{\bfseries\abstractname}\ \@abstract
 \par\vskip 5mm}
