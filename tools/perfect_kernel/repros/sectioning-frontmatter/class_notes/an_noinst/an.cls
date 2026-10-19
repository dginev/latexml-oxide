% A trimmed copy of an.cls (Astronomische Nachrichten, Wiley-VCH, 2006-01-13; not in TeX Live): its `\institute`
% store (an.cls:416), the affiliation block that reads it (`\institutename`, :418-440), the defaults its own
% `\maketitle` gives the stores a paper leaves unset (`\ch@ckobl`, :511-515, called at :528-532), its `\keywords`
% (:354-360) and the title page that prints both (`\@maketitle`, :703, :715).
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{an}[2006/01/13 trimmed copy]
\LoadClass{article}
\def\keywordname{\textbf{Key words}}
\newcommand{\keywords}[1]{%
\if!#1!%
  \let\@keywords\relax%
\else%
  \def\@keywords{\textbf{\keywordname}\quad#1\ignorespaces\vskip6pt}%
\fi}
\keywords{}
\newcounter{inst}
\newdimen\instindent
\def\institute#1{\gdef\@institute{#1}}
\def\institutename{\par
 \begingroup
 \parindent=0pt
 \parskip=0pt
 \setcounter{inst}{1}%
 \setbox0=\vbox{\def\thanks##1{}\@institute}
 \small
 \ignorespaces
 \@institute\par
 \endgroup}
\def\ch@ckobl#1#2{\@ifundefined{@#1}
 {\typeout{Warning: Missing
\expandafter\string\csname#1\endcsname}%
  \csname #1\endcsname{#2}}
 {}}
\def\maketitle{\par
\ch@ckobl{title}{A title should be given}
\ch@ckobl{author}{Name(s) and initial(s) of author(s) should be given}
\ch@ckobl{institute}{Address(es) of author(s) should be given}
\begingroup\@maketitle\endgroup}
\def\@maketitle{\newpage
 \normalfont
 {\fontsize{14.4}{17}\bfseries\selectfont\@title}\par
 \vskip 4.5mm\relax
 \noindent\ignorespaces{\normalsize\bfseries\selectfont{\@author}}%
 \vskip10pt\relax
 \institutename
 \vskip 5mm
 \noindent
 \@keywords
 \par\vskip 5mm}
