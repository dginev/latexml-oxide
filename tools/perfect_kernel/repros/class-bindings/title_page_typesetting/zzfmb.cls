% The title-page shapes address_in_a_vbox_with_a_class_line.tex reads (review 68x r2 probe kb).
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{zzfmb}
\LoadClass{article}
\def\@jinfo{Vol.~1, 2026}
\newcommand\address[1]{\gdef\@address{#1}}
\gdef\@address{}
\newcommand\journal[1]{\gdef\@journal{#1}}
\gdef\@journal{Journal of Default Placeholders}
\def\@maketitle{\vbox{\centering{\LARGE\@title\par}\@jinfo\par\@author\par\@address\par}}
\def\ps@jhead{\def\@oddhead{\hfil\@journal}\let\@evenhead\@oddhead}
\AtBeginDocument{\thispagestyle{plain}}
