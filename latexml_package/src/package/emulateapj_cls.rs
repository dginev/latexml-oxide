use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: emulateapj.cls.ltxml — Seems to be equivalent to aastex.
  // LoadClass('aastex', withoptions => 1);
  load_class("aastex", Vec::new(), Tokens!())?;
  RequireResource!("ltx-apj.css");
  RequirePackage!("emulateapj");
});
