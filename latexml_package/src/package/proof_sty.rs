use crate::engine::base_utilities::split_tokens;
use crate::prelude::*;
use crate::xmath_helpers::i_apply;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: proof.sty.ltxml
  // The premises can be separated by "&" (which is NOT being used for alignment!)
  DefMath!("\\lx@proof@logical@and", "\u{2003}", role => "ADDOP", meaning => "and");

  // Perl proof.sty.ltxml L20-28: split the argument on T_ALIGN (`&`),
  // which in proof.sty separates premises (not for column alignment).
  // Single clause → pass through; multiple → wrap with I_apply around
  // \lx@proof@logical@and so the semantic structure is an n-ary AND.
  DefMacro!("\\lx@proof@split@and {}", sub[(tokens)] {
    let clauses = split_tokens(tokens, vec![T_ALIGN!()]);
    if clauses.is_empty() {
      Ok(Tokens!())
    } else if clauses.len() == 1 {
      Ok(clauses.into_iter().next().unwrap())
    } else {
      Ok(i_apply(&[], Tokens!(T_CS!("\\lx@proof@logical@and")), clauses))
    }
  });

  // \lx@proof@stack{}{}{}{}:
  // An extremely contrived stack of premises and conclusion.
  // Args are $top, $middle (if any), $bottom, $sidelabel (if any)
  DefConstructor!("\\lx@proof@stack{}{}{}{}",
    "<ltx:XMArray vattach='bottom'><ltx:XMRow><ltx:XMCell>#1</ltx:XMCell>?#4(<ltx:XMCell rowspan='3'>#4</ltx:XMCell>)()</ltx:XMRow>?#2(<ltx:XMRow><ltx:XMCell>#2</ltx:XMCell></ltx:XMRow>)()<ltx:XMRow><ltx:XMCell>#3</ltx:XMCell></ltx:XMRow></ltx:XMArray>");

  // Put 1 or 2 bars over the conclusion (possibly stretched)
  DefConstructor!("\\lx@proof@bars OptionalMatch:= {}",
    "<ltx:XMApp><ltx:XMTok role='OVERACCENT'>\u{203E}</ltx:XMTok>?#1(<ltx:XMApp><ltx:XMTok role='OVERACCENT'>\u{203E}</ltx:XMTok>)()<ltx:XMWrap>#2</ltx:XMWrap>?#1(</ltx:XMApp>)()</ltx:XMApp>");

  // \infer: complex sub{} body — uses I_dual, I_apply, I_symbol, I_arg, Invocation
  // Stub: render as \ensuremath with a simple stack layout
  DefMacro!("\\infer OptionalMatch:* OptionalMatch:= [] {}{}",
    "\\ensuremath{\\lx@proof@stack{\\lx@proof@split@and{#5}}{}{\\lx@proof@bars #2{#4}}{#3}}");

  // \deduce: complex sub{} body — similar to \infer but no bars
  // Stub: render as \ensuremath with a simple stack layout
  DefMacro!("\\deduce [] {}{}",
    "\\ensuremath{\\lx@proof@stack{\\lx@proof@split@and{#3}}{#1}{#2}{}}");
});
