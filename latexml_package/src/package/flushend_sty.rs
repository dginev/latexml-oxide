//! flushend.sty — flush/ragged column balancing (no-op in LaTeXML)
//! Perl: flushend.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Nothing to do, really.
  DefMacro!("\\flushend",        "");
  DefMacro!("\\flushcolsend",    "");
  DefMacro!("\\raggedend",       "");
  DefMacro!("\\raggedcolsend",   "");
  DefMacro!("\\atColsEnd{}",     "");
  DefMacro!("\\atColsBreak{}",   "");
  DefMacro!("\\showcolsendrule", "");
});
