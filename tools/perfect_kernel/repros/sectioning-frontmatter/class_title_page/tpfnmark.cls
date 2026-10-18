% The shape of sig-alternate-05-2015.cls:726-737, :1025-1049 (1505.04771): the class's `\maketitle` defines
% `\@makefnmark` around its title (copied from article.cls:206, reading `\@thefnmark`, which latex.ltx defines only
% when a footnote is made), peeks with `\futurelet`, and then prints the `\toappear` text its setter wrapped.
\ProvidesClass{tpfnmark}
\LoadClass{article}
\def\@toappear{}
\long\def\toappear#1{\def\@toappear{\parbox[b]{20pc}{#1}}}
\def\maketitle{\par
 \begingroup
   \def\thefootnote{\fnsymbol{footnote}}%
   \def\@makefnmark{\hbox to 0pt{$^{\@thefnmark}$\hss}}%
   \futurelet\tp@next\relax
   \ifx\tp@next\@empty\fi
   \@maketitle
 \endgroup
 \setcounter{footnote}{0}%
 \let\maketitle\relax
 \let\@maketitle\relax
 \gdef\@thanks{}\gdef\@author{}\gdef\@title{}\let\thanks\relax
 \par\noindent\@toappear\par}
