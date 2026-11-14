% The shape of nowfnt.cls:552-600 (1912.00312): a helper whose parameter text is delimited by `\@nil`, called by the
% class's own title-page code, and defining the macro the code reads next.
\ProvidesClass{tpdelim}
\LoadClass{article}
\def\tp@split#1,#2\@nil{\def\tp@first{#1}\def\tp@rest{#2}}
\newcommand{\series}[1]{\gdef\@series{#1}}
\def\@series{}
\renewcommand{\maketitle}{\par
  \expandafter\tp@split\@series,\@nil
  \noindent Series: \tp@first\par}
