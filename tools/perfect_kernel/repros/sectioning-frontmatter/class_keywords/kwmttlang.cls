% The shape of mtt.cls:176-181 with its language-chosen label (1310.0585, 1310.0589, 1212.3890): the class's own
% `{abstract}`, with an optional language argument, defines `\keywords` for its body as an `\item` of its list, its
% label a macro the abstract's language selects (`\mtt@keywords`) — not readable without running the class's code.
\ProvidesClass{kwmttlang}
\LoadClass{article}
\def\kw@en{Keywords}
\def\kw@fr{Mots-cl\'es}
\renewenvironment{abstract}[1][en]{%
  \parindent=0pt
  \def\kw@lang{#1}%
  \def\kw@keywords{\csname kw@\kw@lang\endcsname}%
  \def\keywords##1{\item\footnotesize{\bfseries\kw@keywords:} \itshape##1}%
  \list{}{}\item\relax}{\endlist}
