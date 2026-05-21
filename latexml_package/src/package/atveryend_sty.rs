//! atveryend.sty — hooks at the very end of the document
//! Perl: atveryend.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  def_macro_noop("\\AfterLastShipout{}")?;
  def_macro_noop("\\AtVeryEndDocument{}")?;
  def_macro_noop("\\BeforeClearDocument{}")?;
  def_macro_noop("\\AtEndAfterFileList{}")?;
  def_macro_noop("\\AtVeryVeryEnd{}")?;
});
