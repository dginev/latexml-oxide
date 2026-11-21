%% A shipped copy of smfart.cls under another name (1401.0915 fsmfart.cls:32,365-372; smfartVS.cls 1110.1467,
%% 1404.0804), cut down to its first-load test: with amsmath loaded already, `\@tempa` is `\@gobbletwo` and eats the
%% `\ifx\relax`, leaving the `\else` and `\fi` unmatched.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{smfart}[2003/10/28 v1.3f]
\LoadClass{article}
\@ifundefined{numberwithin}%
  {\let\@tempa\relax}%
  {\let\@tempa\@gobbletwo}
\@tempa
\ifx\relax\mathfrak \expandafter\@gobbletwo
  \else \let\mathfrak\relax \fi
\RequirePackage{amsfonts}\relax
\RequirePackage{amsmath}\relax
\newcommand\smfcopymacro{SMF-COPY-MACRO}
\endinput
