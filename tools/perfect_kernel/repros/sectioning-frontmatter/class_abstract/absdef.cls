% A class's `{abstract}` begin code that defines what its body prints, local to the environment's group.
\ProvidesClass{absdef}
\LoadClass{article}
\renewenvironment{abstract}{\def\abstracthead{X}\par}{\par}
