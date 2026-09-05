use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: lastpage.sty.ltxml
  // Really nothing to do other than try(!) to arrange that lastpage labels
  // the last something?
  at_end_document(TokenizeInternal!(r"\label{lastpage}"))?;
  // lastpagemodern.sty:73 `\gdef\lastpage@lastpage{??}` (the value before the
  // second run) — lastpage-example prints it directly.
  RawTeX!(r"\gdef\lastpage@lastpage{??}\gdef\lastpage@lastpageHy{??}");
});
