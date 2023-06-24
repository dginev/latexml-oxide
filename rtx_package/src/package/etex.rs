use crate::package::*;

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
    let state = state!();
    let mouth_opt = if let Some(Stored::Mouth(mouth)) = state.lookup_value(&file_key) {
      Some(Rc::clone(mouth)) } else { None };
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

  DefMacro!("\\eTeXrevision", { Explode!(".2") });
  DefRegister!("\\eTeXversion" => Number::new(2));

  // \currentgrouplevel
  DefRegister!("\\currentgrouplevel", Number!(0), readonly => true,
    getter => { get_frame_depth() });

  // \currentgrouptype returns group types from 0..16 ; but what IS a "group type"?
  DefRegister!("\\currentgrouptype", Number!(0), readonly => true);

  // \ifcsname stuff \endcsname
  DefConditional!("\\ifcsname CSName", sub[(t)] {
    lookup_meaning(&t).is_some()
  });

  // \ifdefined <token>
  DefConditional!("\\ifdefined Token", sub[(t)] {
    lookup_meaning(&t).is_some()
  });

  // # ???
  DefRegister!("\\lastnodetype", Number::new(0));

  // #======================================================================
  // # 3.4 Generalization of the \mark concept: a class of \marks
  // # but since we don't manage Pages...

  DefPrimitive!("\\marks Number GeneralText", None);
  DefMacro!("\\topmarks Number", sub[(num)] {});
  DefMacro!("\\firstmarks Number", sub[(num)] {});
  DefMacro!("\\botmarks Number", sub[(num)] {});
  DefMacro!("\\splitfirstmarks Number", sub[(num)] {});
  DefMacro!("\\splitbotmarks Number", sub[(num)] {});

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

  // # \showtokens <generaltext>
  // DefPrimitive!("\\showtokens GeneralText",  sub {
  //   Note("> " . writableTokens($_[1]));
  //   Note($_[0]->getLocator->toString());
  //   return; });

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

  // TODO:
  // DefConstructor('\middle Token', '#1',
  //   afterConstruct => sub {
  //     my ($document) = @_;
  //     my $current = $document->getNode;
  //     my $delim = $document->getLastChildElement($current) || $current;
  //     $document->setAttribute($delim, role     => 'MIDDLE');
  //     $document->setAttribute($delim, stretchy => 'true');
  //     return; });

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
  fn etex_readexpr(
    rtype: RegisterType,
  ) -> Result<RegisterValue> {
    let value = etex_readexpr_i( rtype, 0)?;
    if let Some(token) = gullet::read_token()? {
      // Skip \relax
      if token != *TOKEN_RELAX {
        gullet::unread_one(token);
      }
    }
    Ok(value)
  }

  fn etex_readexpr_i(
    rtype: RegisterType,
    prec: usize,
  ) -> Result<RegisterValue> {
    // Read a first value
    let token = match gullet::read_x_non_space()? {
      Some(t) => t,
      None => return Ok(RegisterValue::default()),
    };
    let mut value = if token == T_OTHER!("(") {
      let i_value = etex_readexpr_i( rtype, 0)?;
      let close = gullet::read_x_token(None, false)?;
      // close parenthesis should have terminated recursive call
      if close.is_none() || !(close == Some(T_OTHER!(")"))) {
        unimplemented!();
        //       Error('expected', ')', $gullet,
        //         "Missing close parenthesis in $type expr.", "Got " . ToString($close));
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
        value = value.add(etex_readexpr_i( rtype, 1)?);
      } else if next == T_OTHER!("-") && prec < 1 {
        value = value.subtract(etex_readexpr_i( rtype, 1)?);
      } else if next == T_OTHER!("*") && prec < 2 {
        // multiplier should be pure number
        value = value.multiply(etex_readexpr_i( RegisterType::Number, 2)?);
      } else if next == T_OTHER!("/") && prec < 2 {
        // denominator should be pure number
        value = value.divideround(etex_readexpr_i( RegisterType::Number, 2)?);
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

  DefPrimitive!("\\pdftexcmds@directlua{}", None);

  // Not really sure where this comes from; pdftex?
  DefRegister!("\\synctex", Number::new(0));
  // #======================================================================
});
