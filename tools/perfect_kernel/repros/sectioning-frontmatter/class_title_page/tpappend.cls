% The shape of wiley-article.cls:213-214, :420-466, :576 (1911.06213, 1808.01384): the class renews `\maketitle` to
% print its own stores, then appends to it with etoolbox's `\appto`.
\ProvidesClass{tpappend}
\LoadClass{article}
\RequirePackage{etoolbox}
\newcommand{\papertopic}[1]{\def\@papertopic{#1}}
\newcommand{\grantinfo}[1]{\def\@grantinfo{#1}}
\def\@papertopic{}\def\@grantinfo{}
\renewcommand{\maketitle}{%
  \par\noindent{\bfseries\@papertopic}\par
  {\Large\@title\par}%
  \noindent\@author\par}
\appto{\maketitle}{\par\noindent Funded by \@grantinfo.\par}
