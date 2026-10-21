% A minimal stand-in for the deepseek.cls copies arXiv papers ship (2401.14196's copy; not in TeX Live, no licence
% stated), written from scratch for deepseek_corresponding_author_footer_note.tex: the corresponding-author note, a token
% register the paper fills (:112), printed only in the title page's footer (`firststyle`, :147-171), which its
% `\maketitle` selects (:263-280), with what the class does there. No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{deepseek}[2024/01/01 repro stand-in]
\LoadClass{article}
\RequirePackage{fancyhdr}
\newtoks\correspondingauthor
\fancypagestyle{firststyle}{\fancyfoot[L]{\setbox0\hbox{\the\correspondingauthor}%
  \ifdim\wd0>0pt *\the\correspondingauthor\fi}}
\renewcommand\maketitle{{\parindent=0pt\raggedright\@title\par\vskip 20pt\@author\par\vskip 30pt}%
  \thispagestyle{firststyle}}
