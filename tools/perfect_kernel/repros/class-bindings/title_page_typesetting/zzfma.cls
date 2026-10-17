% The title-page shapes address_after_a_font_macro.tex reads (review 68x r2 probe ka).
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{zzfma}
\LoadClass{article}
\def\@addrfont{\itshape}
\newcommand\address[1]{\gdef\@address{#1}}
\gdef\@address{}
\newcommand\journal[1]{\gdef\@journal{#1}}
\gdef\@journal{Journal of Default Placeholders}
\def\@maketitle{\begin{center}{\LARGE\@title\par}{\large\@author\par}{\@addrfont\@address\par}\end{center}}
\RequirePackage{fancyhdr}
\fancyhead[L]{\@journal}
\pagestyle{fancy}
