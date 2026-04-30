use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: revsymb.sty.ltxml — REVTeX symbol definitions

  DefMath!("\\lambdabar", "\u{03BB}\u{0304}");
  DefConstructor!("\\mathbb{}", "#1", bounded => true, require_math => true,
    font => {family => "blackboard", series => "medium", shape => "upright"});
  DefMacro!("\\Bbb{}", "\\mathbb{#1}");

  // Bold delimiter constructors — stubbed as simple pass-through.
  // Perl: DefConstructor('\biglb TeXDelimiter', '#1', ...) uses the
  // TeXDelimiter parameter type that Rust doesn't have yet (see WISDOM #41).
  // All 8 \biglb/\bigrb/\Biglb/\Bigrb/\bigglb/\biggrb/\Bigglb/\Biggrb
  // entries are DefConstructor↔DefMacro DP mismatches from this root cause;
  // porting TeXDelimiter as a ParameterType would collapse them back to
  // audit-clean shape. Current forwarding to \mathopen/\mathclose + \big/\Big
  // preserves the visible delimiter rendering.
  //
  // Intentional divergence (WISDOM #44 class: blocked-on-parameter-type):
  // these 8 DefConstructor → DefMacro flips are a single-root-cause cluster
  // — porting TeXDelimiter closes all 8 at once. Meanwhile the \big/\Big
  // expansion keeps delimiter sizing authentic even if emission is not
  // <ltx:XMWrap open=.../>. Audit counts 8 flips; all 8 share this root.
  DefMacro!("\\biglb", "\\mathopen\\big");
  DefMacro!("\\bigrb", "\\mathclose\\big");
  DefMacro!("\\Biglb", "\\mathopen\\Big");
  DefMacro!("\\Bigrb", "\\mathclose\\Big");
  DefMacro!("\\bigglb", "\\mathopen\\bigg");
  DefMacro!("\\biggrb", "\\mathclose\\bigg");
  DefMacro!("\\Bigglb", "\\mathopen\\Bigg");
  DefMacro!("\\Biggrb", "\\mathclose\\Bigg");

  DefMath!("\\gtrsim", "\u{2273}", role => "RELOP", meaning => "greater-than-or-equivalent-to");
  DefMath!("\\lesssim", "\u{2272}", role => "RELOP", meaning => "less-than-or-similar-to");
  Let!("\\agt", "\\gtrsim");
  Let!("\\alt", "\\lesssim");

  DefMath!("\\precsim", "\u{227E}", role => "RELOP", meaning => "precedes-or-equivalent-to");
  DefMath!("\\succsim", "\u{227F}", role => "RELOP", meaning => "succeeds-or-equivalent-to");
  Let!("\\altprecsim", "\\precsim");
  Let!("\\altsuccsim", "\\succsim");

  DefMath!("\\overcirc{}", "\u{030A}", operator_role => "OVERACCENT");
  DefMath!("\\dddot{}", "\u{02D9}\u{02D9}\u{02D9}", operator_role => "OVERACCENT");
  Let!("\\overdots", "\\dddot");
  DefMath!("\\triangleq", "\u{225C}", role => "RELOP");
  Let!("\\corresponds", "\\triangleq");

  DefMath!("\\loarrow{}", "\u{20D6}", operator_role => "OVERACCENT");
  DefMath!("\\roarrow{}", "\u{20D7}", operator_role => "OVERACCENT");
  DefConstructor!("\\openone", "1",
    font => {family => "blackboard", series => "medium", shape => "upright"});
  DefMath!("\\overstar{}", "\u{0359}", operator_role => "OVERACCENT");
  DefMath!("\\tensor{}", "\u{20E1}", operator_role => "OVERACCENT");
});
