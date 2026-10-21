% A stand-in written from scratch for the lines of 2503.14633's sagej.cls (SAGE's class, "All rights reserved"; not in
% TeX Live) that the sagej binding reads (an excerpt, not a working class): its supplemental-material statement
% (:447-449), set under a heading "Supplemental Material" — the spelling of that copy, which the binding looks for
% (sagej_cls.rs: `\bfseries Supplemental Material}`). No code of the class is copied.
\newenvironment{sm}[1]{\subsection*{\normalsize\sagesf\bfseries Supplemental Material}\begin{refsize}\noindent#1}%
  {\end{refsize}}
