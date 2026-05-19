use crate::prelude::*;
// placeins.sty — float barriers (no-op since floats stay where found)

/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  DeclareOption!("section", None);
  DeclareOption!("above", None);
  DeclareOption!("below", None);
  DeclareOption!("verbose", None);
  // Perl placeins.sty.ltxml L21: `ProcessOptions()` consumes the
  // declared no-ops so user-side `\usepackage[section]{placeins}`
  // doesn't leave unprocessed options behind. Rust was missing this.
  ProcessOptions!();
  def_macro_noop("\\FloatBarrier")?;
});
