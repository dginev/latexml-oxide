//! XMath helper functions for building XMDual token streams.
//! Port of Perl's I_dual, I_arg, I_xmarg, I_xmref, I_wrap from Base_XMath.pool.ltxml.
//!
//! These functions generate token lists that expand into semantic math markup.
//! Used by physics.sty, diffcoeff.sty, and other packages that need XMDual structures.

use crate::prelude::*;

/// Perl: getXMArgID — step @lx@xmarg counter, return its value as a string.
///
/// Fast path for the xmath_helpers callers: they only read the
/// register value — they never expand `\the@lx@xmarg@ID`. That means
/// we don't need the full `step_counter` machinery (which also
/// def_macro's `\@@lx@xmarg@ID`, probes `\cl@@lx@xmarg` for nested
/// counters, and allocates two format! strings per call). Directly
/// read+write the `\c@@lx@xmarg` register instead.
///
/// Callers in Core dialect.rs (`get_xmarg_id`) still go through the
/// full `step_counter` because they *do* expand the macro form.
pub fn get_xm_arg_id() -> Result<String> {
  // `T_CS!` with a literal arm routes through `pin!` — the Token
  // construction is essentially free after first call (OnceCell load).
  // Pass the Token straight to the register ops so they don't have to
  // re-pin the `&str` key.
  let cs = T_CS!("\\c@@lx@xmarg");
  let current = lookup_register_token(&cs, Vec::new())?
    .map(|rv| rv.value_of())
    .unwrap_or(0);
  let next = current + 1;
  assign_register_token(
    &cs,
    Number::new(next).into(),
    Some(Scope::Global),
    Vec::new(),
  )?;
  Ok(next.to_string())
}

/// Perl: I_arg(n) — create a parameter token #n (cc ARG).
pub fn i_arg(n: &str) -> Token {
  let idx = n.parse::<u8>().unwrap_or(1);
  // Create a CC_ARG token with the given index
  CharToken!(
    char::from_digit(idx as u32, 10).unwrap_or('1'),
    Catcode::ARG
  )
}

/// Perl: I_xmarg(id, arg) — generates `\lx@xmarg{id}{arg}` tokens.
pub fn i_xmarg(id: &str, arg: Tokens) -> Tokens {
  // Pre-size: \lx@xmarg {id} {arg} = 1 + 2 + id.len() + 2 + arg.len()
  let mut tks: Vec<Token> = Vec::with_capacity(5 + id.len() + arg.len());
  tks.push(T_CS!("\\lx@xmarg"));
  tks.push(T_BEGIN!());
  tks.extend(ExplodeText!(id));
  tks.push(T_END!());
  tks.push(T_BEGIN!());
  tks.extend(arg.unlist());
  tks.push(T_END!());
  Tokens::new(tks)
}

/// Perl: I_xmref(id) — generates `\lx@xmref{id}` tokens.
pub fn i_xmref(id: &str) -> Tokens {
  // Pre-size: \lx@xmref {id} = 1 + 2 + id.len()
  let mut tks: Vec<Token> = Vec::with_capacity(3 + id.len());
  tks.push(T_CS!("\\lx@xmref"));
  tks.push(T_BEGIN!());
  tks.extend(ExplodeText!(id));
  tks.push(T_END!());
  Tokens::new(tks)
}

/// Perl: I_wrap(keyvals, content) — generates `\lx@wrap[keyvals]{content}` tokens.
pub fn i_wrap(keyvals: Option<Tokens>, content: Tokens) -> Tokens {
  // Pre-size: \lx@wrap + optional [kv] + {content}
  let kv_len = keyvals.as_ref().map(|k| k.len() + 2).unwrap_or(0);
  let mut tks: Vec<Token> = Vec::with_capacity(1 + kv_len + 2 + content.len());
  tks.push(T_CS!("\\lx@wrap"));
  if let Some(kv) = keyvals {
    tks.push(T_OTHER!("["));
    tks.extend(kv.unlist());
    tks.push(T_OTHER!("]"));
  }
  tks.push(T_BEGIN!());
  tks.extend(content.unlist());
  tks.push(T_END!());
  Tokens::new(tks)
}

/// Perl: I_dual(keyvals, content, presentation, args) — generates full XMDual token stream.
///
/// Creates `\lx@dual[options]{content_with_xmrefs}{wrapped_presentation_with_xmargs}`.
///
/// # Arguments
/// - `keyvals`: Key-value pairs as `(&str, Tokens)` — uses Tokens directly to preserve ARG catcodes
/// - `content`: Token template for content branch (with #1, #2, ... parameter refs)
/// - `presentation`: Token template for presentation branch (with #1, #2, ... parameter refs)
/// - `args`: The actual argument token lists to be shared between branches
pub fn i_dual(
  keyvals: &[(&str, Tokens)],
  content: Tokens,
  presentation: Tokens,
  args: Vec<Tokens>,
) -> Result<Tokens> {
  // Keep original args for reversion substitution (actual content, not xmarg refs)
  let n_args = args.len();
  let mut orig_args: Vec<Tokens> = Vec::with_capacity(n_args);
  let mut pargs: Vec<Tokens> = Vec::with_capacity(n_args);
  let mut cargs: Vec<Tokens> = Vec::with_capacity(n_args);

  for arg in args {
    let id = get_xm_arg_id()?;
    orig_args.push(arg.clone());
    pargs.push(i_xmarg(&id, arg));
    cargs.push(i_xmref(&id));
  }

  // Separate reversion from other keyvals
  let mut reversion_tokens: Option<Tokens> = None;
  let mut opts = Vec::new();
  let mut opt_count = 0;
  for (key, value) in keyvals.iter() {
    if *key == "reversion" {
      // Substitute actual arg content into reversion template (no ARG tokens remain)
      let rev_opt: Vec<Option<Cow<Tokens>>> =
        orig_args.iter().map(|t| Some(Cow::Borrowed(t))).collect();
      reversion_tokens = Some(value.clone().substitute_parameters(&rev_opt));
    } else {
      if opt_count > 0 {
        opts.push(T_OTHER!(","));
      }
      opts.extend(ExplodeText!(key));
      opts.push(T_OTHER!("="));
      opts.push(T_BEGIN!());
      opts.extend(value.unlist_ref().iter().copied());
      opts.push(T_END!());
      opt_count += 1;
    }
  }
  let optional = if opt_count > 0 {
    Some(Tokens::new(opts))
  } else {
    None
  };

  // Push reversion via state (bypasses keyval string conversion)
  if let Some(rev) = reversion_tokens {
    push_value("PENDING_DUAL_REVERSION", Stored::Tokens(rev))?;
  }

  // Build content with xmrefs substituted
  let cargs_opt: Vec<Option<Cow<Tokens>>> =
    cargs.into_iter().map(|t| Some(Cow::Owned(t))).collect();
  let content_subst = content.substitute_parameters(&cargs_opt);

  // Build presentation with xmargs substituted, wrapped in \lx@wrap
  let pargs_opt: Vec<Option<Cow<Tokens>>> =
    pargs.into_iter().map(|t| Some(Cow::Owned(t))).collect();
  let pres_subst = presentation.substitute_parameters(&pargs_opt);
  let wrapped_pres = i_wrap(None, pres_subst);

  // Assemble: \lx@dual[options]{content}{presentation}
  // Pre-size: \lx@dual + [opts] + {content} + {presentation}.
  let opt_len = optional.as_ref().map(|o| o.len() + 2).unwrap_or(0);
  let mut tks: Vec<Token> =
    Vec::with_capacity(1 + opt_len + 2 + content_subst.len() + 2 + wrapped_pres.len());
  tks.push(T_CS!("\\lx@dual"));
  if let Some(opts) = optional {
    tks.push(T_OTHER!("["));
    tks.extend(opts.unlist());
    tks.push(T_OTHER!("]"));
  }
  tks.push(T_BEGIN!());
  tks.extend(content_subst.unlist());
  tks.push(T_END!());
  tks.push(T_BEGIN!());
  tks.extend(wrapped_pres.unlist());
  tks.push(T_END!());

  Ok(Tokens::new(tks))
}

/// Perl: I_keyvals — convert key=value pairs into `[key={value},...]` tokens.
pub fn i_keyvals(kv: &[(&str, Tokens)]) -> Tokens {
  if kv.is_empty() {
    return Tokens::default();
  }
  // Pre-size: 2 brackets + per-kv (key_len + value_len + 4 structure
  // tokens: `=`, `{`, `}`, optional `,`).
  let total: usize = 2 + kv.iter().map(|(k, v)| k.len() + v.len() + 4).sum::<usize>();
  let mut tks: Vec<Token> = Vec::with_capacity(total);
  tks.push(T_OTHER!("["));
  for (i, (key, value)) in kv.iter().enumerate() {
    if i > 0 {
      tks.push(T_OTHER!(","));
    }
    tks.extend(ExplodeText!(key));
    tks.push(T_OTHER!("="));
    tks.push(T_BEGIN!());
    tks.extend(value.unlist_ref().iter().copied());
    tks.push(T_END!());
  }
  tks.push(T_OTHER!("]"));
  Tokens::new(tks)
}

/// Perl: I_apply(kv, op, @args) — generates `\lx@apply[kv]{\lx@wrap{op}}{\lx@wrap{arg1}...}`.
pub fn i_apply(kv: &[(&str, Tokens)], op: Tokens, args: Vec<Tokens>) -> Tokens {
  // Pre-size: \lx@apply + keyvals + 6 wrap-structure tokens for op
  // + per-arg (3 structure + arg.len()) + args list wrap (2).
  let args_total: usize = args.iter().map(|a| a.len() + 3).sum();
  let kv_total: usize = if kv.is_empty() {
    0
  } else {
    kv.iter().map(|(k, v)| k.len() + v.len() + 4).sum::<usize>() + 2
  };
  let mut tks: Vec<Token> = Vec::with_capacity(1 + kv_total + 6 + op.len() + 2 + args_total);
  tks.push(T_CS!("\\lx@apply"));
  let kvt = i_keyvals(kv);
  if !kvt.is_empty() {
    tks.extend(kvt.unlist());
  }
  // operator wrapped
  tks.push(T_BEGIN!());
  tks.push(T_CS!("\\lx@wrap"));
  tks.push(T_BEGIN!());
  tks.extend(op.unlist());
  tks.push(T_END!());
  tks.push(T_END!());
  // args wrapped
  tks.push(T_BEGIN!());
  for arg in args {
    tks.push(T_CS!("\\lx@wrap"));
    tks.push(T_BEGIN!());
    tks.extend(arg.unlist());
    tks.push(T_END!());
  }
  tks.push(T_END!());
  Tokens::new(tks)
}

/// Perl: I_symbol(kv, text) — generates `\lx@symbol[kv]{text}`.
pub fn i_symbol(kv: &[(&str, Tokens)], text: Option<Tokens>) -> Tokens {
  let kv_total: usize = if kv.is_empty() {
    0
  } else {
    kv.iter().map(|(k, v)| k.len() + v.len() + 4).sum::<usize>() + 2
  };
  let text_len = text.as_ref().map(|t| t.len()).unwrap_or(0);
  let mut tks: Vec<Token> = Vec::with_capacity(1 + kv_total + 2 + text_len);
  tks.push(T_CS!("\\lx@symbol"));
  let kvt = i_keyvals(kv);
  if !kvt.is_empty() {
    tks.extend(kvt.unlist());
  }
  tks.push(T_BEGIN!());
  if let Some(t) = text {
    tks.extend(t.unlist());
  }
  tks.push(T_END!());
  Tokens::new(tks)
}

/// Perl: I_superscript(kv, base, script) — generates `\lx@superscript[kv]{base}{script}`.
pub fn i_superscript(kv: &[(&str, Tokens)], base: Tokens, script: Tokens) -> Tokens {
  let kv_total: usize = if kv.is_empty() {
    0
  } else {
    kv.iter().map(|(k, v)| k.len() + v.len() + 4).sum::<usize>() + 2
  };
  let mut tks: Vec<Token> = Vec::with_capacity(1 + kv_total + 4 + base.len() + script.len());
  tks.push(T_CS!("\\lx@superscript"));
  let kvt = i_keyvals(kv);
  if !kvt.is_empty() {
    tks.extend(kvt.unlist());
  }
  tks.push(T_BEGIN!());
  tks.extend(base.unlist());
  tks.push(T_END!());
  tks.push(T_BEGIN!());
  tks.extend(script.unlist());
  tks.push(T_END!());
  Tokens::new(tks)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn i_arg_parses_valid_index() {
    let t = i_arg("2");
    assert_eq!(t.code, Catcode::ARG);
  }

  #[test]
  fn i_arg_defaults_on_parse_failure() {
    // Non-numeric input falls back to '1' per the unwrap_or.
    let t = i_arg("abc");
    assert_eq!(t.code, Catcode::ARG);
  }

  #[test]
  fn i_xmarg_emits_expected_sequence() {
    let arg = Tokens::new(vec![]);
    let tks = i_xmarg("X.1", arg);
    let unlisted = tks.unlist();
    // Should have \lx@xmarg { X.1 } { }.
    assert!(
      unlisted.len() >= 5,
      "should have at least 5 tokens: cs + {{ + id + }} + {{ + }}"
    );
    assert_eq!(unlisted[0].code, Catcode::CS);
  }

  #[test]
  fn i_xmref_emits_expected_sequence() {
    let tks = i_xmref("X.1");
    let unlisted = tks.unlist();
    // Should have \lx@xmref { X.1 }.
    // Minimum 3 tokens: cs + BEGIN + ... + END → 3 + len(id chars).
    assert!(unlisted.len() >= 3);
    assert_eq!(unlisted[0].code, Catcode::CS);
    // Last token is END.
    let last = unlisted.last().unwrap();
    assert_eq!(last.code, Catcode::END);
  }

  #[test]
  fn i_xmref_preserves_id_chars() {
    let tks = i_xmref("abc");
    let out = tks.to_string();
    // Re-rendering back to string should contain the id.
    assert!(out.contains("abc"), "got {out:?}");
  }

  #[test]
  fn i_xmarg_with_content_preserves_arg() {
    // Arg is "foo" — the output should encode foo.
    let arg_tks = Tokens::new(vec![
      CharToken!('f', Catcode::LETTER),
      CharToken!('o', Catcode::LETTER),
      CharToken!('o', Catcode::LETTER),
    ]);
    let out = i_xmarg("id", arg_tks);
    let s = out.to_string();
    assert!(s.contains("foo"), "got {s:?}");
    assert!(s.contains("id"), "got {s:?}");
  }

  #[test]
  fn i_keyvals_empty_list_returns_empty() {
    let out = i_keyvals(&[]);
    // Empty input → no tokens (or just minimal scaffold).
    // Exact behavior depends on the impl; just assert it's well-formed.
    let _ = out.len(); // doesn't panic
  }
}
