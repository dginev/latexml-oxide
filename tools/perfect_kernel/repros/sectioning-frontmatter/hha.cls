% An excerpt of hha.cls (the copy math0409234 ships), its frontmatter commands (:77-166), for
% compositio_family_old_variant.tex.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{hha}
\LoadClass[11pt]{article}
\def\HHA@err#1{\ClassError{hha}{#1}\@ehd}
\def\HHA@warn#1{\ClassWarning{hha}{#1}}
\renewcommand*\author[2][]{%
  \gdef\HHA@doauthors{\@authors\@andorcomma\relax\@andorcomma\relax\par}%
  \g@addto@macro\@curr@author{\\*#2}%
  \g@addto@macro\@authors{\@andorcomma,{\ignorespaces\MakeTextUppercase{#2}\unskip}}}
\def\@authors{\@gobbletwo}
\def\@curr@author{\@gobbletwo}
\def\address#1{%
  \ifx\@curr@author\HHA@gobble
    \HHA@err{\noexpand\address with no \string\author}\@ehd
  \else
    \expandafter\g@addto@macro\expandafter\@addresses\expandafter{%
         \expandafter\filbreak\@curr@author\hha@address{#1}}%
    \let\@curr@author\HHA@gobble
  \fi}
\def\@addresses{}
\AtEndDocument{\par\@addresses}
\def\hha@address#1{\\*[6pt]\begin{tabular}{@{}l}#1\end{tabular}\par}
\def\HHA@gobble{\@gobbletwo}
\def\email#1{%
  \ifx\@curr@author\HHA@gobble
    \HHA@err{\noexpand\email with no \string\author}\@ehd
  \else
    \g@addto@macro\@curr@author{\quad\texttt{#1}}%
  \fi}
\def\homepage#1{%
  \ifx\@curr@author\HHA@gobble
    \HHA@err{\noexpand\homepage with no \string\author}\@ehd
  \else
    \g@addto@macro\@curr@author{\quad\url{#1}}%
  \fi}
\AtBeginDocument{\providecommand\url[1]{\texttt{#1}}}
\def\keywords#1{%
  \gdef\@keywords{%
    {Key words and phrases:\/} \ignorespaces#1\unskip}}
\def\@keywords{\HHA@warn{No keywords}}
\def\classification#1{%
  \gdef\@classification{%
    2000 Mathematics Subject Classification: \ignorespaces#1\unskip}}
\let\subjclass\classification
\def\received#1{%
  \gdef\@received{Received #1}}
\def\submitted#1{%
  \gdef\@submitted{({\itshape communicated by #1})}}
\endinput
