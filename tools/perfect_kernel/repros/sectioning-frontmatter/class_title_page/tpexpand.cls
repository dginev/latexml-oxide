% A class `\maketitle` that copies a store with `\expandafter\def\expandafter\tp@x\expandafter{\@venue}`: `\tp@x` is
% what it defines, and `\@venue` runs (one expansion) as the definition does (75a review nit 7).
\ProvidesClass{tpexpand}
\LoadClass{article}
\def\@venue{}\newcommand\venue[1]{\def\@venue{#1}}
\renewcommand{\maketitle}{\par
  \expandafter\def\expandafter\tp@x\expandafter{\@venue}%
  \noindent Venue: \tp@x\par}
