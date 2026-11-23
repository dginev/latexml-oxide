% A minimal stand-in for iaus.cls's title page (iaus.cls:824-868, the class 0801.1806 loads; ~1,665 fleet papers),
% written from scratch for maketitle_argument_class_title_page_closes_its_group.tex: its `\maketitle` calls an
% `\@maketitle` that takes an argument. No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{mtarg}
\input{size10.clo}
\def\@maketitle#1{\par{\centering\@title\par}}
\newcommand\maketitle{\par\begingroup\@maketitle{n}\endgroup}
