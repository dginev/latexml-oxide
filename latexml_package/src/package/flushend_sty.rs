//! flushend.sty — flush/ragged column balancing (no-op in LaTeXML)
//! Perl: flushend.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Nothing to do, really.
  def_macro_noop("\\flushend")?;
  def_macro_noop("\\flushcolsend")?;
  def_macro_noop("\\raggedend")?;
  def_macro_noop("\\raggedcolsend")?;
  def_macro_noop("\\atColsEnd{}")?;
  def_macro_noop("\\atColsBreak{}")?;
  def_macro_noop("\\showcolsendrule")?;
});
