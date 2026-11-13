% A minimal stand-in for jcm.cls (the Compositio-family class quant-ph0412067 ships — not in TeX Live; copyright
% David Carlisle, no licence stated), written from scratch for compositio_family_old_variant.tex: it defines only the
% interface the Compositio family's binding (latexml_contrib/src/compositio_cls.rs) reads — the family's
% `\@curr@author` store and the two-argument `\homepage{url}{text}` (jcm.cls:157-162). No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{jcm}[1997/01/01 repro stand-in]
\LoadClass{article}
\newcommand\@curr@author{}
\renewcommand\author[2][]{\gdef\@curr@author{#2}\gdef\@author{#2}}
\newcommand\homepage[2]{\g@addto@macro\@curr@author{, \href{#1}{\texttt{#2}}}}
\endinput
