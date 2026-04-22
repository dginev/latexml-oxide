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
    // Perl L26-29: scriptpos => ($star ? \&doScriptpos : 'post') — starred form
    // gets dynamic mid/post from current display style; bare form is always 'post'.
    // revert_as => 'context' so source-export emits the user-facing CS name
    // rather than the operatorname expansion. Both were previously dropped.
    let opts = MathPrimitiveOptions {
      role: Some(if has_star { "OPERATOR" } else { "OPFUNCTION" }.to_string()),
      font: Some(fontmap!(family => "serif", series => "medium", shape => "upright").into()),
      scriptpos: if has_star { None } else { Some("post".to_string()) },
      dynamic_scriptpos: has_star,
      revert_as: Some(std::borrow::Cow::Borrowed("context")),
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

  // Operator variants — Perl L33-38 ships scriptpos => \&doScriptpos so the
  // operators sit mid (under/over) in display style and post (sub/super) in
  // inline. Without it, Rust statically rendered everything as 'post', giving
  // wrong placement in display-mode formulas.
  DefMath!("\\injlim", "inj lim",
    role => "LIMITOP", meaning => "injective-limit", dynamic_scriptpos => true);
  DefMath!("\\projlim", "proj lim",
    role => "LIMITOP", meaning => "projective-limit", dynamic_scriptpos => true);

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
  DefMacro!(
    "\\qopname{}{}{}",
    "\\mathop{#3}\\csname n#2limits@\\endcsname"
  );
});
