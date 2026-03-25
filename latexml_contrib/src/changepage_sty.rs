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
  DefEnvironment!("{adjustwidth} OptionalMatch:* []{}{}", "#body");
  DefMacro!("\\changetext{}{}{}{}{}", "");
  DefMacro!("\\changepage{}{}{}{}{}{}{}{}{}", "");
});
