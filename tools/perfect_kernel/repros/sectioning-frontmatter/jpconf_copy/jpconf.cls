% A minimal stand-in, written from scratch, for IOP's jpconf.cls (2004/05/04 v1.0, Journal of Physics: Conference
% Series; 0705.0202's and 1011.0523's copies), which is not in TeX Live. The real class is standalone; this one builds
% on article and keeps what the repros read: the author block typeset where it stands (:90-102), the e-mail and address
% blocks (:104-110: `\eads`, `\ead`, `\mailto`, `\address`, each an item of an `{indented}` list) and that list
% (:496-499), and its notes' marks, defined as :130-153 defines them.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{jpconf}[2004/05/04 v1.0 Journal of Physics: Conference Series (stand-in)]
\LoadClass{article}
\newdimen\mathindent \mathindent=6pc
\def\indented{\list{}{\itemsep=0pt\labelsep=0pt\itemindent=0pt\labelwidth=0pt\leftmargin=\mathindent
  \topsep=0pt\partopsep=0pt\parsep=0pt}\footnotesize\rm}
\let\endindented=\endlist
\renewcommand{\author}[1]{\vspace*{1.5pc}\begin{indented}\item[]\normalsize\bf\raggedright #1\end{indented}\smallskip}
\newcommand{\address}[1]{\begin{indented}\item[]\rm\raggedright #1\end{indented}}
\def\mailto#1{{\tt #1}}
\def\ead#1{\vspace*{5pt}\address{E-mail: \mailto{#1}}}
\def\eads#1{\vspace*{5pt}\address{E-mail: #1}}
% its notes' marks (:130-153): a footnote's mark `\fnsymbol{footnote}`, and `\@fnsymbol` the arabic `\thefootnote` as a
% superscript (99 a star)
\def\footnoterule{}%
\setcounter{footnote}{0}
\long\def\@makefntext#1{\parindent 1em\noindent
 \makebox[1em][l]{\footnotesize\rm$\m@th{\fnsymbol{footnote}}$}%
 \footnotesize\rm #1}
\def\@makefnmark{\normalfnmark}
\def\normalfnmark{\hbox{${\fnsymbol{footnote}}\m@th$}}
\def\@thefnmark{\fnsymbol{footnote}}
\def\footnote{\protect\pfootnote}
\def\pfootnote{\@ifnextchar[{\@xfootnote}{\stepcounter{\@mpfn}%
       \begingroup\let\protect\noexpand
       \xdef\@thefnmark{\thempfn}\endgroup
     \@footnotemark\@footnotetext}}
\def\@xfootnote[#1]{\setcounter{footnote}{#1}%
   \addtocounter{footnote}{-1}\footnote}
\def\@fnsymbol#1{\ifnum\thefootnote=99\hbox{*}\else^{\thefootnote}\fi\relax}
