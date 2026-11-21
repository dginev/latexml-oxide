% stand-in for edpsmath.cls:317,326 (2004.12879, 1110.0287, 2603.24523, 1501.03645): placeholders \@subjclass and
% \@keywords that take an argument; edpsproc.clo:9 empties \@setsubjclass, so amsart's \maketitle never prints them
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{subjclass_placeholder}
\LoadClass{amsart}
\def\@subjclass#1{{--- Please, give AMS classification codes ---}}
\def\@keywords#1{{--- Please, give keywords ---}}
\let\@setsubjclass\relax
\let\@setkeywords\relax
