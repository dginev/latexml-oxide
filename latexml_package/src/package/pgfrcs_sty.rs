use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: pgfrcs.sty.ltxml
  InputDefinitions!("pgfutil-common", extension => Some(Cow::Borrowed("tex")));
  InputDefinitions!("pgfutil-latex",  extension => Some(Cow::Borrowed("def")));
  InputDefinitions!("pgfrcs.code",    extension => Some(Cow::Borrowed("tex")));
});
