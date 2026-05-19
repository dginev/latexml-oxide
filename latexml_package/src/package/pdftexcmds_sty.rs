//! pdftexcmds.sty — pdfTeX utility commands
//! Perl: pdftexcmds.sty.ltxml
//! Everything is in pdfTeX.pool already; just require iftex.
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
  RequirePackage!("iftex");
  // Stubs for the pdftex-primitive wrappers that bmpsize (TL L51-53)
  // and other oberdiek packages probe via `\ifx\csname pdf@filedump
  // \endcsname\relax`. The raw load of pdftexcmds.sty would create
  // these conditionally; we bind instead so define defensively.
  // Witness 2406.02536, 2406.03347 (bmpsize "pdfTeX 1.30 or newer").
  def_macro_noop("\\pdf@filedump{}{}{}")?;
  def_macro_noop("\\pdf@mdfivesum{}")?;
  def_macro_noop("\\pdf@filemdfivesum{}")?;
  DefMacro!("\\pdf@filesize{}", "0");
  def_macro_noop("\\pdf@filemoddate{}")?;
  DefMacro!("\\pdf@strcmp{}{}", "0");
  // \pdf@shellescape returns the shell-escape level (0 = disabled).
  // Used by packages probing whether \write18 is available. We don't
  // execute shell commands; return 0.
  DefMacro!("\\pdf@shellescape", "0");
  // Other common pdftexcmds wrappers — return safe defaults.
  def_macro_noop("\\pdf@unescapehex{}")?;
  DefMacro!("\\pdf@escapestring{}", "#1");
  DefMacro!("\\pdf@escapename{}", "#1");
  DefMacro!("\\pdf@escapehex{}", "#1");
  def_macro_noop("\\pdf@primitive{}")?;
});
