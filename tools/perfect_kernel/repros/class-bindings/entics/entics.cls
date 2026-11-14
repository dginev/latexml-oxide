\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{entics}[2022/02/20 a stand-in for entics.cls (esl.dtx), written for this repro]
\LoadClass{article}
% entics.cls:707-735, :860-930 (esl.dtx): `\thanksref{key}` is no command but a delimiter `\author` reads its argument
% by (`\proc@elem`, `\elem@nothanksref`), the keys' note numbers printed as a superscript after the name; `\address`
% prints its text under the authors. Cut down to that (one key, after the name).
\newcounter{fnote}
\def\thanks{\@ifnextchar[{\thanks@key}{\thanks@key[]}}
\def\thanks@key[#1]#2{\refstepcounter{fnote}\ifx\relax#1\relax\else\label{#1}\fi
  \g@addto@macro\@notes{\par\noindent\textsuperscript{\thefnote}#2}}
\let\@notes\@empty
\def\author#1{\author@parse#1\thanksref\relax}
\def\author@parse#1\thanksref#2\relax{\g@addto@macro\@authors{#1}%
  \ifx\relax#2\relax\else\author@key#2\fi\g@addto@macro\@authors{\par}}
\def\author@key#1#2{\g@addto@macro\@authors{\textsuperscript{\ref{#1}}}}
\let\@authors\@empty
\def\address#1{\g@addto@macro\@authors{{\itshape#1}\par}}
\def\frontmatter{}\def\endfrontmatter{{\centering\@authors}\@notes\par}
% entics.cls:449: the end-of-proof mark, an open box set in math
\def\qed{\relax\ifmmode\hfill \Box\else\unskip\nobreak\hfill $\Box$\fi}
