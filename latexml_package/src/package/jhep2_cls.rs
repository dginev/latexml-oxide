use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: JHEP2.cls.ltxml
  // LoadClass("JHEP", withoptions => 1);
  // (this class loads no JHEP.cls: `jhep_cls::jhep_definitions`)
  jhep_cls::jhep_definitions()?;
});
