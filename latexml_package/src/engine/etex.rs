use crate::prelude::*;
use latexml_core::common::glue::FillCode;

/// Resolve a FontDef token to an Rc<Font>: look up fontinfo via definition CS name, fall back to
/// current font. Perl: $font = $STATE->lookupValue('font')->merge(%$fontinfo);
fn fontchar_lookup_font(font_tok: &Token) -> Option<Rc<Font>> {
  // Resolve through definition to get actual CS name (e.g. \font -> \tenrm)
  let key = if let Ok(Some(defn)) = lookup_definition(font_tok) {
    s!("fontinfo_{}", defn.get_cs_name())
  } else {
    s!("fontinfo_{}", font_tok)
  };
  with_value(&key, |v| {
    if let Some(Stored::Font(f)) = v {
      Some(Rc::clone(f))
    } else {
      None
    }
  })
  .or_else(lookup_font)
}

LoadDefinitions!({
  // See http://tex.loria.fr/moteurs/etex_ref.html
  // Or better yet, see the full manual
  // http://texdoc.net/texmf-dist/doc/etex/base/etex_man.pdf
  // Section 3. The new features

  //======================================================================
  // 3.1 Additional control over expansion
  // \protected associates with the next defn
  // (note that it isn't actually used anywhere).
  DefPrimitive!("\\protected", {
    set_prefix("protected");
  },
  is_prefix => true);

  // \detokenize
  DefMacro!("\\detokenize GeneralText", sub[(text)] {
    Explode!(writable_tokens(&text))
  });

  // When building an expanded token list, the tokens resulting from the expansion
  // of \unexpanded are not expanded further (this is the same behaviour as is
  // exhibited by the tokens resulting from the expansion of
  // \the〈token variable〉in both TEX and ε-TEX).
  DefMacro!("\\unexpanded GeneralText", "#1");

  // ======================================================================
  // 3.2. Provision for re-scanning already read text

  // \readline; like \read, but only spaces & other
  DefMacro!("\\readline Number SkipKeyword:to SkipSpaces Token", sub[(port, token)] {
    let file_key = format!("input_file:{port}");
    let mouth_opt = with_value(&file_key, |v| match v {
      Some(Stored::Mouth(mouth)) => Some(Rc::clone(mouth)),
      _ => None,
    });
    if let Some(mouth) = mouth_opt {
      let mut raw_line = mouth.borrow_mut().read_raw_line(false).unwrap_or_default();
      // DG: Can't we do this \endlinechar check in readRawLine ?!
      // DG:  and can't we make it *faster* ?
      if let Some(eol) = lookup_definition(&T_CS!("\\endlinechar"))? {
        let eolv   = eol.value_of(Vec::new()).unwrap_or_default().value_of();
        if (eolv > 0) && (eolv <= 255) {
          raw_line.push(eolv as u8 as char);
        }
      } else {
        raw_line.push('\r');
      }

      DefMacro!(token, None, Tokens!(Explode!(raw_line)));
    }
  });

  DefMacro!("\\scantokens GeneralText", sub[(tokens)] {
    gullet::open_mouth(
      Mouth::new(&writable_tokens(&tokens), None)?, true);
    Tokens!()
  });

  //======================================================================
  // 3.3 Environmental enquiries

  DefMacro!("\\eTeXrevision", { Explode!(".6") });
  DefRegister!("\\eTeXversion" => Number::new(2));

  // Conditional almost tracks this information, but not quite in the needed form. Punt!
  DefRegister!("\\currentiflevel",  Number!(0), readonly => true);
  DefRegister!("\\currentifbranch", Number!(0), readonly => true);
  DefRegister!("\\currentiftype",   Number!(0), readonly => true);

  // \currentgrouplevel
  DefRegister!("\\currentgrouplevel", Number!(0), readonly => true,
    getter => { get_frame_depth() });

  // \currentgrouptype returns group types from 0..16 ; but what IS a "group type"?
  DefRegister!("\\currentgrouptype", Number!(0), readonly => true);

  // \ifcsname stuff \endcsname
  // Uses CSNameQuiet — unlike \csname, \ifcsname does NOT emit errors
  // for non-expandable CS tokens encountered during expansion (TeX §506-507)
  DefConditional!("\\ifcsname CSNameQuiet", sub[(t)] {
    lookup_meaning(&t).is_some()
  });

  // \ifdefined <token>
  DefConditional!("\\ifdefined Token", sub[(t)] {
    lookup_meaning(&t).is_some()
  });

  // \ifincsname — eTeX (TeX §506-507): true when expansion is happening
  // inside a `\csname...\endcsname` construction. LaTeXML does not have
  // a separate "inside csname" mode (it expands eagerly), so this is
  // always false — matching Perl LaTeXML's same shortcut.
  DefConditional!("\\ifincsname", { false });

  // # ???
  DefRegister!("\\lastnodetype", Number::new(0));

  // \fontcharht <font><8bit>
  // \fontcharwd <font><8bit>
  // \fontchardp <font><8bit>
  // \fontcharic <font><8bit>
  DefParameterType!(FontDef, sub[_inner, _extra] {
    gullet::read_token()?.unwrap_or(T_CS!("\\relax"))
  });

  // Perl: $font = $STATE->lookupValue('font')->merge(%$fontinfo);
  // Rust stores the full Font at fontinfo_{cs}, so we use it directly.
  // If not found, fall back to the current font from state.
  DefRegister!("\\fontcharht FontDef Number", Dimension::new(0),
  readonly => true,
  getter => sub[args] {
    let font_tok = args.remove(0).expected_token();
    let code = args.remove(0).expect_number().value_of();
    let font_rc = fontchar_lookup_font(&font_tok);
    if let Some(font) = font_rc {
      if let Some(ch) = char::from_u32(code as u32) {
        let mut buf = [0u8; 4];
        let key = ch.encode_utf8(&mut buf);
        let (_, h, _) = font.compute_string_size(key, SymHashMap::default());
        return Some(RegisterValue::Dimension(h));
      }
    }
    Some(RegisterValue::Dimension(Dimension::new(0)))
  });

  DefRegister!("\\fontcharwd FontDef Number", Dimension::new(0),
  readonly => true,
  getter => sub[args] {
    let font_tok = args.remove(0).expected_token();
    let code = args.remove(0).expect_number().value_of();
    let font_rc = fontchar_lookup_font(&font_tok);
    if let Some(font) = font_rc {
      if let Some(ch) = char::from_u32(code as u32) {
        let mut buf = [0u8; 4];
        let key = ch.encode_utf8(&mut buf);
        let (w, _, _) = font.compute_string_size(key, SymHashMap::default());
        return Some(RegisterValue::Dimension(w));
      }
    }
    Some(RegisterValue::Dimension(Dimension::new(0)))
  });

  DefRegister!("\\fontchardp FontDef Number", Dimension::new(0),
  readonly => true,
  getter => sub[args] {
    let font_tok = args.remove(0).expected_token();
    let code = args.remove(0).expect_number().value_of();
    let font_rc = fontchar_lookup_font(&font_tok);
    if let Some(font) = font_rc {
      if let Some(ch) = char::from_u32(code as u32) {
        let mut buf = [0u8; 4];
        let key = ch.encode_utf8(&mut buf);
        let (_, _, d) = font.compute_string_size(key, SymHashMap::default());
        return Some(RegisterValue::Dimension(d));
      }
    }
    Some(RegisterValue::Dimension(Dimension::new(0)))
  });

  // Perl: also computes via computeStringSize but notes "Not actually computed here (yet)"
  DefRegister!("\\fontcharic FontDef Number", Dimension::new(0),
  readonly => true,
  getter => sub[args] {
    let _font_tok = args.remove(0);
    let _code     = args.remove(0);
    Some(RegisterValue::Dimension(Dimension::new(0)))
  });

  // \parshapeindent, \parshapelength, \parshapedimen
  // Assuming parshape is stored as an even list of [indent0, length0, indent1, length1, ...]
  // These access the indentation or length of the n-th (1-based) line, or the last line.
  DefRegister!("\\parshapeindent Number", Dimension::new(0),
  readonly => true,
  getter => sub[args] {
    let n = {
      let v = args.remove(0).value_of();
      if v < 0 { 0usize } else { v as usize }
    };
    if n == 0 {
      return Some(RegisterValue::Dimension(Dimension::new(0)));
    }
    with_value("parshape", |v| {
      if let Some(Stored::VecDequeStored(shape)) = v {
        let idx = 2 * n - 2;
        let d = if idx < shape.len() {
          shape.get(idx)
        } else {
          // fallback: second-to-last element (last indent)
          shape.get(shape.len().saturating_sub(2))
        };
        d.and_then(|s| if let Stored::Dimension(dim) = s { Some(*dim) } else { None })
          .map(RegisterValue::Dimension)
          .or(Some(RegisterValue::Dimension(Dimension::new(0))))
      } else {
        Some(RegisterValue::Dimension(Dimension::new(0)))
      }
    })
  });

  DefRegister!("\\parshapelength Number", Dimension::new(0),
  readonly => true,
  getter => sub[args] {
    let n = {
      let v = args.remove(0).value_of();
      if v < 0 { 0usize } else { v as usize }
    };
    if n == 0 {
      return Some(RegisterValue::Dimension(Dimension::new(0)));
    }
    with_value("parshape", |v| {
      if let Some(Stored::VecDequeStored(shape)) = v {
        let idx = 2 * n - 1;
        let d = if idx < shape.len() {
          shape.get(idx)
        } else {
          // fallback: last element
          shape.back()
        };
        d.and_then(|s| if let Stored::Dimension(dim) = s { Some(*dim) } else { None })
          .map(RegisterValue::Dimension)
          .or(Some(RegisterValue::Dimension(Dimension::new(0))))
      } else {
        Some(RegisterValue::Dimension(Dimension::new(0)))
      }
    })
  });

  DefRegister!("\\parshapedimen Number", Dimension::new(0),
  readonly => true,
  getter => sub[args] {
    let n = {
      let v = args.remove(0).value_of();
      if v < 0 { 0usize } else { v as usize }
    };
    if n == 0 {
      return Some(RegisterValue::Dimension(Dimension::new(0)));
    }
    with_value("parshape", |v| {
      if let Some(Stored::VecDequeStored(shape)) = v {
        let primary_idx = n - 1;
        let d = if primary_idx < shape.len() {
          shape.get(primary_idx)
        } else {
          // fallback: odd n-1 → last (a length), even n-1 → second-to-last (an indent)
          let fallback_idx = if (n - 1) % 2 == 0 {
            shape.len().saturating_sub(2)
          } else {
            shape.len().saturating_sub(1)
          };
          shape.get(fallback_idx)
        };
        d.and_then(|s| if let Stored::Dimension(dim) = s { Some(*dim) } else { None })
          .map(RegisterValue::Dimension)
          .or(Some(RegisterValue::Dimension(Dimension::new(0))))
      } else {
        Some(RegisterValue::Dimension(Dimension::new(0)))
      }
    })
  });

  // #======================================================================
  // # 3.4 Generalization of the \mark concept: a class of \marks
  // # but since we don't manage Pages...

  DefPrimitive!("\\marks Number GeneralText", None);
  DefMacro!("\\topmarks Number", None);
  DefMacro!("\\firstmarks Number", None);
  DefMacro!("\\botmarks Number", None);
  DefMacro!("\\splitfirstmarks Number", None);
  DefMacro!("\\splitbotmarks Number", None);

  // #======================================================================
  // # 3.5 Bi-directional typesetting: the TeX--XeT primitives

  // # Should these simply ouput some unicode direction changers,
  // # [Things like:
  // #  202A;LEFT-TO-RIGHT EMBEDDING;Cf;0;LRE;;;;;N;;;;;
  // #  202B;RIGHT-TO-LEFT EMBEDDING;Cf;0;RLE;;;;;N;;;;;
  // #  202C;POP DIRECTIONAL FORMATTING;Cf;0;PDF;;;;;N;;;;;
  // #  202D;LEFT-TO-RIGHT OVERRIDE;Cf;0;LRO;;;;;N;;;;;
  // #  202E;RIGHT-TO-LEFT OVERRIDE;Cf;0;RLO;;;;;N;;;;;
  // # ]
  // # or do we need to do some more intelligent tracking of modes
  // # and directionality?
  // # Presumably we can't rely on the material itself being directional.

  // By leaving this 0, we're saying "Don't use these features"!
  DefRegister!("\\TeXXeTstate" => Number::new(0));

  DefMacro!("\\beginL", None);
  DefMacro!("\\beginR", None);
  DefMacro!("\\endL", None);
  DefMacro!("\\endR", None);

  DefRegister!("\\predisplaydirection" => Number::new(0)); // ???

  // #======================================================================
  // # 3.6 Additional debugging features
  DefRegister!("\\interactionmode" => Number::new(0));

  // # Should show all open groups & their type.
  DefPrimitive!("\\showgroups", None);

  // \showtokens <generaltext> — logs the token text (no document output)
  DefPrimitive!("\\showtokens GeneralText", sub[(tokens)] {
    Note!(s!("> {}", writable_tokens(&tokens)));
  });

  DefRegister!("\\tracingassigns"    => Number::new(0)); // ???
  DefRegister!("\\tracinggroups"     => Number::new(0));
  DefRegister!("\\tracingifs"        => Number::new(0)); // ???
  DefRegister!("\\tracingscantokens" => Number::new(0));
  DefRegister!("\\tracingnesting"    => Number::new(0));
  DefRegister!("\\savingvdiscards"   => Number::new(0));
  DefRegister!("\\savinghyphcodes"   => Number::new(0));

  // #======================================================================
  // # 3.7 Miscellaneous primitives

  // # \everyeof
  // # NOTE: These tokens are NOT used anywhere (yet?)
  DefRegister!("\\everyeof", Tokens!());

  // \middle Token — sets role='MIDDLE' and stretchy='true' on the produced delimiter element
  DefConstructor!("\\middle Token", "#1",
  after_construct => sub[document, _whatsit] {
    let current = document.get_node().clone();
    let delim_opt = current.get_child_nodes()
      .into_iter()
      .filter(|n| n.get_type() == Some(NodeType::ElementNode))
      .last();
    let mut delim = delim_opt.unwrap_or_else(|| current.clone());
    document.set_attribute(&mut delim, "role", "MIDDLE")?;
    document.set_attribute(&mut delim, "stretchy", "true")?;
  });

  // \unless someif
  DefConditional!("\\unless Token", sub[(if_token)] {
    if let Some(Stored::Conditional(defn)) = lookup_definition_stored(&if_token)? {
      if defn.conditional_type == ConditionalType::If {
        if let Some(ref test) = defn.test {
          // Invert the if's test!
          let args = defn.read_arguments()?;
          return Ok(!(test( args)?));
        }
      }
    }
    let msg = s!("\\unless should not be followed by {}",if_token.stringify());
    Error!("unexpected", if_token, msg);
    false
  });

  // #======================================================================
  // # \numexpr, \dimexpr, \gluexpr, \muexpr
  // # These read tokens doing simple parsing until \relax or the parse fails.
  // # since we don't know where it ends, we can't easily use Parse::RecDescent.
  // # They also act like a Register!
  // # $type is one of Number, Dimension, Glue or MuGlue
  fn is_relax_meaning(token: &Token) -> bool {
    if *token == *TOKEN_RELAX {
      return true;
    }
    if token.get_catcode() != Catcode::CS {
      return false;
    }
    matches!(state::lookup_meaning(token),
      Some(Stored::Primitive(ref p)) if *p.get_cs() == *TOKEN_RELAX)
  }

  fn etex_readexpr(rtype: RegisterType) -> Result<RegisterValue> {
    let value = etex_readexpr_i(rtype, 0)?;
    if let Some(token) = gullet::read_token()? {
      // Skip \relax or token with \relax meaning (\__int_eval_end: etc.)
      if !is_relax_meaning(&token) {
        gullet::unread_one(token);
      }
    }
    Ok(value)
  }

  fn etex_readexpr_i(rtype: RegisterType, prec: usize) -> Result<RegisterValue> {
    // Read a first value
    let token = match gullet::read_x_non_space()? {
      Some(t) => t,
      None => return Ok(RegisterValue::default()),
    };
    let mut value = if token == T_OTHER!("(") {
      let i_value = etex_readexpr_i(rtype, 0)?;
      let close = gullet::read_x_token(None, false, None)?;
      // close parenthesis should have terminated recursive call
      if close.is_none() || close != Some(T_OTHER!(")")) {
        let got = close
          .map(|t| t.stringify().to_string())
          .unwrap_or_else(|| "EOF".to_string());
        let message = format!("Missing close parenthesis in {:?} expr. Got {}", rtype, got);
        Error!("expected", ")", message);
      }
      i_value
    } else {
      // Read core TeX value/register
      gullet::unread_one(token);
      gullet::read_value(rtype)?
    };

    // Now check for a following operator(s) & operand(s) (respecting precedence)
    while let Some(next) = gullet::read_x_non_space()? {
      if next == *TOKEN_RELAX {
        gullet::unread_one(next); // leave the \relax for top-level to strip off.
        break;
      } else if next == T_OTHER!("+") && prec < 1 {
        value = value.add(etex_readexpr_i(rtype, 1)?);
      } else if next == T_OTHER!("-") && prec < 1 {
        value = value.subtract(etex_readexpr_i(rtype, 1)?);
      } else if next == T_OTHER!("*") && prec < 2 {
        // multiplier should be pure number
        value = value.multiply(etex_readexpr_i(RegisterType::Number, 2)?);
      } else if next == T_OTHER!("/") && prec < 2 {
        // denominator should be pure number
        value = value.divideround(etex_readexpr_i(RegisterType::Number, 2)?);
      } else {
        // anything else, we're done.
        gullet::unread_one(next);
        break;
      }
    }
    Ok(value)
  }

  DefParameterType!(NumExpr, sub[_inner, _extra] {
    etex_readexpr( RegisterType::Number)?
  });
  DefParameterType!(DimExpr, sub[_inner, _extra] {
    etex_readexpr( RegisterType::Dimension)?
  });
  DefParameterType!(GlueExpr, sub[_inner, _extra] {
    etex_readexpr( RegisterType::Glue)?
  });
  DefParameterType!(MuExpr, sub[_inner, _extra] {
    etex_readexpr( RegisterType::MuGlue)?
  });

  DefRegister!("\\numexpr NumExpr", Number::new(0), getter => sub[args] {
    args.remove(0).expect_number()
  });
  DefRegister!("\\dimexpr DimExpr", Dimension::new(0), getter => sub[args] {
    args.remove(0).expect_dimension()
  });
  DefRegister!("\\glueexpr GlueExpr", Glue::new(0), getter => sub[args] {
    args.remove(0).expect_glue()
  });
  DefRegister!("\\muexpr MuExpr", MuGlue::new(0), getter => sub[args] {
    args.remove(0).expect_mu_glue()
  });

  // Parts of Glue
  DefRegister!("\\gluestretchorder Glue", Number::new(0),
  getter => sub[args] {
    let g = args.remove(0).expect_glue();
    let order: i64 = match g.pfill {
      None                  => 0,
      Some(FillCode::Fil)   => 1,
      Some(FillCode::Fill)  => 2,
      Some(FillCode::Filll) => 3,
    };
    Some(RegisterValue::Number(Number::new(order)))
  });

  DefRegister!("\\glueshrinkorder Glue", Number::new(0),
  getter => sub[args] {
    let g = args.remove(0).expect_glue();
    let order: i64 = match g.mfill {
      None                  => 0,
      Some(FillCode::Fil)   => 1,
      Some(FillCode::Fill)  => 2,
      Some(FillCode::Filll) => 3,
    };
    Some(RegisterValue::Number(Number::new(order)))
  });

  DefRegister!("\\gluestretch Glue", Dimension::new(0),
  getter => sub[args] {
    let g = args.remove(0).expect_glue();
    Some(RegisterValue::Dimension(Dimension::new(g.plus.unwrap_or(0))))
  });

  DefRegister!("\\glueshrink Glue", Dimension::new(0),
  getter => sub[args] {
    let g = args.remove(0).expect_glue();
    Some(RegisterValue::Dimension(Dimension::new(g.minus.unwrap_or(0))))
  });

  DefPrimitive!("\\pagediscards", None);
  DefPrimitive!("\\splitdiscards", None);
  DefMacro!("\\reserveinserts{}", None);

  DefPrimitive!("\\pdftexcmds@directlua{}", None);

  // \lastlinefit
  DefRegister!("\\lastlinefit", Number::new(0));

  // Penalty array registers (\interlinepenalties, \clubpenalties, etc.)
  // Mirrors Perl's eTeXPenaltiesGetter / eTeXPenaltiesSetter helpers.
  // The getter reads an index N from the gullet (not from args), returns the N-th penalty.
  // The setter receives the count N as its assigned value, then reads N penalties from the gullet.
  fn etex_penalties_getter(name: &str) -> Option<RegisterValue> {
    let n = gullet::read_number().unwrap_or_default().value_of();
    let n = if n < 0 { 0i64 } else { n } as usize;
    if n == 0 {
      return Some(RegisterValue::Number(Number::new(0)));
    }
    with_value(name, |v| {
      if let Some(Stored::VecDequeStored(p)) = v {
        let idx = n - 1;
        let item = if idx < p.len() { p.get(idx) } else { p.back() };
        item
          .and_then(|s| {
            if let Stored::Number(num) = s {
              Some(*num)
            } else {
              None
            }
          })
          .map(RegisterValue::Number)
          .or(Some(RegisterValue::Number(Number::new(0))))
      } else {
        Some(RegisterValue::Number(Number::new(0)))
      }
    })
  }

  fn etex_penalties_setter(name: &str, value: RegisterValue) {
    let n = value.value_of();
    let n = if n < 0 { 0i64 } else { n } as usize;
    let mut penalties = VecDeque::new();
    for _ in 0..n {
      let p = gullet::read_number().unwrap_or_default();
      penalties.push_back(Stored::Number(p));
    }
    assign_value(
      name,
      if n > 0 {
        Stored::VecDequeStored(penalties)
      } else {
        Stored::None
      },
      None,
    );
  }

  DefRegister!("\\interlinepenalties", Number::new(0),
    getter => sub[_args] { etex_penalties_getter("interlinepenalties") },
    setter => sub[value, _scope, _args] { etex_penalties_setter("interlinepenalties", value) });

  DefRegister!("\\clubpenalties", Number::new(0),
    getter => sub[_args] { etex_penalties_getter("clubpenalties") },
    setter => sub[value, _scope, _args] { etex_penalties_setter("clubpenalties", value) });

  DefRegister!("\\widowpenalties", Number::new(0),
    getter => sub[_args] { etex_penalties_getter("widowpenalties") },
    setter => sub[value, _scope, _args] { etex_penalties_setter("widowpenalties", value) });

  DefRegister!("\\displaywidowpenalties", Number::new(0),
    getter => sub[_args] { etex_penalties_getter("displaywidowpenalties") },
    setter => sub[value, _scope, _args] { etex_penalties_setter("displaywidowpenalties", value) });

  // Not really sure where this comes from; pdftex?
  DefRegister!("\\synctex", Number::new(0));
  // #======================================================================
});
