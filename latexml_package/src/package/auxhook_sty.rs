use crate::prelude::*;

/// DEP-20 helper for empty-body `DefPrimitive!("\\cs[opt-spec]", None);` stubs.
/// Mirrors `def_macro_noop` but routes through `def_primitive` so the CS
/// is registered as a digestion-time primitive rather than an expandable
/// macro. Body=None is treated as a no-op primitive (no Box emitted).
fn def_primitive_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  def_primitive(cs_tok, params, None, PrimitiveOptions::default())?;
  Ok(())
}


#[rustfmt::skip]
LoadDefinitions!({
  // Perl: auxhook.sty.ltxml
  def_primitive_noop("\\AddLineBeginAux{}")?;
  def_primitive_noop("\\AddLineBeginMainAux{}")?;
  def_primitive_noop("\\AddLineBeginPartAux{}")?;
});
