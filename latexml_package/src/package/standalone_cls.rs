use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: standalone.cls.ltxml
  InputDefinitions!("standalone", noltxml => true,
    extension => Some(Cow::Borrowed("cls")));
});
