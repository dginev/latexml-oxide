use crate::engine::base_utilities::split_tokens;
use crate::prelude::*;
use crate::xmath_helpers::{i_apply, i_arg, i_dual, i_symbol};

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

  // Perl proof.sty.ltxml L60-71: \infer (* or =) [label] {lower}{uppers}
  // builds an XMDual — content branch records
  //   I_apply(I_symbol(meaning => "infer"|"multistep-infer"), lower, uppers-split)
  // presentation branch wraps \lx@proof@stack around the lower + bars + label.
  // Args shared between branches (in order): lower, split-and(uppers), label.
  DefMacro!("\\infer OptionalMatch:* OptionalMatch:= [] {}{}",
    sub[(multistep, double, label, lower, uppers)] {
      let meaning = if multistep.is_some() { "multistep-infer" } else { "infer" };
      let content = i_apply(
        &[],
        i_symbol(&[("meaning", Tokenize!(meaning))], None),
        vec![Tokens!(i_arg("1")), Tokens!(i_arg("2"))],
      );
      let presentation = if multistep.is_some() {
        // Invocation(\lx@proof@stack, I_arg(2), \vdots, I_arg(1), I_arg(3))
        Invocation!(T_CS!("\\lx@proof@stack"), vec![
          Tokens!(i_arg("2")),
          Tokens!(T_CS!("\\vdots")),
          Tokens!(i_arg("1")),
          Tokens!(i_arg("3")),
        ])
      } else {
        // Invocation(\lx@proof@bars, double, I_arg(1))
        let bars = Invocation!(T_CS!("\\lx@proof@bars"), vec![
          double.clone().unwrap_or_default(),
          Tokens!(i_arg("1")),
        ]);
        // Invocation(\lx@proof@stack, I_arg(2), undef, bars, I_arg(3))
        Invocation!(T_CS!("\\lx@proof@stack"), vec![
          Tokens!(i_arg("2")),
          Tokens!(),
          bars,
          Tokens!(i_arg("3")),
        ])
      };
      let uppers_split =
        Invocation!(T_CS!("\\lx@proof@split@and"), vec![uppers.clone()]);
      let cmd = i_dual(&[], content, presentation, vec![
        lower.clone(),
        uppers_split,
        label.clone().unwrap_or_default(),
      ])?;
      let mut out: Vec<Token> = Vec::with_capacity(3 + cmd.len());
      out.push(T_CS!("\\ensuremath"));
      out.push(T_BEGIN!());
      out.extend(cmd.unlist());
      out.push(T_END!());
      Ok(Tokens::new(out))
    }
  );

  // Perl proof.sty.ltxml L74-81: \deduce [label] {lower}{uppers}
  // Similar to \infer but no bars; label (if any) replaces the bars row.
  DefMacro!("\\deduce [] {}{}", sub[(label, lower, uppers)] {
    let content = i_apply(
      &[],
      i_symbol(&[("meaning", Tokenize!("deduce"))], None),
      vec![Tokens!(i_arg("1")), Tokens!(i_arg("2"))],
    );
    // Invocation(\lx@proof@stack, I_arg(2), label, I_arg(1))
    // Perl passes only 3 args to \lx@proof@stack{}{}{}{}; 4th (sidelabel)
    // defaults to empty.
    let presentation = Invocation!(T_CS!("\\lx@proof@stack"), vec![
      Tokens!(i_arg("2")),
      label.clone().unwrap_or_default(),
      Tokens!(i_arg("1")),
      Tokens!(),
    ]);
    let uppers_split =
      Invocation!(T_CS!("\\lx@proof@split@and"), vec![uppers.clone()]);
    let cmd = i_dual(&[], content, presentation, vec![
      lower.clone(),
      uppers_split,
    ])?;
    let mut out: Vec<Token> = Vec::with_capacity(3 + cmd.len());
    out.push(T_CS!("\\ensuremath"));
    out.push(T_BEGIN!());
    out.extend(cmd.unlist());
    out.push(T_END!());
    Ok(Tokens::new(out))
  });
});
