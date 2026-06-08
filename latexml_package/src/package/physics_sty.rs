//! physics.sty — semantic physics macros
//! Perl: physics.sty.ltxml (729 lines)
//!
//! Faithful port using I_dual infrastructure for semantic markup (XMDual).
use crate::prelude::*;
use crate::xmath_helpers::*;

/// Perl: %physics_delimiters
fn physics_delimiters(s: &str) -> Option<&'static str> {
  match s {
    "(" => Some(")"),
    "[" => Some("]"),
    "|" => Some("|"),
    _ => None,
  }
}

/// Perl: phys_readSize — returns (no_stretch, size_token)
/// 0 = no stretch (*), 1 = stretchy (\left/\right), Token = specific size
fn phys_read_size() -> Result<(bool, Option<Token>)> {
  let mut no_stretch = false;
  let mut size_tok: Option<Token> = None;
  let next = gullet::read_token()?;
  let mut pending = next;
  if let Some(ref t) = pending {
    if t.text == pin!("*") {
      no_stretch = true;
      pending = gullet::read_token()?;
    }
  }
  if let Some(ref t) = pending {
    let s = t.to_string();
    if s.starts_with('\\') && (s == "\\big" || s == "\\Big" || s == "\\bigg" || s == "\\Bigg") {
      size_tok = Some(*t);
      pending = gullet::read_token()?;
    }
  }
  if let Some(t) = pending {
    gullet::unread_one(t);
  }
  Ok((no_stretch, size_tok))
}

/// Perl: phys_revSize — reversion tokens for size
fn phys_rev_size(no_stretch: bool, size_tok: &Option<Token>) -> Vec<Token> {
  if let Some(ref sz) = size_tok {
    vec![*sz]
  } else if no_stretch {
    vec![T_OTHER!("*")]
  } else {
    vec![]
  }
}

/// Perl: phys_open — opening fence with sizing
fn phys_open(no_stretch: bool, size_tok: &Option<Token>, delim: Tokens) -> Tokens {
  if delim.is_empty() {
    return Tokens::default();
  }
  if let Some(sz) = size_tok {
    Tokens::new([vec![*sz], delim.unlist()].concat())
  } else if no_stretch {
    delim
  } else {
    Tokens::new([vec![T_CS!("\\left")], delim.unlist()].concat())
  }
}

/// Perl: phys_mid — middle fence with sizing
fn phys_mid(no_stretch: bool, size_tok: &Option<Token>, delim: Tokens) -> Tokens {
  if delim.is_empty() {
    return Tokens::default();
  }
  if let Some(sz) = size_tok {
    Tokens::new([vec![*sz], delim.unlist()].concat())
  } else if no_stretch {
    delim
  } else {
    Tokens::new([vec![T_CS!("\\middle")], delim.unlist()].concat())
  }
}

/// Perl: phys_close — closing fence with sizing
fn phys_close(no_stretch: bool, size_tok: &Option<Token>, delim: Tokens) -> Tokens {
  if delim.is_empty() {
    return Tokens::default();
  }
  if let Some(sz) = size_tok {
    Tokens::new([vec![*sz], delim.unlist()].concat())
  } else if no_stretch {
    delim
  } else {
    Tokens::new([vec![T_CS!("\\right")], delim.unlist()].concat())
  }
}

/// Perl: phys_readArg — read TeX {} arg or delimited arg.
/// Returns (arg, open_token, close_token).
fn phys_read_arg(
  required: bool,
  delimiters: fn(&str) -> Option<&'static str>,
) -> Result<(Option<Tokens>, Option<Token>, Option<Token>)> {
  let next = gullet::read_token()?;
  if let Some(ref t) = next {
    if t.get_catcode() == Catcode::BEGIN {
      gullet::unread_one(*t);
      let arg = gullet::read_arg(ExpansionLevel::Off)?;
      return Ok((Some(arg), None, None));
    }
    let s = t.to_string();
    if let Some(close_str) = delimiters(&s) {
      let open_tok = *t;
      let close_tok = Token::from(close_str);
      let mut tokens = Vec::new();
      let mut level = 1i32;
      let mut _blevel = 0i32;
      while let Some(tok) = gullet::read_token()? {
        let cc = tok.get_catcode();
        if cc == Catcode::END {
          _blevel -= 1;
          tokens.push(tok);
        } else if cc == Catcode::BEGIN {
          _blevel += 1;
          tokens.push(tok);
        } else if tok == close_tok {
          level -= 1;
          if level == 0 {
            break;
          }
          tokens.push(tok);
        } else if tok == open_tok {
          level += 1;
          tokens.push(tok);
        } else {
          tokens.push(tok);
        }
      }
      return Ok((Some(Tokens::new(tokens)), Some(open_tok), Some(close_tok)));
    }
    gullet::unread_one(*t);
  }
  if required {
    // Error: expected open delimiter
  }
  Ok((None, None, None))
}

/// Perl: phys_readArg with no delimiters — just TeX {} arg
fn phys_read_arg_tex() -> Result<Option<Tokens>> {
  let next = gullet::read_token()?;
  if let Some(ref t) = next {
    if t.get_catcode() == Catcode::BEGIN {
      gullet::unread_one(*t);
      let arg = gullet::read_arg(ExpansionLevel::Off)?;
      return Ok(Some(arg));
    }
    gullet::unread_one(*t);
  }
  Ok(None)
}

/// Perl: phys_revArg — reversion tokens for arg
fn phys_rev_arg(arg_ref: Tokens, open: &Option<Token>, close: &Option<Token>) -> Tokens {
  if let Some(o) = open {
    Tokens::new([vec![*o], arg_ref.unlist(), vec![*close.as_ref().unwrap()]].concat())
  } else {
    Tokens::new([vec![T_BEGIN!()], arg_ref.unlist(), vec![T_END!()]].concat())
  }
}

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("amsmath");

  //======================================================================
  // Automatic bracing — Perl physics.sty.ltxml L132-900
  //======================================================================
  //
  // **Umbrella WISDOM #44 intentional divergence** — applies to the
  // entire physics math-macro family below (not just \quantity):
  //
  //   \quantity, \lx@physics@fenced, \lx@physics@fencedII,
  //   \lx@physics@operator, \lx@physics@operatorP, \lx@physics@ReIm,
  //   \qqtext, \qcc, \lx@physics@diff, \lx@physics@deriv,
  //   \ket, \bra, \lx@physics@qm@product, \expectationvalue,
  //   \matrixelement
  //   + matrix family: \identitymatrix, \xmatrix, \paulimatrix,
  //   \diagonalmatrix, \antidiagonalmatrix, \lx@physics@mat
  //   (~22 entries total; DP-audit flags all of them intentionally)
  //
  // Perl defines each as a DefConstructor that runs custom digest-
  // time size-reading + delimiter-reading, then emits the fenced
  // XMApp/XMDual shape directly via `DefMath`-style XML template.
  //
  // Rust ports each as a DefPrimitive that does the size + delimiter
  // read manually via the `phys_read_size` / `phys_read_arg` /
  // `gullet::read_arg` helpers (all defined in this file), composes
  // the presentation + content + reversion tokens, and `gullet::unread`s
  // the result for normal math-parser absorption.
  //
  // Rationale (WISDOM #44): the Rust-native gullet API gives finer
  // control over the multi-token lookahead these physics macros need
  // (TeX-level OptionalMatch of size modifiers like `\big`/`\Big`,
  // delimiter pair peeking, XMDual reversion construction), and the
  // unread-presentation path produces the same observable XMApp /
  // XMDual shape as Perl's direct-emit DefConstructor. Kind-wise the
  // audit counts ~16 DefConstructor → DefPrimitive flips in this
  // file, all under the same rationale. Individual entries don't
  // re-carry the WISDOM #44 tag to avoid comment noise.

  // DefMacro (expansion-time), not DefPrimitive — it reads a delimited `(…)`/`[…]`
  // body via `phys_read_arg`, so inside an alignment a digestion-time primitive would
  // let the column scan grab the body's `&`/`\\` first (same WISDOM #44 alignment bug
  // fixed for `\mqty` and `\lx@physics@operatorP`). Return the dual.
  DefMacro!("\\quantity", {
    let (no_stretch, size_tok) = phys_read_size()?;
    let (arg, open, close) = phys_read_arg(true, physics_delimiters)?;
    let arg = arg.unwrap_or_default();
    let arg1 = Tokens::new(vec![i_arg("1")]);

    // Build reversion
    let mut rev_tks: Vec<Token> = vec![T_CS!("\\quantity")];
    rev_tks.extend(phys_rev_size(no_stretch, &size_tok));
    rev_tks.extend(phys_rev_arg(arg1, &open, &close).unlist());
    let reversion = Tokens::new(rev_tks);

    // Content: just the argument (no apparent semantics)
    let content = Tokens::new(vec![i_arg("1")]);

    // Presentation: open + arg + close
    let open_tks = open.map(|t| Tokenize!(&t.to_string()))
      .unwrap_or_else(|| Tokenize!("\\lbrace"));
    let close_tks = close.map(|t| Tokenize!(&t.to_string()))
      .unwrap_or_else(|| Tokenize!("\\rbrace"));
    let mut pres = Vec::new();
    pres.extend(phys_open(no_stretch, &size_tok, open_tks).unlist());
    pres.push(i_arg("1"));
    pres.extend(phys_close(no_stretch, &size_tok, close_tks).unlist());
    let presentation = Tokens::new(pres);

    let result = i_dual(
      &[("reversion", reversion)],
      content, presentation, vec![arg],
    )?;
    Ok(result)
  });
  Let!("\\qty", "\\quantity");

  // Perl: \lx@physics@fenced — fenced stuff with optional semantics
  DefPrimitive!("\\lx@physics@fenced{}{}{}{}{}", sub[(cs, semantic, function, open, close)] {
    let cs_tks = cs;
    let semantic_str = semantic.to_string();
    let semantic_opt = if semantic_str.is_empty() { None } else { Some(semantic_str.as_str()) };
    let function_tks = function;
    let open_tks = open;
    let close_tks = close;
    let (no_stretch, size_tok) = phys_read_size()?;
    let arg = gullet::read_arg(ExpansionLevel::Off)?;
    let arg1 = Tokens::new(vec![i_arg("1")]);

    // Reversion: \cs size {#1}
    let mut rev_tks: Vec<Token> = Vec::new();
    rev_tks.extend(cs_tks.unlist());
    rev_tks.extend(phys_rev_size(no_stretch, &size_tok));
    rev_tks.extend(phys_rev_arg(arg1.clone(), &None, &None).unlist());
    let reversion = Tokens::new(rev_tks);

    // Content
    let content = if let Some(sem) = semantic_opt {
      i_apply(&[], i_symbol(&[("meaning", Tokenize!(sem))], None), vec![arg1])
    } else {
      arg1
    };

    // Presentation: [function] open #1 close
    let mut pres = Vec::new();
    if !function_tks.is_empty() {
      pres.extend(function_tks.unlist());
    }
    pres.extend(phys_open(no_stretch, &size_tok, open_tks).unlist());
    pres.push(i_arg("1"));
    pres.extend(phys_close(no_stretch, &size_tok, close_tks).unlist());
    let presentation = Tokens::new(pres);

    let result = i_dual(
      &[("reversion", reversion)],
      content, presentation, vec![arg],
    )?;
    gullet::unread(result);
  });

  DefMacro!("\\pqty", "\\lx@physics@fenced{\\pqty}{}{}{(}{)}");
  DefMacro!("\\bqty", "\\lx@physics@fenced{\\bqty}{}{}{[}{]}");
  DefMacro!("\\vqty", "\\lx@physics@fenced{\\vqty}{}{}{\u{007C}}{\u{007C}}");
  DefMacro!("\\Bqty", "\\lx@physics@fenced{\\Bqty}{}{}{\\{}{\\}}");
  DefMacro!("\\absolutevalue", "\\lx@physics@fenced{\\absolutevalue}{absolute-value}{}{\\vert}{\\vert}");
  DefMacro!("\\norm", "\\lx@physics@fenced{\\norm}{norm}{}{\\|}{\\|}");
  Let!("\\abs", "\\absolutevalue");

  // Perl: \evaluated — fenced arg, then read sub/superscript limits.
  // Perl kind: DefMacro with gullet-level sub body returning I_dual(...).
  // Rust kind: DefPrimitive with imperative stomach-level body — same
  // structured XMDual output, parsed at digest time instead of expansion
  // time. WISDOM #44 (not #41 — #41 covers math-mode ParameterType
  // adaptations; the kind shift here is the expandability difference).
  // Practically safe because physics notation is math-mode stomach-time
  // only; no call site is known to wrap `\evaluated` in `\edef`.
  // WISDOM #44 verified 2026-04-23: zero `\edef`/`\ifx`/`\expandafter`
  // uses of `\evaluated` across LaTeXML/lib + ar5iv-bindings.
  // DefMacro (expansion-time) — reads a delimited `(…|`/`[…|` arg via `phys_read_arg`
  // (then optional `_`/`^` limits); same WISDOM #44 alignment reason as `\mqty`/
  // `\lx@physics@operatorP`. Return the dual.
  DefMacro!("\\evaluated", {
    let (no_stretch, size_tok) = phys_read_size()?;
    let _c = Token::from("|");
    let (arg, open, close) = phys_read_arg(true, |s| {
      match s {
        "(" | "[" => Some("|"),
        _ => None,
      }
    })?;
    let arg = arg.unwrap_or_default();
    // Read optional sub/superscript
    let mut lower: Option<Tokens> = None;
    let mut upper: Option<Tokens> = None;
    loop {
      let next = gullet::read_token()?;
      if let Some(t) = next {
        if lower.is_none() && t.get_catcode() == Catcode::SUB {
          lower = Some(gullet::read_arg(ExpansionLevel::Off)?);
        } else if upper.is_none() && t.get_catcode() == Catcode::SUPER {
          upper = Some(gullet::read_arg(ExpansionLevel::Off)?);
        } else {
          gullet::unread_one(t);
          break;
        }
      } else { break; }
    }
    let a1 = Tokens::new(vec![i_arg("1")]);
    let mut all_args = vec![arg];
    let mut content_args = vec![a1.clone()];
    let mut rev = vec![T_CS!("\\evaluated")];
    rev.extend(phys_rev_size(no_stretch, &size_tok));
    rev.extend(phys_rev_arg(a1, &open, &close).unlist());

    let mut pres_suffix = Vec::new();
    if let Some(lo) = lower {
      let a2 = Tokens::new(vec![i_arg("2")]);
      all_args.push(lo);
      content_args.push(a2.clone());
      rev.push(T_SUB!());
      rev.extend(phys_rev_arg(a2, &None, &None).unlist());
      pres_suffix.push(T_SUB!());
      pres_suffix.push(T_BEGIN!());
      pres_suffix.push(i_arg("2"));
      pres_suffix.push(T_END!());
    }
    if let Some(up) = upper {
      let an = Tokens::new(vec![i_arg(&(all_args.len() + 1).to_string())]);
      all_args.push(up);
      content_args.push(an.clone());
      rev.push(T_SUPER!());
      rev.extend(phys_rev_arg(an, &None, &None).unlist());
      pres_suffix.push(T_SUPER!());
      pres_suffix.push(T_BEGIN!());
      pres_suffix.push(i_arg(&all_args.len().to_string()));
      pres_suffix.push(T_END!());
    }
    let reversion = Tokens::new(rev);
    let content = i_apply(&[],
      i_symbol(&[("meaning", Tokenize!("evaluated-at"))], None), content_args);
    let open_tks = open.map(|t| Tokenize!(&t.to_string())).unwrap_or_else(|| Tokenize!("."));
    let close_tks = close.map(|_| Tokenize!("|")).unwrap_or_else(|| Tokenize!("|"));
    let mut pres = Vec::new();
    pres.extend(
      i_wrap(None, Tokens::new([
        phys_open(no_stretch, &size_tok, open_tks).unlist(),
        vec![i_arg("1")],
        phys_close(no_stretch, &size_tok, close_tks).unlist(),
      ].concat())).unlist()
    );
    pres.extend(pres_suffix);
    let presentation = Tokens::new(pres);
    let result = i_dual(&[("reversion", reversion)], content, presentation, all_args)?;
    Ok(result)
  });
  Let!("\\eval", "\\evaluated");

  // Perl: \order — O(arg) with meaning=order, function=\ordersymbol
  DefMacro!("\\ordersymbol", r"\mathcal{O}");
  // Intentional — WISDOM #44, see physics umbrella L178.
  DefMacro!("\\order", "\\lx@physics@fenced{\\order}{order}{\\ordersymbol}{(}{)}");

  // Perl: \lx@physics@fencedII — 2-argument fenced
  DefPrimitive!("\\lx@physics@fencedII{}{}{}{}{}", sub[(cs, semantic, _function, open, close)] {
    let cs_tks = cs;
    let semantic_str = semantic.to_string();
    let open_tks = open;
    let close_tks = close;
    let (no_stretch, size_tok) = phys_read_size()?;
    let arg1_tok = gullet::read_arg(ExpansionLevel::Off)?;
    let arg2_tok = gullet::read_arg(ExpansionLevel::Off)?;
    let a1 = Tokens::new(vec![i_arg("1")]);
    let a2 = Tokens::new(vec![i_arg("2")]);

    // Reversion
    let mut rev = Vec::new();
    rev.extend(cs_tks.unlist());
    rev.extend(phys_rev_size(no_stretch, &size_tok));
    rev.extend(phys_rev_arg(a1.clone(), &None, &None).unlist());
    rev.extend(phys_rev_arg(a2.clone(), &None, &None).unlist());
    let reversion = Tokens::new(rev);

    // Content: apply(symbol(meaning), arg1, arg2)
    let content = i_apply(&[], i_symbol(&[("meaning", Tokenize!(&semantic_str))], None),
      vec![a1, a2]);

    // Presentation: open arg1 , arg2 close
    let mut pres = Vec::new();
    pres.extend(phys_open(no_stretch, &size_tok, open_tks).unlist());
    pres.push(i_arg("1"));
    pres.push(T_OTHER!(","));
    pres.push(i_arg("2"));
    pres.extend(phys_close(no_stretch, &size_tok, close_tks).unlist());
    let presentation = Tokens::new(pres);

    let result = i_dual(
      &[("reversion", reversion)],
      content, presentation, vec![arg1_tok, arg2_tok],
    )?;
    gullet::unread(result);
  });

  DefMacro!("\\commutator", "\\lx@physics@fencedII{\\commutator}{commutator}{}{[}{]}");
  DefMacro!("\\anticommutator", "\\lx@physics@fencedII{\\anticommutator}{anticommutator}{}{\\{}{\\}}");
  DefMacro!("\\poissonbracket", "\\lx@physics@fencedII{\\poissonbracket}{poisson-bracket}{}{\\{}{\\}}");
  Let!("\\comm", "\\commutator");
  Let!("\\acomm", "\\anticommutator");
  Let!("\\pb", "\\poissonbracket");

  //======================================================================
  // Vector Notation
  // Perl L229-231: \lx@physics@mathbfit is a DefConstructor with bounded +
  // requireMath and a bold+italic+serif font merge. The starred form of
  // \vectorbold / \vectorarrow / \vectorunit routes through it to render
  // italic vectors; non-starred falls through to \mathbf (upright bold).
  // The prior Rust port collapsed all three to \mathbf, losing the starred
  // italic case entirely.
  DefConstructor!("\\lx@physics@mathbfit{}", "#1",
    bounded => true, require_math => true,
    font => { shape => "italic", family => "serif", series => "bold", forcebold => true },
    reversion => "{\\bf\\it#1}");
  DefMacro!("\\vectorbold OptionalMatch:* {}",
    "\\lx@wrap[role=ID]{\\ifx.#1.\\mathbf{#2}\\else\\lx@physics@mathbfit{#2}\\fi}");
  DefMacro!("\\vectorarrow OptionalMatch:* {}",
    "\\lx@wrap[role=ID]{\\lx@math@overrightarrow{\\ifx.#1.\\mathbf{#2}\\else\\lx@physics@mathbfit{#2}\\fi}}");
  DefMacro!("\\vectorunit OptionalMatch:* {}",
    "\\lx@wrap[role=ID]{\\hat{\\ifx.#1.\\mathbf{#2}\\else\\lx@physics@mathbfit{#2}\\fi}}");
  Let!("\\vb", "\\vectorbold");
  Let!("\\va", "\\vectorarrow");
  Let!("\\vu", "\\vectorunit");

  DefMath!("\\dotproduct", None, "\u{22C5}", role => "MULOP", meaning => "dot-product");
  DefMath!("\\crossproduct", None, "\u{00D7}", role => "MULOP", meaning => "cross-product");
  Let!("\\vdot", "\\dotproduct");
  Let!("\\cross", "\\crossproduct");
  // Intentional — WISDOM #44, see physics umbrella L178.
  Let!("\\cp", "\\crossproduct");

  // Perl: \lx@physics@operator — operator with optional delimited arg
  // DefMacro (expansion-time) — reads a delimited arg via `phys_read_arg`; same
  // WISDOM #44 alignment reason as `\mqty`/`\lx@physics@operatorP`. Return the dual.
  DefMacro!("\\lx@physics@operator{}{}{}", sub[(cs, semantic, function)] {
    let cs_tks = cs;
    let semantic_str = semantic.to_string();
    let function_tks = function;
    let cfunc = i_symbol(&[("meaning", Tokenize!(&semantic_str))], None);
    let (no_stretch, size_tok) = phys_read_size()?;
    let (arg, open, close) = phys_read_arg(false, physics_delimiters)?;

    if let Some(arg_tks) = arg {
      let a1 = Tokens::new(vec![i_arg("1")]);
      // Has argument: apply(cfunc, arg)
      let mut rev = Vec::new();
      rev.extend(cs_tks.unlist());
      rev.extend(phys_rev_size(no_stretch, &size_tok));
      rev.extend(phys_rev_arg(a1.clone(), &open, &close).unlist());
      let reversion = Tokens::new(rev);

      let content = i_apply(&[], cfunc, vec![a1]);
      let open_tks = open.map(|t| Tokenize!(&t.to_string())).unwrap_or_default();
      let close_tks = close.map(|t| Tokenize!(&t.to_string())).unwrap_or_default();
      let mut pres = Vec::new();
      pres.extend(function_tks.unlist());
      pres.extend(phys_open(no_stretch, &size_tok, open_tks).unlist());
      pres.push(i_arg("1"));
      pres.extend(phys_close(no_stretch, &size_tok, close_tks).unlist());
      let presentation = Tokens::new(pres);

      let result = i_dual(
        &[("reversion", reversion)],
        content, presentation, vec![arg_tks],
      )?;
      Ok(result)
    } else {
      // No argument: just the operator symbol
      let result = i_dual(
        &[("role", Tokenize!("OPERATOR")), ("reversion", cs_tks)],
        cfunc, function_tks, vec![],
      )?;
      Ok(result)
    }
  });

  DefMacro!("\\gradient", "\\lx@physics@operator{\\gradient}{gradient}{\\nabla}");
  DefMacro!("\\divergence", "\\lx@physics@operator{\\divergence}{divergence}{\\nabla\\cdot}");
  DefMacro!("\\curl", "\\lx@physics@operator{\\curl}{curl}{\\nabla\\cross}");
  DefMacro!("\\laplacian", "\\lx@physics@operator{\\laplacian}{laplacian}{\\nabla^2}");
  Let!("\\grad", "\\gradient");
  Let!("\\divisionsymbol", "\\div");
  Let!("\\div", "\\divergence");

  //======================================================================
  // Intentional — WISDOM #44, see physics umbrella L178.
  // Operators with power
  // Perl: \lx@physics@operatorP — operator with optional power and paren-delimited arg

  // MUST be a DefMacro (expansion-time), not a DefPrimitive — see WISDOM #44's
  // alignment-exception. This operator reads a delimited/optional argument
  // (`[power]` via `read_optional`, `(arg)` via `phys_read_arg`); as a
  // digestion-time primitive an enclosing eqnarray's column scan saw the `\\`
  // inside that argument BEFORE this code consumed it — so `\tr\big[ A \\ B \big]`
  // (the trace arg straddling an eqnarray row break) leaked its `\\` into the
  // eqnarray → `\lx@begin@alignment … mode-switch to math due to
  // \lx@begin@inline@math` + `equationgroup`-in-`XMath` cascade (witness
  // 2003.02721, 13 errors, Perl 0). Perl's `\lx@physics@operatorP` is a DefMacro
  // (expansion-time), so it grabs the argument first. Return the dual(s) instead
  // of `gullet::unread`.
  DefMacro!("\\lx@physics@operatorP{}{}{}", sub[(cs, semantic, function)] {
    let cs_tks = cs;
    let semantic_str = semantic.to_string();
    let function_tks = function;
    let cfunc = i_symbol(&[("meaning", Tokenize!(&semantic_str))], None);
    let pfunc = function_tks;
    let (no_stretch, size_tok) = phys_read_size()?;
    let power = gullet::read_optional(None)?;
    let (arg, open, close) = phys_read_arg(false, |s| {
      if s == "(" { Some(")") } else { None }
    })?;

    if arg.is_none() {
      // No argument — return bare operator followed by the (re-bracketed) power.
      let result = i_dual(
        &[("reversion", cs_tks)],
        cfunc, pfunc, vec![],
      )?;
      let mut out: Vec<Token> = result.unlist();
      if let Some(ref pwr) = power {
        out.push(T_OTHER!("["));
        out.extend_from_slice(pwr.unlist_ref());
        out.push(T_OTHER!("]"));
      }
      Ok(Tokens::new(out))
    } else {
      let arg_tks = arg.unwrap();
      let a1 = Tokens::new(vec![i_arg("1")]);

      let mut content_op = cfunc.clone();
      let mut pres_func = pfunc.clone();
      let mut all_args = vec![arg_tks];
      let mut rev = Vec::new();
      rev.extend(cs_tks.unlist());
      rev.extend(phys_rev_size(no_stretch, &size_tok));

      if let Some(pwr) = power {
        // With power: cfunc^power applied to arg
        let a2 = Tokens::new(vec![i_arg("2")]);
        content_op = i_apply(&[], i_symbol(&[("meaning", Tokenize!("power"))], None),
          vec![cfunc, a2.clone()]);
        pres_func = i_superscript(
          &[("role", Tokenize!("OPFUNCTION"))],
          pfunc, a2);
        rev.push(T_OTHER!("["));
        rev.push(i_arg("2"));
        rev.push(T_OTHER!("]"));
        all_args.push(pwr);
      }

      rev.extend(phys_rev_arg(a1.clone(), &open, &close).unlist());
      let reversion = Tokens::new(rev);
      let content = i_apply(&[], content_op, vec![a1]);

      let open_tks = open.map(|t| Tokenize!(&t.to_string())).unwrap_or_else(|| Tokenize!("("));
      let close_tks = close.map(|t| Tokenize!(&t.to_string())).unwrap_or_else(|| Tokenize!(")"));
      let mut pres = Vec::new();
      pres.extend(pres_func.unlist());
      pres.extend(phys_open(no_stretch, &size_tok, open_tks).unlist());
      pres.push(i_arg("1"));
      pres.extend(phys_close(no_stretch, &size_tok, close_tks).unlist());
      let presentation = Tokens::new(pres);

      let result = i_dual(
        &[("reversion", reversion)],
        content, presentation, all_args,
      )?;
      Ok(result)
    }
  });

  // All trig/math operators via \lx@physics@operatorP
  // Perl: loops through @operators list
  DefMacro!("\\sine", "\\lx@physics@operatorP{\\sin}{sine}{\\operatorname{sin}}");
  DefMacro!("\\cosine", "\\lx@physics@operatorP{\\cos}{cosine}{\\operatorname{cos}}");
  DefMacro!("\\tangent", "\\lx@physics@operatorP{\\tan}{tangent}{\\operatorname{tan}}");
  DefMacro!("\\cosecant", "\\lx@physics@operatorP{\\csc}{cosecant}{\\operatorname{csc}}");
  DefMacro!("\\secant", "\\lx@physics@operatorP{\\sec}{secant}{\\operatorname{sec}}");
  DefMacro!("\\cotangent", "\\lx@physics@operatorP{\\cot}{cotangent}{\\operatorname{cot}}");
  DefMacro!("\\hypsine", "\\lx@physics@operatorP{\\sinh}{hypsine}{\\operatorname{sinh}}");
  DefMacro!("\\hypcosine", "\\lx@physics@operatorP{\\cosh}{hypcosine}{\\operatorname{cosh}}");
  DefMacro!("\\hyptangent", "\\lx@physics@operatorP{\\tanh}{hyptangent}{\\operatorname{tanh}}");
  DefMacro!("\\arcsine", "\\lx@physics@operatorP{\\arcsin}{arcsine}{\\operatorname{arcsin}}");
  DefMacro!("\\arccosine", "\\lx@physics@operatorP{\\arccos}{arccosine}{\\operatorname{arccos}}");
  DefMacro!("\\arctangent", "\\lx@physics@operatorP{\\arctan}{arctangent}{\\operatorname{arctan}}");
  DefMacro!("\\exponential", "\\lx@physics@operatorP{\\exp}{exponential}{\\operatorname{exp}}");
  DefMacro!("\\logarithm", "\\lx@physics@operatorP{\\log}{logarithm}{\\operatorname{log}}");
  DefMacro!("\\naturallogarithm", "\\lx@physics@operatorP{\\ln}{natural-logarithm}{\\operatorname{ln}}");
  DefMacro!("\\determinant", "\\lx@physics@operatorP{\\det}{determinant}{\\operatorname{det}}");
  DefMacro!("\\Probability", "\\lx@physics@operatorP{\\Pr}{probability}{\\operatorname{Pr}}");

  // Operator names ported from Perl long→short mapping
  DefMacro!("\\sin", "\\lx@physics@operatorP{\\sin}{sine}{\\operatorname{sin}}");
  DefMacro!("\\cos", "\\lx@physics@operatorP{\\cos}{cosine}{\\operatorname{cos}}");
  DefMacro!("\\tan", "\\lx@physics@operatorP{\\tan}{tangent}{\\operatorname{tan}}");
  DefMacro!("\\csc", "\\lx@physics@operatorP{\\csc}{cosecant}{\\operatorname{csc}}");
  DefMacro!("\\sec", "\\lx@physics@operatorP{\\sec}{secant}{\\operatorname{sec}}");
  DefMacro!("\\cot", "\\lx@physics@operatorP{\\cot}{cotangent}{\\operatorname{cot}}");
  DefMacro!("\\sinh", "\\lx@physics@operatorP{\\sinh}{hyperbolic-sine}{\\operatorname{sinh}}");
  DefMacro!("\\cosh", "\\lx@physics@operatorP{\\cosh}{hyperbolic-cosine}{\\operatorname{cosh}}");
  DefMacro!("\\tanh", "\\lx@physics@operatorP{\\tanh}{hyperbolic-tangent}{\\operatorname{tanh}}");
  DefMacro!("\\arcsin", "\\lx@physics@operatorP{\\arcsin}{arcsine}{\\operatorname{arcsin}}");
  DefMacro!("\\arccos", "\\lx@physics@operatorP{\\arccos}{arccosine}{\\operatorname{arccos}}");
  DefMacro!("\\arctan", "\\lx@physics@operatorP{\\arctan}{arctangent}{\\operatorname{arctan}}");
  DefMacro!("\\exp", "\\lx@physics@operatorP{\\exp}{exponential}{\\operatorname{exp}}");
  DefMacro!("\\log", "\\lx@physics@operatorP{\\log}{logarithm}{\\operatorname{log}}");
  DefMacro!("\\ln", "\\lx@physics@operatorP{\\ln}{natural-logarithm}{\\operatorname{ln}}");
  DefMacro!("\\det", "\\lx@physics@operatorP{\\det}{determinant}{\\operatorname{det}}");
  DefMacro!("\\Pr", "\\lx@physics@operatorP{\\Pr}{probability}{\\operatorname{Pr}}");

  // Let long names point to short names (Perl: Let('\sine', '\sin') etc.)
  Let!("\\asin", "\\arcsin");
  Let!("\\acos", "\\arccos");
  Let!("\\atan", "\\arctan");
  Let!("\\asine", "\\arcsin");
  Let!("\\acosine", "\\arccos");
  Let!("\\atangent", "\\arctan");

  DefMacro!("\\arccsc", "\\lx@physics@operatorP{\\arccsc}{arccosecant}{\\operatorname{arccsc}}");
  DefMacro!("\\arcsec", "\\lx@physics@operatorP{\\arcsec}{arcsecant}{\\operatorname{arcsec}}");
  DefMacro!("\\arccot", "\\lx@physics@operatorP{\\arccot}{arccotangent}{\\operatorname{arccot}}");
  DefMacro!("\\csch", "\\lx@physics@operatorP{\\csch}{hyperbolic-cosecant}{\\operatorname{csch}}");
  DefMacro!("\\sech", "\\lx@physics@operatorP{\\sech}{hyperbolic-secant}{\\operatorname{sech}}");
  Let!("\\hypcosecant", "\\csch");
  Let!("\\hypsecant", "\\sech");
  DefMacro!("\\hypcotangent", "\\lx@physics@operatorP{\\coth}{hyperbolic-cotangent}{\\operatorname{coth}}");
  Let!("\\acsc", "\\arccsc");
  Let!("\\asec", "\\arcsec");
  Let!("\\acot", "\\arccot");
  Let!("\\arccosecant", "\\arccsc");
  Let!("\\arcsecant", "\\arcsec");
  Let!("\\arccotangent", "\\arccot");
  Let!("\\acosecant", "\\arccsc");
  Let!("\\asecant", "\\arcsec");
  Let!("\\acotangent", "\\arccot");

  DefMacro!("\\trace", "\\lx@physics@operatorP{\\tr}{trace}{\\operatorname{tr}}");
  DefMacro!("\\Trace", "\\lx@physics@operatorP{\\Tr}{trace}{\\operatorname{Tr}}");
  DefMacro!("\\rank", "\\lx@physics@operatorP{\\rank}{rank}{\\operatorname{rank}}");
  DefMacro!("\\erf", "\\lx@physics@operatorP{\\erf}{error-function}{\\operatorname{erf}}");
  DefMacro!("\\Res", "\\lx@physics@operatorP{\\Res}{residue}{\\operatorname{Res}}");
  DefMacro!("\\principalvalue", "\\lx@physics@operatorP{\\principalvalue}{principal-value}{\\mathcal{P}}");
  DefMacro!("\\PV", "\\lx@physics@operatorP{\\PV}{principal-value}{\\operatorname{P.V.}}");
  Let!("\\tr", "\\trace");
  Let!("\\Tr", "\\Trace");
  // Intentional — WISDOM #44, see physics umbrella L178.
  Let!("\\pv", "\\principalvalue");

  // Perl: \lx@physics@ReIm — Re/Im with optional braced arg
  DefPrimitive!("\\lx@physics@ReIm{}{}{}{}", sub[(cs, semantic, raw, function)] {
    let cs_tks = cs;
    let semantic_str = semantic.to_string();
    let raw_tks = raw;
    let function_tks = function;
    let (no_stretch, size_tok) = phys_read_size()?;
    let cfunc = i_symbol(&[("meaning", Tokenize!(&semantic_str))], None);
    let arg = phys_read_arg_tex()?;

    if let Some(arg_tks) = arg {
      let a1 = Tokens::new(vec![i_arg("1")]);
      let mut rev = Vec::new();
      rev.extend(cs_tks.unlist());
      rev.extend(phys_rev_size(no_stretch, &size_tok));
      rev.extend(phys_rev_arg(a1.clone(), &None, &None).unlist());
      let reversion = Tokens::new(rev);

      let content = i_apply(&[], cfunc, vec![a1]);
      let mut pres = Vec::new();
      pres.extend(function_tks.unlist());
      pres.extend(phys_open(no_stretch, &size_tok, Tokenize!("\\lbrace")).unlist());
      pres.push(i_arg("1"));
      pres.extend(phys_close(no_stretch, &size_tok, Tokenize!("\\rbrace")).unlist());
      let presentation = Tokens::new(pres);

      let result = i_dual(&[("reversion", reversion)], content, presentation, vec![arg_tks])?;
      gullet::unread(result);
    } else {
      // Bare — just the operator symbol
      let result = i_dual(
        &[("role", Tokenize!("OPERATOR")), ("reversion", raw_tks)],
        cfunc, function_tks, vec![])?;
      gullet::unread(result);
    }
  });

  // Save old \Re and \Im BEFORE redefining (Perl: Let at L329-330, before DefMacro at L349-350)
  Let!("\\real", "\\Re");
  Let!("\\imaginary", "\\Im");

  DefMacro!("\\Re", "\\lx@physics@ReIm{\\Re}{real-part}{\\real}{\\operatorname{Re}}");
  DefMacro!("\\Im", "\\lx@physics@ReIm{\\Im}{imaginary-part}{\\imaginary}{\\operatorname{Im}}");

  //======================================================================
  // Intentional — WISDOM #44, see physics umbrella L178.
  // Quick quad text
  // Perl: OptionalMatch:* — * means no leading \quad
  // \mbox is used instead of \text for proper text mode handling
  DefPrimitive!("\\qqtext", {
    let star = gullet::read_match(&[&Tokenize!("*")])?.is_some();
    let arg = gullet::read_arg(ExpansionLevel::Off)?;
    let mut tks = Vec::new();
    if !star { tks.push(T_CS!("\\quad")); }
    tks.push(T_CS!("\\mbox"));
    tks.push(T_BEGIN!());
    tks.extend(arg.unlist());
    tks.push(T_END!());
    tks.push(T_CS!("\\quad"));
  // Intentional — WISDOM #44, see physics umbrella L178.
    gullet::unread(Tokens::new(tks));
  });
  DefMacro!("\\qcomma", r",\quad");
  DefPrimitive!("\\qcc", {
    let star = gullet::read_match(&[&Tokenize!("*")])?.is_some();
    let mut tks = Vec::new();
    if !star { tks.push(T_CS!("\\quad")); }
    tks.push(T_CS!("\\mbox"));
    tks.push(T_BEGIN!());
    tks.extend(Tokenize!("c.c.").unlist());
    tks.push(T_END!());
    tks.push(T_CS!("\\quad"));
    gullet::unread(Tokens::new(tks));
  });
  Let!("\\qq", "\\qqtext");
  Let!("\\qc", "\\qcomma");
  // Perl: foreach word, DefMacroI('\q'.$word, 'OptionalMatch:*', '\mbox{\ifx.#1.\quad\fi'.$word.'\quad}')
  DefMacro!("\\qif", r"\mbox{\quad if\quad}");
  DefMacro!("\\qthen", r"\mbox{\quad then\quad}");
  DefMacro!("\\qelse", r"\mbox{\quad else\quad}");
  DefMacro!("\\qotherwise", r"\mbox{\quad otherwise\quad}");
  DefMacro!("\\qunless", r"\mbox{\quad unless\quad}");
  DefMacro!("\\qgiven", r"\mbox{\quad given\quad}");
  DefMacro!("\\qusing", r"\mbox{\quad using\quad}");
  DefMacro!("\\qassume", r"\mbox{\quad assume\quad}");
  DefMacro!("\\qsince", r"\mbox{\quad since\quad}");
  DefMacro!("\\qlet", r"\mbox{\quad let\quad}");
  DefMacro!("\\qfor", r"\mbox{\quad for\quad}");
  DefMacro!("\\qall", r"\mbox{\quad all\quad}");
  DefMacro!("\\qeven", r"\mbox{\quad even\quad}");
  DefMacro!("\\qodd", r"\mbox{\quad odd\quad}");
  DefMacro!("\\qinteger", r"\mbox{\quad integer\quad}");
  DefMacro!("\\qand", r"\mbox{\quad and\quad}");
  DefMacro!("\\qor", r"\mbox{\quad or\quad}");
  DefMacro!("\\qas", r"\mbox{\quad as\quad}");
  DefMacro!("\\qin", r"\mbox{\quad in\quad}");

  //======================================================================
  // Derivatives
  // Intentional — WISDOM #44, see physics umbrella L178.
  Let!("\\flatfrac", "\\ifrac");

  // Perl: \lx@physics@diff — differential operator.
  // DefMacro (expansion-time) — reads a delimited `(…)` arg via `phys_read_arg`; same
  // WISDOM #44 alignment reason as `\mqty`/`\lx@physics@operatorP`. Return the dual.
  DefMacro!("\\lx@physics@diff{}{}{}", sub[(cs, semantic, diff)] {
    let cs_tks = cs;
    let semantic_str = semantic.to_string();
    let diff_tks = diff;
    let cfunc = i_symbol(&[("meaning", Tokenize!(&semantic_str))], None);
    let pfunc = i_wrap(Some(Tokenize!("role=DIFFOP")), diff_tks);
    let degree = gullet::read_optional(None)?;
    let (arg, open, close) = phys_read_arg(false, |s| {
      if s == "(" { Some(")") } else { None }
    })?;

    let a1 = Tokens::new(vec![i_arg("1")]);
    let mut all_args: Vec<Tokens> = Vec::new();

    // Reversion
    let mut rev = Vec::new();
    rev.extend(cs_tks.unlist());
    if degree.is_some() {
      rev.push(T_OTHER!("["));
      rev.push(i_arg("2"));
      rev.push(T_OTHER!("]"));
    }
    if let Some(ref _a) = arg {
      rev.extend(phys_rev_arg(a1.clone(), &open, &close).unlist());
    }
    let reversion = Tokens::new(rev);

    let content;
    let presentation;

    if let Some(arg_tks) = arg {
      all_args.push(arg_tks);
      if let Some(deg) = degree {
        let a2 = Tokens::new(vec![i_arg("2")]);
        all_args.push(deg);
        content = i_apply(&[], cfunc, vec![a1, a2.clone()]);
        presentation = Tokens::new([
          i_superscript(&[("role", Tokenize!("DIFFOP"))], pfunc, a2).unlist(),
          phys_open(false, &None, open.map(|t| Tokenize!(&t.to_string())).unwrap_or_default()).unlist(),
          vec![i_arg("1")],
          phys_close(false, &None, close.map(|t| Tokenize!(&t.to_string())).unwrap_or_default()).unlist(),
        ].concat());
      } else {
        content = i_apply(&[], cfunc, vec![a1]);
        presentation = Tokens::new([
          pfunc.unlist(),
          phys_open(false, &None, open.map(|t| Tokenize!(&t.to_string())).unwrap_or_default()).unlist(),
          vec![i_arg("1")],
          phys_close(false, &None, close.map(|t| Tokenize!(&t.to_string())).unwrap_or_default()).unlist(),
        ].concat());
      }

      let result = i_dual(&[("reversion", reversion)], content, presentation, all_args)?;
      Ok(result)
    } else if let Some(deg) = degree {
      let a2 = Tokens::new(vec![i_arg("2")]);
      all_args.push(deg);
      content = i_apply(&[], i_symbol(&[("meaning", Tokenize!("functional-power"))], None),
        vec![cfunc, a2.clone()]);
      presentation = i_superscript(&[("role", Tokenize!("DIFFOP"))], pfunc, a2);

      let result = i_dual(
        &[("role", Tokenize!("DIFFOP")), ("reversion", reversion)],
        content, presentation, all_args)?;
      Ok(result)
    } else {
      // Bare differential: just the symbol
      let result = i_dual(
        &[("role", Tokenize!("DIFFOP")), ("reversion", reversion)],
        cfunc, pfunc, vec![])?;
      Ok(result)
    }
  });

  DefMacro!("\\differential", "\\lx@physics@diff{\\differential}{differential}{\\mathrm{d}}");
  DefMacro!("\\variation", "\\lx@physics@diff{\\variation}{variation}{\\delta}");
  Let!("\\dd", "\\differential");
  Let!("\\var", "\\variation");

  // Intentional — WISDOM #44, see physics umbrella L178.
  // Perl: \lx@physics@deriv — derivative (complex multi-arg parsing)
  // Handles: \dv{var}, \dv{f}{x}, \dv[n]{f}{x}, \dv{var}(expr), \dv*{f}{x}
  // For partial: \pdv{f}{x}{y} (double derivative)
  DefPrimitive!("\\lx@physics@deriv{}{}{}", sub[(cs, semantic, diff)] {
    let cs_tks = cs;
    let semantic_str = semantic.to_string();
    let diff_tks = diff;
    let cfunc = i_symbol(&[("meaning", Tokenize!(&semantic_str))], None);
    let pfunc = i_wrap(Some(Tokenize!("role=DIFFOP")), diff_tks);

    let inline = gullet::read_match(&[&Tokenize!("*")])?.is_some();
    let degree = gullet::read_optional(None)?;
    let tmp1 = gullet::read_arg(ExpansionLevel::Off)?; // 1st required: var1 or expr
    let (tmp2, open, close) = phys_read_arg(false, |s| {
      if s == "(" { Some(")") } else { None }
    })?;

    // For partial derivatives: try to read a 3rd {} arg (2nd var)
    let mut tmp3: Option<Tokens> = None;
    if semantic_str.starts_with("partial") {
      if let Some(ref _t2) = tmp2 {
        if open.is_none() {
          // tmp2 was a {} arg, try for 3rd
          let (t3, o3, _c3) = phys_read_arg(false, |s| {
            if s == "(" { Some(")") } else { None }
          })?;
          if o3.is_none() { // only accept {} arg, not (arg)
            tmp3 = t3;
          }
        }
      }
    }

    // Determine expr, var, var2
    let (expr, var, var2) = if tmp3.is_some() {
      (Some(tmp1), tmp2.unwrap(), tmp3)
    } else if let Some(t2) = tmp2 {
      if open.is_some() {
        (Some(t2), tmp1, None) // \dv{var}(expr)
      } else {
        (Some(tmp1), t2, None) // \dv{expr}{var}
      }
    } else {
      (None, tmp1, None) // \dv{var}
    };

    // Check if expr is empty
    let expr = expr.filter(|e| !e.is_empty());

    let frac_cs = if inline { T_CS!("\\ifrac") } else { T_CS!("\\frac") };

    if let Some(v2) = var2 {
      // Double derivative: \pdv{f}{x}{y}
      let a1 = Tokens::new(vec![i_arg("1")]); // expr
      let a2 = Tokens::new(vec![i_arg("2")]); // var1
      let a3 = Tokens::new(vec![i_arg("3")]); // var2

      let mut rev = Vec::new();
      rev.extend(cs_tks.unlist());
      if inline { rev.push(T_OTHER!("*")); }
      rev.extend(phys_rev_arg(a1.clone(), &None, &None).unlist());
      rev.extend(phys_rev_arg(a2.clone(), &None, &None).unlist());
      rev.extend(phys_rev_arg(a3.clone(), &None, &None).unlist());
      let reversion = Tokens::new(rev);

      let op = i_apply(&[], cfunc, vec![a2.clone(), Tokenize!("1"), a3.clone(), Tokenize!("1")]);
      let content = if expr.is_some() {
        i_apply(&[], op, vec![a1])
      } else { op };

      // Presentation: \frac{d^2 expr}{dx dy}
      let mut numer = Vec::new();
      numer.extend(i_superscript(&[("role", Tokenize!("DIFFOP"))], pfunc.clone(), Tokenize!("2")).unlist());
      if expr.is_some() { numer.push(i_arg("1")); }
      let mut denom = Vec::new();
      denom.extend(i_apply(&[], pfunc.clone(), vec![a2]).unlist());
      denom.extend(i_apply(&[], pfunc, vec![a3]).unlist());
      let pres = Tokens::new([
        vec![frac_cs, T_BEGIN!()],
        numer,
        vec![T_END!(), T_BEGIN!()],
        denom,
        vec![T_END!()],
      ].concat());

      let has_expr = expr.is_some();
      let mut args = Vec::new();
      if let Some(e) = expr { args.push(e); } else { args.push(Tokens::default()); }
      args.push(var);
      args.push(v2);

      let mut kv = vec![("reversion", reversion)];
      if !has_expr { kv.push(("role", Tokenize!("DIFFOP"))); }
      let result = i_dual(&kv, content, pres, args)?;
      gullet::unread(result);
    } else {
      // Single-variable derivative
      let a1 = Tokens::new(vec![i_arg("1")]); // expr (or placeholder)
      let a2 = Tokens::new(vec![i_arg("2")]); // var

      let mut rev = Vec::new();
      rev.extend(cs_tks.unlist());
      if inline { rev.push(T_OTHER!("*")); }

      let mut all_args: Vec<Tokens> = Vec::new();

      if degree.is_some() {
        rev.push(T_OTHER!("["));
        rev.push(i_arg("3"));
        rev.push(T_OTHER!("]"));
      }

      if open.is_some() {
        // \dv{var}(expr) — var first, then expr in parens
        rev.extend(phys_rev_arg(a2.clone(), &None, &None).unlist());
        rev.extend(phys_rev_arg(a1.clone(), &open, &close).unlist());
      } else if expr.is_some() {
        // \dv{expr}{var}
        rev.extend(phys_rev_arg(a1.clone(), &None, &None).unlist());
        rev.extend(phys_rev_arg(a2.clone(), &None, &None).unlist());
      } else {
        // \dv{var} alone
        rev.extend(phys_rev_arg(a2.clone(), &None, &None).unlist());
      }
      let reversion = Tokens::new(rev);

      let op = i_apply(&[], cfunc, vec![a2.clone(),
        if degree.is_some() { Tokens::new(vec![i_arg("3")]) } else { Tokens::default() }]);

      let content = if expr.is_some() {
        i_apply(&[], op, vec![a1])
      } else { op };

      let has_expr = expr.is_some();
      let has_open = open.is_some();

      // Presentation: \frac{d^n expr}{dx^n} or \frac{d expr}{dx}
      let mut numer = Vec::new();
      if degree.is_some() {
        let a3 = Tokens::new(vec![i_arg("3")]);
        numer.extend(i_superscript(&[("role", Tokenize!("DIFFOP"))], pfunc.clone(), a3).unlist());
      } else {
        numer.extend_from_slice(pfunc.unlist_ref());
      }
      if has_expr && !has_open {
        numer.push(i_arg("1"));
      }
      let mut denom = Vec::new();
      if degree.is_some() {
        let a3 = Tokens::new(vec![i_arg("3")]);
        denom.extend(i_superscript(&[], i_apply(&[], pfunc.clone(), vec![a2]), a3).unlist());
      } else {
        denom.extend(i_apply(&[], pfunc, vec![a2]).unlist());
      }
      let mut pres_tks = vec![frac_cs, T_BEGIN!()];
      pres_tks.extend(numer);
      pres_tks.push(T_END!());
      pres_tks.push(T_BEGIN!());
      pres_tks.extend(denom);
      pres_tks.push(T_END!());
      if has_expr && has_open {
        pres_tks.push(T_CS!("\\lx@ApplyFunction"));
        pres_tks.extend(phys_open(false, &None, open.map(|t| Tokenize!(&t.to_string())).unwrap_or_default()).unlist());
        pres_tks.push(i_arg("1"));
        pres_tks.extend(phys_close(false, &None, close.map(|t| Tokenize!(&t.to_string())).unwrap_or_default()).unlist());
      }
      let presentation = Tokens::new(pres_tks);
      if let Some(e) = expr {
        all_args.push(e);
      }
      all_args.push(var);
      if let Some(deg) = degree {
        all_args.push(deg);
      }

      let mut kv = vec![("reversion", reversion)];
      if !has_expr { kv.push(("role", Tokenize!("DIFFOP"))); }
      let result = i_dual(&kv, content, presentation, all_args)?;
      gullet::unread(result);
    }
  });

  DefMacro!("\\derivative", "\\lx@physics@deriv{\\derivative}{derivative}{\\mathrm{d}}");
  DefMacro!("\\partialderivative", "\\lx@physics@deriv{\\partialderivative}{partial-derivative}{\\partial}");
  DefMacro!("\\functionalderivative", "\\lx@physics@deriv{\\functionalderivative}{functional-derivative}{\\delta}");
  Let!("\\dv", "\\derivative");
  Let!("\\pdv", "\\partialderivative");
  Let!("\\pderivative", "\\partialderivative");
  Let!("\\fdv", "\\functionalderivative");

  //======================================================================
  // Intentional — WISDOM #44, see physics umbrella L178.
  // Dirac bra-ket notation

  // Perl: \ket{} — |arg⟩ with meaning=ket
  DefPrimitive!("\\ket", {
    let (no_stretch, size_tok) = phys_read_size()?;
    let arg = gullet::read_arg(ExpansionLevel::Off)?;
    let a1 = Tokens::new(vec![i_arg("1")]);

    let mut rev = vec![T_CS!("\\ket")];
    rev.extend(phys_rev_size(no_stretch, &size_tok));
    rev.extend(phys_rev_arg(a1.clone(), &None, &None).unlist());
    let reversion = Tokens::new(rev);

    let content = i_apply(&[], i_symbol(&[("meaning", Tokenize!("ket"))], None), vec![a1]);
    let mut pres = Vec::new();
    pres.extend(phys_open(no_stretch, &size_tok, Tokenize!("\\vert")).unlist());
    pres.push(i_arg("1"));
    pres.extend(phys_close(no_stretch, &size_tok, Tokenize!("\\rangle")).unlist());
    let presentation = Tokens::new(pres);

    let result = i_dual(&[("reversion", reversion)], content, presentation, vec![arg])?;
    gullet::unread(result);
  // Intentional — WISDOM #44, see physics umbrella L178.
  });

  // Perl: \bra{} — ⟨arg| with meaning=bra, auto-joins to \braket
  DefPrimitive!("\\bra", {
    let no_stretch = gullet::read_match(&[&Tokenize!("*")])?.is_some();
    let arg = gullet::read_arg(ExpansionLevel::Off)?;
    let a1 = Tokens::new(vec![i_arg("1")]);

    // Check if followed by \ket → join to braket
    if gullet::read_match(&[&Tokenize!("\\ket")])?.is_some() {
      let no_stretch2 = gullet::read_match(&[&Tokenize!("*")])?.is_some();
      let arg2 = gullet::read_arg(ExpansionLevel::Off)?;
      let a2 = Tokens::new(vec![i_arg("2")]);
      let final_stretch = !no_stretch && !no_stretch2;

      let mut rev = vec![T_CS!("\\bra")];
      if no_stretch { rev.push(T_OTHER!("*")); }
      rev.extend(phys_rev_arg(a1.clone(), &None, &None).unlist());
      rev.push(T_CS!("\\ket"));
      if no_stretch2 { rev.push(T_OTHER!("*")); }
      rev.extend(phys_rev_arg(a2.clone(), &None, &None).unlist());
      let reversion = Tokens::new(rev);

      let content = i_apply(&[],
        i_symbol(&[("meaning", Tokenize!("inner-product"))], None),
        vec![a1, a2]);
      let mut pres = Vec::new();
      pres.extend(phys_open(!final_stretch, &None, Tokenize!("\\langle")).unlist());
      pres.push(i_arg("1"));
      pres.extend(phys_mid(!final_stretch, &None, Tokenize!("\\vert")).unlist());
      pres.push(i_arg("2"));
      pres.extend(phys_close(!final_stretch, &None, Tokenize!("\\rangle")).unlist());
      let presentation = Tokens::new(pres);

      let result = i_dual(&[("reversion", reversion)], content, presentation, vec![arg, arg2])?;
      gullet::unread(result);
    } else {
      // Plain bra
      let mut rev = vec![T_CS!("\\bra")];
      if no_stretch { rev.push(T_OTHER!("*")); }
      rev.extend(phys_rev_arg(a1.clone(), &None, &None).unlist());
      let reversion = Tokens::new(rev);

      let content = i_apply(&[],
        i_symbol(&[("meaning", Tokenize!("bra"))], None), vec![a1]);
      // Stretchy by default (Perl parity); `no_stretch` passed directly — see the
      // `\matrixelement` note. A prior `!no_stretch` made plain `\bra` default
      // non-stretchy, diverging from Perl.
      let mut pres = Vec::new();
      pres.extend(phys_open(no_stretch, &None, Tokenize!("\\langle")).unlist());
      pres.push(i_arg("1"));
      pres.extend(phys_close(no_stretch, &None, Tokenize!("\\vert")).unlist());
      let presentation = Tokens::new(pres);

      let result = i_dual(&[("reversion", reversion)], content, presentation, vec![arg])?;
      gullet::unread(result);
    }
  // Intentional — WISDOM #44, see physics umbrella L178.
  });

  // Perl: \innerproduct — ⟨arg1|arg2⟩
  DefPrimitive!("\\lx@physics@qm@product{}{}{}{}{}", sub[(cs, semantic, open, middle, close)] {
    let cs_tks = cs;
    let semantic_str = semantic.to_string();
    let open_tks = open;
    let middle_tks = middle;
    let close_tks = close;
    let no_stretch = gullet::read_match(&[&Tokenize!("*")])?.is_some();
    let arg0 = gullet::read_arg(ExpansionLevel::Off)?;
    let argx = phys_read_arg_tex()?;
    let arg1 = argx.unwrap_or_else(|| arg0.clone());
    let a1 = Tokens::new(vec![i_arg("1")]);
    let a2 = Tokens::new(vec![i_arg("2")]);

    let mut rev = Vec::new();
    rev.extend(cs_tks.unlist());
    if no_stretch { rev.push(T_OTHER!("*")); }
    rev.extend(phys_rev_arg(a1.clone(), &None, &None).unlist());
    rev.extend(phys_rev_arg(a2.clone(), &None, &None).unlist());
    let reversion = Tokens::new(rev);

    let content = i_apply(&[],
      i_symbol(&[("meaning", Tokenize!(&semantic_str))], None),
      vec![a1, a2]);
    // Stretchy by default (Perl parity); `no_stretch` passed directly — see the
    // `\matrixelement` note. A prior `!no_stretch` made `\innerproduct`/`\outerproduct`
    // default non-stretchy, diverging from Perl.
    let mut pres = Vec::new();
    pres.extend(phys_open(no_stretch, &None, open_tks).unlist());
    pres.push(i_arg("1"));
    pres.extend(phys_mid(no_stretch, &None, middle_tks).unlist());
    pres.push(i_arg("2"));
    pres.extend(phys_close(no_stretch, &None, close_tks).unlist());
    let presentation = Tokens::new(pres);

    let result = i_dual(&[("reversion", reversion)], content, presentation, vec![arg0, arg1])?;
    gullet::unread(result);
  });

  DefMacro!("\\innerproduct",
    "\\lx@physics@qm@product{\\innerproduct}{inner-product}{\\langle}{\\vert}{\\rangle}");
  DefMacro!("\\outerproduct",
  // Intentional — WISDOM #44, see physics umbrella L178.
    "\\lx@physics@qm@product{\\outerproduct}{outer-product}{\\vert}{\\rangle\\langle}{\\vert}");

  // Perl: \expectationvalue — ⟨arg⟩ or ⟨arg2|arg1|arg2⟩
  DefPrimitive!("\\expectationvalue", {
    let cfunc = i_symbol(&[("meaning", Tokenize!("expectation-value"))], None);
    // ** means stretchy (default), * means no stretch, plain means stretchy
    let size = if gullet::read_match(&[&Tokenize!("*")])?.is_some() {
      gullet::read_match(&[&Tokenize!("*")])?.is_some()
    } else { true };
    let no_stretch = !size;
    let open_tks = phys_open(no_stretch, &None, Tokenize!("\\langle"));
    let middle_tks = phys_mid(no_stretch, &None, Tokenize!("\\vert"));
    let close_tks = phys_close(no_stretch, &None, Tokenize!("\\rangle"));
    let arg0 = gullet::read_arg(ExpansionLevel::Off)?;
    let arg1 = phys_read_arg_tex()?;
    let a1 = Tokens::new(vec![i_arg("1")]);

    if let Some(arg1_tks) = arg1 {
      // With second arg: ⟨arg1|arg0|arg1⟩
      let a2 = Tokens::new(vec![i_arg("2")]);
      let a3 = Tokens::new(vec![i_arg("3")]);
      let mut rev = vec![T_CS!("\\expectationvalue")];
      if no_stretch { rev.push(T_OTHER!("*")); }
      rev.extend(phys_rev_arg(a1.clone(), &None, &None).unlist());
      rev.extend(phys_rev_arg(a2.clone(), &None, &None).unlist());
      let reversion = Tokens::new(rev);
      let content = i_apply(&[], cfunc, vec![a1, a2, a3]);
      let mut pres = Vec::new();
      pres.extend(open_tks.unlist());
      pres.push(i_arg("2"));
      pres.extend_from_slice(middle_tks.unlist_ref());
      pres.push(i_arg("1"));
      pres.extend(middle_tks.unlist());
      pres.push(i_arg("3"));
      pres.extend(close_tks.unlist());
      let presentation = Tokens::new(pres);
      let result = i_dual(&[("reversion", reversion)], content, presentation,
        vec![arg0, arg1_tks.clone(), arg1_tks])?;
      gullet::unread(result);
    } else {
      // Simple: ⟨arg0⟩
      let mut rev = vec![T_CS!("\\expectationvalue")];
      if no_stretch { rev.push(T_OTHER!("*")); }
      rev.extend(phys_rev_arg(a1.clone(), &None, &None).unlist());
      let reversion = Tokens::new(rev);
      let content = i_apply(&[], cfunc, vec![a1]);
      let mut pres = Vec::new();
      pres.extend(open_tks.unlist());
      pres.push(i_arg("1"));
      pres.extend(close_tks.unlist());
      let presentation = Tokens::new(pres);
      let result = i_dual(&[("reversion", reversion)], content, presentation, vec![arg0])?;
      gullet::unread(result);
    }
  // Intentional — WISDOM #44, see physics umbrella L178.
  });

  // Perl: \matrixelement — ⟨arg1|arg2|arg3⟩
  DefPrimitive!("\\matrixelement", {
    let cfunc = i_symbol(&[("meaning", Tokenize!("expectation-value"))], None);
    let no_stretch = if gullet::read_match(&[&Tokenize!("*")])?.is_some() {
      gullet::read_match(&[&Tokenize!("*")])?.is_none()
    } else { false };
    // Default (no `*`) is stretchy (`\left…\middle…\right`), matching Perl's
    // physics.sty.ltxml (all bra-ket delimiters default stretchy="true"). The
    // `no_stretch` variable already means "no stretch" (true only when `*` given),
    // so pass it DIRECTLY — an earlier `!no_stretch` double-negated it, making the
    // default non-stretchy. Beyond the rendering divergence, the bare-VERTBAR
    // (non-stretchy) `⟨#1|#2|#3⟩` form hits the open VERTBAR-modulus grammar
    // ambiguity: a compound `#2` between two bare `|` parses as bare modulus,
    // dissolving the `\lx@xmarg` that carries the XMDual's shared id → dangling
    // `\lx@xmref` → Post `expected:id Cannot find a node`. The stretchy
    // `\middle\vert` form parses correctly. Witnesses 2205.06843, 2211.16395,
    // 2306.04445 (physics `\matrixelement`/`\mel`). See SYNC_STATUS.md.
    let open_tks = phys_open(no_stretch, &None, Tokenize!("\\langle"));
    let middle_tks = phys_mid(no_stretch, &None, Tokenize!("\\vert"));
    let close_tks = phys_close(no_stretch, &None, Tokenize!("\\rangle"));
    let arg0 = gullet::read_arg(ExpansionLevel::Off)?;
    let arg1 = gullet::read_arg(ExpansionLevel::Off)?;
    let arg2 = gullet::read_arg(ExpansionLevel::Off)?;
    let a1 = Tokens::new(vec![i_arg("1")]);
    let a2 = Tokens::new(vec![i_arg("2")]);
    let a3 = Tokens::new(vec![i_arg("3")]);

    let mut rev = vec![T_CS!("\\matrixelement")];
    if no_stretch { rev.push(T_OTHER!("*")); }
    rev.extend(phys_rev_arg(a1.clone(), &None, &None).unlist());
    rev.extend(phys_rev_arg(a2.clone(), &None, &None).unlist());
    rev.extend(phys_rev_arg(a3.clone(), &None, &None).unlist());
    let reversion = Tokens::new(rev);

    let content = i_apply(&[], cfunc, vec![a2, a1, a3]);
    let mut pres = Vec::new();
    pres.extend(open_tks.unlist());
    pres.push(i_arg("1"));
    pres.extend_from_slice(middle_tks.unlist_ref());
    pres.push(i_arg("2"));
    pres.extend(middle_tks.unlist());
    pres.push(i_arg("3"));
    pres.extend(close_tks.unlist());
    let presentation = Tokens::new(pres);

    let result = i_dual(&[("reversion", reversion)], content, presentation, vec![arg0, arg1, arg2])?;
    gullet::unread(result);
  });

  Let!("\\braket", "\\innerproduct");
  Let!("\\ip", "\\innerproduct");
  Let!("\\dyad", "\\outerproduct");
  Let!("\\ketbra", "\\outerproduct");
  Let!("\\op", "\\outerproduct");
  Let!("\\expval", "\\expectationvalue");
  Let!("\\ev", "\\expectationvalue");
  Let!("\\matrixel", "\\matrixelement");
  Let!("\\mel", "\\matrixelement");

  //======================================================================
  // Intentional — WISDOM #44, see physics umbrella L178.
  // Matrix macros

  // Perl: \identitymatrix{n} — generates n×n identity matrix content
  DefPrimitive!("\\identitymatrix{}", sub[(n)] {
    let n_val: usize = n.to_string().parse().unwrap_or(2);
    let mut tks = Vec::new();
    for i in 0..n_val {
      if i > 0 { tks.push(T_CS!("\\\\")); }
      for j in 0..n_val {
        if j > 0 { tks.push(T_ALIGN!()); }
        tks.push(T_OTHER!(if i == j { "1" } else { "0" }));
      }
    }
    gullet::unread(Tokens::new(tks));
  // Intentional — WISDOM #44, see physics umbrella L178.
  });

  // Perl: \xmatrix *{item}{n}{m}
  DefPrimitive!("\\xmatrix{}{}{}", sub[(_item, n, m)] {
    let item_tks = _item;
    let n_val: usize = n.to_string().parse().unwrap_or(2);
    let m_val: usize = m.to_string().parse().unwrap_or(2);
    let mut tks = Vec::new();
    for i in 0..n_val {
      if i > 0 { tks.push(T_CS!("\\\\")); }
      for j in 0..m_val {
        if j > 0 { tks.push(T_ALIGN!()); }
        tks.extend_from_slice(item_tks.unlist_ref());
      }
    }
    gullet::unread(Tokens::new(tks));
  });

  DefMacro!("\\zeromatrix{}{}", "\\xmatrix{0}{#1}{#2}");

  // Perl physics.sty.ltxml L622: `alias => 'i'` — reversion emits `i` rather
  // than the internal `\lx@physics@iunit` CS name. Without it, MathML `name=`
  // Intentional — WISDOM #44, see physics umbrella L178.
  // and `tex=` attributes leak the private helper name to downstream consumers.
  DefMath!("\\lx@physics@iunit", None, "\\mathit{i}",
    meaning => "imaginary-unit", alias => "i");
  // Perl physics.sty.ltxml L623-634: `\paulimatrix{n}` constructs the
  // matrix-cell tokens DIRECTLY (T_OTHER, T_ALIGN, T_CS) rather than
  // round-tripping through Tokenize/TokenizeInternal — the Tokenize
  // approach mistokenizes `\lx@physics@iunit` (catcode @ = 12 splits
  // it into `\lx` + `@physics@iunit` text) and TokenizeInternal leaks
  // raw `0&-i\\i&0` past the surrounding `\smallmatrixquantity(...)`
  // tex-attr scaffold. Match Perl exactly.
  DefPrimitive!("\\paulimatrix{}", sub[(n)] {
    let n_val: usize = n.to_string().parse().unwrap_or(0);
    let tks = match n_val {
      0 => Tokens!(T_OTHER!("1"), T_ALIGN!(), T_OTHER!("0"), T_CS!("\\\\"),
                   T_OTHER!("0"), T_ALIGN!(), T_OTHER!("1")),
      1 => Tokens!(T_OTHER!("0"), T_ALIGN!(), T_OTHER!("1"), T_CS!("\\\\"),
                   T_OTHER!("1"), T_ALIGN!(), T_OTHER!("0")),
      2 => Tokens!(T_OTHER!("0"), T_ALIGN!(), T_OTHER!("-"),
                   T_CS!("\\lx@physics@iunit"), T_CS!("\\\\"),
                   T_CS!("\\lx@physics@iunit"), T_ALIGN!(), T_OTHER!("0")),
      3 => Tokens!(T_OTHER!("1"), T_ALIGN!(), T_OTHER!("0"), T_CS!("\\\\"),
                   T_OTHER!("0"), T_ALIGN!(), T_OTHER!("-"), T_OTHER!("1")),
      _ => Tokens::default(),
    };
    gullet::unread(tks);
  // Intentional — WISDOM #44, see physics umbrella L178.
  });

  // Perl: \diagonalmatrix[zero]{diag,diag,...} (physics.sty.ltxml L636-654).
  // Split the diagonal entries on `,` at the TOKEN level (Perl
  // `SplitTokens($diag, T_OTHER(','))`), NOT by `to_string()`+re-tokenize:
  // the string round-trip drops the inter-token space that separates a
  // control word from a following letter, so e.g. `\vb h` collapses into
  // the undefined CS `\vbh`. Witness 2004.07845 (`\dmat{\vb h \vdot
  // \vb*{\sigma}, ...}` → `\vbh`/`\tildeN`/`\tilded` undefined; Perl
  // converts it cleanly).
  DefPrimitive!("\\diagonalmatrix[]{}", sub[(z, diag)] {
    let z_tok = match z { Some(t) if !t.is_empty() => t.unlist(), _ => vec![T_SPACE!()] };
    let items = crate::engine::base_utilities::split_tokens(diag, vec![T_OTHER!(",")]);
    let n = items.len();
    let mut tks = Vec::new();
    for (i, item) in items.iter().enumerate() {
      if i > 0 { tks.push(T_CS!("\\\\")); }
      for j in 0..n {
        if j > 0 { tks.push(T_ALIGN!()); }
        if i == j {
          tks.extend(item.clone().unlist());
        } else {
          tks.extend(z_tok.clone());
        }
      }
    }
    gullet::unread(Tokens::new(tks));
  // Intentional — WISDOM #44, see physics umbrella L178.
  });

  // Perl: \antidiagonalmatrix[zero]{diag,diag,...} (physics.sty.ltxml
  // L655-672). Same token-level split as \diagonalmatrix.
  DefPrimitive!("\\antidiagonalmatrix[]{}", sub[(z, diag)] {
    let z_tok = match z { Some(t) if !t.is_empty() => t.unlist(), _ => vec![T_SPACE!()] };
    let items = crate::engine::base_utilities::split_tokens(diag, vec![T_OTHER!(",")]);
    let n = items.len();
    let mut tks = Vec::new();
    for i in 0..n {
      if i > 0 { tks.push(T_CS!("\\\\")); }
      for j in 0..n {
        if j > 0 { tks.push(T_ALIGN!()); }
        if j == n - i - 1 {
          tks.extend(items[n - i - 1].clone().unlist());
        } else {
          tks.extend(z_tok.clone());
        }
      }
    }
    gullet::unread(Tokens::new(tks));
  });
  // Intentional — WISDOM #44, see physics umbrella L178.

  // Perl: \lx@physics@mat — wraps matrix content in an env, with delimiters
  // Reads optional * then required arg (TeX {} or delimiter-fenced)
  //
  // This MUST be a DefMacro (expandable), NOT a DefPrimitive — matching Perl
  // `DefMacro('\lx@physics@mat{}{}{}{}{}', sub {…})` (physics.sty.ltxml L677). The
  // matrix body is read here via `phys_read_arg` (a delimited `(…)`/`[…]` read, not
  // a brace group). As a digestion-time PRIMITIVE, an alignment's column scan would
  // see the matrix's own `&`/`\\` BEFORE this code consumes them — so `\mqty(a&b\\c&d)`
  // inside an `eqnarray` leaked its `&`/`\\` into the eqnarray, splitting the row and
  // orphaning the `\left(`/`\right)` fences → `\lx@begin@alignment … mode-switch to
  // restricted_horizontal due to \lx@begin@inmath@text` + "Unbalanced \right" cascade
  // (witness 2007.06211: revtex4-1 + physics, 11 errors, Perl 0). As an EXPANSION-time
  // macro it grabs `(…)` first (like Perl), so the alignment never sees the inner
  // `&`/`\\`. Return the dual instead of `gullet::unread`.
  DefMacro!("\\lx@physics@mat{}{}{}{}{}", sub[(cs, semantic, env, defopen, defclose)] {
    let cs_tks = cs;
    let semantic_str = semantic.to_string();
    let semantic_opt = if semantic_str.is_empty() { None } else { Some(semantic_str.as_str()) };
    let env_str = env.to_string();
    let defopen_tks = defopen;
    let defclose_tks = defclose;
    let _alt = gullet::read_match(&[&Tokenize!("*")])?.is_some();

    let cfunc = semantic_opt.map(|s| i_symbol(&[("meaning", Tokenize!(s))], None));

    // Read the body: either {} or delimiter-fenced
    let (body, open, close) = phys_read_arg(true, physics_delimiters)?;
    let body = body.unwrap_or_default();

    // Wrap body in matrix environment tokens
    let mut matrix_tks = vec![T_CS!(&format!("\\{env_str}"))];
    matrix_tks.extend(body.unlist());
    matrix_tks.push(T_CS!(&format!("\\end{env_str}")));
    let matrix = Tokens::new(matrix_tks);

    let a1 = Tokens::new(vec![i_arg("1")]);
    let mut rev = Vec::new();
    rev.extend(cs_tks.unlist());
    rev.extend(phys_rev_arg(a1.clone(), &open, &close).unlist());
    let reversion = Tokens::new(rev);

    let content = if let Some(cf) = cfunc {
      i_apply(&[], cf, vec![a1])
    } else {
      a1
    };

    // Presentation: open + matrix + close (using default fences if no explicit ones)
    let open_fence = open.map(|t| Tokenize!(&t.to_string())).unwrap_or(defopen_tks);
    let close_fence = close.map(|t| Tokenize!(&t.to_string())).unwrap_or(defclose_tks);
    let mut pres = Vec::new();
    if !open_fence.is_empty() {
      pres.extend(phys_open(false, &None, open_fence).unlist());
    }
    pres.push(i_arg("1"));
    if !close_fence.is_empty() {
      pres.extend(phys_close(false, &None, close_fence).unlist());
    }
    let presentation = Tokens::new(pres);

    let result = i_dual(&[("reversion", reversion)], content, presentation, vec![matrix])?;
    Ok(result)
  });

  // Perl: \lx@physics@matrix / \lx@physics@smallmatrix environments
  DefMacro!("\\lx@physics@matrix", "\\lx@ams@matrix{datameaning=matrix}");
  DefMacro!("\\endlx@physics@matrix", "\\lx@end@ams@matrix");
  DefMacro!("\\lx@physics@smallmatrix", "\\lx@ams@matrix{datameaning=matrix,style=\\scriptsize}");
  DefMacro!("\\endlx@physics@smallmatrix", "\\lx@end@ams@matrix");

  DefMacro!("\\matrixquantity", "\\lx@physics@mat{\\matrixquantity}{}{lx@physics@matrix}{}{}");
  DefMacro!("\\pmqty{}", "\\lx@physics@mat{\\pmqty}{}{lx@physics@matrix}{(}{)}");
  DefMacro!("\\Pmqty{}", "\\lx@physics@mat{\\Pmqty}{}{lx@physics@matrix}{(}{)}");
  DefMacro!("\\bmqty{}", "\\lx@physics@mat{\\bmqty}{}{lx@physics@matrix}{[}{]}");
  DefMacro!("\\vmqty{}", "\\lx@physics@mat{\\vmqty}{}{lx@physics@matrix}{\\vert}{\\vert}");
  DefMacro!("\\smallmatrixquantity", "\\lx@physics@mat{\\smallmatrixquantity}{}{lx@physics@smallmatrix}{}{}");
  DefMacro!("\\spmqty{}", "\\lx@physics@mat{\\spmqty}{}{lx@physics@smallmatrix}{(}{)}");
  DefMacro!("\\sPmqty{}", "\\lx@physics@mat{\\sPmqty}{}{lx@physics@smallmatrix}{(}{)}");
  DefMacro!("\\sbmqty{}", "\\lx@physics@mat{\\sbmqty}{}{lx@physics@smallmatrix}{[}{]}");
  DefMacro!("\\svmqty{}", "\\lx@physics@mat{\\svmqty}{}{lx@physics@smallmatrix}{\\vert}{\\vert}");
  DefMacro!("\\matrixdeterminant", "\\lx@physics@mat{\\matrixdeterminant}{determinant}{matrix}{\\vert}{\\vert}");
  DefMacro!("\\smallmatrixdeterminant", "\\lx@physics@mat{\\smallmatrixdeterminant}{determinant}{smallmatrix}{\\vert}{\\vert}");

  Let!("\\imat", "\\identitymatrix");
  Let!("\\xmat", "\\xmatrix");
  Let!("\\zmat", "\\zeromatrix");
  Let!("\\pmat", "\\paulimatrix");
  Let!("\\dmat", "\\diagonalmatrix");
  Let!("\\admat", "\\antidiagonalmatrix");
  Let!("\\mqty", "\\matrixquantity");
  Let!("\\smqty", "\\smallmatrixquantity");
  Let!("\\mdet", "\\matrixdeterminant");
  Let!("\\smdet", "\\smallmatrixdeterminant");
});
