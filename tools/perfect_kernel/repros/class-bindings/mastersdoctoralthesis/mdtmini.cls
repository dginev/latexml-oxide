\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{mdtmini}[2023/01/01 MastersDoctoralThesis.cls cut down]
\LoadClass{book}
\RequirePackage{xparse}
% MastersDoctoralThesis.cls:218
\DeclareDocumentCommand{\author}{m}{\newcommand{\authorname}{#1}\renewcommand{\@author}{#1}}
