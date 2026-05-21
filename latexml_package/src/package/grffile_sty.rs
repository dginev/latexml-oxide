//! grffile.sty — extended file name support for graphics
//! Perl: grffile.sty.ltxml
//! LaTeXML can handle filenames with spaces natively.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("graphicx");
  def_macro_noop("\\grffilesetup{}")?;
});
