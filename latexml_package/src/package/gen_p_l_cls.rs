use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: gen-p-l.cls.ltxml — Generic AMS Proceedings
  // LoadClass('amsprocs', withoptions => 1);
  load_class_with_options("amsprocs", Tokens!())?;
});
