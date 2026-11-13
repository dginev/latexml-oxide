use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: gen-j-l.cls.ltxml — Generic AMS Journal
  // LoadClass('amsart', withoptions => 1);
  LoadClass!("amsart", withoptions => true);
});
