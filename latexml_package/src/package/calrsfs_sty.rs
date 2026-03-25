use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: calrsfs.sty.ltxml — defines \mathcal to use \mathscr
  Let!("\\mathrsfs", "\\mathscr");
  Let!("\\mathcal",  "\\mathscr");
});
