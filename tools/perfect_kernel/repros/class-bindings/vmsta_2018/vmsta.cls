\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{vmsta}[2018/09/11 vmsta.cls v2 cut down]
\LoadClass{article}
\RequirePackage{etoolbox}
% 1805.02511's style/vmsta.cls (2018+ copies, cut down): `\author[labels]{…\ead[label=e1]{…}}` stored for the title
% page (:753-792), `\ead` (:1263-1297) adding the e-mail to the title page's e-mail line and counting it, the
% `\address[label]{…}` list (:871-920), and the `{frontmatter}` end checking for an e-mail (:550-565)
\newcounter{author}
\def\author@list{}
\def\author{\@ifnextchar[{\author@optarg}{\author@optarg[]}}
\def\author@optarg[#1]#2{\stepcounter{author}%
  \def\fnms##1{##1}\def\snm##1{##1}\def\inits##1{}%
  \setbox\z@\hbox{#2}% (the class's `\proc@elem` runs the content, :971-981: its `\ead` adds the e-mail)
  \csgdef{author\the\c@author}{#2}%
  \ifx\author@list\@empty\xdef\author@list{author\the\c@author}\else\xappto\author@list{,author\the\c@author}\fi}
\newtoks\email@author@toks
\newcounter{countead}
\providecommand\ead[2][]{\stepcounter{countead}\global\email@author@toks\expandafter{\the\email@author@toks #2 }}
\newcounter{address}
\def\address@list{}
\def\address{\@ifnextchar[{\address@optarg}{\address@noptarg}}
\def\address@optarg[#1]#2{\refstepcounter{address}\csgdef{address\the\c@address}{\textsuperscript{#1}#2}%
  \ifx\address@list\@empty\xdef\address@list{address\the\c@address}\else\xappto\address@list{,address\the\c@address}\fi
  \ignorespaces}
\def\address@noptarg#1{\refstepcounter{address}\csgdef{address\the\c@address}{#1}%
  \ifx\address@list\@empty\xdef\address@list{address\the\c@address}\else\xappto\address@list{,address\the\c@address}\fi
  \ignorespaces}
\def\maketitle{\par{\centering\bfseries\renewcommand\ead[2][]{}%
  \def\do##1{\csuse{##1}\par}\expandafter\docsvlist\expandafter{\author@list}}%
  {\itshape\def\do##1{\csuse{##1}\par}\expandafter\docsvlist\expandafter{\address@list}}%
  \the\email@author@toks\par}
\newenvironment{frontmatter}{}{\maketitle
  \ifnum\c@author=0\relax\else\ifnum\c@countead=0\relax
    \ClassError{vmsta}{At least one email address is required!}{}\fi\fi}
