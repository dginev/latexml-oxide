%%
%% MastersDoctoralThesis.cls (Version 1.6, 27/8/17; http://www.LaTeXTemplates.com; authors Vel and Johannes Böttcher)
%% Class license: LPPL v1.3c (http://www.latex-project.org/lppl)
%%
%% MODIFIED and RENAMED: mdtmini.cls is a modified, shortened copy of one line of MastersDoctoralThesis.cls (as
%% 2603.21941 ships it), made for latexml-oxide's repros (tools/perfect_kernel/repros/class-bindings/
%% mastersdoctoralthesis/), not the original work.
%%
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{mdtmini}[2023/01/01 MastersDoctoralThesis.cls cut down]
\LoadClass{book}
\RequirePackage{xparse}
% MastersDoctoralThesis.cls:218
\DeclareDocumentCommand{\author}{m}{\newcommand{\authorname}{#1}\renewcommand{\@author}{#1}}
