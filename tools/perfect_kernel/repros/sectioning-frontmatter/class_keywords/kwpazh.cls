% A stand-in written from scratch for the shape of astlb.cls:91-205 (1202.2889's copy; pazhastl.cls, astro-ph0306137;
% pazhb_eng.cls, 1507.06302; pazhb.cls, 2103.05801 — Astronomy Letters' classes, not in TeX Live, no licence stated):
% the abstract's body is collected into a token register, `\endabstract` calls `\maketitle`, and the class's
% `\@maketitle` typesets the collected body inside `{pazhabstract}`, the only place `\keywords` is defined. No code of
% the class is copied.
\ProvidesClass{kwpazh}
\LoadClass{article}
\newtoks\kpz@body
% `\begin{abstract}` reads its body up to its own `\end{abstract}` (an inner environment's `\end` kept in the body)
\def\abstract{\kpz@collect}
\long\def\kpz@collect#1\end#2{\def\kpz@name{#2}\def\kpz@want{abstract}%
  \ifx\kpz@name\kpz@want
    \global\kpz@body\expandafter{\the\kpz@body\textbf{Abstract ---} #1}\expandafter\kpz@close
  \else
    \global\kpz@body\expandafter{\the\kpz@body #1\end{#2}}\expandafter\kpz@collect
  \fi}
\def\kpz@close{\end{abstract}}
\def\endabstract{\maketitle}
\newenvironment{pazhabstract}{\def\keywords##1{\par\medskip Key words:~~\emph{##1}\par\medskip}\small
  \list{}{\leftmargin=0.06\linewidth \rightmargin=\leftmargin}\item\relax}{\endlist}
\def\@maketitle{\begin{center}{\large\bfseries\@title\par}{\bfseries\@author\par}\end{center}%
  \begin{pazhabstract}\the\kpz@body\end{pazhabstract}}
