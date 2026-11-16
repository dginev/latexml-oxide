use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: amsaddr.sty.ltxml
  RequirePackage!("ams_support");
  // amsaddr.sty:43 `\let\emails\@empty`, the e-mail list its `\email` fills (:44-50); here `\email` is the frontmatter's
  // contact, and a document's own `\email` adds to the list (2606.23853, 2606.25954, 2212.11488, 1801.06576, 2307.06418:
  // `\g@addto@macro\emails`; OD #1036). Its `\@maketitle` prints the list as a footnote (:113, `\@setemails`
  // :102-108, "E-mail address" for one `\author`, counted in `g@author`, :25-28): a note on the document here.
  RawTeX!(
    r"\let\emails\@empty
\newcounter{g@author}
\let\lx@amsaddr@addto@authors\lx@ams@addto@authors
\def\lx@ams@addto@authors{\stepcounter{g@author}\lx@amsaddr@addto@authors}
\def\@setemails{%
  \ifnum\theg@author > 1
    \mbox{{\itshape E-mail addresses}:\space}{\ttfamily\emails}.
  \else
    \mbox{{\itshape E-mail address}:\space}{\ttfamily\emails}.
  \fi}"
  );
  AddToMacro!("\\lx@maketitle@body", r"\ifx\@empty\emails\else\lx@frontmatter@footnotes{\@footnotetext{\@setemails}}\fi");
});
