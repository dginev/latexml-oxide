use crate::package::*;
static LEAD_W_COLON_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\w+):").unwrap());
static UNTIL_SPEC: Lazy<Regex> = Lazy::new(|| Regex::new("^\\w?Until(\\w*):").unwrap());
static EXCEPTION_MACRO_NAMES_FOR_MEANING: Lazy<Regex> =
  Lazy::new(|| Regex::new("^\\\\(?:(?:un)?expanded|detokenize)$").unwrap());
//=======================
// -- Main Definitions --
//=======================
LoadDefinitions!({
  // The following special cases are built-in to Definition
  DefConditional!("\\else");
  DefConditional!("\\or");
  DefConditional!("\\fi");
  DefConditional!("\\ifcase Number");

  DefConditional!("\\ifnum Number Token Number", sub[(u,rel,v)] {
    compare(u.value_of(), rel, v.value_of())
  });
  DefConditional!("\\ifdim Dimension Token Dimension", sub[(u,rel,v)] {
    compare(u.value_of(), rel, v.value_of())
  });
  DefConditional!("\\ifodd Number", sub[(u)] {
    u.value_of() % 2 == 1
  });

  // NOTE: We don't KNOW if we're in vertical, horizontal or inner mode!!!!!!!
  DefConditional!("\\ifvmode", { false });
  DefConditional!("\\ifhmode", { false });
  DefConditional!("\\ifinner", { false });
  DefConditional!("\\ifmmode", { LookupBool!("IN_MATH") });

  DefParameterType!(ExpandedIfToken, sub[_inner, _extra] {
    let token_opt = gullet::read_x_token(Some(false), false)?.map(|t| {
      // Also resolve \let variants:
      if let Some(Stored::Token(meaning)) = lookup_meaning(&t) {
        meaning
      } else {
        t
      }});
    let token = match token_opt {
      Some(t) => t,
      None => {
        Error!("expected", "ExpandedIfToken",
          "conditional expected a token argument, came back empty. Falling back to \\@empty");
        T_CS!("\\@empty")
      }};
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

  DefConditional!("\\if ExpandedIfToken ExpandedIfToken", sub[(left,right)] {
    left.get_charcode() == right.get_charcode()
  });

  DefConditional!("\\ifcat ExpandedIfToken ExpandedIfToken", sub[(left,right)] {
    left.get_catcode() == right.get_catcode()
  });

  DefConditional!("\\ifx Token Token", sub[(left,right)] {
    x_equals(&left, &right)
  });

  DefConditional!("\\ifvoid Number", sub[(arg)] { classify_box(arg)?.is_empty() });
  DefConditional!("\\ifhbox Number", sub[(arg)] { classify_box(arg)? == "hbox" });
  DefConditional!("\\ifvbox Number", sub[(arg)] { classify_box(arg)? == "vbox" });

  DefConditional!("\\iftrue", { true });
  DefConditional!("\\iffalse", { false });

  //======================================================================
  // This makes \relax disappear completely after digestion
  // (which seems most TeX like).
  DefPrimitive!("\\relax", None);

  DefMacro!("\\number Number", sub[(num)] { Explode!(num.value_of()) });

  // define it here (only approxmiately), since it's already useful.
  Let!("\\protect", "\\relax");

  DefMacro!("\\romannumeral Number", sub[(num)] { roman!(num.value_of()) });

  // 1) Knuth, The TeXBook, page 40, paragraph 1, Chapter 7: How TEX Reads What You Type.
  // suggests all characters except spaces are returned in category code Other, i.e. Explode()
  DefMacro!("\\string Token", sub[(token)] {
    let mut s = token.to_string();
    if s.starts_with('\\') {
      s = escapechar() + &s[1..];
    }
    Explode!(s)
  });

  DefMacro!(T_CS!("\\jobname"), None, Tokens!()); // Set to the filename by initialization

  DefMacro!(
    T_CS!("\\fontname"),
    None,
    Tokens::new(Explode!("fontname not implemented"))
  );

  // Not sure about this yet...
  // NOTE: Lots of back-and-forth mangle with definition vs cs; don't do that!
  DefMacro!("\\meaning Token", sub[(token)] {
    let mut meaning = String::from("undefined");
    if let Some(definition) = if token == T_ALIGN!() {
      Some(Stored::Token(token))
    } else {
      lookup_meaning(&token)
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
          meaning = register.get_address().to_string();
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
            Some(ExpansionBody::Tokens(tks)) => writable_tokens(tks)
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

  DefParameterType!(CSName, reader => reader!( _inner, _extra, {
    let mut cs = escapechar();
    let endcsname_token = T_CS!("\\endcsname");
    // keep newlines from having \n inside!
    while let Some(token) = gullet::read_x_token(Some(true), true)? {
      if token == endcsname_token {
        break;
      }
      match token.get_catcode() {
        Catcode::CS => {
          if lookup_definition(&token)?.is_some() {
            let message =
              s!("The control sequence {:?} should not appear between \\csname and \\endcsname",
                token);
            Error!("unexpected", token, message);
          } else {
            let message = s!("The token {:?} is not defined", token);
            Error!("undefined", token, message);
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

  DefMacro!("\\csname CSName", sub[(token)] {
    if lookup_meaning(&token).is_none() {
      let relax_meaning = lookup_meaning(&TOKEN_RELAX).unwrap();
      assign_meaning(&token,
        relax_meaning, None);
    }
    token
  });

  DefPrimitive!("\\endcsname", sub[()] {
    Error!("unexpected" ,"\\endcsname", "Extra \\endcsname");
  });

  DefMacro!("\\expandafter Token Token", sub[(tok, xtok)] {
    let mut tokens : Vec<Token> = vec![tok];
    if let Some(defn) = lookup_expandable(&xtok, false)? {
      state::local_current_token(xtok);
      let invoked = defn.invoke( true)?;
      if !invoked.is_empty() {
        tokens.append(&mut invoked.unlist()); // Expand $xtok ONCE ONLY!
      }
      state::expire_current_token();
    } else if lookup_meaning(&xtok).is_none() {
      // Undefined token is an error, as expansion is expected.
      // BUT The unknown token is NOT consumed, (see TeX B book, item 367)
      // since probably in a real TeX run it would have been defined.
      state::generate_error_stub(&xtok)?;
      tokens.push(xtok);
    } else {
      tokens.push(xtok);
    };
    Ok(Tokens::new(tokens))
  });

  // Replace the next token with it's not-expanded variant
  DefMacro!(T_CS!("\\noexpand"), None, {
    if let Some(token) = gullet::read_token()? {
      vec![token.with_dont_expand()?]
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
  // about avoiding multiple borrows that relate to "state::
  // when mutability is involved.
  // For now I have changed to DefPrimitive, so that there is a clear access to the
  // stomach, but we may require some special-case treatment in other pieces of code...
  DefMacro!("\\input", "\\ltx@input");
  DefPrimitive!("\\ltx@input TeXFileName", sub[(name)] {
    let mut tks = name.unlist();
    // If given a LaTeX-style argument, strip braces
    if tks.len() > 1 && tks.first().unwrap().get_catcode() == Catcode::BEGIN
      && tks.last().unwrap().get_catcode() == Catcode::END {
      tks.remove(0);
      tks.pop();
      // and load LaTeX.pool if not already
      if !lookup_bool("LaTeX.pool_loaded") {
        LoadPool!("LaTeX");
      }
    }
    let reloadable_opts = InputOptions { reloadable: true, ..InputOptions::default() };
    input(&Tokens::new(tks).to_string(), reloadable_opts)?;
  });

  // Note that TeX doesn't actually close the mouth;
  // it just flushes it so that it will close the next time it's read!
  DefMacro!(T_CS!("\\endinput"), None, {
    gullet_mut!().flush_mouth();
  });

  // \the<internal quantity>
  DefMacro!("\\the Register", sub[args] {
    if let ArgWrap::RegisterDefinition(dbox) = args.remove(0) {
      let (rtoken, inner) = *dbox;
      // let register_type = defn.borrow().register_type;
      //     if (!$type) {
      //       my $cs = ToString($defn->getCS);
      //       Error('unexpected', "\\the$cs", $gullet,
      //     "You can't use $cs after \\the"); return (); }
      let defn = rtoken.to_register()
        .expect("if a Register parameter provides a token, it must have a Register definition.");
      let value = defn.value_of(inner)
        .unwrap_or_else(|| RegisterValue::Tokens(Tokens!()));
      // In all cases, these should be OTHER, except for space. (!?)
      let mut tokens : Vec<Token> = match value {
        RegisterValue::Tokens(ts) => ts.unlist(),
        RegisterValue::Token(t) => vec![t],
        rv => Explode!(rv.to_string()),
      };
      tokens
    } else {
      Error!("expected", "<register>", "a register was expected to be here");
      Vec::new()
    }
  });
});

// Hmm... I wonder, should getString itself be dealing with escapechar?
fn escapechar() -> String {
  let code: i64 = match state::lookup_register("\\escapechar", Vec::new()).unwrap() {
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
  // NOTE: One would expect this to be best written as an advanced match state::ent
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
    let err = || {Error!("expected", "<relationaltoken>", message); Ok(())};
    err().ok();
    false
  }
}
