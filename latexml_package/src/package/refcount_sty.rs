//! refcount.sty — reference counting (raw TeX passthrough)
//! Perl: InputDefinitions('refcount', type => 'sty', noltxml => 1)
use crate::prelude::*;
use std::borrow::Cow;

LoadDefinitions!({
  InputDefinitions!("refcount", noltxml => true, reloadable => true, extension => Some(Cow::Borrowed("sty")));
});
