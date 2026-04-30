use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: gen-m-l.cls.ltxml — Generic AMS Monograph
  // LoadClass('amsbook', withoptions => 1);
  load_class_with_options("amsbook", Tokens!())?;
});
