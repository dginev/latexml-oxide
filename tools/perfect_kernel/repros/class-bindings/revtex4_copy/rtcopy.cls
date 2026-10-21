%%
%% Copyright (c) 1999, 2001 The American Physical Society (revtex4.cls; ltxutil portions licensed from W. E. Baxter)
%%
%% Licensed under the LPPL (revtex4.cls: http://www.ctan.org/tex-archive/macros/latex/base/lppl.txt).
%%
%% MODIFIED: this is a modified and shortened copy of part of revtex4.cls (v4.0, 2001/08/03, as qm2008_abs.cls copies
%% it), made for latexml-oxide's repros (tools/perfect_kernel/repros/class-bindings/revtex4_copy/), not the original
%% work: it keeps only the author/collaboration bookkeeping (:164-182, :4215-4330), on article.
%%
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{rtcopy}[2001/08/03 v4.0 revtex4 copy]
\LoadClass{article}
% revtex4.cls (qm2008_abs.cls:164-182, 4215-4330), cut down
\long\def\true@sw#1#2{#1}
\long\def\false@sw#1#2{#2}
\def\@booleantrue#1{\let#1\true@sw}
\def\@booleanfalse#1{\let#1\false@sw}
\def\@ifx#1{\ifx#1\expandafter\true@sw\else\expandafter\false@sw\fi}
\def\@author@cleared{{}{}}
\def\@author@init{\let\@author\@author@cleared}
\def\@authorclear@sw{\@ifx{\@author\@author@cleared}}
\AtBeginDocument{\@author@init}
\renewcommand\author{\@author@def{}}
\def\collaboration{\@author@def{\@booleantrue\collaboration@sw}}
\def\@author@def#1#2{\move@AU #1\def\@author{{#2}{}}}
\def\move@AU{\@authorclear@sw{}{\collaboration@sw{\typeout{CO}}{\typeout{AU}}}\@author@init\@booleanfalse\collaboration@sw}
\newcommand\affiliation[1]{\move@AU}
