use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: proof.sty.ltxml
  // The premises can be separated by "&" (which is NOT being used for alignment!)
  DefMath!("\\lx@proof@logical@and", "\u{2003}", role => "ADDOP", meaning => "and");

  // \lx@proof@split@and{}: complex sub{} body — splits tokens on T_ALIGN
  // and wraps multiple clauses with I_apply(\lx@proof@logical@and, ...)
  // Stub: just pass through the argument unchanged
  DefMacro!("\\lx@proof@split@and{}", "#1");

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
