% mtarg.cls (iaus.cls:824-868, 0801.1806) with a `\maketitle` that errs when its title is empty, as it is in the
% replay of the class's `\maketitle` (the fields read as empty), for
% maketitle_argument_class_replay_dropped_deposits_its_text.tex. No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{mtargerr}
\input{size10.clo}
\def\@maketitle#1{\par{\centering\@title\par}\textit{A Subtitle Written Here}\par}
\newcommand\maketitle{\par\begingroup\ifx\@title\@empty\@latex@error{No title}\@ehc\fi\@maketitle{n}\endgroup}
