//! Stub for doclicense.sty (Creative Commons license metadata).
use latexml_package::prelude::*;

LoadDefinitions!({
  // License metadata — frontmatter-only, no rendered XML.
  def_macro_noop("\\doclicenseURL")?;
  def_macro_noop("\\doclicenseName")?;
  def_macro_noop("\\doclicenseLongName")?;
  def_macro_noop("\\doclicenseLongType")?;
  def_macro_noop("\\doclicenseNameRef")?;
  def_macro_noop("\\doclicenseLongNameRef")?;
  def_macro_noop("\\doclicenseText")?;
  def_macro_noop("\\doclicenseLongText")?;
  def_macro_noop("\\doclicenseImage[]")?;
  def_macro_noop("\\doclicenseLogo")?;
  // doclicense.sty:222 `\doclicenseThis`: the centred minipage layout of the
  // (already no-op) sub-macros (beautynote).
  def_macro_noop("\\doclicenseThis")?;
});
