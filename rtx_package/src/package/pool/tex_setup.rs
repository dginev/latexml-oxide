use crate::package::*;
LoadDefinitions!(state, {
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
     let mut read_tokens: Vec<Token> = vec![T_BEGIN!()];
     if !inner.is_empty() {
      for inner_opt in inner.into_iter() {
        let mut reverted_inner = match inner_opt {
          ParameterExtra::ParametersOption(Some(inner_p)) => inner_p.revert_arguments(vec![Tokens::new(arg.clone())], gullet, state)?,
          _ => arg.iter().map(|t| t.revert()).collect()
        };
        read_tokens.append(&mut reverted_inner);
      }
     } else {
       let mut arg_reverted = arg.iter().map(|a| a.revert()).collect();
       read_tokens.append(&mut arg_reverted);
     }
     read_tokens.push(T_END!());
     Ok(Tokens::new(read_tokens))
    })
  );

  DefParameterType!("Optional",
    reader => reader!(gullet, inner, _extra, state, {
      let mut value = gullet.read_optional(state)?;
      // TODO: Default
      // if (!$value && $default) {
      //   $value = $default; }
      if !inner.is_empty() {
        for inner_opt in inner {
          if let Some(inner_p) = inner_opt {
            value = inner_p.reparse_argument(gullet, value, state);
          }
        }
      }
      Ok(value)
    }),
    optional => true,
    reversion => reversion!(gullet, arg, inner, state, {
      // TODO : default!
      if !arg.is_empty() {
        let mut read_tokens: Vec<Token> = vec![T_OTHER!(s!("["))];
        let mut reverted_arg = if inner.is_empty() {
            arg.iter().map(|t| t.revert()).collect()
        } else {
          let mut value = Vec::new();
          for inner_opt in inner.iter() {
            value = match inner_opt {
              ParameterExtra::ParametersOption(Some(inner)) => inner.revert_arguments(vec![Tokens::new(arg.clone())], gullet, state)?,
              _ => arg.iter().map(|t| t.revert()).collect()
            }
          }
          value
        };
        read_tokens.append(&mut reverted_arg);
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
      Ok(Tokens!(T_OTHER!("")))
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
      let until = until.into_iter().map(|x| if let ParameterExtra::Token(t) = x {
        t
      } else {
        T_OTHER!("")
      }).collect();
      gullet.read_until(until, state)
    })
    // reversion: |arg, until| { vec![Revert!(arg), Revert!(until)] },
  );

  DefParameterType!("Skip1Space",
    reader => reader!(gullet, inner, _extra, state, {
      gullet.skip_one_space(state);
      Ok(Tokens!())
    }),
    novalue => true
  );

  // Read a matching keyword, eg. Match:=
  DefParameterType!("Match",
    reader => reader!(gullet, _inner, extra, state, {
      let extra_tokens : Vec<Token> = extra.into_iter().filter(|e|
      if let ParameterExtra::Token(t) = e {
          true
        } else {
          false
        }
      ).map(|x| x.into()).collect();
      match gullet.read_match(&extra_tokens, state)? {
        Some(t) => Ok(Tokens!(t)),
        None => Ok(Tokens!())
      }
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
      if !inner.is_empty() {
        for inner_opt in inner.into_iter() {
          let mut reverted_inner = match inner_opt { // TODO: the revert_arguments arg type is confusing me!
            ParameterExtra::ParametersOption(Some(inner_p)) => inner_p.revert_arguments(vec![Tokens::new(arg.clone())], gullet, state)?,
            _ => arg.iter().map(|t| t.revert()).collect()
          };
          read_tokens.append(&mut reverted_inner);
        }
      } else {
        let mut reverted_arg = arg.iter().map(|t| t.revert()).collect();
        read_tokens.append(&mut reverted_arg);
      }
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
       let mut reverted_arg = arg.iter().map(|a| a.revert()).collect();
       read_tokens.append(&mut reverted_arg);
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
  reader_predigest => undigested!(),
  reversion => reversion!(gullet, arg, inner, state, {
    let mut read_tokens = vec!(T_BEGIN!());
    let mut reverted_arg = arg.iter().map(|a| a.revert()).collect();
    read_tokens.append(&mut reverted_arg);
    read_tokens.push(T_END!());
    Ok(Tokens::new(read_tokens))
  }));

  // Read a LaTeX-style optional argument (ie. in []), but it will not be digested.
  DefParameterType!("OptionalUndigested",
  reader => reader!(gullet, inner, _extra, state, { gullet.read_optional(state) }),
  reader_predigest => undigested!(),
  optional => true,
  reversion => reversion!(gullet, arg, inner, state, {
    if arg.is_empty() {
      Ok(Tokens!())
    } else {
      let mut read_tokens = vec!(T_OTHER!("["));
      let mut reverted_arg = arg.iter().map(|a| a.revert()).collect();
      read_tokens.append(&mut reverted_arg);
      read_tokens.push(T_OTHER!("]"));
      Ok(Tokens::new(read_tokens))
    }
  }));

  // Same, but not necessarily writable
  DefParameterType!("Register", reader => reader!(gullet, inner, _extra, state, {
      let token = gullet.read_x_token(false, false, state)?;
      let defn = match token {
        None => None,
        Some(ref t) => state.lookup_register_definition(t)
      };
      match defn {
        Some(register) => {
          let mut invoked = vec![token.clone().unwrap()];
          for arg in register.read_arguments(gullet, state)? {
            invoked.append(&mut arg.unlist());
          }
          return Ok(Tokens::new(invoked));
        },
        None => {
          error!(target:"expected:<register>", "A <register> was supposed to be here. Got {:?}", token);
          // if isDefinable!(token) {
          //   DefRegisterI!(token, None, Tokens!(), state);
          //   return Tokens!(defn);
          // }
        }
      }
      Ok(Tokens!())
    }),
    reversion => reversion!(gullet, arg, inner, state, {
      // my ($var) = @_;
      // my ($defn, @args) = @$var;
      // my $params = $defn->getParameters;
      // return Tokens($defn->getCS, ($params ? $params->revertArguments(@args) : ()));
      Ok(Tokens!())
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
        gullet.unread(&Tokens::new(toks));

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
    reader_predigest => undigested!()
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

  // Read a floating point number
  // DefParameterType!("Float",
  //   reader => reader!(gullet, inner, _extra, state, {
  //     gullet.read_float(state)?.to_token().into()
  //   })
  // );

  // Read a Glue (aka skip)
  DefParameterType!("Glue",
    reader => reader!(gullet, inner, _extra, state, {
      gullet.read_glue(state)?.to_token().into()
    })
  );

  // Read until the next (balanced) open brace {
  // used for the last TeX-style delimited argument
  DefParameterType!("UntilBrace",
    reader => reader!(gullet, inner, _extra, state, {
      gullet.read_until_brace(state)
    })
  );

  //**********************************************************************
  // LaTeX has a very particular notion of "Undefined",
  // so let's get that squared away at the outset; it's useful for TeX, too!
  // Naturally, it uses \csname to check, which ends up DEFINING the possibly undefined macro as \relax
  DefMacro!("\\@ifundefined{}{}{}", sub[gullet, args, inner_state] {
    unpack!(args=>name, if_token, else_token);
    let cs = T_CS!(&s!("\\{}", Expand!(name,gullet,inner_state).to_string()));
    if IsDefined!(&cs, inner_state) {
      Ok(else_token)
    } else {
      Let!(cs, "\\relax", inner_state); // Yuck, but traditional!
      Ok(if_token)
    }
  });

  // sub isDefinable {
  //   my ($token) = @_;
  //   return unless $token;
  //   my $meaning = LookupMeaning($token);
  //   my $name = $token->getString; $name =~ s/^\\//;
  //   return (((!defined $meaning) || ($meaning eq LookupMeaning(T_CS('\relax'))))
  //       && (($name ne 'relax') && ($name !~ /^end/))); }
});
