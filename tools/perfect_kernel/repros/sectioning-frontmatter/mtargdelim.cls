% mtargerr.cls (iaus.cls:824-868, 0801.1806) with an `\@maketitle` whose parameter is delimited (`#1.`), for
% maketitle_delimited_argument_class_keeps_its_text_red.tex. No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{mtargdelim}
\input{size10.clo}
\def\@maketitle#1.{\par{\centering\@title\par}\textit{A Subtitle Written Here}\par}
\newcommand\maketitle{\par\begingroup\@maketitle n.\endgroup\ifx\@title\@empty\@latex@error{No title}\@ehc\fi}
