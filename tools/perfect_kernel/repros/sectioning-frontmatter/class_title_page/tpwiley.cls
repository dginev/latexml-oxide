% The shape of wiley-article.cls:206-214, :416-433, :470-489 (1808.01384, 1911.06213): the class stores its
% correspondence block, e-mail, funding line, paper type and journal section; its `\maketitle` prints the type and
% the section, and only its own abstract environment prints the correspondence block and the funding line.
\ProvidesClass{tpwiley}
\LoadClass{article}
\newcommand{\corraddress}[1]{\def\@corraddress{#1}}
\newcommand{\corremail}[1]{\def\@corremail{#1}}
\newcommand{\fundinginfo}[1]{\def\@fundinginfo{#1}}
\newcommand{\papertype}[1]{\def\@papertype{#1}}
\newcommand{\paperfield}[1]{\def\@paperfield{#1}}
\renewcommand{\maketitle}{%
  \noindent{\bfseries\MakeUppercase{\@papertype}}\\
  {\small\@paperfield}\par
  {\Large\bfseries\@title\par}%
  \noindent\@author\par}
\newcommand{\tp@affilmetadata}{%
  \textbf{Correspondence}\\
  \@corraddress\\
  Email: \@corremail\par
  \textbf{Funding information}\\
  \@fundinginfo\par}
\newenvironment{tp@abstract}{\par\tp@affilmetadata\par\noindent\textbf{Abstract.} }{\par}
\renewcommand{\abstract}{\tp@abstract}
\renewcommand{\endabstract}{\endtp@abstract}
