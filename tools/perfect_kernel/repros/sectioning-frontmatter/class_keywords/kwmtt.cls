% The shape of mtt.cls:176-181 (1310.0585's copy; mttr.cls, 1212.3890): the class's own `{abstract}`, with an optional
% language argument, defines `\keywords` for its body.
\ProvidesClass{kwmtt}
\LoadClass{article}
\renewenvironment{abstract}[1][en]{%
  \parindent=0pt
  \def\keywords##1{\par\noindent{\bfseries Keywords:} \itshape##1}%
  \list{}{}\item\relax}{\endlist}
