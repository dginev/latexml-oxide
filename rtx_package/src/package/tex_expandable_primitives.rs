use crate::package::*;
static LEAD_W_COLON_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\w+):").unwrap());
static UNTIL_SPEC: Lazy<Regex> = Lazy::new(|| Regex::new("^\\w?Until(\\w*):").unwrap());
static EXCEPTION_MACRO_NAMES_FOR_MEANING: Lazy<Regex> =
  Lazy::new(|| Regex::new("^\\\\(?:(?:un)?expanded|detokenize)$").unwrap());
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
    compare(u.value_of(), rel, v.value_of())
  });
  DefConditional!("\\ifdim Dimension Token Dimension", sub[gullet, (u,rel,v), state] {
    compare(u.value_of(), rel, v.value_of())
  });
  DefConditional!("\\ifodd Number", sub[gullet, (u), state] {
    u.value_of() % 2 == 1
  });

  // NOTE: We don't KNOW if we're in vertical, horizontal or inner mode!!!!!!!
  DefConditional!("\\ifvmode", { false });
  DefConditional!("\\ifhmode", { false });
  DefConditional!("\\ifinner", { false });
  DefConditional!("\\ifmmode", { LookupBool!("IN_MATH") });

  DefParameterType!(ExpandedIfToken, sub[gullet, _inner, _extra, state] {
    let token_opt = gullet.read_x_token(Some(false), false, state)?.map(|t| {
      // Also resolve \let variants:
      let meaning_opt = state.lookup_meaning(&t);
      if let Some(Stored::Token(ref meaning)) = meaning_opt.as_deref() {
        meaning.clone()
      } else {
        t
      }});
    let token = token_opt.unwrap_or_else(|| {
      Error!("expected", "ExpandedIfToken", gullet, state,
        "conditional expected a token argument, came back empty. Falling back to \\@empty");
      T_CS!("\\@empty") });
    if token.has_smuggled() {    // marked dont_expand
      let smuggled = token.get_dont_expand().as_ref().unwrap();
      if smuggled.get_catcode() == Catcode::ACTIVE {
        // treat as active character, if originally such
        token.without_dont_expand()
      } else { // otherwise, treat as relax for comparisons
        T_RELAX!()
      }
    } else {   // normal case, treat token as-is
      token
    }
  });

  DefConditional!("\\if ExpandedIfToken ExpandedIfToken", sub[gullet, (left,right), state] {
    left.get_charcode() == right.get_charcode()
  });

  DefConditional!("\\ifcat ExpandedIfToken ExpandedIfToken", sub[gullet, (left,right), state] {
    left.get_catcode() == right.get_catcode()
  });

  DefConditional!("\\ifx Token Token", sub[gullet, (left,right), state] {
    state.x_equals(&left, &right)
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

  DefMacro!("\\number Number", sub[gullet, (num), state] { Explode!(num.value_of()) });

  // define it here (only approxmiately), since it's already useful.
  Let!("\\protect", "\\relax");

  DefMacro!("\\romannumeral Number", sub[gullet, (num), state] { roman!(num.value_of()) });

  // 1) Knuth, The TeXBook, page 40, paragraph 1, Chapter 7: How TEX Reads What You Type.
  // suggests all characters except spaces are returned in category code Other, i.e. Explode()
  DefMacro!("\\string Token", sub[gullet, (token), state] {
    let mut s = token.to_string();
    if s.starts_with('\\') {
      s = escapechar(state) + &s[1..];
    }
    Explode!(s)
  });

  DefMacro!(T_CS!("\\jobname"), None, Tokens!()); // Set to the filename by initialization

  DefMacro!(
    T_CS!("\\fontname"),
    None,
    Tokens::new(Explode!("fontname not implemented"))
  );

  DefMacro!("\\meaning Token", sub[gullet, (token), state] {
    let mut meaning = String::from("undefined");
    if let Some(definition) = if token == T_ALIGN!() {
      Some(Stored::Token(token))
    } else {
      state.lookup_meaning(&token).map(|d| d.into_owned())
    } {
      // First, if this definition is a primitive|conditional|constructor,
      // check to see if it has an alias, which would allow us to work with a token
      let definition : Stored = match definition {
        Stored::Primitive(primitive) =>
          Stored::Token(primitive.get_cs_or_alias().into_owned()),
        Stored::Constructor(constructor) =>
          Stored::Token(constructor.get_cs_or_alias().into_owned()),
        Stored::Conditional(cond) =>
          Stored::Token(cond.get_cs_or_alias().into_owned()),
        other => other
      };
      // TODO: Also check for fontinfo_ when implemented

      // Now that we've tried to obtain an expandable definition, do the TeX dance:
      match definition {
        Stored::Token(t) => {
          let cc = t.get_catcode();
          let text = if cc == Catcode::SPACE {
            String::from(" ")
          } else {
            t.to_string()
          };
          meaning = String::from(cc.meaning());
          if !meaning.is_empty() {
            meaning.push(' ');
          }
          meaning.push_str(&text);
        },
        Stored::Register(register) => {
          let value = register.value_of(vec![],state);
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
          meaning = format!("{prefix}{literal_value}");
        },
        Stored::Expandable(expandable) => {
          // short-circuit some troublesome discrepancies with TeX, which end up macros on our end,
          // but \meaning expects as primitives in the CTAN ecosystem.
          let cs = expandable.get_cs_or_alias().to_string();
          // These exceptions could be extended further, as we add more .sty/.cls support
          if EXCEPTION_MACRO_NAMES_FOR_MEANING.is_match(&cs) {
            return Ok(Tokens::new(Explode!(cs)));
          }
          let params = match expandable.get_parameters() {
            Some(ps) => ps.get_parameters(),
            None => Vec::new()
          };
          let mut spec_parts : Vec<Cow<str>> = Vec::new();
          let mut p_trailer = "";
          // params.iter().map(|param| LEAD_W_COLON_RE.replace(&param.spec,"") ).collect();
          let mut arg_index = 0;
          for param in params.iter() {
            let mut p_spec = Cow::Borrowed("");
            match &*param.spec {
              "RequireBrace" => {
                // tex's \meaning prints out the required braces for "\def\a#{}" variants
                p_trailer = "{";
                p_spec    = Cow::Borrowed("{");
              },
              "UntilBrace" => {
                p_trailer = "{";
                arg_index+=1;
                p_spec = Cow::Owned(format!("#{arg_index}{p_spec}"));
              }
              other if other.starts_with("Match:") => {
                // just match, don't increment arg index
                p_spec = LEAD_W_COLON_RE.replace(other,"");
              },
              other if UNTIL_SPEC.is_match(other) => {
                // implied argument at this slot
                p_spec = LEAD_W_COLON_RE.replace(other,"");
                arg_index +=1 ;
                p_spec = Cow::Owned(s!("#{arg_index}{p_spec}"));
              },
              other => { // regular parameter, increment
              // skip the latexml-only requirement params, but only here,
              // since Match also have "novalue" set.
                if param.novalue {
                  continue;
                }
                arg_index+=1;
                p_spec = Cow::Owned(s!("#{arg_index}"));
              }
            }
            spec_parts.push(p_spec);
          }
          let mut spec : String = spec_parts.join("");
          spec = spec.replace("{}","");
          spec = spec.replace("Token","");

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
            // TODO: How to print closures? This follows Perl's raw pointer format
            Some(ExpansionBody::Closure(exp)) => format!("CODE({:p})", Rc::as_ptr(exp)),
            Some(ExpansionBody::Tokens(tks)) => writable_tokens(tks, state)
          };
          meaning = format!("{prefixes}macro:{spec}->{expansion}{p_trailer}");
        },
        e => { // are there other cases that could occur here? should we handle them?
          panic!("this may be a missing case in \\meaning's implementation: {e}");
        }
      }
    }
    ExplodeChars!(meaning)
  });

  //======================================================================

  DefParameterType!(CSName, reader => reader!(gullet, _inner, _extra, state, {
    let mut cs = escapechar(state);
    let endcsname_token = T_CS!("\\endcsname");
    // keep newlines from having \n inside!
    while let Some(token) = gullet.read_x_token(Some(true), true, state)? {
      if token == endcsname_token {
        break;
      }
      match token.get_catcode() {
        Catcode::CS => {
          if let Some(defn) = state.lookup_definition(&token) {
            let message =
              s!("The control sequence {:?} should not appear between \\csname and \\endcsname",
                token);
            Error!("unexpected", token, gullet, state, message);
          } else {
            let message = s!("The token {:?} is not defined", token);
            Error!("undefined", token, gullet, state, message);
          }
        },
        Catcode::SPACE => {  // Keep newlines from having \n!
          cs.push(' ');
        },
        _ => {
          token.with_str(|s| cs.push_str(s));
        }
      };
    }
    T_CS!(cs)
  }));

  DefMacro!("\\csname CSName", sub[gullet, (token), state] {
    if state.lookup_meaning(&token).is_none() {
      state.assign_meaning(&token, TOKEN_RELAX.with(|t_relax|
        state.lookup_meaning(t_relax)).unwrap().into_owned(), None);
    }
    token
  });

  DefPrimitive!("\\endcsname", sub[stomach, (), state] {
    Error!("unexpected" ,"\\endcsname", stomach, state, "Extra \\endcsname");
  });

  DefMacro!("\\expandafter Token Token", sub[gullet, (tok, xtok), state] {
    let mut tokens : Vec<Token> = vec![tok];
    if let Some(defn) = state.lookup_expandable(&xtok, false) {
      state.local_current_token(xtok);
      let invoked = defn.invoke(gullet, true, state)?;
      if !invoked.is_empty() {
        tokens.append(&mut invoked.unlist()); // Expand $xtok ONCE ONLY!
      }
      state.expire_current_token();
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

  // Replace the next token with it's not-expanded variant
  DefMacro!(T_CS!("\\noexpand"), None, sub[gullet, _args, state] {
    if let Some(token) = gullet.read_token(state)? {
      vec![token.with_dont_expand(state)?]
    } else {
      // Missing token likely the result of "{\noexpand}" for which TeX would be unperturbed
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
    let mut tks = name.unlist();
    // If given a LaTeX-style argument, strip braces
    if tks.len() > 1 && tks.first().unwrap().get_catcode() == Catcode::BEGIN
      && tks.last().unwrap().get_catcode() == Catcode::END {
      tks.remove(0);
      tks.pop();
      // and load LaTeX.pool if not already
      if !state.lookup_bool("LaTeX.pool_loaded") {
        LoadPool!("LaTeX");
      }
    }
    let reloadable_opts = InputOptions { reloadable: true, ..InputOptions::default() };
    input(&Tokens::new(tks).to_string(), reloadable_opts, stomach, state)?;
  });

  // Note that TeX doesn't actually close the mouth;
  // it just flushes it so that it will close the next time it's read!
  DefMacro!(T_CS!("\\endinput"), None, sub[gullet, _args, state] {
    gullet.flush_mouth(state);
  });

  // \the<internal quantity>
  DefMacro!("\\the Register", sub[gullet, args, state] {
    if let ArgWrap::RegisterDefinition(dbox) = args.remove(0) {
      let (rtoken, inner) = *dbox;
      // let register_type = defn.borrow().register_type;
      //     if (!$type) {
      //       my $cs = ToString($defn->getCS);
      //       Error('unexpected', "\\the$cs", $gullet,
      //     "You can't use $cs after \\the"); return (); }
      let defn = rtoken.to_register(state)
        .expect("if a Register parameter provides a token, it must have a Register definition.");
      let value = defn.value_of(inner, state)
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
fn escapechar(state: &mut State) -> String {
  let code: i64 = match state.lookup_register("\\escapechar", Vec::new()) {
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

fn compare(u: i64, rel: Token, v: i64) -> bool {
  // NOTE: One would expect this to be best written as an advanced match statement
  // however, due to the shallow comparison of Cow<str> the Cow::Borrowed("<") and
  // Cow::Owned("<") variants will NOT be equal via a destructuring match.
  // However, since we've defined our own PartialEq trait over Token, an equality comparison
  // will produce the right behavior
  if rel == T_OTHER!("<") || rel == T_CS!("\\@@<") {
    u < v
  } else if rel == T_OTHER!("=") {
    u == v
  } else if rel == T_OTHER!(">") || rel == T_CS!("\\@@>") {
    u > v
  } else {
    let message = s!(
      "Expected a relational token for comparision. Got {:?} (cc {:?})",
      rel,
      rel.get_catcode()
    );
    Error!("expected", "<relationaltoken>", None, None, message);
    false
  }
}
