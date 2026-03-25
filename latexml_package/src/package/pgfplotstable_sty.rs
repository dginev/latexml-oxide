use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: pgfplotstable.sty.ltxml
  InputDefinitions!("pgfplotstable", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
