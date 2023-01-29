use crate::package::*;
lazy_static! {
  static ref LEAD_W_COLON_RE: Regex = Regex::new(r"^(\w+):").unwrap();
}
//=======================
// -- Main Definitions --
//=======================
LoadDefinitions!(outer_state, {
  // The following special cases are built-in to Definition
  DefConditional!("\\else");
  DefConditional!("\\or");
  DefConditional!("\\fi");
  DefConditional!("\\ifcase Number");

  DefConditional!("\\ifnum Number Token Number", sub[gullet, (u,rel,v), state] {
    compare(u, rel, v)
  });
  DefConditional!("\\ifdim Dimension Token Dimension", sub[gullet, (u,rel,v), state] {
    compare(Number::new(u.value_of()), rel, Number::new(v.value_of()))
  });
  DefConditional!("\\ifodd Number", sub[gullet, (u), state] {
    let uint = u.value_of();
    uint % 2 == 1
  });

  // NOTE: We don't KNOW if we're in vertical, horizontal or inner mode!!!!!!!
  DefConditional!("\\ifvmode", { false });
  DefConditional!("\\ifhmode", { false });
  DefConditional!("\\ifinner", { false });
  DefConditional!("\\ifmmode", { LookupBool!("IN_MATH") });

  DefConditional!("\\if XToken XToken", sub[gullet, (token1,token2), state] {
    token1.get_charcode() == token2.get_charcode()
  });

  DefConditional!("\\ifcat XToken XToken", sub[gullet, (token1,token2), state] {
    token1.get_catcode() == token2.get_catcode()
  });

  DefConditional!("\\ifx Token Token", sub[gullet, (token1,token2), state] {
    XEquals!(&token1, &token2)
  });

  DefConditional!("\\ifvoid Number", sub[_g, (arg), state] { classify_box(arg, state).is_empty() });
  DefConditional!("\\ifhbox Number", sub[_g, (arg), state] { classify_box(arg, state) == "hbox" });
  DefConditional!("\\ifvbox Number", sub[_g, (arg), state] { classify_box(arg, state) == "vbox" });

  DefConditional!("\\iftrue", { true });
  DefConditional!("\\iffalse", { false });

  //======================================================================
  // This makes \relax disappear completely after digestion
  // (which seems most TeX like).
  DefPrimitive!("\\relax", None);
  //// However, this keeps a box, so it can appear in UnTeX
  ////// DefPrimitive('\relax',undef);
  //// But if you do that, you've got to watch out since it usually
  //// shouldn't be a box; See the isRelax code in handleScripts, below

  DefMacro!("\\number Number", sub[gullet, (num), state] {
    let num_str = num.value_of();
    Explode!(num_str)
  });

  // define it here (only approxmiately), since it's already useful.
  Let!("\\protect", "\\relax");

  DefMacro!("\\romannumeral Number", sub[gullet, (num), state] { roman!(num.value_of()) });

  // # 1) Knuth, The TeXBook, page 40, paragraph 1, Chapter 7: How TEX Reads What You Type.
  // # suggests all characters except spaces are returned in category code Other, i.e. Explode()
  DefMacro!("\\string Token", sub[gullet, (token), state] {
    let mut s = token.to_string();
    if s.starts_with('/') {
      s = escapechar(state) + &s;
    }
    Explode!(s)
  });

  DefMacro!(T_CS!("\\jobname"), None, Tokens!()); // Set to the filename by initialization

  DefMacro!(T_CS!("\\fontname"), None, Tokens::new(Explode!("fontname not implemented")));

  DefMacro!("\\meaning Token", sub[gullet, (token), state] {
    let mut meaning = String::from("undefined");
    let definition_opt = if token == T_ALIGN!() {
      Some(Stored::Token(token))
    } else {
      state.lookup_meaning(&token)
    };
    if let Some(definition) = definition_opt {
       // First, if this definition is a primitive or constructor, check to see if it has an alias, which would allow us to work with a token
       let definition : Stored = match definition {
         Stored::Primitive(primitive) => Stored::Token(primitive.get_cs_or_alias().into_owned()),
         Stored::Constructor(constructor) => Stored::Token(constructor.get_cs_or_alias().into_owned()),
         other => other
       };
      // Now that we've tried to obtain an expandable definition, do the TeX dance:
      match definition {
        Stored::Token(t) => {
          let cc = t.get_catcode();
          let text = if cc == Catcode::SPACE {
            " "
          } else {
            t.get_string()
          };
          meaning = String::from(cc.meaning());
          if !meaning.is_empty() {
            meaning.push(' ');
          }
          meaning.push_str(text);
        },
        Stored::Register(register) => {
          let value = register.value_of(ArrayVec::default(),state);
          let register_type = register.register_type().unwrap();
          let prefix = match register_type {
            RegisterType::Glue | RegisterType::MuGlue =>  "\\skip",
            RegisterType::Dimension => "\\dimen",
            _ => "\\count"
          };
          let literal_value : String = if register_type != RegisterType::Any {
            if let Some(v) = value {
              v.value_of().to_string()
            } else {
              String::new()
            }
          } else {
            String::new()
          };
          // Should we be more careful to distinguish between latex and tex counters?
          meaning = s!("{}{}",prefix, literal_value);
        },
        Stored::Expandable(expandable) => {
          let (params,argcount) = if let Some(ltxps) = expandable.get_parameters() {
            (ltxps.get_parameters(), ltxps.get_num_args())
          } else {
            (Vec::new(), 0)
          };
          let specparts : Vec<Cow<str>> = params.iter().map(|param| LEAD_W_COLON_RE.replace(&param.spec,"") ).collect();
          let mut spec = String::new();
          for (index, part) in specparts.iter().take(argcount).enumerate() {
            spec.push_str(part);
            spec.push('#');
            spec.push_str(&(index+1).to_string());
            spec = spec.replace("{}","");
            spec = spec.replace("Token","");
          }
          let mut prefixes = String::new();
          if expandable.is_protected {
            prefixes.push_str("\\protected");
          }
          if expandable.is_long {
            prefixes.push_str("\\long");
          }
          if expandable.is_outer {
            prefixes.push_str("\\outer");
          }
          if !prefixes.is_empty() {
            prefixes.push(' ');
          }
          let expansion = match expandable.get_expansion() {
            None => String::new(),
            Some(exp) => exp.to_string()
          };
          meaning = s!("{}macro:{}->{}",prefixes, spec, expansion);
        },
        e => { // are there other cases that could occur here? should we handle them?
          panic!("this may be a missing case in \\meaning's implementation: {e}");
        }
      }
    }
    Explode!(meaning)
  });

  //======================================================================

  DefParameterType!(CSName, reader => reader!(gullet, inner, extra, state, {
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
          let message = s!("The control sequence {:?} should not appear between \\csname and \\endcsname", token);
          Error!("unexpected", token, gullet, state, message);
        } else {
          let message = s!("The token {:?} is not defined", token);
          Error!("undefined", token, gullet, state, message);
        }
      } else if cc == Catcode::SPACE {  // Keep newlines from having \n!
        cs.push(' ');
      } else {
        cs.push_str(s);
      }
    }
    T_CS!(cs)
  }));

  DefMacro!("\\csname CSName", sub[gullet, (token), state] {
    if LookupMeaning!(&token).is_none() {
      state.assign_meaning(&token, state.lookup_meaning(&T_CS!("\\relax")).unwrap(), None);
    }
    token
  });

  DefPrimitive!("\\endcsname", sub[stomach, (), state] {
    Error!("unexpected" ,"\\endcsname", stomach, state, "Extra \\endcsname");
  });

  DefMacro!("\\expandafter Token Token", sub[gullet, (tok, xtok), state] {
    let mut tokens : Vec<Token> = vec![tok];
    if let Some(defn) = state.lookup_expandable(&xtok, false) {
      state.current_token=Some(Arc::new(xtok.clone()));
      let invoked = defn.invoke(gullet, true, state)?;
      if !invoked.is_empty() {
        tokens.append(&mut invoked.unlist()); // Expand $xtok ONCE ONLY!
      }
    } else if state.lookup_meaning(&xtok).is_none() {
      // Undefined token is an error, as expansion is expected.
      // BUT The unknown token is NOT consumed, (see TeX B book, item 367)
      // since probably in a real TeX run it would have been defined.
      state.generate_error_stub(gullet, &xtok)?;
      tokens.push(xtok);
    } else {
      tokens.push(xtok);
    };
    Ok(Tokens::new(tokens))
  });

  // Insert magic token that Gullet knows not to expand the next one.
  DefMacro!(T_CS!("\\noexpand"), None, sub[gullet, args, state] {
    if let Some(token) = gullet.read_token(state) {
      vec![token.with_dont_expand(state)?]
    } else {
      Vec::new()
    }
  });

  DefMacro!(T_CS!("\\topmark"), None, Tokens!());
  DefMacro!(T_CS!("\\firstmark"), None, Tokens!());
  DefMacro!(T_CS!("\\botmark"), None, Tokens!());
  DefMacro!(T_CS!("\\splitfirstmark"), None, Tokens!());
  DefMacro!(T_CS!("\\splitbotmark"), None, Tokens!());

  // using input() from DefMacro is actually an incredible ordeal.
  // I tried several variations of arranging the types, but Rust is quite strict
  // about avoiding multiple borrows that relate to "state"
  // when mutability is involved.
  // For now I have changed to DefPrimitive, so that there is a clear access to the
  // stomach, but we may require some special-case treatment in other pieces of code...
  DefMacro!("\\input", "\\ltx@input");
  DefPrimitive!("\\ltx@input TeXFileName", sub[stomach, (name), state] {
    input(&name.to_string(), InputOptions::default(), stomach, state)?;
  });

  // Note that TeX doesn't actually close the mouth;
  // it just flushes it so that it will close the next time it's read!
  DefMacro!(T_CS!("\\endinput"), None, sub[gullet, _args, state] {
    let mut mouth = gullet.get_mouth_mut().unwrap();
    let line_opt = if !mouth.is_eol(state) {
      gullet.read_raw_line(state)
    } else {
      None
    };
    gullet.flush_mouth(state);
    if let Some(line) = line_opt {
      gullet.unread(Tokenize!(&line));
    }
  });

  // \the<internal quantity>
  DefMacro!("\\the Register", sub[gullet, args, state] {
    if let ArgWrap::RegisterDefinition((rtoken, mut inner)) = args.remove(0) {
      // let register_type = defn.borrow().register_type;
      //     if (!$type) {
      //       my $cs = ToString($defn->getCS);
      //       Error('unexpected', "\\the$cs", $gullet, "You can't use $cs after \\the"); return (); }
      let defn = rtoken.to_register(state).expect("if a Register parameter provides a token, it must have a Register definition.");
      let tiny_inner :ArrayVec<[ArgWrap;9]> = inner.drain(..min(9,inner.len())).collect();
      let value = defn.value_of(tiny_inner, state)
        .unwrap_or_else(|| RegisterValue::Tokens(Tokens!()));
      // In all cases, these should be OTHER, except for space. (!?)
      let mut tokens : Vec<Token> = match value {
        RegisterValue::Tokens(ts) => ts.unlist(),
        RegisterValue::Token(t) => vec![t],
        rv => Explode!(rv.to_string()),
      };
      tokens
    } else {
      Error!("expected", "<register>", gullet, state, "a register was expected to be here");
      Vec::new()
    }
  });
});

// Hmm... I wonder, should getString itself be dealing with escapechar?
fn escapechar(state: &State) -> String {
  let code: i32 = match state.lookup_register("\\escapechar", ArrayVec::default()) {
    Some(RegisterValue::Number(v)) => v.value_of(),
    _ => -1,
  };
  if (0..=255).contains(&code) {
    let char_code = (code as u8) as char;
    char_code.to_string()
  } else {
    String::new()
  }
}

fn compare(u: Number, rel: Token, v: Number) -> Result<bool> {
  let u = u.value_of();
  let v = v.value_of();
  // NOTE: One would expect this to be best written as an advanced match statement
  // however, due to the shallow comparison of Cow<str> the Cow::Borrowed("<") and
  // Cow::Owned("<") variants will NOT be equal via a destructuring match.
  // However, since we've defined our own PartialEq trait over Token, an equality comparison
  // will produce the right behavior
  if rel == T_OTHER!("<") || rel == T_CS!("\\@@<") {
    Ok(u < v)
  } else if rel == T_OTHER!("=") {
    Ok(u == v)
  } else if rel == T_OTHER!(">") || rel == T_CS!("\\@@>") {
    Ok(u > v)
  } else {
    let message = s!("Expected a relational token for comparision. Got {:?} (cc {:?})", rel, rel.get_catcode());
    Error!("expected", "<relationaltoken>", None, None, message);
    Ok(false)
  }
}
