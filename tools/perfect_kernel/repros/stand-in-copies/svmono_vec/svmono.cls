% The vector code of svmono.cls v5.x (0910.4696's copy, :81-83 the options, :672-688 the test in the class body,
% :1414 \fig@type set to figure later), around a book class: enough for pdflatex and for the binding, which reads the
% copy for this code.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{svmono}[2009/10/01 vector code of svmono.cls v5.x]
\DeclareOption{vecphys}{\def\vec@style{phys}}
\DeclareOption{vecarrow}{\def\vec@style{arrow}}
\DeclareOption*{\PassOptionsToClass{\CurrentOption}{book}}
\ProcessOptions
\LoadClass{book}
% redefines the \vec accent to a bold character - if desired
\def\fig@type{arrow}% temporarily abused
\ifx\vec@style\fig@type\else
\@ifundefined{vec@style}{%
 \def\vec#1{\ensuremath{\mathchoice
                     {\mbox{\boldmath$\displaystyle\mathbf{#1}$}}
                     {\mbox{\boldmath$\textstyle\mathbf{#1}$}}
                     {\mbox{\boldmath$\scriptstyle\mathbf{#1}$}}
                     {\mbox{\boldmath$\scriptscriptstyle\mathbf{#1}$}}}}%
}
{\def\vec#1{\ensuremath{\mathchoice
                     {\mbox{\boldmath$\displaystyle#1$}}
                     {\mbox{\boldmath$\textstyle#1$}}
                     {\mbox{\boldmath$\scriptstyle#1$}}
                     {\mbox{\boldmath$\scriptscriptstyle#1$}}}}%
}
\fi
\def\fig@type{figure}
