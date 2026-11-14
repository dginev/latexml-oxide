% A minimal stand-in for hha.cls (Homology, Homotopy and Applications, the class math0409234 ships — not in TeX Live; it
% states no licence), written from scratch for compositio_family_old_variant.tex: it defines only the interface the
% repro and the Compositio family's binding (latexml_contrib/src/compositio_cls.rs) use, with what the class does
% there; line numbers are those of math0409234's copy. No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{hha}[2004/01/01 repro stand-in]
\LoadClass[11pt]{article}
% :77-86 `\author[short]{name}`: the names in capitals on the title page; the current author's lines gather in
% `\@curr@author` (the family's store, which the binding looks for)
\newcommand\@curr@author{}
\newcommand\hhs@names{}
\renewcommand\author[2][]{\g@addto@macro\hhs@names{\MakeUppercase{#2}\par}\gdef\@curr@author{#2}}
% :104-137 the one-argument address, e-mail and homepage, printed after the paper (the binding counts the homepage's
% one argument)
\newcommand\hhs@contacts{}
\newcommand\address[1]{\g@addto@macro\hhs@contacts{\par\noindent #1}}
\newcommand\email[1]{\g@addto@macro\hhs@contacts{\par\noindent\texttt{#1}}}
\newcommand\homepage[1]{\g@addto@macro\hhs@contacts{\par\noindent\texttt{#1}}}
\AtEndDocument{\par\hhs@contacts}
% :142-166 what the title page prints: the keywords ("Key words and phrases:"), the classification (the binding reads
% its label, "2000 Mathematics Subject Classification:", off this definition), the received date and the communicating
% editor
\newcommand\hhs@notes{}
\newcommand\keywords[1]{\g@addto@macro\hhs@notes{\par Key words and phrases: #1}}
\newcommand\classification[1]{\g@addto@macro\hhs@notes{\par 2000 Mathematics Subject Classification: #1}}
\let\subjclass\classification
\newcommand\received[1]{\g@addto@macro\hhs@notes{\par Received #1}}
\newcommand\submitted[1]{\g@addto@macro\hhs@notes{\par(\textit{communicated by #1})}}
\renewcommand\maketitle{{\centering{\Large\@title\par}\hhs@names}\hhs@notes\par}
\endinput
