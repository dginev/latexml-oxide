use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: makeidx.sty.ltxml
  DefPrimitive!("\\makeindex", None, locked => true);
  DefMacro!("\\see{}{}", "\\emph{\\seename} #1");
  DefMacro!("\\seealso{}{}", "\\emph{\\alsoname} #1");
  DefMacro!("\\printindex", "\\begin{theindex}\\end{theindex}", locked => true);
  DefMacro!("\\seename", "see");
  DefMacro!("\\alsoname", "see also");
});
