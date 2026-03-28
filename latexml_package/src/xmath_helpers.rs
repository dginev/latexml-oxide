//! XMath helper functions for building XMDual token streams.
//! Port of Perl's I_dual, I_arg, I_xmarg, I_xmref, I_wrap from Base_XMath.pool.ltxml.
//!
//! These functions generate token lists that expand into semantic math markup.
//! Used by physics.sty, diffcoeff.sty, and other packages that need XMDual structures.

use crate::prelude::*;

/// Perl: getXMArgID — step @lx@xmarg counter, return its value as a string.
pub fn get_xm_arg_id() -> Result<String> {
  step_counter("@lx@xmarg", false)?;
  let val = state::lookup_register("\\c@@lx@xmarg", Vec::new())?
    .map(|rv| rv.value_of())
    .unwrap_or(0);
  Ok(val.to_string())
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
/// - `keyvals`: Optional HashMap of key-value attributes (reversion, meaning, role, etc.)
/// - `content`: Token template for content branch (with #1, #2, ... parameter refs)
/// - `presentation`: Token template for presentation branch (with #1, #2, ... parameter refs)
/// - `args`: The actual argument token lists to be shared between branches
pub fn i_dual(
  keyvals: Option<&std::collections::HashMap<String, String>>,
  content: Tokens,
  presentation: Tokens,
  args: Vec<Tokens>,
) -> Result<Tokens> {
  let mut revargs: Vec<Tokens> = Vec::new();
  let mut pargs: Vec<Tokens> = Vec::new();
  let mut cargs: Vec<Tokens> = Vec::new();

  for arg in args {
    let id = get_xm_arg_id()?;
    revargs.push(Tokens::new(vec![i_arg(&id)]));
    pargs.push(i_xmarg(&id, arg));
    cargs.push(i_xmref(&id));
  }

  // Build optional keyvals string
  let optional = if let Some(kv) = keyvals {
    let mut opts = Vec::new();
    for (key, value) in kv {
      if !opts.is_empty() {
        opts.push(T_OTHER!(","));
      }
      opts.extend(ExplodeText!(key));
      opts.push(T_OTHER!("="));
      opts.push(T_BEGIN!());
      // For reversion keys, substitute parameter refs
      let value_toks = Tokenize!(value);
      if key.ends_with("reversion") {
        let rev_opt: Vec<Option<Cow<Tokens>>> = revargs.iter().map(|t| Some(Cow::Borrowed(t))).collect();
        opts.extend(value_toks.substitute_parameters(&rev_opt).unlist());
      } else {
        opts.extend(value_toks.unlist());
      }
      opts.push(T_END!());
    }
    Some(Tokens::new(opts))
  } else {
    None
  };

  // Build content with xmrefs substituted
  let cargs_opt: Vec<Option<Cow<Tokens>>> = cargs.into_iter().map(|t| Some(Cow::Owned(t))).collect();
  let content_subst = content.substitute_parameters(&cargs_opt);

  // Build presentation with xmargs substituted, wrapped in \lx@wrap
  let pargs_opt: Vec<Option<Cow<Tokens>>> = pargs.into_iter().map(|t| Some(Cow::Owned(t))).collect();
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
