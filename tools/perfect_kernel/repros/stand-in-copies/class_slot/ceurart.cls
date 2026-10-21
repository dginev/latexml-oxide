%%
%% ceurart.cls: Dmitry S. Kulyabov (Typesetting articles for CEUR Workshop Proceedings)
%%
%% Released under the LaTeX Project Public License v1.3c or later; see http://www.latex-project.org/lppl.txt
%%
%% MODIFIED: a reduced model of ceurart.cls (v0.6.1; three of its option lines kept), made for latexml-oxide's
%% repros (tools/perfect_kernel/repros/stand-in-copies/class_slot/), not the original work.
%%
% A reduced model of ceurart.cls (2509.11496, 2608.23214, 2310.02658, 2505.16276): its declaration, its options
% passed to scrartcl, its own `\LoadClass{scrartcl}` (:59-62) and a KOMA call after it (:63), a package, its own
% `{keywords}` environment (:1783; there a box its title page prints, with `\sep` a comma) and an extra of its own.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{ceurart}[2022/04/01 reduced]
\DeclareOption*{\PassOptionsToClass{\CurrentOption}{scrartcl}}
\PassOptionsToClass{a4paper,twoside=false}{scrartcl}
\ProcessOptions\relax
\LoadClass{scrartcl}
\KOMAoptions{DIV=10}
\RequirePackage{amsmath}
\newenvironment{keywords}{\par\noindent\textbf{Keywords}\par\def\sep{\unskip,}}{\par}
\newcommand\ceurextra{CEUR}
