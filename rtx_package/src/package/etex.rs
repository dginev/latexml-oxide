use crate::package::*;

LoadDefinitions!(outer_state, {
  // See http://tex.loria.fr/moteurs/etex_ref.html
  // Or better yet, see the full manual
  // http://texdoc.net/texmf-dist/doc/etex/base/etex_man.pdf
  // Section 3. The new features

  //======================================================================
  // 3.1 Additional control over expansion
  // \protected associates with the next defn
  // (note that it isn't actually used anywhere).
  DefPrimitive!("\\protected", sub[_stomach, _args, state] {
    state.set_prefix("protected");
  },
  is_prefix => true);

  // \detokenize
  DefMacro!("\\detokenize GeneralText", sub[gullet, (text), state] {
    Explode!(writable_tokens(&text, state)?)
  });

  // When building an expanded token list, the tokens resulting from the expansion
  // of \unexpanded are not expanded further (this is the same behaviour as is
  // exhibited by the tokens resulting from the expansion of
  // \the〈token variable〉in both TEX and ε-TEX).
  DefMacro!("\\unexpanded GeneralText", sub[gullet, (text), state] { text });

  // ======================================================================
  // 3.2. Provision for re-scanning already read text

  // \readline; like \read, but only spaces & other
  DefMacro!("\\readline Number SkipKeyword:to SkipSpaces Token", sub[gullet, (port, token), state] {
    let mouth_opt = if let Some(Stored::Mouth(mouth)) = LookupValue!(&format!("input_file:{port}")) {
      Some(Arc::clone(mouth)) } else { None };
    if let Some(mouth) = mouth_opt {
      let mut raw_line = mouth.write().unwrap().read_raw_line(false, state).unwrap_or_default();
      // DG: Can't we do this \endlinechar check in readRawLine ?!
      // DG:  and can't we make it *faster* ?
      if let Some(eol) = state.lookup_definition(&T_CS!("\\endlinechar")) {
        let eolv   = eol.value_of(Vec::new(), state).unwrap_or_default().value_of();
        if (eolv > 0) && (eolv <= 255) {
          raw_line.push(eolv as u8 as char);
        }
      } else {
        raw_line.push('\r');
      }

      DefMacro!(token, None, Tokens!(Explode!(raw_line)));
    }
  });

  DefMacro!("\\scantokens GeneralText", sub[gullet, (tokens), state] {
    gullet.open_mouth(
      Mouth::new(&writable_tokens(&tokens, state)?, None, state)?, true);
    Tokens!()
  });

  // #======================================================================
  // # 3.3 Environmental enquiries

  DefMacro!("\\eTeXrevision", sub[_gullet,_args,_state] { Explode!(".2") });
  DefRegister!("\\eTeXversion" => Number!(2));

  // \currentgrouplevel
  DefRegister!("\\currentgrouplevel", Number!(0), readonly => true,
    getter => sub[_args, state] { state.get_frame_depth() });

  // \currentgrouptype returns group types from 0..16 ; but what IS a "group type"?
  DefRegister!("\\currentgrouptype", Number!(0), readonly => true);

  // \ifcsname stuff \endcsname
  DefConditional!("\\ifcsname CSName", sub[gullet, (t), state] {
    state.lookup_meaning(&t).is_some()
  });

  // \ifdefined <token>
  DefConditional!("\\ifdefined Token", sub[gullet, (t), state] {
    state.lookup_meaning(&t).is_some()
  });

  // # ???
  DefRegister!("\\lastnodetype", Number::new(0));

  // #======================================================================
  // # 3.4 Generalization of the \mark concept: a class of \marks
  // # but since we don't manage Pages...

  DefPrimitive!("\\marks Number GeneralText", None);
  DefMacro!("\\topmarks Number", sub[gullet, (num), state] {});
  DefMacro!("\\firstmarks Number", sub[gullet, (num), state] {});
  DefMacro!("\\botmarks Number", sub[gullet, (num), state] {});
  DefMacro!("\\splitfirstmarks Number", sub[gullet, (num), state] {});
  DefMacro!("\\splitbotmarks Number", sub[gullet, (num), state] {});

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

  DefMacro!("\\beginL", "");
  DefMacro!("\\beginR", "");
  DefMacro!("\\endL", "");
  DefMacro!("\\endR", "");

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

  // DefConstructor('\middle Token', '#1',
  //   afterConstruct => sub {
  //     my ($document) = @_;
  //     my $current = $document->getNode;
  //     my $delim = $document->getLastChildElement($current) || $current;
  //     $document->setAttribute($delim, role     => 'MIDDLE');
  //     $document->setAttribute($delim, stretchy => 'true');
  //     return; });

  // # \unless someif
  DefConditional!("\\unless Token", sub[gullet, (if_token), state] {
    if let Some(Stored::Conditional(defn)) = state.lookup_definition_stored(&if_token) {
      if defn.conditional_type == ConditionalType::If {
        if let Some(ref closure) = defn.test {
          // Invert the if's test!
          let args = defn.read_arguments(gullet, state)?;
          return Ok(!(closure(gullet, args, state)?));
        }
      }
    }
    let msg = s!("\\unless should not be followed by {}",if_token.stringify());
    Error!("unexpected", if_token, gullet, state, msg);
    false
  });

  // #======================================================================
  // # \numexpr, \dimexpr, \gluexpr, \muexpr
  // # These read tokens doing simple parsing until \relax or the parse fails.
  // # since we don't know where it ends, we can't easily use Parse::RecDescent.
  // # They also act like a Register!
  // # $type is one of Number, Dimension, Glue or MuGlue
  fn etex_readexpr(gullet: &mut Gullet, rtype: RegisterType, state: &mut State) -> Result<RegisterValue> {
    let value = etex_readexpr_i(gullet, rtype, 0, state)?;
    if let Some(token) = gullet.read_token(state) {
      // Skip \relax
      if !(token == T_RELAX) {
        gullet.unread_one(token);
      }
    }
    Ok(value)
  }

  fn etex_readexpr_i(gullet: &mut Gullet, rtype: RegisterType, prec: usize, state: &mut State) -> Result<RegisterValue> {
    // Read a first value
    let token = match gullet.read_x_non_space(state)? {
      Some(t) => t,
      None => return Ok(RegisterValue::default()),
    };
    let mut value = if token == T_OTHER!("(") {
      let i_value = etex_readexpr_i(gullet, rtype, 0, state)?;
      let close = gullet.read_x_token(None, false, state)?; // close parenthesis should have terminated recursive call
      if close.is_none() || !(close == Some(T_OTHER!(")"))) {
        unimplemented!();
        //       Error('expected', ')', $gullet,
        //         "Missing close parenthesis in $type expr.", "Got " . ToString($close));
      }
      i_value
    } else {
      // Read core TeX value/register
      gullet.unread_one(token);
      gullet.read_value(rtype, state)?
    };

    // Now check for a following operator(s) & operand(s) (respecting precedence)
    while let Some(next) = gullet.read_x_non_space(state)? {
      if next == T_RELAX {
        gullet.unread_one(next); // leave the \relax for top-level to strip off.
        break;
      } else if next == T_OTHER!("+") && prec < 1 {
        value = value.add(etex_readexpr_i(gullet, rtype, 1, state)?);
      } else if next == T_OTHER!("-") && prec < 1 {
        value = value.subtract(etex_readexpr_i(gullet, rtype, 1, state)?);
      } else if next == T_OTHER!("*") && prec < 2 {
        // multiplier should be pure number
        value = value.multiply(etex_readexpr_i(gullet, RegisterType::Number, 2, state)?);
      } else if next == T_OTHER!("/") && prec < 2 {
        // denominator should be pure number
        value = value.divideround(etex_readexpr_i(gullet, RegisterType::Number, 2, state)?);
      } else {
        // anything else, we're done.
        gullet.unread_one(next);
        break;
      }
    }
    Ok(value)
  }

  DefParameterType!(NumExpr, sub[gullet, _inner, _extra, state] {
    etex_readexpr(gullet, RegisterType::Number, state)?
  });
  DefParameterType!(DimExpr, sub[gullet, _inner, _extra, state] {
    etex_readexpr(gullet, RegisterType::Dimension, state)?
  });
  DefParameterType!(GlueExpr, sub[gullet, _inner, _extra, state] {
    etex_readexpr(gullet, RegisterType::Glue, state)?
  });
  DefParameterType!(MuExpr, sub[gullet, _inner, _extra, state] {
    etex_readexpr(gullet, RegisterType::MuGlue, state)?
  });

  DefRegister!("\\numexpr NumExpr", Number::new(0), getter => sub[args, _state] {
    args.remove(0).expect_number()
  });
  DefRegister!("\\dimexpr DimExpr", Dimension::new(0), getter => sub[args, _state] {
    args.remove(0).expect_dimension()
  });
  DefRegister!("\\glueexpr GlueExpr", Glue::new(0), getter => sub[args, _state] {
    args.remove(0).expect_glue()
  });
  DefRegister!("\\muexpr MuExpr", MuGlue::new(0), getter => sub[args, _state] {
    args.remove(0).expect_mu_glue()
  });

  DefPrimitive!("\\pdftexcmds@directlua{}", None);

  // Not really sure where this comes from; pdftex?
  DefRegister!("\\synctex", Number::new(0));
  // #======================================================================
});
