use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("book");
  RequirePackage!("geometry");
  RequirePackage!("setspace");
  RawTeX!(
    r"\newif\ifmargins@equal
\newlength{\margin@base}
\newlength{\margin@offset}
\setlength{\margin@offset}{0.5in}
\newcommand*{\degree}    [1]{\gdef\@degree{#1}}
\newcommand*{\gradyear}  [1]{\gdef\@gradyear{#1}}
\newcommand*{\department}[2][Department of]%
  {\gdef\@deptpre{#1}\gdef\@deptname{#2}}
\author    {(author)}
\title     {(title)}
\degree    {(degree)}
\gradyear  {(gradyear)}
\department{(department)}
\newenvironment*{dedication}%
  {\thispagestyle{plain}
   \null
   \vfill
   \begin{flushright}}%
  {\end{flushright}
   \vfill
   \vfill
   \clearpage}
\newenvironment*{acknowledgements}%
  {\thispagestyle{plain}
   \begin{center}
   \section*{Acknowledgements}
   \end{center}
   \begingroup}%
  {\endgroup\clearpage}
\let\ocleardoublepage\cleardoublepage
\def\cleardoublepage{{\newpage\pagestyle{plain}\ocleardoublepage}}
\newcommand{\headerstyle}[1]{\footnotesize\MakeUppercase{#1}}
\setlength\abovecaptionskip{1ex}
\setlength\belowcaptionskip{1ex}"
  );
});
