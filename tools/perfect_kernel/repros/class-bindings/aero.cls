% An excerpt of aero.cls (latexstudio.net, v1.0 2022/01/04, shipped with 2610.12827, not in TeX Live): its option
% handling, the title-page keys (:95-118), the running head (:120-126) and its `\maketitle` (:128-148), verbatim; the
% layout and the many packages of the rest left out.
\NeedsTeXFormat{LaTeX2e}[1999/12/01]
\ProvidesClass{aero}[2022/01/04, v1.0 by latexstudio.net]
\newif\if@linenumberon\global\@linenumberonfalse%
\DeclareOption{lineno}{\global\@linenumberontrue}%
\DeclareOption*{\PassOptionsToClass{\CurrentOption}{article}}
\ProcessOptions\relax
\LoadClass[oneside,a4paper,11pt]{article}
\RequirePackage{expl3}
\RequirePackage{fancyhdr}
\RequirePackage{marvosym}
\let\linenumbershook\empty
\newcommand{\cor}{{\rm (\Letter)}}
\ExplSyntaxOn
\keys_define:nn { ADkeys } {
  name     . tl_set:N  = \l_ADkeys_name_tl,
  name     . initial:n = ,
  title    . tl_set:N  = \l_ADkeys_title_tl,
  title    . initial:n = Paper~title:~Preparation~of~papers~for~\textit{Astrodynamics},
  author   . tl_set:N  = \l_ADkeys_author_tl,
  author   . initial:n = {First~A.~Author$^{1}$,~Second~B.~Author$^{2}$,~and~Third~C.~Author$^{3}$},
  address  . tl_set:N  = \l_ADkeys_address_tl,
  address  . initial:n = {1.~Business~or~academic~affiliation~of~First~A.~Author,~with~city,~post~code,~and~country},
  email    . tl_set:N  = \l_ADkeys_email_tl,
  email    . initial:n = Corresponding~author's~name~and~email~address,
  abstract . tl_set:N  = \l_ADkeys_abstract_tl,
  abstract . initial:n = Abstract~content,
  keywords . tl_set:N  = \l_ADkeys_keywords_tl,
  keywords . initial:n = keywords~content,
  url      . tl_set:N  = \l_ADkeys_url_tl,
  url      . initial:n = ,
}
\NewDocumentCommand\ADsetup{ +m }{
  \keys_set:nn{ADkeys}{#1}
}
\fancypagestyle{plain}{
  \fancyhf{}
  \renewcommand{\headrulewidth}{0pt}
  \fancyhead[L]{\itshape \l_ADkeys_name_tl}
  \fancyhead[R]{\l_ADkeys_url_tl}
  \fancyfoot[C]{\thepage}
}
\RenewDocumentCommand\maketitle{}{
  \pagestyle{plain}
  \linenumbershook
  \parindent 0pt
  \begingroup
  {\bfseries\LARGE\l_ADkeys_title_tl}\par
  \bigskip
  \l_ADkeys_author_tl\par
  \bigskip
  {\itshape\hangafter1\hangindent 1.06em \def\sep{\par \hangafter1\hangindent 1.06em} \l_ADkeys_address_tl}\par
  \bigskip
  {\Letter\ \l_ADkeys_email_tl}\par
  \bigskip
  \textbf{Abstract}\\
  \l_ADkeys_abstract_tl\par
  \bigskip
  \textbf{Keywords}\\
  {\def\sep{~·\ \ } \l_ADkeys_keywords_tl}
  \endgroup
  \parindent 1em
}
\ExplSyntaxOff
\endinput
