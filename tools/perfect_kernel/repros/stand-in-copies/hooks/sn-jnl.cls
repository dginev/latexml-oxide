\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{sn-jnl}[2023/06/05 v2.1]
\LoadClass{article}
\newcommand\snextra{SN-EXTRA}
\newenvironment{appendices}{\appendix}{}
\AtBeginDocument{%
\let\oldappendices\appendices
\let\oldendappendices\endappendices
\renewenvironment{appendices}{\begin{oldappendices}%
  \gdef\theequation{\Alph{section}\arabic{equation}}}{\end{oldappendices}}%
\gdef\thetable{T\arabic{table}}}
