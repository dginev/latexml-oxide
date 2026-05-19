use latexml_package::prelude::*;

LoadDefinitions!({
  DeclareOption!("strict", "");
  DefConditional!("\\ifoddpage");
  DefConditional!("\\ifstrictpagecheck");
  DefConditional!("\\ifcpstrict");
  DefConditional!("\\ifcpoddpage");
  DefMacro!("\\strictpagecheck", "");
  DefMacro!("\\easypagecheck", "");
  DefMacro!("\\pmemlabel{}", "");
  DefMacro!("\\newpmemlabel{}{}", "");
  DefMacro!("\\pmemlabelref{}", "");
  DefMacro!("\\checkoddpage", "");
  DefMacro!("\\cplabelprefix", "");
  DefMacro!("\\cplabel{}", "");
  DefMacro!("\\newcplabel{}{}", "");
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
  DefMacro!("\\changetext{}{}{}{}{}", "");
  DefMacro!("\\changepage{}{}{}{}{}{}{}{}{}", "");
});
