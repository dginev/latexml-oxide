use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: gen-j-l.cls.ltxml — Generic AMS Journal
  // LoadClass('amsart', withoptions => 1);
  load_class("amsart", Vec::new(), Tokens!())?;
});
