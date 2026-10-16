\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{memomini}[2014/05/27 memo-l.cls cut down]
\LoadClass{amsbook}[2008/07/02]
% memo-l.cls:45-46, :87-93 and :116-125
\newcommand\PII{\def\@PII}
\PII{}
\def\tableofcontents{%
    \@starttoc{toc}\contentsname
    \ifvoid\abstractbox\else\@setabstract\fi
    \let\tableofcontents\@empty
}
\def\@makeabsfootnotes{%
    \begingroup
        \ifx\@empty\@dateposted\else\@footnotetext{Published on \@dateposted}\fi
        \ifx\@empty\@PII\else\@footnotetext{\@PII}\fi
    \endgroup
}
\AtBeginDocument{\@makeabsfootnotes}
