use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: pgfmath.sty.ltxml
  RequirePackage!("pgfrcs");
  RequirePackage!("pgfkeys");
  InputDefinitions!("pgfmath.code", extension => Some(Cow::Borrowed("tex")));
});
