//! atveryend.sty — hooks at the very end of the document
//! Perl: atveryend.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\AfterLastShipout{}",    "");
  DefMacro!("\\AtVeryEndDocument{}",   "");
  DefMacro!("\\BeforeClearDocument{}", "");
  DefMacro!("\\AtEndAfterFileList{}",  "");
  DefMacro!("\\AtVeryVeryEnd{}",       "");
});
