use crate::prelude::*;

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
  DefConditional!("\\ifmmode", { lookup_bool("IN_MATH") });

  DefParameterType!(ExpandedIfToken, sub[_inner, _extra] {
    let token_opt = gullet::read_x_token(Some(false), true)?;
    match token_opt {
      Some(t) => t,
      None => {
        Error!("expected", "ExpandedIfToken",
          "conditional expected a token argument, came back empty. Falling back to \\@empty");
        T_CS!("\\@empty")
      }}
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
  DefPrimitive!(T_CS!("\\relax"), None, {});
  // Internal token produced by Gullet in response to \dont_expand;
  // Acts like \relax, but isn't equal to it.
  DefPrimitive!(T_CS!("\\special_relax"), None, { });

  DefMacro!(T_CS!("\\jobname"), None, Tokens!()); // Set to the filename by initialization

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

  DefPrimitive!("\\endcsname", {
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

  // If next token is expandable, prefix it with the internal marker \dont_expand
  // That token is never defined, explicitly handled in Gullet & should never escape the Gullet
  DefMacro!(T_CS!("\\noexpand"), None, {
    if let Some(token) = gullet::read_token()? {
      if state::is_dont_expandable(&token) {
        vec![T_CS!("\\dont_expand"), token]
      } else {
        vec![token]
      }
    } else {
      // Missing token likely the result of "{\noexpand}" for which TeX would be unperturbed
      Vec::new()
    }
  });
  DefPrimitive!(T_CS!("\\dont_expand"), None, {
    Error!("misdefined", "\\dont_expand",
      "The token \\dont_expand should never reach Stomach!"); });

  DefMacro!(T_CS!("\\topmark"), None, Tokens!());
  DefMacro!(T_CS!("\\firstmark"), None, Tokens!());
  DefMacro!(T_CS!("\\botmark"), None, Tokens!());
  DefMacro!(T_CS!("\\splitfirstmark"), None, Tokens!());
  DefMacro!(T_CS!("\\splitbotmark"), None, Tokens!());


  // \the<internal quantity>
  DefMacro!("\\the Register", sub[args] {
    let [rdef] : [_; 1] = args.try_into().unwrap();
    if let ArgWrap::RegisterDefinition(dbox) = rdef {
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
      match value {
        RegisterValue::Tokens(ts) => ts.unlist(),
        RegisterValue::Token(t) => vec![t],
        rv => Explode!(rv.to_string()),
      }
    } else {
      Error!("expected", "<register>", "a register was expected to be here");
      Vec::new()
    }
  });
});

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
