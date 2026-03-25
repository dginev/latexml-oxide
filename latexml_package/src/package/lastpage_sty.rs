use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: lastpage.sty.ltxml
  // Really nothing to do other than try(!) to arrange that lastpage labels
  // the last something?
  RawTeX!("\\AtEndDocument{\\label{lastpage}}");
});
