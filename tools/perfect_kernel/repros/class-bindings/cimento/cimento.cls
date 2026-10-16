\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{cimento}[2005/01/01 cimento.cls cut down]
% self-contained, as cimento.cls is (no \LoadClass); the page setup of minimal.cls
\renewcommand\normalsize{\fontsize{10pt}{12pt}\selectfont}
\setlength{\textwidth}{6.5in}
\setlength{\textheight}{8in}
\pagenumbering{arabic}
% cimento.cls:680, :704-706, :708-757: the author list ends with `|`; `\ETC`, defined in `\maketitle` only, keeps
% the rest of it on the title page (`\scrunch` drops it for the running head)
\catcode`\|=\active
\renewcommand{\author}[1]{\gdef\@author@{#1|}}
\gdef\unscrunch|#1|{#1}
\gdef\scrunch|#1|{}
\def\@maketitle{{\centering\scshape\@title\par\@author@\par}}
\newcommand{\maketitle}{\par\begingroup\def\ETC{\unskip\unscrunch|}\def|{}\@maketitle\endgroup}
\catcode`\|=12
