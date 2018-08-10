use package::*;
pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  RegisterNamespace!("ltx", "http://dlmf.nist.gov/LaTeXML");
  RegisterNamespace!("svg", "http://www.w3.org/2000/svg");
  RegisterNamespace!("xlink", "http://www.w3.org/1999/xlink"); // Needed for SVG
                                                               // Not directly used, but let's stake out the ground
  RegisterNamespace!("m", "http://www.w3.org/1998/Math/MathML");
  RegisterNamespace!("xhtml", "http://www.w3.org/1999/xhtml");

  DefMacro!("\\@empty", "");

  //======================================================================
  // Core ID functionality.
  //======================================================================
  // DOCUMENTID is the ID of the document
  // AND prefixes IDs on all other elements.
  if !state.documentid.is_empty() {
    let docid = state.documentid.clone();
    // Wrap in T_OTHER so funny chars don't screw up (no space!)
    DefMacroI!(T_CS!("\\thedocument@ID"), None, T_OTHER!(docid));
  } else {
    Let!("\\thedocument@ID", "\\@empty");
  }
  // TODO:
  // NewCounter!("@XMARG", "document", idprefix: "XM");

  // Optionally, add ID's to ALL nodes.
  // By default, this is OFF;
  // Set to 1 (or \usepackage[ids]{latexml}) to enable.
  // Set to 0 (or \usepackage[noids]{latexml}) to disable.

  Tag!("ltx:*", after_open => tagsub!(document, node, state, {
    // If GENERATE_IDS is true, we'll assign an ID to EVERY element,
    // EXCEPT ltx:document which only gets an id from an EXPLICIT \thedocument@id.
    let tag = document.get_node_qname(&node, state);
    if tag != "ltx:document"
      && tag != "ltx:XMWrap"    // No auto-generated id on wrap???
      && state.lookup_bool("GENERATE_IDS") {
        // TODO:
        // GenerateID!(document, node, state);
    }
  }));

  //======================================================================
  Tag!("ltx:document",
    after_open => sub!(|document, node, state| {
      document.process_pending_resources(state)
    }),
    after_close => sub!(|document, node, state| {
      document.process_pending_resources(state)
    })
  );

  RequireResource!("LaTeXML.css");

  //**********************************************************************
  // CORE TeX; Built-in commands.
  //**********************************************************************

  // ======================================================================
  // Define parsers for standard parameter types.
  DefParameterType!("Plain",
    reader => reader!(gullet, inner, _extra, state, {
      let mut value: Tokens = gullet.read_arg(state)?;
      for inner_opt in inner {
        if let Some(inner_p) = inner_opt {
          value = inner_p.reparse_argument(gullet, value, state);
        }
      }
      Ok(value)
    }),
    reversion => reversion!(gullet, arg, inner, state, {
     // let mut reverted_inner;
     println!("-- Plain reversion for arg: {:?}", arg);
     let mut read_tokens: Vec<Token> = vec![T_BEGIN!()];
     if !inner.is_empty() {
      for inner_opt in inner.into_iter() {
        let mut reverted_inner = match inner_opt {
          Some(inner_p) => inner_p.revert_arguments(vec![Tokens::new(arg.clone())], gullet, state)?,
          None => Vec::new()
        };
        read_tokens.append(&mut reverted_inner);
      }
     } else {
       read_tokens.append(&mut arg); // TODO: implement Revert(arg)
     }
     read_tokens.push(T_END!());
     Ok(Tokens::new(read_tokens))
    })
  );

  DefParameterType!("Optional",
    reader => reader!(gullet, inner, _extra, state, {
      // TODO: default !!!
      // let value = gullet.read_optional(state);
      // if (!$value && $default) {
      //   $value = $default; }
      // elsif ($inner) {
      //   ($value) = $inner->reparseArgument($gullet, $value); }
      // value
      gullet.read_optional(state)
    }),
    optional => true,
    reversion => reversion!(gullet, arg, inner, state, {
      // TODO : default!
      if !arg.is_empty() {
        let mut read_tokens: Vec<Token> = vec![T_OTHER!(s!("["))];
        // TODO: ($inner ? $inner->revertArguments($arg) : Revert($arg)),
        read_tokens.push(T_OTHER!(s!("]")));
        Ok(Tokens::new(read_tokens))
      } else {
        Ok(Tokens!())
      }
    })
  );

  // Skip any spaces, but don't contribute an argument.
  DefParameterType!("SkipSpaces",
    reader => reader!(gullet, inner, _extra, state, {
      gullet.skip_spaces(state);
      Ok(Tokens!())
    }),
    novalue => true
  );

  // // This is a peculiar type of argument of the form
  // //   <general text> = <filler>{<balanced text><right brace>
  // // however, <filler> does get expanded while searching for the initial {
  // // which IS required in contrast to a general argument; ie a single token is not correct.
  // DefParameterType!("GeneralText",Parameter{
  // reader: reader!(gullet, inner, _extra, state, {
  //   let open = gullet.read_x_token();
  //     if open.equals(T_BEGIN!()) {
  //       gullet.read_balanced()
  //     } else {
  //       // Error("expected", "{", $gullet,
  //       //   "Expected <general text> here");
  //       open
  //     }
  //   }),
  //   ..Parameter::default()
  // }, state);

  DefParameterType!("Until",
    reader => reader!(gullet, _inner, until, state, {
      gullet.read_until(until, state)
    })
    // reversion: |arg, until| { vec![Revert!(arg), Revert!(until)] },
  );

  // DefParameterType!("Skip1Space",Parameter{
  // reader: reader!(gullet, inner, _extra, state, {
  //   gullet.skip_one_space();
  //     vec![]
  //   }),
  //   novalue: true,
  //   ..Parameter::default()
  // }, state);

  // Read a matching keyword, eg. Match:=
  DefParameterType!("Match",
    reader => reader!(gullet, _inner, extra, state, {
      gullet.read_match(extra, state)
    })
  );

  // Read a keyword; eg. Keyword:to
  // (like Match, but ignores catcodes)
  // DefParameterType!("Keyword",
  //   Parameter {
  //     reader: reader!(gullet, inner, _extra, state, {
  //       gullet.read_keyword(state);
  //     }), ..Parameter::default()
  //   }, state);

  // Read balanced material (?)
  DefParameterType!("Balanced",
    reader => reader!(gullet, _inner, _extra, state, {
      gullet.read_balanced(state)
    })
  );

  // Read a Semiverbatim argument; ie w/ most catcodes neutralized.
  DefParameterType!("Semiverbatim",
    reader => reader!(gullet, inner, _extra, state, { gullet.read_arg(state) }),
    reversion => reversion!(gullet, arg, inner, state, {
      // let mut reverted_inner;
      let mut read_tokens: Vec<Token> = vec![T_BEGIN!()];
      // for inner_opt in inner.into_iter() {
      //   reverted_inner = match inner_opt {
      //     Some(inner_p) => inner_p.revert_arguments(arg, state),
      //     None => Revert(arg)
      //   };
      // }
      // TODO : push reverted_inner to the read_tokens
      read_tokens.push(T_END!());
      Ok(Tokens::new(read_tokens))
    }),
    semiverbatim => true);

  // Read a LaTeX-style optional argument (ie. in []), but the contents read as Semiverbatim.
  DefParameterType!("OptionalSemiverbatim",
    reader => reader!(gullet, inner, _extra, state, { gullet.read_optional(state)}),
    semiverbatim => true,
    optional => true,
    reversion => reversion!(gullet, arg, inner, state, {
     if !arg.is_empty() {
       let mut read_tokens = vec![T_OTHER!(s!("["))];
       // TODO: add these: Revert!(arg, state)
       read_tokens.push(T_OTHER!(s!("]")));
       Ok(Tokens::new(read_tokens))
     } else {
       Ok(Tokens!())
     }
    })
  );

  // Read an argument that will not be digested.
  DefParameterType!("Undigested",
  reader => reader!(gullet, inner, _extra, state, { gullet.read_arg(state)}),
  undigested => true,
  reversion => reversion!(gullet, arg, inner, state, {
    unimplemented!()
    // TODO: add to read_tokens Revert!(arg, state)
    // let read_tokens = Tokens!(T_BEGIN!(), T_END!());
    // Ok(read_tokens)
  }));

  // Read a LaTeX-style optional argument (ie. in []), but it will not be digested.
  DefParameterType!("OptionalUndigested", 
  reader => reader!(gullet, inner, _extra, state, { gullet.read_optional(state) }),
  undigested => true, optional => true,
   // TODO
   reversion => reversion!(gullet, arg, inner, state, {
     unimplemented!()
     // ($_[0] ? (T_OTHER('['), Revert($_[0]), T_OTHER(']')) : ()); });
   })
  );

  // Read a token as used when defining it, ie. it may be enclosed in braces.
  DefParameterType!("DefToken",
    reader => reader!(gullet, inner, _extra, state, {
      let mut token = gullet.read_token(state);
      let begin_token = Some(T_BEGIN!());
      let space_token = T_SPACE!();

      while token == begin_token {
        let mut toks : Vec<Token> = gullet.read_balanced(state)?.unlist().into_iter().filter(|t| *t != space_token).collect();
        let mut new_tokens = toks.split_off(1);
        gullet.unread(Tokens::new(toks));

        token = if new_tokens.is_empty() {
          None
        } else {
          new_tokens.pop()
        };
      }
      match token {
        Some(t) => Ok(Tokens!(t)),
        None => Ok(Tokens!())
      }
    }),
    undigested => true
  );

  // Read the next token
  DefParameterType!("Token",
    reader => reader!(gullet, inner, _extra, state, {
      if let Some(t) = gullet.read_token(state) {
        Ok(Tokens!(t))
      } else {
        Ok(Tokens!())
      }
    })
  );

  // Read the next token, after expanding any expandable ones.
  DefParameterType!("XToken",
    reader => reader!(gullet, inner, _extra, state, {
      if let Some(t) = gullet.read_x_token(false, false, state)? {
        Ok(Tokens!(t))
      } else {
        Ok(Tokens!())
      }
    })
  );

  // Read a number
  DefParameterType!("Number",
    reader => reader!(gullet, inner, _extra, state, {
      gullet.read_number(state)?.to_token().into()
    })
  );

  // // Read a floating point number
  // DefParameterType!("Float",Parameter{
  // reader: reader!(gullet, inner, _extra, state, {
  // state: &mut State| {     gullet.read_float()
  //   }),
  //   ..Parameter::default()
  // }, state);

  // Read until the next (balanced) open brace {
  // used for the last TeX-style delimited argument
  DefParameterType!("UntilBrace",
    reader => reader!(gullet, inner, _extra, state, {
      gullet.read_until_brace(state)
    })
  );

  Ok(())
}
