% A class abstract whose one-argument `\keywords` only warns (the shape of beilstein.cls:854's
% `\newcommand{\keywords}[1]{\ClassWarning{beilstein}{Keywords should not be part of …}}`, fbithesis.cls:272): not a
% setter of the keywords, the class's own definition.
\ProvidesClass{kwwarn}
\LoadClass{article}
\renewenvironment{abstract}{%
  \def\keywords##1{\ClassWarning{kwwarn}{Keywords are not part of this document type}}%
  \quotation}{\endquotation}
