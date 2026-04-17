use crate::prelude::*;
LoadDefinitions!({
  RequirePackage!("amsgen");

  // \DeclareMathOperator*{cs}{text}
  //
  // Use `.untex()` instead of `.to_string()` — the latter concatenates
  // token texts with no separator, so `{\rm Aut}` (where the space after
  // `\rm` was swallowed by control-word tokenization) becomes `{\rmAut}`,
  // which tokenizes back as the single undefined CS `\rmAut`. `untex()`
  // inserts a space at CS→letter boundaries (tokens.rs L392-405), so the
  // round-trip through `def_math`'s internal `mouth::tokenize_internal`
  // preserves the correct token structure.
  //
  // Perl avoids this entirely by passing Tokens directly to DefMathI via
  // `Invocation(T_CS('\operatorname'), $star, $text)` — no stringify
  // round-trip. Rust's `def_math` takes String, so we use the TeX-safe
  // stringifier.
  //
  // Fixes sandbox papers 0806.2705 (`\rmTr`) and 0808.0535 (`\rmAut`/
  // `\rmSpan`) whose `\DeclareMathOperator{\X}{{\rm X}}` patterns would
  // otherwise produce undefined `\rmX` errors.
  DefPrimitive!("\\DeclareMathOperator OptionalMatch:* {Token} {}", sub[(star, cs, text)] {
    let text_str = text.untex();
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

  // Perl: amsopn.sty.ltxml — var limit operators
  DefMath!("\\varlimsup", "\\overline{\\operatorname{lim}}",
    role => "LIMITOP", meaning => "limit-supremum");
  DefMath!("\\varliminf", "\\underline{\\operatorname{lim}}",
    role => "LIMITOP", meaning => "limit-infimum");
  DefMath!("\\varinjlim", "\\underrightarrow{\\operatorname{lim}}",
    role => "LIMITOP", meaning => "injective-limit");
  DefMath!("\\varprojlim", "\\underleftarrow{\\operatorname{lim}}",
    role => "LIMITOP", meaning => "projective-limit");

  DefMacro!("\\nolimits@", "\\nolimits");
  DefMacro!("\\nmlimits@", "\\displaylimits");
  DefMacro!("\\qopname{}{}{}", "\\mathop{#3}\\csname n#2limits@\\endcsname");
});
