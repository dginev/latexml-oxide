% A reduced model of ceurart.cls (2509.11496, 2608.23214, 2310.02658, 2505.16276): its declaration, its options
% passed to scrartcl, its own `\LoadClass{scrartcl}` (:59-62), a package and an extra of its own.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{ceurart}[2022/04/01 reduced]
\DeclareOption*{\PassOptionsToClass{\CurrentOption}{scrartcl}}
\PassOptionsToClass{a4paper,twoside=false}{scrartcl}
\ProcessOptions\relax
\LoadClass{scrartcl}
\RequirePackage{amsmath}
\newcommand\ceurextra{CEUR}
