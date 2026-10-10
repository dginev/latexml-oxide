% An excerpt of compositio.cls (the copy 2606.19241 ships), its frontmatter commands (:117-189), for
% compositio_family_frontmatter.tex.
\NeedsTeXFormat{LaTeX2e}[1995/06/01]
\ProvidesClass{compositio}
\LoadClass[twoside,onecolumn,11pt]{article}
\def\CPM@err#1{\ClassError{compositio}{#1}\@ehd}
\def\CPM@warn#1{\ClassWarning{compositio}{#1}}
\renewcommand*\author[2][]{%
  \gdef\CPM@doauthors{%
    \@authors\@andorcomma\relax\@andorcomma\relax}%
  \g@addto@macro\@curr@author{\\*#2}%
  \gdef\@curr@author@name{#2}%
  \g@addto@macro\@authors{%
    \@andorcomma,{\ignorespaces#2\unskip}}}%
\def\@authors{\@gobbletwo}
\def\CPM@doauthors{\CPM@warn{No author specified}}
\def\@curr@author{\@gobbletwo}
\def\@andorcomma#1#2\@andorcomma#3{%
  \ifx#3\relax
    \ifx#1\relax%
      \let\@tempa\relax
    \else
    \def\@tempa{\space\andname\space #2\@gobbletwo}%
    \fi
  \else
    \def\@tempa{#1 #2\@andorcomma#3}%
  \fi
  \@tempa}
\providecommand\andname{and}
\newcommand\address[2][]{%
  \ifx\@curr@author\CPM@gobble
    \CPM@err{\noexpand\address with no \string\author}\@ehd
  \else
    \expandafter\g@addto@macro\expandafter\@addresses\expandafter{%
         \expandafter\filbreak\@curr@author\CPM@address{#2}}%
    \let\@curr@author\CPM@gobble
  \fi}
\newcommand\curraddr[2][]{%
\g@addto@macro\@addresses{\CPM@address{\textit{Current address:} #2}}}
\def\@addresses{}
\AtEndDocument{\par\@addresses}
\def\CPM@address#1{\\*\begingroup\def\\{, }#1\endgroup}
\def\CPM@gobble{\@gobbletwo}
\newcommand\email[2][]{%
  \ifx\@curr@author\CPM@gobble
    \CPM@err{\noexpand\email should come after corresponding \string\author}\@ehd
  \else
    \g@addto@macro\@curr@author{\quad\CPM@email{#2}}%
  \fi}
\def\CPM@email#1{\textsf{#1}}
\def\thanks#1{%
  \ifx\@thanks\@empty\else\CPM@warn{Only one \string\thanks\space allowed!}\fi
  \g@addto@macro\@thanks{\footnotetext{\indent#1}}}
\let\@thanks\@empty
\def\keywords#1{%
  \gdef\@keywords{%
    {\itshape Keywords:\/} \ignorespaces#1\unskip}}
\let\@keywords\@empty
\newcommand\classification[2][]{%
  \gdef\@classification{%
    {\textit{2010 Mathematics Subject Classification}} \ignorespaces#2\unskip}}
\let\subjclass\classification
\def\@classification{\CPM@warn{No MSC classification}\textit{2010
Mathematics Subject Classification} ????}
\newcommand{\dedication}[1]{\gdef\@dedication{#1}}
\let\@dedication\@empty
\def\received#1{%
  \gdef\@received{Received #1}}
\def\@received{\CPM@warn{No received date}Received 0000}
% \maketitle (:209-271), its layout trimmed to the fields it typesets
\def\maketitle{%
  \ifx\@curr@author\CPM@gobble
  \else
    \begingroup
      \def\@curr@author{\@curr@author@name}%
      \CPM@warn{No \string\address\space entry for \@curr@author}%
    \endgroup
    \address\relax
  \fi
  \begingroup
  \centering
  \LARGE\textbf{\@title}\par
  \Large
  \CPM@doauthors\par
  \ifx\@dedication\@empty\else\normalsize\textit{\@dedication}\par\fi
  \footnotetext{\@received}%
  \footnotetext{\@classification}%
  \ifx\@keywords\@empty\else\footnotetext{\@keywords}\fi
  \@thanks
  \endgroup
  \let\maketitle\relax}
\endinput
