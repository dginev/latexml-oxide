% The shape of astlb.cls:91-205 (1202.2889's copy; pazhastl.cls, astro-ph0306137; pazhb_eng.cls, 1507.06302; pazhb.cls,
% 2103.05801): the abstract's body is collected into a token register, `\endabstract` calls `\maketitle`, and the
% class's `\@maketitle` typesets it inside `{pazhabstract}`, the only place `\keywords` is defined.
\ProvidesClass{kwpazh}
\LoadClass{article}
\def\@maketitle{%
  \begin{center}{\large\bfseries \@title\par}{\bfseries\@author\par}\end{center}%
  \begin{pazhabstract}\the\abstract@toks\end{pazhabstract}}
\newtoks\abstract@toks  \abstract@toks{}
\def\abstract{\def\@abstract@{abstract}%
  {\ifnum0=`}\fi
  \@abstract@getbody}
\def\endabstract{\maketitle}
\long\def\@abstract@getbody#1\end{%
  \global\abstract@toks\expandafter{\the\abstract@toks \textbf{Abstract ---}#1}%
  \@abstract@findend}
\def\@abstract@findend#1{%
  \def\@tempa{#1}%
  \ifx\@tempa\@abstract@
    \expandafter\@abstract@end
  \else
     \global\abstract@toks\expandafter{\the\abstract@toks\end{#1}}%
     \expandafter\expandafter\expandafter\@abstract@getbody
  \fi}
\def\@abstract@end{\ifnum0=`{\fi}%
  \expandafter\end\expandafter{\@abstract@}}
\newenvironment{pazhabstract}{
  \def\keywords##1{\medskip Key words:~~\emph{##1}\medskip}
  \small
  \list{}{\rightmargin 0.0575\linewidth\leftmargin\rightmargin}%
\item\relax}{\endlist}
