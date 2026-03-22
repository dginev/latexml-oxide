//! balance.sty — two-column balancing (no-op in LaTeXML)
//! Perl: balance.sty.ltxml
use crate::prelude::*;

LoadDefinitions!({
  DefMacro!("\\balance", None);
  DefMacro!("\\nobalance", None);
});
