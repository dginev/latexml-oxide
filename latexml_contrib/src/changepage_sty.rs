use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  DeclareOption!("strict", "");
  DefConditional!("\\ifoddpage");
  DefConditional!("\\ifstrictpagecheck");
  DefConditional!("\\ifcpstrict");
  DefConditional!("\\ifcpoddpage");
  def_macro_noop("\\strictpagecheck")?;
  def_macro_noop("\\easypagecheck")?;
  def_macro_noop("\\pmemlabel{}")?;
  def_macro_noop("\\newpmemlabel{}{}")?;
  def_macro_noop("\\pmemlabelref{}")?;
  def_macro_noop("\\checkoddpage")?;
  def_macro_noop("\\cplabelprefix")?;
  def_macro_noop("\\cplabel{}")?;
  def_macro_noop("\\newcplabel{}{}")?;
  // Mode must be internal_vertical so `$$`-display-math inside the env
  // is recognised by the dollar handler (tex_math.rs's
  // `BOUND_MODE.ends_with("vertical")` gate). Without this, witness
  // 2305.09826: `$$\log_2(x)$$` inside `\begin{adjustwidth}{...}{...}`
  // emitted Error:unexpected:_ "Script _ can only appear in math mode"
  // for every `_` in the formula because the wrapping env's default
  // restricted_horizontal mode kept the `$` handler from entering
  // display math. See sibling fix in
  // latexml_package/src/package/changepage_sty.rs.
  DefEnvironment!("{adjustwidth} OptionalMatch:* []{}{}", "#body",
    mode => "internal_vertical");
  def_macro_noop("\\changetext{}{}{}{}{}")?;
  def_macro_noop("\\changepage{}{}{}{}{}{}{}{}{}")?;
});
