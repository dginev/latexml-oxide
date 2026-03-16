use crate::prelude::*;
LoadDefinitions!({
  RequirePackage!("amsgen");

  // \DeclareMathOperator*{cs}{text}
  DefPrimitive!("\\DeclareMathOperator OptionalMatch:* {Token} {}", sub[(star, cs, text)] {
    let text_str = text.to_string();
    let has_star = star.is_some();
    let opts = MathPrimitiveOptions {
      role: Some(if has_star { "OPERATOR" } else { "OPFUNCTION" }.to_string()),
      font: Some(fontmap!(family => "serif", series => "medium", shape => "upright").into()),
      ..Default::default()};
    def_math(cs, None, text_str, opts)?;
  });

  // \operatorname*{text}
  DefConstructor!("\\operatorname OptionalMatch:* {}",
    "<ltx:XMWrap role='#role' scriptpos='#scriptpos'>#2</ltx:XMWrap>",
    bounded => true, require_math => true,
    font => { family => "serif", series => "medium", shape => "upright" },
    properties => sub[args] {
      let starred = args[0].is_some();
      let role = if starred { "OPERATOR" } else { "OPFUNCTION" };
      let scriptpos = if starred { "mid" } else { "post" };
      Ok(stored_map!("role" => role, "scriptpos" => scriptpos))
    });

  DefConstructor!("\\operatornamewithlimits {}",
    "<ltx:XMWrap role='OPERATOR' scriptpos='mid'>#1</ltx:XMWrap>",
    bounded => true, require_math => true,
    font => { family => "serif", series => "medium", shape => "upright" });

  // Operator variants
  DefMath!("\\injlim", "inj lim",
    role => "LIMITOP", meaning => "injective-limit");
  DefMath!("\\projlim", "proj lim",
    role => "LIMITOP", meaning => "projective-limit");

  DefMacro!("\\nolimits@", "\\nolimits");
  DefMacro!("\\nmlimits@", "\\displaylimits");
  DefMacro!("\\qopname{}{}{}", "\\mathop{#3}\\csname n#2limits@\\endcsname");
});
