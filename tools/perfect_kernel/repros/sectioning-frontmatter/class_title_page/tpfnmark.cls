% A stand-in written from scratch for the shape of sig-alternate-05-2015.cls:726-737, :1025-1049 (1505.04771; ACM's
% class, no licence stated for it): the class's `\maketitle` defines `\@makefnmark` around its title (reading
% `\@thefnmark`, which latex.ltx defines only when a footnote is made), peeks with `\futurelet`, and then prints the
% `\toappear` text its setter wrapped. No code of the class is copied.
\ProvidesClass{tpfnmark}
\LoadClass{article}
\newcommand\tpf@appear{}
\long\def\toappear#1{\def\tpf@appear{\parbox[b]{20pc}{#1}}}
\renewcommand\maketitle{\par
  \begingroup
    \renewcommand\thefootnote{\fnsymbol{footnote}}%
    \def\@makefnmark{\hbox to 0pt{$^{\@thefnmark}$\hss}}%
    \futurelet\tpf@next\relax
    \ifx\tpf@next\@empty\fi
    \@maketitle
  \endgroup
  \setcounter{footnote}{0}%
  \let\maketitle\relax\let\@maketitle\relax
  \gdef\@thanks{}\gdef\@author{}\gdef\@title{}\let\thanks\relax
  \par\noindent\tpf@appear\par}
