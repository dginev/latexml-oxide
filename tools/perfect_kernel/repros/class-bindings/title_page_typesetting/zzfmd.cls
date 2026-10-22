% The title-page shapes value_in_the_line_stays_the_class_line.tex reads (79a): the address printed in one line with
% the class's text "Vol. 1, 2026", and the e-mail in one line with `\@submitted`, a store of a setter no table names
% (`\submitted`), empty until the document sets it (tlp.cls:627's shape).
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{zzfmd}
\LoadClass{article}
\def\@jinfo{Vol.~1, 2026}
\newcommand\address[1]{\gdef\@address{#1}}
\gdef\@address{}
\newcommand\submitted[1]{\gdef\@submitted{#1}}
\gdef\@submitted{}
\newcommand\email[1]{\gdef\@email{#1}}
\gdef\@email{}
\def\@maketitle{\begin{center}{\LARGE\@title\par}{\large\@author\par}{\@jinfo, \@address\par}%
  {\itshape submitted \@submitted; e-mail \@email\par}\end{center}}
