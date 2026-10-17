\NeedsTeXFormat{LaTeX2e}
\def\ClassName@CiM{cimart}
\ProvidesClass{\ClassName@CiM}
\LoadClass{article}
\RequirePackage{amsmath, amssymb, amsthm}
\newtheoremstyle{cmdefinition}{1em}{1ex}{}{}{\bfseries\sffamily}{.}{ }{}
\theoremstyle{cmdefinition}
\newtheorem{definition}{Definition}[section]
\newtheoremstyle{cmplain}{1em}{1ex}{\itshape}{}{\bfseries\sffamily}{.}{ }{}
\theoremstyle{cmplain}
\newtheorem{theorem}[definition]{Theorem}
