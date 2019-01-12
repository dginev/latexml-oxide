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
  NewCounter!("@XMARG", "document", idprefix => "XM");

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
        generate_id(document, node, "", state)?;
    }
  }));

  //======================================================================
  Tag!("ltx:document",
    after_open => sub!(|document, node, state| {
      document.process_pending_resources(state)
    })
  );
  RequireResource!("LaTeXML.css");

  //======================================================================
  // The default "initial context" for XML+RDFa specifies some default
  // terms and prefixes, but no default vocabulary.
  // Ought to have a default for @vocab, but settable?
  // can we detect use of simple "term"s in attributes so we know whether we need @vocab?
  // Ought to have a default set of prefixes from RDFa Core,
  // but allow prefixes to be added.
  // Probably ought to scan rdf attributes for all uses of prefixes,
  // and include them in @prefix
  // The following prefixes are listed in http://www.w3.org/2011/rdfa-context/rdfa-1.1
  let rdf_prefixes = map!(
    "cc"      => "http://creativecommons.org/ns#",
    "ctag"    => "http://commontag.org/ns#",
    "dc"      => "http://purl.org/dc/terms/",
    "dcterms" => "http://purl.org/dc/terms/",
    "ical"    => "http://www.w3.org/2002/12/cal/icaltzd#",
    "foaf"    => "http://xmlns.com/foaf/0.1/",
    "gr"      => "http://purl.org/goodrelations/v1#",
    "grddl"   => "http://www.w3.org/2003/g/data-view#",
    "ma"      => "http://www.w3.org/ns/ma-ont#",
    "og"      => "http://ogp.me/ns#",
    "owl"     => "http://www.w3.org/2002/07/owl#",
    "rdf"     => "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
    "rdfa"    => "http://www.w3.org/ns/rdfa#",
    "rdfs"    => "http://www.w3.org/2000/01/rdf-schema#",
    "rev"     => "http://purl.org/stuff/rev#",
    "rif"     => "http://www.w3.org/2007/rif#",
    "rr"      => "http://www.w3.org/ns/r2rml#",
    "schema"  => "http://schema.org/",
    "sioc"    => "http://rdfs.org/sioc/ns#",
    "skos"    => "http://www.w3.org/2004/02/skos/core#",
    "skosxl"  => "http://www.w3.org/2008/05/skos-xl#",
    "v"       => "http://rdf.data-vocabulary.org/#",
    "vcard"   => "http://www.w3.org/2006/vcard/ns#",
    "void"    => "http://rdfs.org/ns/void#",
    "xhv"     => "http://www.w3.org/1999/xhtml/vocab#",
    "xml"     => "http://www.w3.org/XML/1998/namespace",
    "xsd"     => "http://www.w3.org/2001/XMLSchema#",
    "wdr"     => "http://www.w3.org/2007/05/powder#",
    "wdrs"    => "http://www.w3.org/2007/05/powder-s#"
  );

  for (k, v) in rdf_prefixes.iter() {
    AssignMapping!("RDFa_prefixes", k => *v);
  }

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

  // This is a peculiar type of argument of the form
  //   <general text> = <filler>{<balanced text><right brace>
  // however, <filler> does get expanded while searching for the initial {
  // which IS required in contrast to a general argument; ie a single token is not correct.
  DefParameterType!(
    "GeneralText",
    reader => reader!(gullet, inner, _extra, state, {
      if let Some(open) = gullet.read_x_token(false ,false, state)? {
        if open == T_BEGIN!() {
          gullet.read_balanced(state)
        } else {
          error!(target:"expected:{", "Expected <general text> here");
          Ok(Tokens!(open))
        }
      } else {
        error!(target:"expected:{", "Expected <general text> here");
        Ok(Tokens!())
      }
    })
  );

  DefParameterType!("Until",
    reader => reader!(gullet, _inner, until, state, {
      let until = until.into_iter().map(|x| if let ParameterExtra::Token(t) = x {
        t
      } else {
        T_OTHER!("")
      }).collect();
      gullet.read_until(until, state)
    }),
    reversion => reversion!(gullet, arg, until, state, {
      let mut rev = Vec::new();
      for t in arg.iter() {
        rev.push(t.revert());
      }
      // TODO: is until operational?
      Ok(Tokens::new(rev))
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

  DefParameterType!("Skip1Space",
    reader => reader!(gullet, inner, _extra, state, {
      gullet.skip_one_space(state);
      Ok(Tokens!())
    }),
    novalue => true
  );

  // Read the next token
  DefParameterType!("Token",
    reader => reader!(gullet, _inner, _extra, state, {
       match gullet.read_token(state) {
         Some(t) => Ok(Tokens!(t)),
         None => {
           error!(target:"expected:Token", "Paramater <Token> found None.");
           Ok(Tokens!())
         }
       }
    })
  );

  // Read the next token, after expanding any expandable ones.
  DefParameterType!("XToken",
    reader => reader!(gullet, inner, _extra, state, {
      if let Some(t) = gullet.read_x_token(false, false, state)? {
        Ok(Tokens!(t))
      } else {
        error!(target:"expected:XToken", "Paramater <XToken> found None.");
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
  DefParameterType!("Float",
    reader => reader!(gullet, inner, _extra, state, {
      gullet.read_float(state)?.to_token().into()
    })
  );

  // ???
  // sub ReadFloat {
  //   my ($gullet) = @_;
  //   $gullet->skipSpaces;
  //   return ($gullet->readFloat || Float(0)); }

  // Read a dimension
  DefParameterType!("Dimension",
    reader => reader!(gullet, inner, _extra, state, {
      gullet.read_dimension(state)?.to_token().into()
    })
  );

  // Read a Glue (aka skip)
  DefParameterType!("Glue",
    reader => reader!(gullet, inner, _extra, state, {
      gullet.read_glue(state)?.to_token().into()
    })
  );

  // Read a MuDimension (math)
  DefParameterType!("MuDimension",
    reader => reader!(gullet, inner, _extra, state, {
      gullet.read_mudimension(state)?.to_token().into()
    })
  );

  // Read a MuGlue (math)
  DefParameterType!("MuGlue",
    reader => reader!(gullet, inner, _extra, state, {
      gullet.read_muglue(state)?.to_token().into()
    })
  );

  // Read until the next (balanced) open brace {
  // used for the last TeX-style delimited argument
  // DefParameterType('UntilBrace', sub {
  //     my ($gullet) = @_;
  //     $gullet->readUntilBrace; });

  // Yet another special case: Require a { but do not read it!!!
  DefParameterType!("RequireBrace",
    reader => reader!(gullet, inner, _extra, state, {
      if !gullet.if_next(T_BEGIN!(), state)? {
        error!(target:"expected:{", "Expected a {{ here");
      }
      T_BEGIN!().into()
    }),
    novalue => true
  );

  DefParameterType!("XUntil",
    reader => reader!(gullet, inner, untils, state, {
      let until : Token = match untils[0] {
        ParameterExtra::Token(ref t) => t.clone(),
        _ => T_OTHER!("")
      }; // Make sure it's a single token!!!
      let mut tokens : Vec<Token> = Vec::new();
      while let Some(token) = gullet.read_x_token(false, false, state)? {
        if token == until {
          break;
        } else if token.get_catcode() == Catcode::BEGIN {
          tokens.push(token);
          tokens.extend(gullet.read_balanced(state)?.unlist());
          tokens.push(T_END!());
        } else if let Some(defn) = state.lookup_definition_stored(&token) {
          let args = defn.read_arguments(gullet, state)?;
          tokens.extend(Invocation!(token, args, gullet, state)?.unlist());
        } else {
          tokens.push(token);
        }
      }
      Ok(Tokens::new(tokens))
    })
  );

  // This is sorta like readbalanced, but expands as it goes.
  // This appears to be needed by certain primitives (eg. \noalign ?)
  // and maybe what we should be using for some Digested ??
  DefParameterType!("Expanded",
    reader => reader!(gullet, inner, untils, state, {
      if let Some(token) = gullet.read_x_token(false, false, state)? {
        let mut tokens   = Vec::new();
        if token.get_catcode() == Catcode::BEGIN {
          let mut level = 1;
          while let Some(token) = gullet.read_x_token(false, false, state)? {
            match token.get_catcode() {
            Catcode::END => {
              level-=1;
              if level <=0 {
                break;
              }
            },
            Catcode::BEGIN => level +=1,
            _ => {}
            };
            tokens.push(token);
          }
          Ok(Tokens::new(tokens))
        } else {
          Ok(Tokens!(token))
        }
      } else {
        error!(target:"expected:Expanded", "was expecting an Expanded parameter value, found nothing.");
        Ok(Tokens!())
      }
    }),
    reversion => reversion!(gullet, arg, inner, state, {
      let arg_rev : Vec<Token> = arg.iter().map(|t| t.revert()).collect();
      let mut tks = Vec::new();
      tks.push(T_BEGIN!());
      tks.extend(arg_rev);
      tks.push(T_END!());
      Ok(Tokens::new(tks))
    })
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
  DefParameterType!("Keyword",
    reader => reader!(gullet, _inner, extra, state, {
      let extra_tokens : Vec<Token> = extra.into_iter().filter(|e|
      if let ParameterExtra::Token(t) = e {
          true
        } else {
          false
        }
      ).map(|x| x.into()).collect();
      let extra_strings: Vec<&str> = extra_tokens.iter().map(|x| x.get_string()).collect();
      match gullet.read_keyword(extra_strings.as_slice(), state)? {
        Some(t) => Ok(t),
        None => Ok(Tokens!())
      }
    })
  );

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

  // Be careful here: if % appears before the initial {, it's still a comment!
  // Also, note that non-typewriter fonts will mess up some chars on digestion!
  DefParameterType!("Verbatim",
    reader => reader!(gullet, inner, _extra, state, {
      gullet.read_until(vec![T_BEGIN!()], state)?;
      state.begin_semiverbatim(Some(vec!['%', '\\']));
      let arg = gullet.read_balanced(state)?;
      state.end_semiverbatim()?;
      Ok(arg)
    }),
    before_digest => beforeproc!(stomach, state, {
      stomach.bgroup(state);
      MergeFont!(family => "typewriter", state);
    }),
    after_digest => afterproc!(stomach, args, state, {
      stomach.egroup(state)?;
    }),
    reversion => reversion!(gullet, arg, inner, state, {
      let mut reverted = vec![T_BEGIN!()];
      let reverted_arg : Vec<Token> = arg.iter().map(|a| a.revert()).collect();
      reverted.extend(reverted_arg);
      reverted.push(T_END!());
      Ok(Tokens::new(reverted))
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

  // Read a keyword value (KeyVals), that will not be digested.
  DefParameterType!("UndigestedKey",
    reader => reader!(gullet, inner, _extra, state, { gullet.read_arg(state) }),
    reader_predigest => undigested!()
  );

  // Read a token as used when defining it, ie. it may be enclosed in braces.
  DefParameterType!("DefToken",
    reader => reader!(gullet, inner, _extra, state, {
      let mut token_opt = gullet.read_token(state);
      while token_opt.is_some() && token_opt != Some(T_BEGIN!()) {
        let mut toks : Vec<Token> = gullet.read_balanced(state)?.unlist()
          .into_iter().filter(|t| *t != T_SPACE!()).collect();
        token_opt = Some(toks.remove(0));
        gullet.unread(&Tokens::new(toks));
      }
      match token_opt {
        Some(t) => Ok(Tokens!(t)),
        None => {
          error!(target:"expected:DefToken", "Expected a DefToken parameter, found nothing.");
          Ok(Tokens!())
        }
      }
    }),
    reader_predigest => undigested!()
  );

  // Read a variable, ie. a token (after expansion) that is a writable register.
  DefParameterType!("Variable",
    reader => reader!(gullet, inner, _extra, state, {
      let token_opt = gullet.read_x_token(false, false, state)?;
      let defn_opt = match token_opt {
        Some(ref token) => state.lookup_definition(token),
        None => None
      };
      if let Some(defn) = defn_opt {
         if defn.is_register() && !defn.is_readonly() {
           unimplemented!() // TODO
          // Ok(Tokens!(defn, defn.read_arguments(gullet, state)?))
         } else {
            error!(target:"expected:<variable>", "A <variable> was supposed to be here\n Got {:?}", token_opt);
            Ok(Tokens!())
         }
      } else {
        error!(target:"expected:<variable>", "A <variable> was supposed to be here\n Got {:?}", token_opt);
        Ok(Tokens!())
      }
    }),
    reversion => reversion!(gullet,args, inner, state, {
      let defn = args.remove(0);
      // my ($defn, @args) = @$var;
      unimplemented!()
      // TODO: What is defn here? what is the intent?
      // let mut reverted = vec![defn.get_cs()];
      // let reverted_args = if let Some(params) = defn.get_parameters() {
      //   params.revert_arguments(args);
      // } else {
      //   Vec::new()
      // };
      // reverted.extend(reverted_args);
      // Ok(Tokens::new(reverted))
    })
  );

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
