\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{deepseek}
% Minimal excerpt of the deepseek.cls copies arXiv papers ship (2401.14196's copy: :112, :147-171, :263-280),
% with the font, logo and header code left out.
\LoadClass{article}
\RequirePackage{fancyhdr}
\newtoks\correspondingauthor
\fancypagestyle{firststyle}{%
  \fancyfoot[L]{%
    \ifdefined\correspondingauthor
    \if\relax\the\correspondingauthor\relax
    \else {*\the\correspondingauthor \\} \fi
    \else \fi}}
\renewcommand{\maketitle}{\bgroup\setlength{\parindent}{0pt}
  {\raggedright \@title\par}\vskip20pt
  {\raggedright \@author\par}\vskip30pt
  \egroup
  \thispagestyle{firststyle}}
