//! ifthen.sty — conditional commands (raw TeX passthrough)
//! Perl: InputDefinitions('ifthen', type => 'sty', noltxml => 1)
use std::borrow::Cow;

use crate::prelude::*;

LoadDefinitions!({
  InputDefinitions!("ifthen", noltxml => true, reloadable => true, extension => Some(Cow::Borrowed("sty")));
});
