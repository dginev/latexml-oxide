%% A stub of an emulateapj.cls copy of 11/26/04 (0704.2079's): its header and the appendix code (:774-805), which
%% zeroes the floats
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{emulateapj}
\def\emulapj@ver{11/26/04}
\LoadClass{article}
\newcounter{remember@figure@num}
\newcounter{remember@table@num}
\renewenvironment{appendix}[0]{
          \setcounter{section}{0}
          \setcounter{subsection}{0}
          \setcounter{equation}{0}
          \setcounter{remember@figure@num}{\c@figure}
          \setcounter{remember@table@num}{\c@table}
          \setcounter{figure}{0}
          \setcounter{table}{0}
          \def\thesection{\Alph{section}}
          \def\theequation{\hbox{\Alph{section}\arabic{equation}}}
          \def\thefigure{\thesection\@arabic\c@figure}
          \def\thetable{\thesection\@arabic\c@table}
          \def\section{\setcounter{equation}{0}\setcounter{figure}{0}\setcounter{table}{0}\@startsection {section}{1}{\z@}
            {10pt}{5pt}{\centering\scshape}}
          }{%
            \setcounter{figure}{\c@remember@figure@num}%
            \setcounter{table}{\c@remember@table@num}%
          }
