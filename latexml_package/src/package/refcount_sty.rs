//! refcount.sty — reference counting (raw TeX passthrough)
//! Perl: InputDefinitions('refcount', type => 'sty', noltxml => 1)
use std::borrow::Cow;

use crate::prelude::*;

LoadDefinitions!({
  InputDefinitions!("refcount", noltxml => true, reloadable => true, extension => Some(Cow::Borrowed("sty")));
});
