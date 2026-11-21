% sig-alternate-05-2015.cls (1703.08365), its citation lines (:1385-1409, :1459-1491) after `\LoadClass{article}`
\ProvidesClass{acmcite}
\LoadClass{article}
\def\@normalcite{%
    \def\@cite##1##2{[##1\if@tempswa , ##2\fi]}%
}
\def\@citeNB{%
    \def\@cite##1##2{##1\if@tempswa , ##2\fi}%
}
\def\@citeRB{%
    \def\@cite##1##2{##1\if@tempswa , ##2\fi]}%
}
\def\start@cite#1#2{%
    \edef\citeauthoryear##1##2##3{%
        ###1%
        \ifnum#2=\z@ \else\ ###2\fi
    }%
    \ifnum#1=\thr@@
        \let\@@cite\@citeyear
    \else
        \let\@@cite\@citenormal
    \fi
    \@ifstar{\@citeNB\@@cite}{\@normalcite\@@cite}%
}
%\def\cite{\start@cite23}
\DeclareRobustCommand\cite{\start@cite23}		% January 2008
\def\citeNP{\cite*}					% No Parentheses e.g. 5
\def\@citenormal{%
    \@ifnextchar [{\@tempswatrue\@citex;}%
{\@tempswafalse\@citex[]}%  	% GERRY FIX FOR BABEL 3/20/2009
}

\def\@citeyear{%
    \@ifnextchar [{\@tempswatrue\@citex,}%
{\@tempswafalse\@citex[]}%	%  GERRY FIX FOR BABEL 3/20/2009
}

\def\@citex#1[#2]#3{%
    \let\@citea\@empty
    \@cite{%
        \@for\@citeb:=#3\do{%
            \@citea
            \def\@citea{#1, }% 	% GERRY FIX FOR BABEL 3/20/2009 -- SO THAT YOU GET [1, 2] IN THE BODY TEXT
            \edef\@citeb{\expandafter\@iden\@citeb}%
            \if@filesw
                \immediate\write\@auxout{\string\citation{\@citeb}}%
            \fi
            \@ifundefined{b@\@citeb}{%
                {\bf ?}%
                \@warning{%
                    Citation `\@citeb' on page \thepage\space undefined%
                }%
            }%
            {\csname b@\@citeb\endcsname}%
        }%
    }{#2}%
}
