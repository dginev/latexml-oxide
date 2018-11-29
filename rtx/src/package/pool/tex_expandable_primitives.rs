use package::*;

// Hmm... I wonder, should getString itself be dealing with escapechar?
fn escapechar(state: &State) -> String {
  let code = match state.lookup_register("\\escapechar", Vec::new()) {
    Some(RegisterValue::Number(v)) => v.value_of(),
    _ => -1,
  };
  if code >= 0 && code <= 255 {
    let char_code = (code as u8) as char;
    char_code.to_string()
  } else {
    String::new()
  }
}

pub fn load_definitions(core_state: &mut State) -> Result<()> {
  SetupBindingMacros!(core_state);

  // DefConditional('\ifnum Number Token Number',       sub { compare($_[1], $_[2], $_[3]); });
  // DefConditional('\ifdim Dimension Token Dimension', sub { compare($_[1], $_[2], $_[3]); });
  // DefConditional('\ifodd Number',                    sub { $_[1]->valueOf % 2; });

  // NOTE: We don't KNOW if we're in vertical, horizontal or inner mode!!!!!!!
  DefConditional!("\\ifvmode", sub[gullet,args,state] {Ok(false)});
  DefConditional!("\\ifhmode", sub[gullet,args,state] {Ok(false)});
  DefConditional!("\\ifinner", sub[gullet,args,state] {Ok(false)});
  
  DefConditional!("\\ifmmode", sub[gullet,args,state] {Ok(state.lookup_bool("IN_MATH"))});

  DefConditional!("\\if XToken XToken", sub[gullet, args, state] {
    unpack!(args=>tokens1, tokens2);
    let token1 : Token = tokens1.into();
    let token2 : Token = tokens2.into();
    Ok(token1.get_charcode() == token2.get_charcode())
  });


  DefConditional!("\\ifx Token Token", sub[gullet, args, state] {
    unpack!(args => tokens1, tokens2);
    let token1 : Token = tokens1.into();
    let token2 : Token = tokens2.into();
    let xequals = XEquals!(&token1, &token2, state);
    Ok(xequals)
  });


  // TODO:
  // DefConditionalI('\iftrue',  undef, sub { 1; });
  // DefConditionalI('\iffalse', undef, sub { 0; });

  //======================================================================
  // This makes \relax disappear completely after digestion
  // (which seems most TeX like).
  DefPrimitiveI!("\\relax", noprimitive!() );
  //// However, this keeps a box, so it can appear in UnTeX
  ////// DefPrimitive('\relax',undef);
  //// But if you do that, you've got to watch out since it usually
  //// shouldn't be a box; See the isRelax code in handleScripts, below

  DefMacro!("\\number Number", sub[gullet, args, state] {
    unpack!(args=>vals);
    let mut args = vals.unlist();
    let num = args.remove(0);
    Ok(Explode!(num.value_of(args, state).unwrap_or_default().to_string()).into())
  });

  // define it here (only approxmiately), since it's already useful.
  Let!("\\protect", "\\relax");


  // DefMacro('\romannumeral Number', sub { roman($_[1]->valueOf); });
  // # Hmm... I wonder, should getString itself be dealing with escapechar?
  // sub escapechar {
  //   my $code = LookupRegister('\escapechar')->valueOf;
  //   return (($code >= 0) && ($code <= 255) ? chr($code) : ''); }


  // # 1) Knuth, The TeXBook, page 40, paragraph 1, Chapter 7: How TEX Reads What You Type.
  // # suggests all characters except spaces are returned in category code Other, i.e. Explode()
  DefMacro!("\\string Token", sub[gullet, args, state] {
    unpack!(args => token);
    let token : Token = token.into();
    let mut s = token.get_string().to_string();
    if s.starts_with('/') {
      s = escapechar(state) + &s; 
    }
    Ok(Explode!(s).into())
  });

  DefMacroI!(T_CS!("\\jobname"), None, Tokens!()); // Set to the filename by initialization

  DefMacroI!(T_CS!("\\fontname"), None, Tokens::new(Explode!("fontname not implemented")));

  DefMacro!("\\meaning Token", sub[gullet, args, state] {
    unpack!(args => token);
    let token : Token = token.into();
    let mut meaning = String::from("undefined");

    if let Some(definition) = state.lookup_meaning(&token) {
      //     if (my $definition = (Equals($tok, T_ALIGN) ? $tok : LookupMeaning($tok))) {
      match definition {
        Stored::Token(t) => {
          let cc = t.get_catcode();
          let text = if cc == Catcode::SPACE {
            " "
          }else {
            t.get_string()
          };
          let meaning = s!("{} {}", cc.meaning(), text);
          Ok(Explode!(meaning).into())
        },
        _ => Ok(Explode!("meaning").into())
        // TODO: Continue implementing ...
      // Stored::Expandable(meaning)
      // Stored::Conditional(meaning)
      // }
      //       if ($type =~ /primitive/i) {
      //         $definition = $definition->getCSorAlias;
      //         $type       = ref $definition;
      //         $type =~ s/^LaTeXML:://; }
      //       if ($type =~ /con(ditional|structor)/i) {
      //         $definition = $definition->getCSorAlias;
      //         $type       = ref $definition;
      //         $type =~ s/^LaTeXML:://; }

      //       elsif ($type =~ /register/i) {   
      //         my $value = $definition->valueOf;
      //         my $register_type = lc(ref $value);
      //         my $prefix = '\count';
      //         if ($register_type && $register_type =~ /glue/) {
      //             $prefix = '\skip'; }
      //         elsif ($register_type && $register_type =~ /dimension/) {
      //             $prefix = '\dimen'; }
      //         my $literal_value = $value->valueOf if $register_type;
      //         # Should we be more careful to distinguish between latex and tex counters?
      //         $meaning = $prefix . $literal_value; }
      //       elsif ($type =~ /expandable/i) {
      //         my $expansion = $definition->getExpansion;
      //         my $ltxps     = $definition->getParameters;
      //         my @params;
      //         my $argcount = 0;
      //         if (defined $ltxps) {
      //           @params   = $ltxps->getParameters;
      //           $argcount = $ltxps->getNumArgs;
      //         }
      //         my $sp;
      //         my @specparts = map { (($sp = $_->{spec}) =~ s/^(\w+):// ? $sp : $sp) } @params;
      //         my $arg = 1;
      //         foreach (@specparts) {
      //           last if ($arg > $argcount);
      //           $_ .= "#$arg";
      //           $arg++; }
      //         my $spec = join("", @specparts);
      //         $spec =~ s/\{\}//g;
      //         $spec =~ s/Token//g;
      //         my $prefixes = join('',
      //           ($definition->isProtected ? '\protected' : ()),
      //           ($definition->isLong      ? '\long'      : ()),
      //           ($definition->isOuter     ? '\outer'     : ()),
      //         );
      //         $meaning = ($prefixes ? $prefixes . ' ' : '') . "macro:" . ToString($spec) . "->" . ToString($expansion); }
      //       Explode($meaning); }
      }
    } else {
      Ok(Explode!("undefined").into())
    }
  });


  //======================================================================


  DefParameterType!("CSName", reader => reader!(gullet, inner, extra, state, {
    let mut cs = escapechar(state);
    // keep newlines from having \n inside!
    while let Some(token) = gullet.read_x_token(true, true, state)? {
      let s = token.get_string();
      if s == "\\endcsname" {
        break;
      }
      let cc = token.get_catcode();
      if cc == Catcode::CS {
        if let Some(defn) = state.lookup_definition(&token) {
          error!(target: &s!("unexpected:{}", token), "The control sequence {:?} should not appear between \\csname and \\endcsname", token);
        } else {
          error!(target: &s!("undefined:{}", token), "The token {:?} is not defined", token);
        } 
      } else if cc == Catcode::SPACE {  // Keep newlines from having \n!
        cs.push(' ');
      } else { 
        cs.push_str(s);
      }
    }
    T_CS!(cs).into()
  }));

  DefMacro!("\\csname CSName", sub[gullet, args, state] {
    unpack!(args => token);
    let token : Token = token.into();
    if state.lookup_meaning(&token).is_none() { 
      Let!(token, "\\relax", state);
    }
    token.into()
  });

  DefPrimitive!("\\endcsname", sub[stomach, whatsit, state] {
    error!(target: "unexpected:\\endcsname", "Extra \\endcsname");
    Ok(Vec::new())
  });

  DefMacro!("\\expandafter Token Token", sub[gullet, args, state] {
    unpack!(args => tok, xtok);
    let mut tokens : Vec<Token> = vec![tok.into()];
    let xtok_single = xtok.clone().into();
    if let Some(defn) = state.lookup_expandable(&xtok_single, false) {
      // Note that IF expandafter ends up expanding a \the in an \edef,
      // that it Overrides the implicit noexpand that \edef would normally use for\the!!
      state.remove_value("NOEXPAND_THE");
      tokens.append(&mut defn.invoke(gullet, state)?.unlist()); // Expand $xtok ONCE ONLY!
    } else {
      tokens.append(&mut xtok.unlist()); 
    };
    Ok(tokens.into())
  });

  // Insert magic token that Gullet knows not to expand the next one.
  DefMacroI!(T_CS!("\\noexpand"), None, T_NOTEXPANDED!());

  DefMacroI!(T_CS!("\\topmark"), None,        Tokens!());
  DefMacroI!(T_CS!("\\firstmark"), None,      Tokens!());
  DefMacroI!(T_CS!("\\botmark"), None,        Tokens!());
  DefMacroI!(T_CS!("\\splitfirstmark"), None, Tokens!());
  DefMacroI!(T_CS!("\\splitbotmark"), None,   Tokens!());

  // DefMacro('\input TeXFileName', sub { Input($_[1]); });

  // # Note that TeX doesn't actually close the mouth;
  // # it just flushes it so that it will close the next time it's read!
  // DefMacroI('\endinput', undef, sub {
  //     my ($gullet) = @_;
  //     my $mouth = $gullet->getMouth;
  //     my $line;
  //     if (!$mouth->isEOL) {
  //       $line = $gullet->readRawLine; }
  //     $gullet->flushMouth;
  //     if ($line) {
  //       $gullet->unread(Tokenize($line)); }
  //     return;
  // });

  // \the<internal quantity>
  DefMacro!("\\the Register", sub[gullet, args, state] {
    unpack!(args => variable);
    let mut args = variable.unlist();
    let defn = args.remove(0).to_register(state);
    match defn {
      None => {
        error!(target:"expected:<register>", "a register was expected to be here");
        Ok(Tokens!())
      },
      Some(defn) => {
        let register_type = defn.borrow().register_type;
        //     if (!$type) {
        //       my $cs = ToString($defn->getCS);
        //       Error('unexpected', "\\the$cs", $gullet, "You can't use $cs after \\the"); return (); }
        let value = defn.value_of(args, state).unwrap_or_else(|| RegisterValue::Tokens(Tokens!()));
        // In all cases, these should be OTHER, except for space. (!?)
        let mut tokens : Vec<Token> = match value {
          RegisterValue::Tokens(ts) => ts.unlist(),
          RegisterValue::Token(t) => vec![t],
          rv => Explode!(rv.to_string()),
        };
        if state.noexpand_the { // See \the for the sense in this.
          tokens = gullet.neutralize_tokens(&tokens, state);
        }
        Ok(Tokens::new(tokens))
      }
    }
  });


  // The following special cases are built-in to Definition
  DefConditional!("\\else",          None);
  DefConditional!("\\or",            None);
  DefConditional!("\\fi",            None);
  DefConditional!("\\ifcase Number", None);

  Ok(())
}
