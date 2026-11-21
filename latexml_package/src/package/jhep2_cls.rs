use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: JHEP2.cls.ltxml
  // LoadClass("JHEP", withoptions => 1);
  // JHEP3.cls, JHEP2.cls and PoS.cls are classes of their own, loading no JHEP.cls (0907.1636's JHEP3.cls, 1011.0095's
  // PoS.cls): the JHEP binding's definitions are made in this class's load, its options this class's
  // (KNOWN_PERL_ERRORS #1300).
  jhep_cls::load_definitions()?;
});
