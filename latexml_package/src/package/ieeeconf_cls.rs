use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ieeeconf.cls.ltxml
  // LoadClass('IEEEtran');
  load_class("IEEEtran", Vec::new(), Tokens!())?;
});
