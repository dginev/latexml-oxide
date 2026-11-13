%% A stub of an emulateapj.cls copy of 6/22/04 (0709.1471's): its header and the appendix code (:755-779), the float
%% zeroing commented out
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{emulateapj}
\def\emulapj@ver{6/22/04}
\LoadClass{article}
\renewenvironment{appendix}[0]{
          \setcounter{section}{0}
          \setcounter{subsection}{0}
          \setcounter{equation}{0}
%  \setcounter{figure}{0}
%  \setcounter{table}{0}
          \def\thesection{\Alph{section}}
          \def\theequation{\hbox{\Alph{section}\arabic{equation}}}
          \def\thefigure{\thesection\@arabic\c@figure}
          \def\thetable{\thesection\@arabic\c@table}
          }{}
