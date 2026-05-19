//! eTeX — enhanced TeX enhancements.
//!
//! Mirrors `LaTeXML/blib/lib/LaTeXML/Engine/eTeX.pool.ltxml` line-by-line
//! (re-ordered 2026-04-26 to match Perl exactly per the pool parity audit).
//! Manual: <http://texdoc.net/texmf-dist/doc/etex/base/etex_man.pdf>
//!
//! Section numbering in comments below references that manual.

use crate::prelude::*;
use latexml_core::common::glue::FillCode;

/// Resolve a FontDef token to an Rc<Font>: look up fontinfo via definition CS name, fall back to
/// current font. Perl: `$font = $STATE->lookupValue('font')->merge(%$fontinfo);`
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


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  // Helpers used by definitions below. Defined first so all defs can refer.

  // Perl L153-193: etex_readexpr / etex_readexpr_i for \numexpr/\dimexpr/etc.
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
    let token = match gullet::read_x_non_space()? {
      Some(t) => t,
      None => return Ok(RegisterValue::default()),
    };
    let mut value = if token == T_OTHER!("(") {
      let i_value = etex_readexpr_i(rtype, 0)?;
      let close = gullet::read_x_token(None, false, None)?;
      if close.is_none() || close != Some(T_OTHER!(")")) {
        let got = close
          .map(|t| t.stringify().to_string())
          .unwrap_or_else(|| "EOF".to_string());
        let message = format!("Missing close parenthesis in {:?} expr. Got {}", rtype, got);
        Error!("expected", ")", message);
      }
      i_value
    } else {
      gullet::unread_one(token);
      gullet::read_value(rtype)?
    };

    while let Some(next) = gullet::read_x_non_space()? {
      if next == *TOKEN_RELAX {
        gullet::unread_one(next);
        break;
      } else if next == T_OTHER!("+") && prec < 1 {
        value = value.add(etex_readexpr_i(rtype, 1)?);
      } else if next == T_OTHER!("-") && prec < 1 {
        value = value.subtract(etex_readexpr_i(rtype, 1)?);
      } else if next == T_OTHER!("*") && prec < 2 {
        value = value.multiply(etex_readexpr_i(RegisterType::Number, 2)?);
      } else if next == T_OTHER!("/") && prec < 2 {
        value = value.divideround(etex_readexpr_i(RegisterType::Number, 2)?);
      } else {
        gullet::unread_one(next);
        break;
      }
    }
    Ok(value)
  }

  // Perl L275-288: eTeXPenaltiesGetter / eTeXPenaltiesSetter
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

  //======================================================================
  // 3.3 Tracing and Diagnostics — Perl L39-43
  DefRegister!("\\tracingassigns"    => Number::new(0));
  DefRegister!("\\tracinggroups"     => Number::new(0));
  DefRegister!("\\tracingifs"        => Number::new(0));
  DefRegister!("\\tracingscantokens" => Number::new(0));
  DefRegister!("\\tracingnesting"    => Number::new(0));

  // Perl L46: \showgroups
  DefPrimitive!("\\showgroups", None);

  // Perl L49-52: \showtokens — logs the token text (no document output)
  DefPrimitive!("\\showtokens GeneralText", sub[(tokens)] {
    Note!(s!("> {}", writable_tokens(&tokens)));
  });

  //======================================================================
  // 3.4 Status Enquiries — Perl L57-80
  DefMacro!("\\eTeXrevision", { Explode!(".6") });
  DefRegister!("\\eTeXversion" => Number::new(2));

  DefRegister!("\\interactionmode" => Number::new(0));

  DefRegister!("\\currentiflevel",  Number!(0), readonly => true);
  DefRegister!("\\currentifbranch", Number!(0), readonly => true);
  DefRegister!("\\currentiftype",   Number!(0), readonly => true);

  DefRegister!("\\currentgrouplevel", Number!(0), readonly => true,
    getter => { get_frame_depth() });

  DefRegister!("\\currentgrouptype", Number!(0), readonly => true);

  DefRegister!("\\lastnodetype", Number::new(0));

  // \fontcharht / \fontcharwd / \fontchardp / \fontcharic — Perl L86-113
  DefParameterType!(FontDef, sub[_inner, _extra] {
    gullet::read_token()?.unwrap_or(T_CS!("\\relax"))
  });
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
  // Perl notes "Not actually computed here (yet)"
  DefRegister!("\\fontcharic FontDef Number", Dimension::new(0),
  readonly => true,
  getter => sub[args] {
    let _font_tok = args.remove(0);
    let _code     = args.remove(0);
    Some(RegisterValue::Dimension(Dimension::new(0)))
  });

  // \parshapeindent / \parshapelength / \parshapedimen — Perl L119-142
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

  //======================================================================
  // 3.5 Expressions — Perl L195-213
  DefParameterType!(NumExpr, sub[_inner, _extra] {
    etex_readexpr(RegisterType::Number)?
  });
  DefParameterType!(DimExpr, sub[_inner, _extra] {
    etex_readexpr(RegisterType::Dimension)?
  });
  DefParameterType!(GlueExpr, sub[_inner, _extra] {
    etex_readexpr(RegisterType::Glue)?
  });
  DefParameterType!(MuExpr, sub[_inner, _extra] {
    etex_readexpr(RegisterType::MuGlue)?
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

  // Parts of Glue — Perl L207-214
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

  //======================================================================
  // 3.6 Additional Registers and Marks — Perl L221-226
  DefPrimitive!("\\marks Number GeneralText", None);
  def_macro_noop("\\topmarks Number")?;
  def_macro_noop("\\firstmarks Number")?;
  def_macro_noop("\\botmarks Number")?;
  def_macro_noop("\\splitfirstmarks Number")?;
  def_macro_noop("\\splitbotmarks Number")?;

  //======================================================================
  // 3.7 Input Handling — Perl L233-258
  DefMacro!("\\readline Number SkipKeyword:to SkipSpaces Token", sub[(port, token)] {
    let file_key = format!("input_file:{port}");
    let mouth_opt = with_value(&file_key, |v| match v {
      Some(Stored::Mouth(mouth)) => Some(Rc::clone(mouth)),
      _ => None,
    });
    if let Some(mouth) = mouth_opt {
      let mut raw_line = mouth.borrow_mut().read_raw_line(false).unwrap_or_default();
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

  DefRegister!("\\everyeof", Tokens!());

  //======================================================================
  // 3.8 Breaking Paragraphs into Lines — Perl L264, L290-301
  DefRegister!("\\lastlinefit", Number::new(0));

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

  //======================================================================
  // 3.9 Math Formulas — Perl L306
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

  //======================================================================
  // 3.10 Hyphenation — Perl L318
  DefRegister!("\\savinghyphcodes" => Number::new(0));

  //======================================================================
  // 3.11 Discarded Items — Perl L322-324
  DefRegister!("\\savingvdiscards" => Number::new(0));
  DefPrimitive!("\\pagediscards", None);
  DefPrimitive!("\\splitdiscards", None);

  //======================================================================
  // 3.12 Expandable Commands — Perl L330-357
  DefConditional!("\\ifdefined Token", sub[(t)] {
    lookup_meaning(&t).is_some()
  });

  // \ifcsname stuff \endcsname.
  // Uses CSNameQuiet — unlike \csname, \ifcsname does NOT emit errors
  // for non-expandable CS tokens encountered during expansion (TeX §506-507).
  DefConditional!("\\ifcsname CSNameQuiet", sub[(t)] {
    lookup_meaning(&t).is_some()
  });

  // \ifincsname — eTeX (TeX §506-507): true when expansion is happening
  // inside a `\csname...\endcsname` construction. LaTeXML does not have
  // a separate "inside csname" mode (it expands eagerly), so this is
  // always false — matching Perl LaTeXML's same shortcut. Rust extra
  // (Perl LaTeXML does not define this), placed near the other
  // expandable-conditional primitives.
  DefConditional!("\\ifincsname", { false });

  DefConditional!("\\unless Token", sub[(if_token)] {
    if let Some(Stored::Conditional(defn)) = lookup_definition_stored(&if_token)? {
      if defn.conditional_type == ConditionalType::If {
        if let Some(ref test) = defn.test {
          let args = defn.read_arguments()?;
          return Ok(!(test(args)?));
        }
      }
    }
    let msg = s!("\\unless should not be followed by {}", if_token.stringify());
    Error!("unexpected", if_token, msg);
    false
  });

  DefMacro!("\\unexpanded GeneralText", "#1");

  DefMacro!("\\detokenize GeneralText", sub[(text)] {
    Explode!(writable_tokens(&text))
  });

  //======================================================================
  // 4.1 Mixed-Direction Typesetting — Perl L367-386
  DefRegister!("\\TeXXeTstate" => Number::new(0));

  def_macro_noop("\\beginL")?;
  def_macro_noop("\\beginR")?;
  def_macro_noop("\\endL")?;
  def_macro_noop("\\endR")?;

  DefRegister!("\\predisplaydirection" => Number::new(0));

  //======================================================================
  // 3.1 Additional control over expansion (positioned after section 4 in
  // the Perl source) — Perl L393
  DefPrimitive!("\\protected", {
    set_prefix("protected");
  },
  is_prefix => true);

  //======================================================================
  // X.X Orphans / pdfTeX-leftover entries — Perl L399-407
  DefPrimitive!("\\pdftexcmds@directlua{}", None);
  DefRegister!("\\synctex", Number::new(0));
  def_macro_noop("\\reserveinserts{}")?;

  //======================================================================
  // etex.sty register-allocator macros (etex.sty L332-348). Real defs
  // use `\et@xglob`/`\et@xloc` to allocate from extended register
  // pools (Numbers 256+ for count/dimen/etc.). For our purposes the
  // semantic is "allocate a new register"; forward to LaTeX's
  // `\newcount`/`\newdimen`/etc. which already exist.
  //
  // Glob* variants allocate globally; loc* variants locally.
  // In LaTeXML's flat-state model these are effectively equivalent.
  //
  // Witness: arXiv:2506.16610 / .16657 / .20642 (papers via etex.sty
  // raw-load + linegoal.sty / etextools / similar). Rust 2 → 0
  // expected, beating Perl=3.
  DefMacro!("\\globcount",  "\\newcount");
  DefMacro!("\\loccount",   "\\newcount");
  DefMacro!("\\globdimen",  "\\newdimen");
  DefMacro!("\\locdimen",   "\\newdimen");
  DefMacro!("\\globskip",   "\\newskip");
  DefMacro!("\\locskip",    "\\newskip");
  DefMacro!("\\globmuskip", "\\newmuskip");
  DefMacro!("\\locmuskip",  "\\newmuskip");
  DefMacro!("\\globbox",    "\\newbox");
  DefMacro!("\\locbox",     "\\newbox");
  DefMacro!("\\globtoks",   "\\newtoks");
  DefMacro!("\\loctoks",    "\\newtoks");
  DefMacro!("\\globmarks",  "\\newmarks");
  DefMacro!("\\locmarks",   "\\newmarks");
});
