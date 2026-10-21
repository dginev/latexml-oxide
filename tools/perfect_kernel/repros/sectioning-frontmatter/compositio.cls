% A minimal stand-in for compositio.cls (Foundation Compositio Mathematica / London Mathematical Society, the class the
% papers ship — not in TeX Live; its notice reserves its distribution to the LMS), written from scratch for
% compositio_family_frontmatter.tex: it defines only the interface the repro and the family's binding
% (latexml_contrib/src/compositio_cls.rs) use, with what the class does there; line numbers are those of 2606.19241's
% copy. No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{compositio}[2002/01/01 repro stand-in]
\LoadClass[twoside,11pt]{article}
% :117-131 `\author[short]{name}`: the names joined by "and" for the title page; the current author's contact lines
% gather in `\@curr@author` (the family's store, which the binding looks for)
\newcommand\@curr@author{}
\newcommand\cps@names{}
\renewcommand\author[2][]{%
  \ifx\cps@names\@empty\gdef\cps@names{#2}\else\g@addto@macro\cps@names{ and #2}\fi
  \gdef\@curr@author{#2}}
% :141-165 the address, current address and e-mail of the current author, printed after the paper
\newcommand\cps@contacts{}
\newcommand\address[2][]{\g@addto@macro\cps@contacts{\par\noindent #2}}
\newcommand\curraddr[2][]{\g@addto@macro\cps@contacts{\par\noindent\textit{Current address:} #2}}
\newcommand\email[2][]{\g@addto@macro\cps@contacts{\par\noindent\textsf{#2}}}
\AtEndDocument{\par\cps@contacts}
% :174-206 what the title page prints in its notes: the thanks, the keywords ("Keywords:"), the classification (the
% binding reads the label, "2010 Mathematics Subject Classification", off this definition), the received date; and the
% dedication under the names
\newcommand\cps@notes{}
\renewcommand\thanks[1]{\g@addto@macro\cps@notes{\footnotetext{#1}}}
\newcommand\keywords[1]{\g@addto@macro\cps@notes{\footnotetext{\textit{Keywords:} #1}}}
\newcommand\classification[2][]{%
  \g@addto@macro\cps@notes{\footnotetext{\textit{2010 Mathematics Subject Classification} #2}}}
\let\subjclass\classification
\newcommand\received[1]{\g@addto@macro\cps@notes{\footnotetext{Received #1}}}
\newcommand\cps@dedication{}
\newcommand\dedication[1]{\gdef\cps@dedication{#1}}
% :209-271 `\maketitle`: the title, the names, the dedication, the notes
\renewcommand\maketitle{{\centering{\LARGE\bfseries\@title\par}{\Large\cps@names\par}%
  \ifx\cps@dedication\@empty\else\textit{\cps@dedication}\par\fi}\cps@notes}
\endinput
