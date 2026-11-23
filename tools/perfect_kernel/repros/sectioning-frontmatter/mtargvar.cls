% Variants of mtarg.cls (iaus.cls:824-868, 0801.1806), one `\maketitle` per class option, for
% maketitle_argument_class_relaxed_after_dropped_replay_keeps_its_text.tex and
% maketitle_argument_class_never_called_prints_none_of_it.tex: `relax`, `empty`, `gdef` err when the title is empty (as
% it is in the replay) after running `\@maketitle{n}` and then switch it off as iaus.cls:847 does (`\global\let…\relax`)
% or with `\@empty` or `\gdef…{}`; `bare` never calls `\@maketitle`. No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{mtargvar}
\input{size10.clo}
\def\@maketitle#1{\par{\centering\@title\par}\textit{A Subtitle Written Here}\par}
\def\mtarg@body{\par\begingroup\@maketitle{n}\endgroup\ifx\@title\@empty\@latex@error{No title}\@ehc\fi}
\DeclareOption{relax}{\newcommand\maketitle{\mtarg@body\global\let\@maketitle\relax}}
\DeclareOption{empty}{\newcommand\maketitle{\mtarg@body\global\let\@maketitle\@empty}}
\DeclareOption{gdef}{\newcommand\maketitle{\mtarg@body\gdef\@maketitle{}}}
\DeclareOption{bare}{\newcommand\maketitle{\par\begingroup{\centering\@title\par}\endgroup}}
\ProcessOptions
