% A class `\maketitle` whose `\csname`-made and `\expandafter`-chained definitions store replacement texts that read
% `\@thefnmark` (latex.ltx defines it only for a footnote): TeX makes the name, or expands the one token after the
% brace, and stores the rest (75a review r2 finding 2).
\ProvidesClass{tpstored}
\LoadClass{article}
\def\@venue{}\newcommand\venue[1]{\def\@venue{#1}}
\renewcommand{\maketitle}{\par
  \expandafter\def\csname tp@mark\endcsname{\textsuperscript{\@thefnmark}}%
  \expandafter\def\expandafter\tp@x\expandafter{\@venue\@thefnmark}%
  \noindent Venue: \@venue\par}
