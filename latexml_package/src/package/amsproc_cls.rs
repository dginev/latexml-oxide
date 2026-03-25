use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: amsproc.cls.ltxml — AMS proceedings article class
  // LoadClass('ams_core', withoptions => 1);
  load_class("ams_core", Vec::new(), Tokens!())?;
});
