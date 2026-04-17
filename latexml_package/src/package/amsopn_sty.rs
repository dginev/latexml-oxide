use crate::prelude::*;
use latexml_core::token::Catcode;

/// Stringify a Tokens arg for `def_math`, preserving space after control words
/// when followed by letters. Plain `Tokens::to_string()` concatenates token
/// texts without separators, so `{\rm Aut}` — whose space after `\rm` was
/// already swallowed by the control-word tokenization — becomes literally
/// `{\rmAut}`, which tokenizes back as the single CS `\rmAut` (undefined).
///
/// Perl avoids this by passing Tokens directly to DefMathI (via
/// `Invocation(T_CS('\operatorname'), $star, $text)`) — no stringify round-
/// trip. Rust's `def_math` takes String, so we insert the missing space at
/// CS→letter boundaries here to preserve tokenizability.
fn tokens_to_tex_safe_string(tokens: &latexml_core::tokens::Tokens) -> String {
  let toks = tokens.unlist_ref();
  let mut out = String::new();
  let mut prev_was_cs_word = false;
  for t in toks {
    if t.code == Catcode::COMMENT { continue; }
    let needs_space = prev_was_cs_word
      && matches!(t.code, Catcode::LETTER | Catcode::OTHER | Catcode::CS);
    if needs_space {
      out.push(' ');
    }
    if t.code == Catcode::ARG { out.push('#'); }
    t.with_str(|s| out.push_str(s));
    // A control word ends in a letter (i.e. the CS name is all letters).
    prev_was_cs_word = t.code == Catcode::CS
      && t.with_str(|s| s.len() > 1
        && s.starts_with('\\')
        && s.chars().skip(1).all(|c| c.is_ascii_alphabetic()));
  }
  out
}

LoadDefinitions!({
  RequirePackage!("amsgen");

  // \DeclareMathOperator*{cs}{text}
  DefPrimitive!("\\DeclareMathOperator OptionalMatch:* {Token} {}", sub[(star, cs, text)] {
    let text_str = tokens_to_tex_safe_string(&text);
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
