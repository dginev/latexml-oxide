% A stand-in written from scratch for the shape of wiley-article.cls:206-214, :416-433, :470-489 (1808.01384,
% 1911.06213; Wiley's template, not in TeX Live, no licence stated for the class): the class stores its correspondence
% block, e-mail, funding line, paper type and journal section; its `\maketitle` prints the type and the section, and only
% its own abstract environment prints the correspondence block and the funding line. No code of the class is copied.
\ProvidesClass{tpwiley}
\LoadClass{article}
\newcommand\corraddress[1]{\gdef\@corraddress{#1}}
\newcommand\corremail[1]{\gdef\@corremail{#1}}
\newcommand\fundinginfo[1]{\gdef\@fundinginfo{#1}}
\newcommand\papertype[1]{\gdef\@papertype{#1}}
\newcommand\paperfield[1]{\gdef\@paperfield{#1}}
\renewcommand\maketitle{\noindent\textbf{\MakeUppercase{\@papertype}}\\{\small\@paperfield}\par
  {\Large\bfseries\@title\par}\noindent\@author\par}
\newcommand\tpw@metadata{\textbf{Correspondence}\\\@corraddress\\Email: \@corremail\par
  \textbf{Funding information}\\\@fundinginfo\par}
\newenvironment{tpw@abstract}{\par\tpw@metadata\par\noindent\textbf{Abstract.} }{\par}
\renewcommand\abstract{\tpw@abstract}
\renewcommand\endabstract{\endtpw@abstract}
