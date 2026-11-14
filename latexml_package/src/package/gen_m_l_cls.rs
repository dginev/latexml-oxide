use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: gen-m-l.cls.ltxml — Generic AMS Monograph
  // LoadClass('amsbook', withoptions => 1);
  LoadClass!("amsbook", withoptions => true);
});
