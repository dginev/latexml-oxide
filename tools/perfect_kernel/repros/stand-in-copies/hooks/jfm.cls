\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{jfm}[2007/01/01 v2.1]
\LoadClass{article}
\newif\ifuseAMSsubequ
\newenvironment{CUPsubequations}{\par[}{]\par}
\newcommand\jfmextra{JFM-EXTRA}
\AtBeginDocument{%
  \ifuseAMSsubequ \else
    \global\let\subequations\CUPsubequations
    \global\let\endsubequations\endCUPsubequations
  \fi}
