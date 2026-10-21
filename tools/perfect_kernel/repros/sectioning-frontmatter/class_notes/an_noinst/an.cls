% A minimal stand-in for an.cls (Astronomische Nachrichten, Wiley-VCH, 2006-01-13 — not in TeX Live; it states no
% licence), written from scratch for an_without_institute_keeps_the_title_block.tex: the `\institute` store (an.cls:416),
% the affiliation block that prints it (`\institutename`, :418-440), the defaults the class's own `\maketitle` gives a
% store the paper left unset, announced on the terminal (`\ch@ckobl`, :511-515, called at :528-532), its `\keywords`
% (:354-360) and the title page that prints them (`\@maketitle`, :703, :715), with what the class does there. No code of
% the class is copied.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{an}[2006/01/13 repro stand-in]
\LoadClass{article}
\newcommand\keywordname{\textbf{Key words}}
\newcommand\keywords[1]{\ifx\relax#1\relax\let\@keywords\relax
  \else\def\@keywords{\textbf{\keywordname}\quad#1\vskip 6pt}\fi}
\let\@keywords\relax
\newcounter{inst}
\newcommand\institute[1]{\gdef\@institute{#1}}
\newcommand\institutename{\par\begingroup\parindent=0pt\parskip=0pt\setcounter{inst}{1}\small\@institute\par\endgroup}
% `\ch@ckobl{store}{default}`: a store the paper never gave is set to the default, with a terminal note
\def\ch@ckobl#1#2{\@ifundefined{@#1}{\typeout{Warning: Missing \expandafter\string\csname#1\endcsname}%
  \csname#1\endcsname{#2}}{}}
\def\maketitle{\par
  \ch@ckobl{title}{A title should be given}%
  \ch@ckobl{author}{Name(s) and initial(s) of author(s) should be given}%
  \ch@ckobl{institute}{Address(es) of author(s) should be given}%
  \begingroup\@maketitle\endgroup}
\def\@maketitle{\newpage\normalfont
  {\fontsize{14.4}{17}\bfseries\selectfont\@title}\par\vskip 4.5mm\relax
  \noindent{\normalsize\bfseries\selectfont{\@author}}\vskip 10pt\relax
  \institutename\vskip 5mm
  \noindent\@keywords\par\vskip 5mm}
