//! grffile.sty — extended file name support for graphics
//! Perl: grffile.sty.ltxml
//! LaTeXML can handle filenames with spaces natively.
use crate::prelude::*;

/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}


#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("graphicx");
  def_macro_noop("\\grffilesetup{}")?;
});
