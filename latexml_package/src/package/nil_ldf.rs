use crate::prelude::*;

/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  // Perl: nil.ldf.ltxml — babel's "nil" (null) language.
  // Define \bbl@languages as an empty stub if not already defined; nil.ldf
  // 2020 expects it to exist. Then load the raw nil.ldf.
  if !IsDefined!(&T_CS!("\\bbl@languages")) {
    def_macro_noop("\\bbl@languages")?;
  }
  InputDefinitions!("nil", extension => Some("ldf".into()), noltxml => true);
});
