%%
%% Copyright 1996, 2000-2002, 2007, 2009, 2011 American Mathematical Society (memo-l.cls)
%%
%% Unlimited copying and redistribution of this file are permitted as long as this file is not modified.
%% Modifications, and distribution of modified versions, are permitted, but only if the resulting file is renamed.
%%
%% MODIFIED and RENAMED: memomini.cls is a modified, shortened copy of part of memo-l.cls (v3.23, 2014/05/27, as
%% 2412.15928 ships it), made for latexml-oxide's repros (tools/perfect_kernel/repros/class-bindings/memo_l/), not the
%% original work.
%%
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
