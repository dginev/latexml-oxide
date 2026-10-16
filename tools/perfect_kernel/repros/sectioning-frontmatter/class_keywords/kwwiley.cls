% The shape of wiley-article.cls:446-489 (1706.00599's copy; nbdt-article.cls:430-473, 1801.08881): `\keywords`
% exists only inside the abstract, which environ's `\NewEnviron` collects.
\ProvidesClass{kwwiley}
\LoadClass{article}
\RequirePackage{environ}
\NewEnviron{kwwiley@abstract}{%
  \newcommand{\keywords}[1]{%
    \vskip8\p@
    \noindent{\bfseries KEYWORDS\par}
    \noindent{\raggedright ##1\par}
  }%
  \begin{quote}\BODY\end{quote}%
}
\renewcommand{\abstract}{\kwwiley@abstract}
\renewcommand{\endabstract}{\endkwwiley@abstract}
