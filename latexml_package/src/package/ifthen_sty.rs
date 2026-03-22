//! ifthen.sty — conditional commands (raw TeX passthrough)
//! Perl: InputDefinitions('ifthen', type => 'sty', noltxml => 1)
use crate::prelude::*;
use std::borrow::Cow;

LoadDefinitions!({
  InputDefinitions!("ifthen", noltxml => true, reloadable => true, extension => Some(Cow::Borrowed("sty")));
});
