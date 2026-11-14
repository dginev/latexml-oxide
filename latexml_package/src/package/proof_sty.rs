use crate::{
  engine::base_utilities::split_tokens,
  prelude::*,
  xmath_helpers::{i_apply, i_arg, i_dual, i_symbol},
};

/// proof.sty:160-166 `\@infer` sets the label and the conclusion as `\hbox{$#1$}`/`\hbox{$#2$}` and each premise in
/// an `\halign` cell `$##$`: math in a box of its own, where an author's `$` is text (`\infer[$MP$]{A}{B}`, the
/// rc_struct3 E1 sample; KNOWN_PERL_ERRORS #907). Each part is digested in `\lx@math@in@hbox@group` (tex_math.rs),
/// scoped to the part; an empty part stays empty. Guard `perfect_kernel_batch69c::dollar_in_proof_and_accents_hbox_math_is_text`.
fn in_hbox_math(tokens: Tokens) -> Tokens {
  if tokens.is_empty() {
    tokens
  } else {
    let mut out = vec![T_CS!("\\lx@math@in@hbox@group"), T_BEGIN!()];
    out.extend(tokens.unlist());
    out.push(T_END!());
    Tokens::new(out)
  }
}

/// One row of proof.sty's premises split at its `&`s, catcode 4 or active (`\lx@proof@split@and`): one premise as
/// itself, several as the n-ary AND of `\lx@proof@logical@and`.
fn split_premises(tokens: Tokens) -> Tokens {
  let clauses: Vec<Tokens> = split_tokens(tokens, vec![T_ALIGN!().into(), T_ACTIVE!('&').into()])
    .into_iter()
    .map(in_hbox_math)
    .collect();
  match clauses.len() {
    0 => Tokens!(),
    1 => clauses.into_iter().next().unwrap(),
    _ => i_apply(&[], Tokens!(T_CS!("\\lx@proof@logical@and")), clauses),
  }
}

/// The premises without each row end's own `*` and `[<skip>]` (the `\\[1pt]` between 1709.05725's premise rows):
/// amsmath's `\math@cr` reads them with `\@ifstar` and `\new@ifnextchar[` (amsmath.sty:1174-1180), and amsgen's
/// `\@ifstar` is a `\new@ifnextchar` too (amsgen.sty:54-63), which skips no space: only a `*` right after the `\\` and
/// a `[` right after that are the row end's. After a space (a token after the control symbol `\\`) they are the
/// premise's own text: pdflatex prints `\infer{E}{s \\ [t] u}` and `{s \\ *[t] u}` in `{align*}` with their `[t]`,
/// 0 errors. That is the rule of amsmath's displays, where proof.sty's papers put `\infer`; outside them (`{array}`,
/// `{eqnarray}`, no amsmath) LaTeX's `\@xarraycr` skips spaces before `[` (latex.ltx:16585) and pdflatex reads
/// `s \\ [t] u`'s `[t]` as a skip, where this keeps it as text. Read before the premises split into rows, whose pieces
/// lose their leading spaces (`split_tokens`).
fn drop_row_skips(tokens: Tokens) -> Tokens {
  let tokens = tokens.unlist();
  let mut out = Vec::with_capacity(tokens.len());
  let mut level = 0usize;
  let mut index = 0;
  while index < tokens.len() {
    let token = tokens[index];
    index += 1;
    match token.get_catcode() {
      Catcode::BEGIN => level += 1,
      Catcode::END => level = level.saturating_sub(1),
      _ => {},
    }
    out.push(token);
    if level == 0 && token == T_CS!("\\\\") {
      if tokens.get(index) == Some(&T_OTHER!("*")) {
        index += 1;
      }
      if tokens.get(index) == Some(&T_OTHER!("["))
        && let Some(close) = tokens[index..].iter().position(|t| *t == T_OTHER!("]"))
      {
        index += close + 1;
      }
    }
  }
  Tokens::new(out)
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: proof.sty.ltxml
  // The premises can be separated by "&" (which is NOT being used for alignment!)
  DefMath!("\\lx@proof@logical@and", "\u{2003}", role => "ADDOP", meaning => "and");

  // Perl proof.sty.ltxml L20-28: split the argument on T_ALIGN (`&`),
  // which in proof.sty separates premises (not for column alignment).
  // Single clause → pass through; multiple → wrap with I_apply around
  // \lx@proof@logical@and so the semantic structure is an n-ary AND.
  // The premises are proof.sty's own `\halign` (proof.sty:164-167, `#3\cr` in its preamble `$##$&&…$##$\cr`): an
  // `&` that is active is a premise separator there too — tikz-cd's cells make it `\pgfmatrixnextcell`
  // (tikz.code.tex:4064-4068), a `\vcenter{\infer{C}{a & b}}` cell (2205.02142) — and a row end, `\cr` or the `\\` an
  // amsmath display lets to it (`\infer[r]{C}{A & \\ B}` in `{align*}`, 2011.13396), ends a row of premises, not the
  // display's: each row is one of `\lx@proof@premise@rows`. Perl splits on the catcode-4 `&` alone, its `\\` reaching
  // the display's alignment. Guards `perfect_kernel_batch77q::{proof_premise_rows_stay_in_the_inference,
  // proof_premises_split_on_an_active_ampersand}`.
  DefMacro!("\\lx@proof@split@and {}", sub[(tokens)] {
    let rows = split_tokens(drop_row_skips(tokens), vec![T_CS!("\\\\").into(), T_CS!("\\cr").into()]);
    if rows.len() <= 1 {
      Ok(split_premises(rows.into_iter().next().unwrap_or_default()))
    } else {
      let mut out = vec![T_CS!("\\lx@proof@premise@rows"), T_BEGIN!()];
      for (index, row) in rows.into_iter().enumerate() {
        if index > 0 {
          out.push(T_CS!("\\lx@proof@premise@rowsep"));
        }
        out.extend([T_CS!("\\lx@proof@premise@row"), T_BEGIN!()]);
        out.extend(split_premises(row).unlist());
        out.push(T_END!());
      }
      out.push(T_END!());
      Ok(Tokens::new(out))
    }
  });
  DefConstructor!("\\lx@proof@premise@rows{}", "<ltx:XMArray>#1</ltx:XMArray>", reversion => "#1");
  DefConstructor!("\\lx@proof@premise@row{}",
    "<ltx:XMRow><ltx:XMCell><ltx:XMArg>#1</ltx:XMArg></ltx:XMCell></ltx:XMRow>",
    reversion => "#1");
  DefConstructor!("\\lx@proof@premise@rowsep", "", reversion => "\\\\");

  // \lx@proof@stack{}{}{}{}:
  // An extremely contrived stack of premises and conclusion.
  // Args are $top, $middle (if any), $bottom, $sidelabel (if any).
  // Perl L44-49 after_digest compares top/bot widths; if top is wider,
  // stretches bot (the bars whatsit) to match. Without this, the bars
  // over the conclusion are only as wide as the conclusion itself.
  DefConstructor!("\\lx@proof@stack{}{}{}{}",
    "<ltx:XMArray vattach='bottom'><ltx:XMRow><ltx:XMCell>#1</ltx:XMCell>?#4(<ltx:XMCell rowspan='3'>#4</ltx:XMCell>)()</ltx:XMRow>?#2(<ltx:XMRow><ltx:XMCell>#2</ltx:XMCell></ltx:XMRow>)()<ltx:XMRow><ltx:XMCell>#3</ltx:XMCell></ltx:XMRow></ltx:XMArray>",
    after_digest => sub[whatsit] {
      let top_w = whatsit.get_arg(1).and_then(|a| a.clone().get_width(None).ok().flatten());
      let bot_w = whatsit.get_arg(3).and_then(|a| a.clone().get_width(None).ok().flatten());
      if let (Some(RegisterValue::Dimension(tw)),
              Some(RegisterValue::Dimension(bw))) = (top_w, bot_w)
        && tw.value_of() > bw.value_of()
          && let Some(bot) = whatsit.get_arg_mut(3) {
            bot.set_property("stretchto", Stored::from(
              RegisterValue::Dimension(tw)
            ));
          }
    });

  // Perl L52-57 emits `width='#stretchto'` on the XMWrap so the bars
  // stretch to the width stashed by \lx@proof@stack's after_digest.
  DefConstructor!("\\lx@proof@bars OptionalMatch:= {}",
    "<ltx:XMApp><ltx:XMTok role='OVERACCENT'>\u{203E}</ltx:XMTok>?#1(<ltx:XMApp><ltx:XMTok role='OVERACCENT'>\u{203E}</ltx:XMTok>)()<ltx:XMWrap width='#stretchto'>#2</ltx:XMWrap>?#1(</ltx:XMApp>)()</ltx:XMApp>");

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
          double.unwrap_or_default(),
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
        Invocation!(T_CS!("\\lx@proof@split@and"), vec![uppers]);
      let cmd = i_dual(&[], content, presentation, vec![
        in_hbox_math(lower),
        uppers_split,
        in_hbox_math(label.unwrap_or_default()),
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
      in_hbox_math(label.unwrap_or_default()),
      Tokens!(i_arg("1")),
      Tokens!(),
    ]);
    let uppers_split =
      Invocation!(T_CS!("\\lx@proof@split@and"), vec![uppers]);
    let cmd = i_dual(&[], content, presentation, vec![
      in_hbox_math(lower),
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
