use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: minimal.cls.ltxml — loads article class
  load_class("article", Vec::new(), Tokens!())?;
});
