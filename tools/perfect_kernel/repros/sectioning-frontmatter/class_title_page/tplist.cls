% The shape of vmsta.cls:84-93, :296-315, :534-545 (1506.02940): the class's `\author` appends each name to its own
% list, and its `\maketitle` prints that list and the dates it stores.
\ProvidesClass{tplist}
\LoadClass{article}
\def\tp@authors{}\let\tp@sep\@empty
\renewcommand{\author}[1]{\g@addto@macro\tp@authors{\tp@sep#1\def\tp@sep{, }}}
\let\@received\@empty
\def\received#1{\gdef\@received{{Received:~\hbox{#1}}}}
\renewcommand{\maketitle}{\par
  {\centering\Large\@title\par}%
  {\centering\bfseries\tp@authors\par}%
  {\centering\small\@received\par}}
