//! xparse.sty — document command parser interface
//! Perl: xparse.sty.ltxml — loads the raw sty with noltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("xparse", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
