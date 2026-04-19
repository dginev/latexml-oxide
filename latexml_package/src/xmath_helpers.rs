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
  let current = state::lookup_register("\\c@@lx@xmarg", Vec::new())?
    .map(|rv| rv.value_of())
    .unwrap_or(0);
  let next = current + 1;
  state::assign_register(
    "\\c@@lx@xmarg",
    latexml_core::common::number::Number::new(next).into(),
    Some(state::Scope::Global),
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
  let mut tks = vec![T_CS!("\\lx@xmarg"), T_BEGIN!()];
  tks.extend(ExplodeText!(id));
  tks.push(T_END!());
  tks.push(T_BEGIN!());
  tks.extend(arg.unlist());
  tks.push(T_END!());
  Tokens::new(tks)
}

/// Perl: I_xmref(id) — generates `\lx@xmref{id}` tokens.
pub fn i_xmref(id: &str) -> Tokens {
  let mut tks = vec![T_CS!("\\lx@xmref"), T_BEGIN!()];
  tks.extend(ExplodeText!(id));
  tks.push(T_END!());
  Tokens::new(tks)
}

/// Perl: I_wrap(keyvals, content) — generates `\lx@wrap[keyvals]{content}` tokens.
pub fn i_wrap(keyvals: Option<Tokens>, content: Tokens) -> Tokens {
  let mut tks = vec![T_CS!("\\lx@wrap")];
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
  let mut orig_args: Vec<Tokens> = Vec::new();
  let mut pargs: Vec<Tokens> = Vec::new();
  let mut cargs: Vec<Tokens> = Vec::new();

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
    state::push_value("PENDING_DUAL_REVERSION", Stored::Tokens(rev))?;
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
  let mut tks = vec![T_CS!("\\lx@dual")];
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
  let mut tks = vec![T_OTHER!("[")];
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
  let mut tks = vec![T_CS!("\\lx@apply")];
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
  let mut tks = vec![T_CS!("\\lx@symbol")];
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
  let mut tks = vec![T_CS!("\\lx@superscript")];
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
