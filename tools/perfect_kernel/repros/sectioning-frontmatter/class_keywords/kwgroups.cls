% Label-only `{keywords}` begin codes whose label is a brace group, one per class option: the group's font and layout
% commands are the label's, not the keywords' — emsprocart.cls:668-670 (1202.4161, 1006.0632, 1610.08545's copies),
% ieice.cls:952-954 (2002.00666, 1604.03276, 2509.21881), lni.cls:157-158 (1604.03313's copy), and the review's minimal
% shapes va, vb, ve.
\ProvidesClass{kwgroups}
\def\keywordname{Keywords}
\DeclareOption{va}{\newenvironment{keywords}{{\noindent\bfseries Keywords.}}{}}
\DeclareOption{vb}{\newenvironment{keywords}{{\bfseries Keywords.\vspace{\z@}}}{}}
\DeclareOption{emsprocart}{\newenvironment{keywords}{%
              \vspace{2pt}\small
                 {\noindent\bfseries{\keywordname.}\vspace{\z@}}}{\par}}% (the copy's end code is the blank line after it)
\DeclareOption{ve}{\newenvironment{keywords}{{\bfseries Keywords:}}{}}
\DeclareOption{ieice}{\newenvironment{keywords}{%
 \noindent\footnotesize\itshape
 {\bfseries key words:\ \ignorespaces}}{\par}}
\DeclareOption{lni}{\newenvironment{keywords}{\fontsize{9}{10}\selectfont
    \noindent{\bfseries Keywords:}}{}}
\ProcessOptions
\LoadClass{article}
