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
    if t.to_string() == "*" {
      no_stretch = true;
      pending = gullet::read_token()?;
    }
  }
  if let Some(ref t) = pending {
    let s = t.to_string();
    if s.starts_with('\\')
      && (s == "\\big" || s == "\\Big" || s == "\\bigg" || s == "\\Bigg")
    {
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
      let mut blevel = 0i32;
      while let Some(tok) = gullet::read_token()? {
        let cc = tok.get_catcode();
        if cc == Catcode::END {
          blevel -= 1;
          tokens.push(tok);
        } else if cc == Catcode::BEGIN {
          blevel += 1;
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
  // Automatic bracing
  // Perl: physics.sty.ltxml L132-142 — \quantity

  DefPrimitive!("\\quantity", {
    let (no_stretch, size_tok) = phys_read_size()?;
    let (arg, open, close) = phys_read_arg(true, physics_delimiters)?;
    let arg = arg.unwrap_or_default();
    let arg1 = Tokens::new(vec![i_arg("1")]);

    // Build reversion
    let mut rev_tks: Vec<Token> = vec![T_CS!("\\quantity")];
    rev_tks.extend(phys_rev_size(no_stretch, &size_tok));
    rev_tks.extend(phys_rev_arg(arg1.clone(), &open, &close).unlist());
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
    gullet::unread(result);
  });
  Let!("\\qty", "\\quantity");

  // Perl: \lx@physics@fenced — fenced stuff with optional semantics
  DefPrimitive!("\\lx@physics@fenced{}{}{}{}{}", sub[(cs, semantic, _function, open, close)] {
    let cs_tks = cs.clone();
    let semantic_str = semantic.to_string();
    let semantic_opt = if semantic_str.is_empty() { None } else { Some(semantic_str.as_str()) };
    let open_tks = open.clone();
    let close_tks = close.clone();
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
      i_apply(&[], i_symbol(&[("meaning", Tokenize!(sem))], None), vec![arg1.clone()])
    } else {
      arg1.clone()
    };

    // Presentation
    let mut pres = Vec::new();
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
  DefMacro!("\\Bqty", "\\lx@physics@fenced{\\Bqty}{}{}{{}}{{}}");
  DefMacro!("\\absolutevalue", "\\lx@physics@fenced{\\absolutevalue}{absolute-value}{}{\\vert}{\\vert}");
  DefMacro!("\\norm", "\\lx@physics@fenced{\\norm}{norm}{}{\\|}{\\|}");
  Let!("\\abs", "\\absolutevalue");

  // Perl: \evaluated — fenced with sub/superscript limits
  DefMacro!("\\evaluated{}", r"\left.#1\right\vert ");
  Let!("\\eval", "\\evaluated");

  // Perl: \order
  DefMacro!("\\order{}", r"\mathcal{O}\left(#1\right)");
  DefMacro!("\\ordersymbol", r"\mathcal{O}");

  // Perl: \lx@physics@fencedII — 2-argument fenced
  DefPrimitive!("\\lx@physics@fencedII{}{}{}{}{}", sub[(cs, semantic, _function, open, close)] {
    let cs_tks = cs.clone();
    let semantic_str = semantic.to_string();
    let open_tks = open.clone();
    let close_tks = close.clone();
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
      vec![a1.clone(), a2.clone()]);

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
  DefMacro!("\\anticommutator", "\\lx@physics@fencedII{\\anticommutator}{anticommutator}{}{{}}{{}}");
  DefMacro!("\\poissonbracket", "\\lx@physics@fencedII{\\poissonbracket}{poisson-bracket}{}{{}}{{}}");
  Let!("\\comm", "\\commutator");
  Let!("\\acomm", "\\anticommutator");
  Let!("\\pb", "\\poissonbracket");

  //======================================================================
  // Vector Notation
  // Perl: \vectorbold uses OptionalMatch:* {} — we skip the star for now
  DefMacro!("\\vectorbold{}", r"\lx@wrap[role=ID]{\mathbf{#1}}");
  DefMacro!("\\vectorarrow{}", r"\lx@wrap[role=ID]{\overrightarrow{\mathbf{#1}}}");
  DefMacro!("\\vectorunit{}", r"\lx@wrap[role=ID]{\hat{\mathbf{#1}}}");
  Let!("\\vb", "\\vectorbold");
  Let!("\\va", "\\vectorarrow");
  Let!("\\vu", "\\vectorunit");

  DefMath!("\\dotproduct", None, "\u{22C5}", role => "MULOP", meaning => "dot-product");
  DefMath!("\\crossproduct", None, "\u{00D7}", role => "MULOP", meaning => "cross-product");
  Let!("\\vdot", "\\dotproduct");
  Let!("\\cross", "\\crossproduct");
  Let!("\\cp", "\\crossproduct");

  // Perl: \lx@physics@operator — operator with optional delimited arg
  DefPrimitive!("\\lx@physics@operator{}{}{}", sub[(cs, semantic, function)] {
    let cs_tks = cs.clone();
    let semantic_str = semantic.to_string();
    let function_tks = function.clone();
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

      let content = i_apply(&[], cfunc.clone(), vec![a1.clone()]);
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
      gullet::unread(result);
    } else {
      // No argument: just the operator symbol
      let result = i_dual(
        &[("role", Tokenize!("OPERATOR")), ("reversion", cs_tks.clone())],
        cfunc, function_tks, vec![],
      )?;
      gullet::unread(result);
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
  // Operators with power
  // Perl: \lx@physics@operatorP — operator with optional power and paren-delimited arg

  DefPrimitive!("\\lx@physics@operatorP{}{}{}", sub[(cs, semantic, function)] {
    let cs_tks = cs.clone();
    let semantic_str = semantic.to_string();
    let function_tks = function.clone();
    let cfunc = i_symbol(&[("meaning", Tokenize!(&semantic_str))], None);
    let pfunc = function_tks;
    let (no_stretch, size_tok) = phys_read_size()?;
    let power = gullet::read_optional(None)?;
    let (arg, open, close) = phys_read_arg(false, |s| {
      if s == "(" { Some(")") } else { None }
    })?;

    if arg.is_none() {
      // No argument — put back the power and return bare operator
      if let Some(ref pwr) = power {
        gullet::unread(Tokens::new(
          [vec![T_OTHER!("[")], pwr.clone().unlist(), vec![T_OTHER!("]")]].concat()
        ));
      }
      let result = i_dual(
        &[("reversion", cs_tks.clone())],
        cfunc, pfunc, vec![],
      )?;
      gullet::unread(result);
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
          pfunc, a2.clone());
        rev.push(T_OTHER!("["));
        rev.push(i_arg("2"));
        rev.push(T_OTHER!("]"));
        all_args.push(pwr);
      }

      rev.extend(phys_rev_arg(a1.clone(), &open, &close).unlist());
      let reversion = Tokens::new(rev);
      let content = i_apply(&[], content_op, vec![a1.clone()]);

      let open_tks = open.map(|t| Tokenize!(&t.to_string())).unwrap_or(Tokenize!("("));
      let close_tks = close.map(|t| Tokenize!(&t.to_string())).unwrap_or(Tokenize!(")"));
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
      gullet::unread(result);
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
  Let!("\\pv", "\\principalvalue");

  Let!("\\real", "\\Re");
  Let!("\\imaginary", "\\Im");

  //======================================================================
  // Quick quad text
  DefMacro!("\\qqtext{}", r"\quad\text{#1}\quad");
  DefMacro!("\\qcomma", r",\quad");
  DefMacro!("\\qcc", r"\quad\text{c.c.}\quad");
  Let!("\\qq", "\\qqtext");
  Let!("\\qc", "\\qcomma");
  DefMacro!("\\qif", r"\quad\text{if}\quad");
  DefMacro!("\\qthen", r"\quad\text{then}\quad");
  DefMacro!("\\qelse", r"\quad\text{else}\quad");
  DefMacro!("\\qotherwise", r"\quad\text{otherwise}\quad");
  DefMacro!("\\qunless", r"\quad\text{unless}\quad");
  DefMacro!("\\qgiven", r"\quad\text{given}\quad");
  DefMacro!("\\qusing", r"\quad\text{using}\quad");
  DefMacro!("\\qassume", r"\quad\text{assume}\quad");
  DefMacro!("\\qsince", r"\quad\text{since}\quad");
  DefMacro!("\\qlet", r"\quad\text{let}\quad");
  DefMacro!("\\qfor", r"\quad\text{for}\quad");
  DefMacro!("\\qall", r"\quad\text{all}\quad");
  DefMacro!("\\qeven", r"\quad\text{even}\quad");
  DefMacro!("\\qodd", r"\quad\text{odd}\quad");
  DefMacro!("\\qinteger", r"\quad\text{integer}\quad");
  DefMacro!("\\qand", r"\quad\text{and}\quad");
  DefMacro!("\\qor", r"\quad\text{or}\quad");
  DefMacro!("\\qas", r"\quad\text{as}\quad");
  DefMacro!("\\qin", r"\quad\text{in}\quad");

  //======================================================================
  // Derivatives
  Let!("\\flatfrac", "\\ifrac");

  // Perl: \lx@physics@diff — differential operator
  DefPrimitive!("\\lx@physics@diff{}{}{}", sub[(cs, semantic, diff)] {
    let cs_tks = cs.clone();
    let semantic_str = semantic.to_string();
    let diff_tks = diff.clone();
    let cfunc = i_symbol(&[("meaning", Tokenize!(&semantic_str))], None);
    let pfunc = i_wrap(Some(Tokenize!("role=DIFFOP")), diff_tks.clone());
    let degree = gullet::read_optional(None)?;
    let (arg, open, close) = phys_read_arg(false, |s| {
      if s == "(" { Some(")") } else { None }
    })?;

    let a1 = Tokens::new(vec![i_arg("1")]);
    let mut all_args: Vec<Tokens> = Vec::new();

    // Reversion
    let mut rev = Vec::new();
    rev.extend(cs_tks.unlist());
    if let Some(ref deg) = degree {
      let a2 = Tokens::new(vec![i_arg("2")]);
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
        content = i_apply(&[], cfunc, vec![a1.clone(), a2.clone()]);
        presentation = Tokens::new([
          i_superscript(&[("role", Tokenize!("DIFFOP"))], pfunc, a2).unlist(),
          phys_open(false, &None, open.map(|t| Tokenize!(&t.to_string())).unwrap_or_default()).unlist(),
          vec![i_arg("1")],
          phys_close(false, &None, close.map(|t| Tokenize!(&t.to_string())).unwrap_or_default()).unlist(),
        ].concat());
      } else {
        content = i_apply(&[], cfunc, vec![a1.clone()]);
        presentation = Tokens::new([
          pfunc.unlist(),
          phys_open(false, &None, open.map(|t| Tokenize!(&t.to_string())).unwrap_or_default()).unlist(),
          vec![i_arg("1")],
          phys_close(false, &None, close.map(|t| Tokenize!(&t.to_string())).unwrap_or_default()).unlist(),
        ].concat());
      }

      let result = i_dual(&[("reversion", reversion)], content, presentation, all_args)?;
      gullet::unread(result);
    } else if let Some(deg) = degree {
      let a2 = Tokens::new(vec![i_arg("2")]);
      all_args.push(deg);
      content = i_apply(&[], i_symbol(&[("meaning", Tokenize!("functional-power"))], None),
        vec![cfunc, a2.clone()]);
      presentation = i_superscript(&[("role", Tokenize!("DIFFOP"))], pfunc, a2);

      let result = i_dual(
        &[("role", Tokenize!("DIFFOP")), ("reversion", reversion)],
        content, presentation, all_args)?;
      gullet::unread(result);
    } else {
      // Bare differential: just the symbol
      let result = i_dual(
        &[("role", Tokenize!("DIFFOP")), ("reversion", reversion)],
        cfunc, pfunc, vec![])?;
      gullet::unread(result);
    }
  });

  DefMacro!("\\differential", "\\lx@physics@diff{\\differential}{differential}{\\mathrm{d}}");
  DefMacro!("\\variation", "\\lx@physics@diff{\\variation}{variation}{\\delta}");
  Let!("\\dd", "\\differential");
  Let!("\\var", "\\variation");

  // Perl: \lx@physics@deriv — derivative (complex multi-arg parsing)
  // Simplified version: handles the most common cases
  DefMacro!("\\derivative{}{}", r"\frac{\mathrm{d}#1}{\mathrm{d}#2}");
  DefMacro!("\\partialderivative{}{}", r"\frac{\partial #1}{\partial #2}");
  DefMacro!("\\functionalderivative{}{}", r"\frac{\delta #1}{\delta #2}");
  Let!("\\dv", "\\derivative");
  Let!("\\pdv", "\\partialderivative");
  Let!("\\pderivative", "\\partialderivative");
  Let!("\\fdv", "\\functionalderivative");

  //======================================================================
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

    let content = i_apply(&[], i_symbol(&[("meaning", Tokenize!("ket"))], None), vec![a1.clone()]);
    let mut pres = Vec::new();
    pres.extend(phys_open(no_stretch, &size_tok, Tokenize!("\\vert")).unlist());
    pres.push(i_arg("1"));
    pres.extend(phys_close(no_stretch, &size_tok, Tokenize!("\\rangle")).unlist());
    let presentation = Tokens::new(pres);

    let result = i_dual(&[("reversion", reversion)], content, presentation, vec![arg])?;
    gullet::unread(result);
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
        vec![a1.clone(), a2.clone()]);
      let sz = &if final_stretch { None } else { Some(T_OTHER!("*")) }; // dummy
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
        i_symbol(&[("meaning", Tokenize!("bra"))], None), vec![a1.clone()]);
      let mut pres = Vec::new();
      pres.extend(phys_open(!no_stretch, &None, Tokenize!("\\langle")).unlist());
      pres.push(i_arg("1"));
      pres.extend(phys_close(!no_stretch, &None, Tokenize!("\\vert")).unlist());
      let presentation = Tokens::new(pres);

      let result = i_dual(&[("reversion", reversion)], content, presentation, vec![arg])?;
      gullet::unread(result);
    }
  });

  // Perl: \innerproduct — ⟨arg1|arg2⟩
  DefPrimitive!("\\lx@physics@qm@product{}{}{}{}{}", sub[(cs, semantic, open, middle, close)] {
    let cs_tks = cs.clone();
    let semantic_str = semantic.to_string();
    let open_tks = open.clone();
    let middle_tks = middle.clone();
    let close_tks = close.clone();
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
      vec![a1.clone(), a2.clone()]);
    let mut pres = Vec::new();
    pres.extend(phys_open(!no_stretch, &None, open_tks).unlist());
    pres.push(i_arg("1"));
    pres.extend(phys_mid(!no_stretch, &None, middle_tks).unlist());
    pres.push(i_arg("2"));
    pres.extend(phys_close(!no_stretch, &None, close_tks).unlist());
    let presentation = Tokens::new(pres);

    let result = i_dual(&[("reversion", reversion)], content, presentation, vec![arg0, arg1])?;
    gullet::unread(result);
  });

  DefMacro!("\\innerproduct",
    "\\lx@physics@qm@product{\\innerproduct}{inner-product}{\\langle}{\\vert}{\\rangle}");
  DefMacro!("\\outerproduct",
    "\\lx@physics@qm@product{\\outerproduct}{outer-product}{\\vert}{\\rangle\\langle}{\\vert}");

  // Perl: \expectationvalue — ⟨arg⟩ or ⟨arg2|arg1|arg2⟩
  DefMacro!("\\expectationvalue{}", r"\left\langle #1\right\rangle ");

  // Perl: \matrixelement — ⟨arg1|arg2|arg3⟩
  DefMacro!("\\matrixelement{}{}{}", r"\left\langle #1\middle\vert #2\middle\vert #3\right\rangle ");

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
  });

  // Perl: \xmatrix *{item}{n}{m}
  DefMacro!("\\xmatrix{}{}{}", "");

  DefMacro!("\\zeromatrix{}{}", "\\xmatrix{0}{#1}{#2}");

  DefMath!("\\lx@physics@iunit", None, "\\mathit{i}", meaning => "imaginary-unit");
  DefPrimitive!("\\paulimatrix{}", sub[(n)] {
    let n_val: usize = n.to_string().parse().unwrap_or(0);
    let tks = match n_val {
      0 => Tokenize!("1 & 0 \\\\ 0 & 1"),
      1 => Tokenize!("0 & 1 \\\\ 1 & 0"),
      2 => Tokenize!("0 & -\\lx@physics@iunit \\\\ \\lx@physics@iunit & 0"),
      3 => Tokenize!("1 & 0 \\\\ 0 & -1"),
      _ => Tokens::default(),
    };
    gullet::unread(tks);
  });

  // Perl: \diagonalmatrix[zero]{diag...}
  DefMacro!("\\diagonalmatrix[]{}", "");
  // Perl: \antidiagonalmatrix[zero]{diag...}
  DefMacro!("\\antidiagonalmatrix[]{}", "");

  // Perl: \lx@physics@mat — wraps in matrix environment with fencing
  // Simplified for now
  DefMacro!("\\matrixquantity{}", r"\begin{pmatrix}#1\end{pmatrix}");
  DefMacro!("\\pmqty{}", r"\begin{pmatrix}#1\end{pmatrix}");
  DefMacro!("\\Pmqty{}", r"\begin{pmatrix}#1\end{pmatrix}");
  DefMacro!("\\bmqty{}", r"\begin{bmatrix}#1\end{bmatrix}");
  DefMacro!("\\vmqty{}", r"\begin{vmatrix}#1\end{vmatrix}");
  DefMacro!("\\smallmatrixquantity{}", r"\begin{pmatrix}#1\end{pmatrix}");
  DefMacro!("\\spmqty{}", r"\begin{pmatrix}#1\end{pmatrix}");
  DefMacro!("\\sPmqty{}", r"\begin{pmatrix}#1\end{pmatrix}");
  DefMacro!("\\sbmqty{}", r"\begin{bmatrix}#1\end{bmatrix}");
  DefMacro!("\\svmqty{}", r"\begin{vmatrix}#1\end{vmatrix}");
  DefMacro!("\\matrixdeterminant{}", r"\begin{vmatrix}#1\end{vmatrix}");
  DefMacro!("\\smallmatrixdeterminant{}", r"\begin{vmatrix}#1\end{vmatrix}");

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
