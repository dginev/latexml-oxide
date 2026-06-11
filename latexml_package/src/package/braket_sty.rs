use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMath!("\\bra{}", "\\langle#1|",            meaning => "bra");
  DefMath!("\\Bra{}", "\\left\\langle#1\\right|", meaning => "bra");
  DefMath!("\\ket{}", "|#1\\rangle",           meaning => "ket");
  DefMath!("\\Ket{}", "\\left|#1\\right\\rangle", meaning => "ket");
  // Perl: alias => '\braket' / '\Braket' makes the no-separator variants
  // round-trip to source as `\braket{x}` rather than `\lx@braket@{x}`.
  // Without it, source-export of `\braket{x}` reified into the internal CS.
  DefMath!("\\lx@braket@{}", "\\langle#1\\rangle",
    meaning => "expectation", alias => "\\braket");
  DefMath!("\\lx@Braket@{}", "\\left\\langle#1\\right\\rangle",
    meaning => "expectation", alias => "\\Braket");
  // Perl L77-88: V/D variants pair an explicit reversion template with
  // alias => '\braket' / '\Braket'. Reversion takes priority for source
  // recovery, but alias is what shows up in MathML / annotations.
  DefMath!("\\lx@braket@V{}{}", "\\langle#1\\,|\\,#2\\rangle",
    meaning => "inner-product", alias => "\\braket", reversion => "\\braket{#1|#2}");
  DefMath!("\\lx@braket@D{}{}", "\\langle#1\\,\\|\\,#2\\rangle",
    meaning => "inner-product", alias => "\\braket", reversion => "\\braket{#1\\|#2}");
  DefMath!("\\lx@Braket@V{}{}", "\\left\\langle#1\\,\\middle|\\,#2\\right\\rangle",
    meaning => "inner-product", alias => "\\Braket", reversion => "\\Braket{#1|#2}");
  DefMath!("\\lx@Braket@D{}{}", "\\left\\langle#1\\,\\middle\\|\\,#2\\right\\rangle",
    meaning => "inner-product", alias => "\\Braket", reversion => "\\Braket{#1\\|#2}");
  // All braket variants (Perl L90-114)
  DefMath!("\\lx@braket@VV{}{}{}", "\\langle#1\\,|#2\\,|\\,#3\\rangle",
    meaning => "quantum-operator-product", alias => "\\braket", reversion => "\\braket{#1|#2|#3}");
  DefMath!("\\lx@braket@VD{}{}{}", "\\langle#1\\,|\\,#2\\,\\|\\,#3\\rangle",
    meaning => "quantum-operator-product", alias => "\\braket", reversion => "\\braket{#1|#2\\|#3}");
  DefMath!("\\lx@braket@DV{}{}{}", "\\langle#1\\,\\|\\,#2\\,|\\,#3\\rangle",
    meaning => "quantum-operator-product", alias => "\\braket", reversion => "\\braket{#1\\|#2|#3}");
  DefMath!("\\lx@braket@DD{}{}{}", "\\langle#1\\,\\|\\,#2\\,\\|\\,#3\\rangle",
    meaning => "quantum-operator-product", alias => "\\braket", reversion => "\\braket{#1\\|#2\\|#3}");
  DefMath!("\\lx@Braket@VV{}{}{}", "\\left\\langle#1\\,\\middle|\\,#2\\,\\middle|\\,#3\\right\\rangle",
    meaning => "quantum-operator-product", alias => "\\Braket", reversion => "\\Braket{#1|#2|#3}");
  DefMath!("\\lx@Braket@VD{}{}{}", "\\left\\langle#1\\,\\middle|\\,#2\\,\\middle\\|\\,#3\\right\\rangle",
    meaning => "quantum-operator-product", alias => "\\Braket", reversion => "\\Braket{#1|#2\\|#3}");
  DefMath!("\\lx@Braket@DV{}{}{}", "\\left\\langle#1\\,\\middle\\|\\,#2\\,\\middle|\\,#3\\right\\rangle",
    meaning => "quantum-operator-product", alias => "\\Braket", reversion => "\\Braket{#1\\|#2|#3}");
  DefMath!("\\lx@Braket@DD{}{}{}", "\\left\\langle#1\\,\\middle\\|\\,#2\\,\\middle\\|\\,#3\\right\\rangle",
    meaning => "quantum-operator-product", alias => "\\Braket", reversion => "\\Braket{#1\\|#2\\|#3}");

  // \braket — splits argument on | bars to dispatch to V/D variants — Perl L57-66.
  //
  // Perl uses Invocation(T_CS('\lx@braket@' . $codes), @args) to pass the
  // part Tokens raw, preserving each token's identity. Our earlier
  // `tokenize_internal(&format!("…{}", parts[0]))` path re-tokenized the
  // stringified arg, and TeX's CS-builder then fused `\mbf r` into a single
  // `\mbfr` CS (no space survived the Display round-trip). The direct
  // Token construction avoids the round-trip entirely.
  DefMacro!("\\braket{}", sub[args] {
    let arg = args[0].clone().into_tokens_result()?;
    let (codes, parts) = split_braket_arg(arg, 2);
    let cs = s!("\\lx@braket@{codes}");
    Ok(build_invocation(&cs, &parts))
  });
  DefMacro!("\\Braket{}", sub[args] {
    let arg = args[0].clone().into_tokens_result()?;
    let (codes, parts) = split_braket_arg(arg, 2);
    let cs = s!("\\lx@Braket@{codes}");
    Ok(build_invocation(&cs, &parts))
  });

  // Set notation (Perl L117-146) — alias matches Perl so the helper CSes
  // round-trip as `\set` / `\Set` rather than `\lx@set@…`.
  DefMath!("\\lx@set@{}", "\\{#1\\}", meaning => "set", alias => "\\set");
  DefMath!("\\lx@Set@{}", "\\left\\{#1\\right\\}", meaning => "set", alias => "\\Set");
  DefMath!("\\lx@set@V{}{}", "\\{#1\\;|\\;#2\\}", meaning => "set", alias => "\\set");
  DefMath!("\\lx@set@D{}{}", "\\{#1\\;\\|\\;#2\\}", meaning => "set", alias => "\\set");
  DefMath!("\\lx@Set@V{}{}", "\\left\\{#1\\;\\middle|\\;#2\\right\\}", meaning => "set", alias => "\\Set");
  DefMath!("\\lx@Set@D{}{}", "\\left\\{#1\\;\\middle\\|\\;#2\\right\\}", meaning => "set", alias => "\\Set");
  // \set/\Set — Perl L117-126 also splits via splitBraketArg (maxbars=1).
  // So `\set{x\|y}` dispatches to `\lx@set@D`, not `\lx@set@V`, preserving
  // the double-bar meaning in the set-builder notation.
  DefMacro!("\\set{}", sub[args] {
    let arg = args[0].clone().into_tokens_result()?;
    let (codes, parts) = split_braket_arg(arg, 1);
    let cs = s!("\\lx@set@{codes}");
    Ok(build_invocation(&cs, &parts))
  });
  DefMacro!("\\Set{}", sub[args] {
    let arg = args[0].clone().into_tokens_result()?;
    let (codes, parts) = split_braket_arg(arg, 1);
    let cs = s!("\\lx@Set@{codes}");
    Ok(build_invocation(&cs, &parts))
  });
});

/// Port of Perl's `splitBraketArg` (braket.sty.ltxml L27-54). Splits a
/// token list on top-level bar separators up to `maxbars` times, while
/// tracking whether each separator was `|` (V) or `\|` (D). Returns the
/// codes string (e.g. "V", "D", "VV", "VD", "DV", "DD") and the N+1
/// parts. Nested braces are respected — a bar inside `{…}` stays in the
/// current part.
///
/// Perl also detects `||` (two adjacent `|` OTHER tokens) as a D code,
/// not two V splits — matches here via the lookahead on `tokens[0]`.
fn split_braket_arg(arg: Tokens, mut maxbars: usize) -> (String, Vec<Tokens>) {
  let vbar = T_OTHER!("|");
  let dbar = T_CS!("\\|");
  let mut codes = String::new();
  let mut parts: Vec<Tokens> = Vec::new();
  let mut current: Vec<Token> = Vec::new();
  let mut depth: i32 = 0;
  let mut tokens: VecDeque<Token> = arg.unlist().into();
  while let Some(t) = tokens.pop_front() {
    if t.get_catcode() == Catcode::BEGIN {
      depth += 1;
    } else if t.get_catcode() == Catcode::END {
      depth -= 1;
    }
    if depth == 0 && maxbars > 0 && t == vbar {
      // `||` — single D split (Perl L38-40)
      if tokens.front() == Some(&vbar) {
        tokens.pop_front();
        codes.push('D');
      } else {
        codes.push('V');
      }
      maxbars -= 1;
      parts.push(Tokens::new(std::mem::take(&mut current)));
    } else if depth == 0 && maxbars > 0 && t == dbar {
      codes.push('D');
      maxbars -= 1;
      parts.push(Tokens::new(std::mem::take(&mut current)));
    } else {
      current.push(t);
    }
  }
  parts.push(Tokens::new(current));
  (codes, parts)
}

/// Build `\cs{arg1}{arg2}…` as a raw Token stream — mirrors Perl's
/// `Invocation(T_CS($cs), @args)`. Each arg is wrapped in an explicit
/// `{…}` group so the downstream macro reads it as a single argument
/// without re-tokenizing a Display-formatted form (which would fuse
/// `\mbf r` into `\mbfr`).
fn build_invocation(cs: impl AsRef<str>, args: &[Tokens]) -> Tokens {
  let total_tokens: usize = args.iter().map(|a| a.len() + 2).sum();
  let mut out: Vec<Token> = Vec::with_capacity(1 + total_tokens);
  out.push(T_CS!(cs.as_ref()));
  for arg in args {
    out.push(T_BEGIN!());
    out.extend(arg.clone().unlist());
    out.push(T_END!());
  }
  Tokens::new(out)
}

#[cfg(test)]
mod tests {
  use latexml_core::state::{State, StateOptions, set_state};

  use super::*;

  fn setup() { set_state(State::new(StateOptions::default())); }

  fn toks(tok_vec: Vec<Token>) -> Tokens { Tokens::new(tok_vec) }

  // ----- split_braket_arg -----

  #[test]
  fn split_braket_arg_no_bar_empty_codes() {
    setup();
    let arg = toks(vec![T_LETTER!("a"), T_LETTER!("b")]);
    let (codes, parts) = split_braket_arg(arg, 2);
    assert_eq!(codes, "");
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].len(), 2);
  }

  #[test]
  fn split_braket_arg_single_vbar_yields_v_code() {
    setup();
    let arg = toks(vec![T_LETTER!("a"), T_OTHER!("|"), T_LETTER!("b")]);
    let (codes, parts) = split_braket_arg(arg, 2);
    assert_eq!(codes, "V");
    assert_eq!(parts.len(), 2);
  }

  #[test]
  fn split_braket_arg_dbar_cs_yields_d_code() {
    setup();
    let arg = toks(vec![T_LETTER!("a"), T_CS!("\\|"), T_LETTER!("b")]);
    let (codes, parts) = split_braket_arg(arg, 2);
    assert_eq!(codes, "D");
    assert_eq!(parts.len(), 2);
  }

  #[test]
  fn split_braket_arg_double_pipe_yields_d_code() {
    setup();
    // `||` at top level → single D split, not two V splits
    let arg = toks(vec![
      T_LETTER!("a"),
      T_OTHER!("|"),
      T_OTHER!("|"),
      T_LETTER!("b"),
    ]);
    let (codes, parts) = split_braket_arg(arg, 2);
    assert_eq!(codes, "D");
    assert_eq!(parts.len(), 2);
  }

  #[test]
  fn split_braket_arg_two_bars_three_parts() {
    setup();
    let arg = toks(vec![
      T_LETTER!("a"),
      T_OTHER!("|"),
      T_LETTER!("b"),
      T_OTHER!("|"),
      T_LETTER!("c"),
    ]);
    let (codes, parts) = split_braket_arg(arg, 2);
    assert_eq!(codes, "VV");
    assert_eq!(parts.len(), 3);
  }

  #[test]
  fn split_braket_arg_mixed_v_d() {
    setup();
    let arg = toks(vec![
      T_LETTER!("a"),
      T_OTHER!("|"),
      T_LETTER!("b"),
      T_CS!("\\|"),
      T_LETTER!("c"),
    ]);
    let (codes, parts) = split_braket_arg(arg, 2);
    assert_eq!(codes, "VD");
    assert_eq!(parts.len(), 3);
  }

  #[test]
  fn split_braket_arg_maxbars_caps_splits() {
    setup();
    // maxbars=1 → only first bar splits; remaining bars stay as content
    let arg = toks(vec![
      T_LETTER!("a"),
      T_OTHER!("|"),
      T_LETTER!("b"),
      T_OTHER!("|"),
      T_LETTER!("c"),
    ]);
    let (codes, parts) = split_braket_arg(arg, 1);
    assert_eq!(codes, "V");
    assert_eq!(parts.len(), 2);
    // Second part retains the second `|` as literal content
    assert_eq!(parts[1].len(), 3);
  }

  #[test]
  fn split_braket_arg_respects_nested_braces() {
    setup();
    // `a{b|c}d` is one part — the inner `|` is inside `{…}`.
    let arg = toks(vec![
      T_LETTER!("a"),
      T_BEGIN!(),
      T_LETTER!("b"),
      T_OTHER!("|"),
      T_LETTER!("c"),
      T_END!(),
      T_LETTER!("d"),
    ]);
    let (codes, parts) = split_braket_arg(arg, 2);
    assert_eq!(codes, "");
    assert_eq!(parts.len(), 1);
  }

  #[test]
  fn split_braket_arg_empty_input() {
    setup();
    let (codes, parts) = split_braket_arg(toks(vec![]), 2);
    assert_eq!(codes, "");
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].len(), 0);
  }

  // ----- build_invocation -----

  #[test]
  fn build_invocation_wraps_each_arg_in_braces() {
    setup();
    let a = toks(vec![T_LETTER!("a")]);
    let b = toks(vec![T_LETTER!("b")]);
    let result = build_invocation("\\lx@braket@V", &[a, b]);
    // Expected: \cs { a } { b }  → 7 tokens
    let list = result.unlist();
    assert_eq!(list.len(), 7);
    assert_eq!(list[0], T_CS!("\\lx@braket@V"));
    assert_eq!(list[1], T_BEGIN!());
    assert_eq!(list[2], T_LETTER!("a"));
    assert_eq!(list[3], T_END!());
    assert_eq!(list[4], T_BEGIN!());
    assert_eq!(list[5], T_LETTER!("b"));
    assert_eq!(list[6], T_END!());
  }

  #[test]
  fn build_invocation_no_args_is_just_cs() {
    setup();
    let result = build_invocation("\\foo", &[]);
    assert_eq!(result.unlist(), vec![T_CS!("\\foo")]);
  }
}
