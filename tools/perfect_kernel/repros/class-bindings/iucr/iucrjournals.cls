\NeedsTeXFormat{LaTeX2e}\ProvidesClass{iucrjournals}[2024/01/01 stand-in for the IUCr class shipped by 2608.11466, 2503.09183]
\LoadClass{article}
\RequirePackage{hyperref}
% iucrjournals.cls:201-206, the mail logo a tikz picture there
\newcommand\IUCr@cemaillogo{\texttt{@}}
\DeclareRobustCommand\IUCr@emaillinkX[1]{\href{mailto:#1}{\IUCr@cemaillogo}}
\DeclareRobustCommand\IUCr@cemaillinkX[1]{\href{mailto:#1}{\IUCr@cemaillogo}}
\newcommand{\IUCrEmaillink}[1]{\,\IUCr@emaillinkX{#1}\,}
\newcommand{\IUCrCemaillink}[1]{\,\IUCr@cemaillinkX{#1}\,}
