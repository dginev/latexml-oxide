use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: JHEP2.cls.ltxml
  // LoadClass("JHEP", withoptions => 1);
  LoadClass!("JHEP", withoptions => true);
});
