use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMath!("\\bra{}", "\\langle#1|",            meaning => "bra");
  DefMath!("\\Bra{}", "\\left\\langle#1\\right|", meaning => "bra");
  DefMath!("\\ket{}", "|#1\\rangle",           meaning => "ket");
  DefMath!("\\Ket{}", "\\left|#1\\right\\rangle", meaning => "ket");
  DefMath!("\\lx@braket@{}", "\\langle#1\\rangle", meaning => "expectation");
  DefMath!("\\lx@Braket@{}", "\\left\\langle#1\\right\\rangle", meaning => "expectation");
  // Perl #2340: reversions use user-facing \braket/\Braket with | separators
  DefMath!("\\lx@braket@V{}{}", "\\langle#1\\,|\\,#2\\rangle",
    meaning => "inner-product", reversion => "\\braket{#1|#2}");
  DefMath!("\\lx@braket@D{}{}", "\\langle#1\\,\\|\\,#2\\rangle",
    meaning => "inner-product", reversion => "\\braket{#1\\|#2}");
  DefMath!("\\lx@Braket@V{}{}", "\\left\\langle#1\\,\\middle|\\,#2\\right\\rangle",
    meaning => "inner-product", reversion => "\\Braket{#1|#2}");
  DefMath!("\\lx@Braket@D{}{}", "\\left\\langle#1\\,\\middle\\|\\,#2\\right\\rangle",
    meaning => "inner-product", reversion => "\\Braket{#1\\|#2}");
  // All braket variants (Perl L90-114)
  DefMath!("\\lx@braket@VV{}{}{}", "\\langle#1\\,|#2\\,|\\,#3\\rangle",
    meaning => "quantum-operator-product", reversion => "\\braket{#1|#2|#3}");
  DefMath!("\\lx@braket@VD{}{}{}", "\\langle#1\\,|\\,#2\\,\\|\\,#3\\rangle",
    meaning => "quantum-operator-product", reversion => "\\braket{#1|#2\\|#3}");
  DefMath!("\\lx@braket@DV{}{}{}", "\\langle#1\\,\\|\\,#2\\,|\\,#3\\rangle",
    meaning => "quantum-operator-product", reversion => "\\braket{#1\\|#2|#3}");
  DefMath!("\\lx@braket@DD{}{}{}", "\\langle#1\\,\\|\\,#2\\,\\|\\,#3\\rangle",
    meaning => "quantum-operator-product", reversion => "\\braket{#1\\|#2\\|#3}");
  DefMath!("\\lx@Braket@VV{}{}{}", "\\left\\langle#1\\,\\middle|\\,#2\\,\\middle|\\,#3\\right\\rangle",
    meaning => "quantum-operator-product", reversion => "\\Braket{#1|#2|#3}");
  DefMath!("\\lx@Braket@VD{}{}{}", "\\left\\langle#1\\,\\middle|\\,#2\\,\\middle\\|\\,#3\\right\\rangle",
    meaning => "quantum-operator-product", reversion => "\\Braket{#1|#2\\|#3}");
  DefMath!("\\lx@Braket@DV{}{}{}", "\\left\\langle#1\\,\\middle\\|\\,#2\\,\\middle|\\,#3\\right\\rangle",
    meaning => "quantum-operator-product", reversion => "\\Braket{#1\\|#2|#3}");
  DefMath!("\\lx@Braket@DD{}{}{}", "\\left\\langle#1\\,\\middle\\|\\,#2\\,\\middle\\|\\,#3\\right\\rangle",
    meaning => "quantum-operator-product", reversion => "\\Braket{#1\\|#2\\|#3}");

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
    let parts: Vec<Tokens> = split_braket_parts(arg);
    let (cs, n) = pick_braket_cs("\\lx@braket@", parts.len());
    Ok(build_invocation(cs, &parts[..n]))
  });
  DefMacro!("\\Braket{}", sub[args] {
    let arg = args[0].clone().into_tokens_result()?;
    let parts: Vec<Tokens> = split_braket_parts(arg);
    let (cs, n) = pick_braket_cs("\\lx@Braket@", parts.len());
    Ok(build_invocation(cs, &parts[..n]))
  });

  // Set notation (Perl L117-146)
  DefMath!("\\lx@set@{}", "\\{#1\\}", meaning => "set");
  DefMath!("\\lx@Set@{}", "\\left\\{#1\\right\\}", meaning => "set");
  DefMath!("\\lx@set@V{}{}", "\\{#1\\;|\\;#2\\}", meaning => "set");
  DefMath!("\\lx@set@D{}{}", "\\{#1\\;\\|\\;#2\\}", meaning => "set");
  DefMath!("\\lx@Set@V{}{}", "\\left\\{#1\\;\\middle|\\;#2\\right\\}", meaning => "set");
  DefMath!("\\lx@Set@D{}{}", "\\left\\{#1\\;\\middle\\|\\;#2\\right\\}", meaning => "set");
  // \set/\Set — split on | for set-builder notation — Perl L117-126
  DefMacro!("\\set{}", sub[args] {
    let arg = args[0].clone().into_tokens_result()?;
    let parts: Vec<Tokens> = split_braket_parts(arg);
    let cs = if parts.len() >= 2 { "\\lx@set@V" } else { "\\lx@set@" };
    let n = if parts.len() >= 2 { 2 } else { 1 };
    Ok(build_invocation(cs, &parts[..n]))
  });
  DefMacro!("\\Set{}", sub[args] {
    let arg = args[0].clone().into_tokens_result()?;
    let parts: Vec<Tokens> = split_braket_parts(arg);
    let cs = if parts.len() >= 2 { "\\lx@Set@V" } else { "\\lx@Set@" };
    let n = if parts.len() >= 2 { 2 } else { 1 };
    Ok(build_invocation(cs, &parts[..n]))
  });
});

/// Split a token list on top-level `|` OTHER tokens. Nested braces are
/// respected: a `|` inside `{…}` stays within the current part. Mirrors
/// Perl's `splitBraketArg` for the single-bar case (we do not yet
/// distinguish `\|` double-bar from `|`, matching the simpler branch of
/// the Perl helper).
fn split_braket_parts(arg: Tokens) -> Vec<Tokens> {
  let vbar = T_OTHER!("|");
  let mut result: Vec<Tokens> = Vec::new();
  let mut current: Vec<Token> = Vec::new();
  let mut depth: i32 = 0;
  for t in arg.unlist() {
    if t.get_catcode() == Catcode::BEGIN {
      depth += 1;
    } else if t.get_catcode() == Catcode::END {
      depth -= 1;
    }
    if depth == 0 && t == vbar {
      result.push(Tokens::new(std::mem::take(&mut current)));
    } else {
      current.push(t);
    }
  }
  result.push(Tokens::new(current));
  result
}

/// Pick the `\lx@braket@…` / `\lx@Braket@…` dispatch target plus the
/// actual argument count to forward, based on how many `|`-separated
/// parts the user supplied.
fn pick_braket_cs(prefix: &str, n_parts: usize) -> (String, usize) {
  match n_parts {
    2 => (format!("{prefix}V"), 2),
    n if n >= 3 => (format!("{prefix}VV"), 3),
    _ => (prefix.to_string(), 1),
  }
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
  use super::*;
  use latexml_core::state::{State, StateOptions, set_state};

  fn setup() { set_state(State::new(StateOptions::default())); }

  fn toks(tok_vec: Vec<Token>) -> Tokens { Tokens::new(tok_vec) }

  // ----- pick_braket_cs -----

  #[test]
  fn pick_braket_cs_one_part_uses_bare_prefix() {
    assert_eq!(
      pick_braket_cs("\\lx@braket@", 1),
      ("\\lx@braket@".to_string(), 1)
    );
    assert_eq!(
      pick_braket_cs("\\lx@braket@", 0),
      ("\\lx@braket@".to_string(), 1)
    );
  }

  #[test]
  fn pick_braket_cs_two_parts_uses_v_suffix() {
    assert_eq!(
      pick_braket_cs("\\lx@braket@", 2),
      ("\\lx@braket@V".to_string(), 2)
    );
    assert_eq!(
      pick_braket_cs("\\lx@Braket@", 2),
      ("\\lx@Braket@V".to_string(), 2)
    );
  }

  #[test]
  fn pick_braket_cs_three_or_more_uses_vv_suffix() {
    assert_eq!(
      pick_braket_cs("\\lx@braket@", 3),
      ("\\lx@braket@VV".to_string(), 3)
    );
    // Extra parts collapse down to the 3-part variant.
    assert_eq!(
      pick_braket_cs("\\lx@braket@", 4),
      ("\\lx@braket@VV".to_string(), 3)
    );
    assert_eq!(
      pick_braket_cs("\\lx@braket@", 5),
      ("\\lx@braket@VV".to_string(), 3)
    );
  }

  // ----- split_braket_parts -----

  #[test]
  fn split_braket_parts_no_bar_returns_one_part() {
    setup();
    let arg = toks(vec![T_LETTER!("a"), T_LETTER!("b")]);
    let parts = split_braket_parts(arg);
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].len(), 2);
  }

  #[test]
  fn split_braket_parts_single_bar_splits_in_two() {
    setup();
    let arg = toks(vec![T_LETTER!("a"), T_OTHER!("|"), T_LETTER!("b")]);
    let parts = split_braket_parts(arg);
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].len(), 1);
    assert_eq!(parts[1].len(), 1);
  }

  #[test]
  fn split_braket_parts_two_bars_splits_in_three() {
    setup();
    let arg = toks(vec![
      T_LETTER!("a"),
      T_OTHER!("|"),
      T_LETTER!("b"),
      T_OTHER!("|"),
      T_LETTER!("c"),
    ]);
    let parts = split_braket_parts(arg);
    assert_eq!(parts.len(), 3);
  }

  #[test]
  fn split_braket_parts_respects_nested_braces() {
    setup();
    // `a{b|c}d` is one part — the inner | is inside `{…}`.
    let arg = toks(vec![
      T_LETTER!("a"),
      T_BEGIN!(),
      T_LETTER!("b"),
      T_OTHER!("|"),
      T_LETTER!("c"),
      T_END!(),
      T_LETTER!("d"),
    ]);
    let parts = split_braket_parts(arg);
    assert_eq!(parts.len(), 1, "| inside braces must stay in current part");
  }

  #[test]
  fn split_braket_parts_empty_input_returns_one_empty_part() {
    setup();
    let parts = split_braket_parts(toks(vec![]));
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].len(), 0);
  }

  #[test]
  fn split_braket_parts_leading_bar_gives_empty_head() {
    setup();
    let arg = toks(vec![T_OTHER!("|"), T_LETTER!("a")]);
    let parts = split_braket_parts(arg);
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].len(), 0);
    assert_eq!(parts[1].len(), 1);
  }

  #[test]
  fn split_braket_parts_trailing_bar_gives_empty_tail() {
    setup();
    let arg = toks(vec![T_LETTER!("a"), T_OTHER!("|")]);
    let parts = split_braket_parts(arg);
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].len(), 1);
    assert_eq!(parts[1].len(), 0);
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
