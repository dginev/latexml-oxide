% The title-page shapes store_in_a_line_with_a_document_value.tex reads (79a r2): the address after the class's line
% ended by `\linebreak`, and the DOI — a store handed on as it is set — in one line with `\@issueinfo`, the store of a
% setter no table names, empty until the document sets it.
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{zzfmf}
\LoadClass{article}
\def\@jinfo{Vol.~1, 2026}
\newcommand\address[1]{\gdef\@address{#1}}
\gdef\@address{}
\newcommand\issueinfo[1]{\gdef\@issueinfo{#1}}
\gdef\@issueinfo{}
\newcommand\doi[1]{\gdef\@doi{#1}}
\gdef\@doi{}
\def\@maketitle{\begin{center}{\LARGE\@title\par}{\large\@author\par}{\@jinfo\linebreak\@address\par}%
  {\itshape \@issueinfo; DOI \@doi\par}\end{center}}
