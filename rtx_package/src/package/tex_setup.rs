use crate::package::*;

LoadDefinitions!({
  RegisterNamespace!("ltx", "http://dlmf.nist.gov/LaTeXML");
  RegisterNamespace!("svg", "http://www.w3.org/2000/svg");
  // Needed for SVG
  RegisterNamespace!("xlink", "http://www.w3.org/1999/xlink");
  // Not directly used, but let's stake out the ground
  RegisterNamespace!("m", "http://www.w3.org/1998/Math/MathML");
  RegisterNamespace!("xhtml", "http://www.w3.org/1999/xhtml");
  // Namespace for arbitrary data attributes (mapped to data-xxx in html5)
  RegisterNamespace!("data" => "http://dlmf.nist.gov/LaTeXML/data");

  DefMacro!("\\@empty", None);

  //======================================================================
  // Core ID functionality.
  //======================================================================
  // DOCUMENTID is the ID of the document
  // AND prefixes IDs on all other elements.
  let doc_id = state!().lookup_string("DOCUMENTID");
  if !doc_id.is_empty() {
    // Wrap in T_OTHER so funny chars don't screw up (no space!)
    let doc_id_token = T_OTHER!(doc_id);
    DefMacro!(T_CS!("\\thedocument@ID"), None, doc_id_token);
  } else {
    Let!("\\thedocument@ID", "\\@empty");
  }
  NewCounter!("@XMARG", "document", idprefix => "XM");

  //======================================================================
  Tag!("ltx:document",
  after_open => sub[document, _node] {
    document.process_pending_resources()?;
  });
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
  DefParameterType!(Plain, sub[inner, _extra] {
      let mut value = ArgWrap::Tokens(gullet::read_arg()?);
      if let Some(inner_ps) = inner {
        // TODO: How many arguments can we expect back? One? Many?
        //       Currently only passing through the first
        value = inner_ps.reparse_argument( value)?.remove(0);
      }
      Ok(value)
    },
    reversion => sub[arg, inner, _extra] {
      // let mut reverted_inner;
      let mut read_tokens: Vec<Token> = vec![T_BEGIN!()];
      read_tokens.extend(if let Some(inner_ps) = inner {
        inner_ps.revert_arguments(vec![Some(Tokens::new(arg))])?
      } else {
        arg.iter().map(|t| t.clone().revert()).collect()
      });
      read_tokens.push(T_END!());
      Ok(Tokens::new(read_tokens))
    });

  DefParameterType!(DefPlain, sub[inner, _extra] {
      let mut value = ArgWrap::Tokens(gullet::read_arg()?);
      if let Some(inner_ps) = inner {
        value = inner_ps.reparse_argument( value)?.remove(0);
      }
      Ok(value)
    },
    pack_parameters => true,
    reversion => sub[arg, inner, _extra] {
     // let mut reverted_inner;
     let mut read_tokens: Vec<Token> = vec![T_BEGIN!()];
     read_tokens.extend(if let Some(inner_ps) = inner {
      inner_ps.revert_arguments(vec![Some(Tokens::new(arg))])?
     } else {
       arg.iter().map(|t| t.clone().revert()).collect()
     });
     read_tokens.push(T_END!());
     Ok(Tokens::new(read_tokens))
    });

  DefParameterType!(Optional, sub[inner, default] {
      let value = gullet::read_optional(None)?;
      if value.is_none() && !default.is_empty() {
        // TODO: Is the default really multiple Vec<Tokens> ? Or just a single Tokens?
        //       the default[0] is suspicious, compared to the original perl "$default"
        ArgWrap::Tokens(default[0].clone())
      } else if let Some(inner_ps) = inner {
        let mut reparsed = inner_ps.reparse_argument( value.into())?;
        if !reparsed.is_empty() {
          reparsed.remove(0)
        } else {
          ArgWrap::None
        }
      } else {
        value.into()
      }
    },
    optional => true,
    reversion => sub[arg, inner, _extra] {
      // TODO: Same question for the type of "arg" as the one above "default" above:
      //  should this be a single `Tokens` rather than a `Vec<Token>`?
      if !arg.is_empty() {
        let mut read_tokens: Vec<Token> = vec![T_OTHER!("[")];
        read_tokens.extend(match inner {
          None => arg.into_iter().map(Token::revert).collect(),
          Some(inner_ps) => inner_ps.revert_arguments(vec![Some(Tokens::new(arg))])?,
        });
        read_tokens.push(T_OTHER!("]"));
        Ok(Tokens::new(read_tokens))
      } else {
        Ok(Tokens!())
      }
    });

  // This is a peculiar type of argument of the form
  //   <general text> = <filler>{<balanced text><right brace>
  // however, <filler> does get expanded while searching for the initial {
  // which IS required in contrast to a general argument; ie a single token is not correct.
  DefParameterType!(GeneralText, sub[_inner, _extra] {
    if let Some(open) = gullet::read_x_token(None, false)? {
      if open.get_catcode() == Catcode::BEGIN {
        gullet::read_balanced(false)?.unwrap_or_default()
      } else {
        Error!("expected","{", "Expected <general text> here");
        Tokens!(open)
      }
    } else {
      Error!("expected","{", "Expected <general text> here");
      Tokens!()
    }
  });

  DefParameterType!(Until, sub[_inner, until_extra] {
    // TODO: how many tokens are in extra?
    gullet::read_until(&until_extra[0])
  },
  reversion => sub[arg, _inner, until] {
    let mut rev = Vec::new();
    for t in arg {
      rev.push(t.revert());
    }
    for ts in until {
      rev.extend(ts.clone().revert());
    }
    Ok(Tokens::new(rev))
  });

  // Skip any spaces, but don't contribute an argument.
  DefParameterType!(SkipSpaces, sub[_inner, _extra] {
    gullet::skip_spaces()?;
  }, novalue => true);

  DefParameterType!(Skip1Space, sub[_inner, _extra] {
    gullet::skip_one_space()?;
  }, novalue => true);

  // Read the next token
  DefParameterType!(Token, sub[_inner, _extra] {
    match gullet::read_token()? {
      Some(t) => Ok(ArgWrap::Token(t)),
      None => {
        Error!("expected", "Token", "Paramater <Token> found None.");
        Ok(ArgWrap::Tokens(Tokens!()))
      }
    }
  });

  // Read the next token, after expanding any expandable ones.
  DefParameterType!(XToken, sub[_inner, _extra] {
    if let Some(t) = gullet::read_x_token(None, false)? {
      Ok(ArgWrap::Token(t))
    } else {
      Error!("expected","XToken", "Paramater <XToken> found None.");
      Ok(ArgWrap::Tokens(Tokens!()))
    }
  });

  // Read a number
  DefParameterType!(Number, sub[_inner, _extra] {
    gullet::read_number()?
  });

  // Read a floating point number
  DefParameterType!(Float, sub[_inner, _extra] {
    gullet::read_float()?
  });

  // ??? DG: is this needed?
  // sub ReadFloat {
  //   my ($gullet) = @_;
  //   $gullet->skipSpaces;
  //   return ($gullet->readFloat || Float(0)); }

  // Read a dimension
  DefParameterType!(Dimension, sub[_inner, _extra] {
    gullet::read_dimension()? });
  // Read a Glue (aka skip)
  DefParameterType!(Glue, sub[_inner, _extra] { gullet::read_glue()? });
  // Read a MuDimension (math)
  DefParameterType!(MuDimension, sub[_inner, _extra] {
    gullet::read_mu_dimension()? });
  // Read a MuGlue (math)
  DefParameterType!(MuGlue, sub[_inner, _extra] { gullet::read_mu_glue()? });

  // Read until the next (balanced) open brace {
  // used for the last TeX-style delimited argument
  DefParameterType!(UntilBrace, sub[_inner, _extra] {
    gullet::read_until_brace()?.unwrap_or_default()
  });

  // Yet another special case: Require a { but do not read it!!!
  DefParameterType!(RequireBrace, sub[_inner, _extra] {
    gullet::read_token()?.map(|tok| {
      gullet_mut!().unread_one(tok.clone());
      if tok.get_catcode() != Catcode::BEGIN {
        let err = || {Error!("expected","{","Expected a {{ here."); Ok(())};
        err().ok();
      }
      tok
    })
  },
  novalue => true);

  DefParameterType!(XUntil, sub[_inner, untils] {
    // Make sure it's a single token!!!
    let until : Token = untils.first().expect("XUntil needs a token Extra.").into();
    let mut tokens : Vec<Token> = Vec::new();
    while let Some(token) = gullet::read_x_token(Some(false), false)? {
      if token == until {
        break;
      } else if token.get_catcode() == Catcode::BEGIN {
        tokens.push(token);
        tokens.extend(gullet::read_balanced(false)?.unwrap_or_default().unlist());
        tokens.push(T_END!());
      } else if let Some(defn) = state!().lookup_definition_stored(&token)? {
        let args = defn.read_arguments()?;
        tokens.extend(Invocation!(token, args).unlist());
      } else {
        tokens.push(token);
      }
    }
    Ok(Tokens::new(tokens))
  });

  // This is sorta like readbalanced, but expands as it goes.
  // This appears to be needed by certain primitives (eg. \noalign ?)
  // and maybe what we should be using for some Digested ??
  DefParameterType!(Expanded, sub[_inner, _untils] {
    if let Some(token) = gullet::read_x_token(Some(false), false)? {
      if token.get_catcode() == Catcode::BEGIN {
        gullet::read_balanced(true)?.unwrap_or_default().without_dont_expand()
      } else {
        Tokens!(token)
      }
    } else {
      Error!("expected","expanded",
        "was expecting an Expanded parameter value, found nothing.");
      Tokens!()
    }
  },
  reversion => sub[arg, _inner, _extra] {
    let mut tks = vec![T_BEGIN!()];
    tks.extend(arg.into_iter().map(Token::revert).collect::<Vec<_>>());
    tks.push(T_END!());
    Ok(Tokens::new(tks))
  });

  // Set state::smuggle_the=true whenever you want to handle special TeX neutralization of
  // tokens created by \the-like primitives.
  //
  // IMPORTANTLY, call packParameters early on the tokens read from the Gullet
  // to enact the neutralization and discard the temporary smuggle flag that is required
  //
  // Whenever possible, use this `DefExpanded` parameter type directly, rather than hand-crafting a
  // new one.
  DefParameterType!(DefExpanded, sub[_inner, _extra] {
      state_mut!().set_smuggle_the(true);
      let expanded = if let Some(token) = gullet::read_x_token(None, false)? {
        if token.get_catcode() == Catcode::BEGIN {
          gullet::read_balanced(true)?.unwrap_or_default()
        } else {
          Tokens!(token)
        }
      } else {
        Error!("Expected", "DefExpanded", "Expected <DefExpanded> here");
        Tokens!()
      };
      state_mut!().expire_smuggle_the();
      Ok(expanded)
    },
    pack_parameters => true,
    reversion      => sub[arg, _inner, _extra] {
      Ok(Tokens!(T_BEGIN!(), Tokens!(arg).revert(), T_END!())) }
  );

  // Read a matching keyword, eg. Match:=
  DefParameterType!(Match, sub[_inner, extra] {
    let extra_refs = extra.iter().collect::<Vec<&Tokens>>();
    gullet::read_match(&extra_refs)?.unwrap_or_default()
  });

  // Read a keyword; eg. Keyword:to
  // (like Match, but ignores catcodes)
  DefParameterType!(Keyword, sub[_inner, extra] {
    let extra_string : String = extra.iter().map(ToString::to_string)
      .collect::<Vec<String>>().join("");
    Ok(
      gullet::read_keyword(&[&extra_string])?.map(|t| Tokens!(T_OTHER!(t)))
        .unwrap_or_default()
    )
  });

  // Read balanced material (?)
  DefParameterType!(Balanced, sub[_inner, _extra] {
    gullet::read_balanced(false)
  });

  // Read a Semiverbatim argument; ie w/ most catcodes neutralized.
  DefParameterType!(Semiverbatim,
    sub[_inner, _extra] { gullet::read_arg() },
    reversion => sub[arg, inner, _extra] {
      // let mut reverted_inner;
      let mut read_tokens: Vec<Token> = vec![T_BEGIN!()];
      read_tokens.extend(if let Some(inner_ps) = inner {
        inner_ps.revert_arguments(vec![Some(Tokens::new(arg))])?
      } else {
        arg.iter().map(|t| t.clone().revert()).collect()
      });
      read_tokens.push(T_END!());
      Ok(Tokens::new(read_tokens))
    },
    semiverbatim => Some(Vec::new()));

  // Read a LaTeX-style optional argument (ie. in []), but the contents read as Semiverbatim.
  DefParameterType!(OptionalSemiverbatim,
    sub[_inner, _extra] { gullet::read_optional(None) },
    semiverbatim => Some(Vec::new()),
    optional => true,
    reversion => sub[arg, _inner, _extra] {
     if !arg.is_empty() {
       let mut read_tokens = vec![T_OTHER!(s!("["))];
       read_tokens.extend(arg.into_iter().map(Token::revert).collect::<Vec<_>>());
       read_tokens.push(T_OTHER!(s!("]")));
       Ok(Tokens::new(read_tokens))
     } else {
       Ok(Tokens!())
     }
    }
  );

  // Be careful here: if % appears before the initial {, it's still a comment!
  // Also, note that non-typewriter fonts will mess up some chars on digestion!
  DefParameterType!(Verbatim, sub[_inner, _extra] {
      gullet::read_until(&Tokens!(T_BEGIN!()))?;
      state_mut!().begin_semiverbatim(Some(&['%', '\\']));
      let arg = gullet::read_balanced(false)?;
      state_mut!().end_semiverbatim()?;
      Ok(arg)
    },
    before_digest => {
      stomach_mut!().bgroup();
      MergeFont!(family => "typewriter");
    },
    after_digest => {
      stomach_mut!().egroup()?;
    },
    reversion => sub[arg, _inner, _extra] {
      let mut reverted = vec![T_BEGIN!()];
      reverted.extend(arg.into_iter().map(Token::revert).collect::<Vec<_>>());
      reverted.push(T_END!());
      Ok(Tokens::new(reverted))
    }
  );

  // Read Verbatim, but allows expanding command sequences
  DefParameterType!(HyperVerbatim, sub[_inner, _extra] {
      gullet::read_until(&Tokens!(T_BEGIN!()))?;
      state_mut!().begin_semiverbatim(Some(&['%']));
      DefMacro!(T_CS!("\\%"),              None, T_OTHER!("%"), scope => Some(Scope::Local));
      DefMacro!(T_CS!("\\#"),              None, T_OTHER!("#"), scope => Some(Scope::Local));
      DefMacro!(T_CS!("\\&"),              None, T_OTHER!("&"), scope => Some(Scope::Local));
      DefMacro!(T_CS!("\\textunderscore"), None, T_OTHER!("_"), scope => Some(Scope::Local));
      state_mut!().let_i(&T_CS!("\\_"), &T_CS!("\\textunderscore"), None);
      DefMacro!(T_CS!("\\hyper@tilde"), None, T_OTHER!("~"), scope => Some(Scope::Local));
      state_mut!().let_i(&T_CS!("\\~"), &T_CS!("\\hyper@tilde"), None);
      state_mut!().let_i(&T_CS!("\\textasciitilde"), &T_CS!("\\hyper@tilde"), None);
      state_mut!().let_i(&T_CS!("\\\\"), &T_CS!("\\@backslashchar"), None);
      // Having prepared, read in the argument, expanding as we go
      let arg = gullet::read_balanced(true)?;
      state_mut!().end_semiverbatim()?;
      arg
    },
    before_digest => {
      stomach_mut!().bgroup();
      MergeFont!(family => "typewriter"); },
    after_digest => {
      stomach_mut!().egroup()?; },
    reversion => sub[arg, _inner, _extra] {
      let mut reverted = vec![T_BEGIN!()];
      reverted.extend(arg.into_iter().map(Token::revert).collect::<Vec<_>>());
      reverted.push(T_END!());
      Ok(Tokens::new(reverted))
    }
  );
  // Read an argument that will not be digested.
  DefParameterType!(Undigested, sub[_inner, _extra] { gullet::read_arg()},
  predigest => sub[arg]{ Ok(arg.undigested()) }
  reversion => sub[arg, _inner, _extra] {
    if arg.is_empty() {
      Ok(Tokens!())
    } else {
      let mut read_tokens = vec!(T_BEGIN!());
      read_tokens.extend(arg.into_iter().map(Token::revert).collect::<Vec<_>>());
      read_tokens.push(T_END!());
      Ok(Tokens::new(read_tokens))
    }
  });

  // Read a LaTeX-style optional argument (ie. in []), but it will not be digested.
  DefParameterType!(OptionalUndigested,
    sub[_inner, _extra] { gullet::read_optional(None) },
    predigest => sub[arg]{ Ok(arg.undigested()) }
    optional => true,
    reversion => sub[arg, _inner, _extra] {
      if arg.is_empty() {
        Ok(Tokens!())
      } else {
        let mut read_tokens = vec!(T_OTHER!("["));
        read_tokens.extend(arg.into_iter().map(Token::revert).collect::<Vec<_>>());
        read_tokens.push(T_OTHER!("]"));
        Ok(Tokens::new(read_tokens))
      }
  });

  // Read a keyword value (KeyVals), that will not be digested.
  DefParameterType!(UndigestedKey, sub[_inner, _extra] {
    gullet::read_arg() },
  predigest => sub[arg]{ Ok(arg.undigested()) });
  DefParameterType!(UndigestedDefKey, sub[_inner, _extra] {
    gullet::read_arg() },
  pack_parameters => true,
  predigest => sub[arg]{ Ok(arg.undigested()) });

  // Read a token as used when defining it, ie. it may be enclosed in braces.
  DefParameterType!(DefToken, sub[_inner, _extra] {
    let mut token = gullet::read_token()?;
    let space_token = T_SPACE!();

    while token.is_some() && token.as_ref().unwrap().get_catcode() == Catcode::BEGIN {
      let mut toks : Vec<Token> = gullet::read_balanced(false)?
        .unwrap_or_default().unlist().into_iter().filter(|t| *t != space_token).collect();
      if !toks.is_empty() {
        token = Some(toks.remove(0));
        if !toks.is_empty() {
          gullet_mut!().unread(Tokens::new(toks));
        }
      } else {
        token = None;
      }
    }
    match token {
      Some(t) => Ok(ArgWrap::Token(t)),
      None => {
        Error!("expected","DefToken",
          "Expected a DefToken parameter, found nothing.");
        Ok(ArgWrap::Tokens(Tokens!()))
      }
    }
  },
  predigest => sub[arg]{ Ok(arg.undigested()) });

  // Stub register for misdefinitions, to avoid a cascade of Errors.
  DefRegister!("\\lx@DUMMY@REGISTER", Tokens!());

  // Read a variable, ie. a token (after expansion) that is a writable register.
  DefParameterType!(Variable, sub[_inner, _extra] {
    let token_opt = gullet::read_x_token(None, false)?;
    let defn_opt = match token_opt {
      Some(ref token) => state_mut!().lookup_register_definition(token),
      None => None
    };
    if let Some(defn) = defn_opt {
        if defn.is_register() && !defn.is_readonly() {
          let args = defn.read_arguments()?;
          // TODO: What is this datatype ?
          // How does it fit the rtx typed interfaces for parameter types?
          // An extension seems required, also due to the Register parameter type right under.
          // Ok(Tokens!(defn_tok, defn_args))
          Ok(ArgWrap::RegisterDefinition(Box::new((token_opt.unwrap(), args))))
        } else {
          let message = s!("A <variable> was supposed to be here\n Got {:?}", token_opt);
          Error!("expected","<variable>", message);
          Ok(ArgWrap::Tokens(Tokens!()))
        }
    } else {
      let message = s!("A <variable> was supposed to be here\n Got {:?}", token_opt);
      Error!("expected","<variable>", message);
      Ok(ArgWrap::Tokens(Tokens!()))
    }
  },
  reversion => sub[args, _inner, _extra] {
    let _defn = args.remove(0);
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
  });

  // Same, but not necessarily writable
  DefParameterType!(Register, sub[_inner, _extra] {
    let token = gullet::read_x_token(None, false)?;
    let defn = match token {
      None => None,
      Some(ref t) => state_mut!().lookup_register_definition(t)
    };
    match defn {
      Some(register) => {
        let args = register.read_arguments()?;
        Ok(ArgWrap::RegisterDefinition(Box::new((token.unwrap(), args))))
      },
      None => {
        let message = s!("A <register> was supposed to be here. Got {:?}", token);
        Error!("expected","<register>", message);
        if let Some(t) = token {
          if is_definable(&t) {
            def_register(t, None, Tokens!(), None)?;
          }
        }
        Ok(ArgWrap::Tokens(Tokens!()))
      }
    }
  },
  // TODO: If we want to revert "arg" in an honest manner, it needs to be an ArgWrap type.
  reversion => sub[_arg, _inner, _extra] {
    // my ($var) = @_;
    // my ($defn, @args) = @$var;
    // my $params = $defn->getParameters;
    // return Tokens($defn->getCS, ($params ? $params->revertArguments(@args) : ()));
    Ok(Tokens!())
  });

  DefParameterType!(TeXFileName, sub[_inner, _extra] {
    use Catcode::*;
    let mut tokens = Vec::new();
    let mut token_opt;
    loop {
      token_opt = gullet::read_x_token(Some(false), false)?;
      if let Some(ref token) = token_opt {
        if matches!(token.get_catcode(), SPACE | EOL | COMMENT | CS) {
          break
        }
      } else { break; }
      if let Some(token) = token_opt {
        tokens.push(token);
      }
    }

    if let Some(token) = token_opt {
      let cc = token.get_catcode();
      if ! matches!(cc, SPACE | EOL | COMMENT) {
        gullet_mut!().unread_one(token);
      }
    }
    // Strip outer "" ???
    let quote = T_OTHER!("\"");
    if tokens.len() > 1 && tokens.first().unwrap() == &quote
      && tokens.last().unwrap() == &quote {
      tokens.remove(0);
      tokens.pop();
    }
    tokens
  });

  DefPrimitive!("\\ltx@loadpool {}", sub[(name)] {
    LoadPool!(&name.to_string());
  });

  // A LaTeX style directory List
  DefParameterType!(DirectoryList, sub[__inner, _extra] {
      // my ($gullet) = @_;
      // $gullet->skipSpaces;
      // if ($gullet->ifNext(T_BEGIN)) {
      //   $gullet->readToken;
      //   my @dirs = ();
      //   $gullet->skipSpaces;
      //   while ($gullet->ifNext(T_BEGIN)) {
      //     # Should these be Semiverbatim ??
      //     push(@dirs, $gullet->readArg);
      //     $gullet->readMatch(T_OTHER(',')); }
      //   $gullet->skipSpaces;
      //   if ($gullet->ifNext(T_END)) {
      //     $gullet->readToken; }
      //   else {
      //     Error('expected', '}', $gullet, "A closing } was supposed to be here"); }
      //   LaTeXML::Core::Array->new(
      //     open => T_BEGIN, close => T_END, itemopen => T_BEGIN, itemclose => T_END,
      //     type => LaTeXML::Package::parseParameters(ToString("Semiverbatim"), "CommaList")->[0],
      //     values => [@dirs]); }
      // else {
      //   Error('expected', 'DirectoryList', $gullet,
      //          "A DirectoryList was supposed to be here"); } });
      unimplemented!();
      Ok(Tokens!())
  });

  // This reads a Box as needed by \raise, \lower, \moveleft, \moveright.
  // Hopefully there are no issues with the box being digested
  // as part of the reader???
  DefParameterType!(MoveableBox, sub[_inner, _extra] {
    gullet::skip_spaces()?;
    if let Some(xtoken) = gullet::read_x_token(None, false)? {
      Tokens!(xtoken)
    } else {
      Tokens!()
    }
  }, predigest => sub[arg] {
    let token = arg.unlist().remove(0);
    let mut stuff = stomach::invoke_token(&token)?;
    if !stuff.is_empty() {
      let tbox = stuff.remove(0);
      let csname = match tbox.data() {
        DigestedData::Whatsit(ref w) =>
          w.borrow().definition.get_cs_name().to_string(),
        _ => tbox.to_string()
      };
      if csname != "\\hbox" && csname != "\\vbox" && csname != "\\vtop" {
        let message = s!("A <box> was supposed to be here.\nGot {}", csname);
        Error!("expected","<box>", message);
        None
      } else {
        Some(tbox)
      }
    } else {
      let message = s!("A <box> was supposed to be here.\nGot none.");
      Error!("expected","<box>", message);
      None
    }
  });

  // Read a parenthesis delimited argument.
  // Note that this does NOT balance () within the argument.
  DefParameterType!(BalancedParen, sub[_inner, _extra] {
    let tok_opt = gullet::read_x_token(None,false)?;
    let is_paren = match tok_opt {
      Some(ref t) => t.with_str(|ts| ts == "("),
      _ => false
    };
    if is_paren {
      gullet::read_until(&Tokens!(T_OTHER!(")"))).map(Some)
    } else {
      if let Some(tok) = tok_opt {
        gullet_mut!().unread_one(tok);
      }
      Ok(None)
    }
  },
  reversion => sub[args, _inner, _extra] {
    Ok(Tokens!(
      T_OTHER!("("), Tokens::new(args).revert(), T_OTHER!(")")
    ))
  });

  // Read a digested argument, digesting as it is being read.
  // The usual macro parameter (generally written as {}) gets tokenized and digested
  // in separate stages, w/o recognizing any special macros or catcode changes within (eg. \url).
  // Rarely, you need a parameter that gets digested AS IT'S READ until ending }.
  // Note that this also recognizes args as \bgroup ... \engroup
  // It is useful when the content would usually need to have been \protect'd
  // in order to correctly deal with catcodes.
  // BEWARE: This is NOT a shorthand for a simple digested {}!
  DefParameterType!(Digested, sub[_inner, _extra] {
      gullet::skip_spaces()?;
      Ok(Tokens!())
    },
    predigest => sub[_arg] {
      let ismath = state!().lookup_bool("IN_MATH");
      let mut list = Vec::new();
      let mut next_token = None;
      while let Some(token) = gullet::read_x_token(Some(false), false )? {
        let is_last = token.get_catcode() != Catcode::SPACE && token != T_RELAX!();
        next_token = Some(token);
        if is_last {
          break;
        }
      }

      if let Some(token) = next_token {
        if token.get_catcode() == Catcode::BEGIN {
          stomach::digest(token)?;
          list.extend(stomach::digest_next_body(None)?);
          list.pop();
        } else {
          list = stomach::invoke_token(&token)?;
        }
      }

      list.retain(|tbox| ! matches!(tbox.data(), DigestedData::Comment(_)));
      let mode = Some(if ismath { TexMode::Math } else { TexMode::Text });
      List { boxes:list,  mode, ..List::default() }
    },
    reversion => sub[args,_inner,_extra] {
      Ok(Tokens!(T_BEGIN!(), Tokens::new(args).revert(), T_END!())) }
  );

  // A variation: Digest until we encounter a given token!
  DefParameterType!(DigestUntil, sub[_inner, _untils] {
      gullet::skip_spaces()?;
      Ok(Tokens!())
    },
    // TODO: To implement this natively, we need "untils" i.e.
    // "extra" passed into "predigest" as well.
    predigest => {
      unimplemented!();
      //   let ismath = state!().lookup_bool("IN_MATH");
      //   stomach::digest_next_body(Some(until))?
    //   my @list   = $state::>getStomach->digestNextBody($until);
    //   @list = grep { ref $_ ne 'LaTeXML::Core::Comment' } @list;
    //   List(@list, mode => ($ismath ? 'math' : 'text'));
      ()
    },
    reversion => sub[args,_inner,_extra] {
      Ok(Tokens!(T_BEGIN!(), Tokens::new(args).revert(), T_END!())) }
  );

  // Reads until the current group has ended.
  // This is useful for environment-like constructs,
  // particularly alignments (which may or may not be actual environments),
  // but which need special treatment of some of their content
  // as the expansion is carried out.
  DefParameterType!(DigestedBody, sub[__inner, _extra] {
      Ok(Tokens!()) // all done in predigestion
    },
    predigest => {
      let ismath   = state!().lookup_bool("IN_MATH");
      let mut list     = stomach::digest_next_body(None)?;
      // In most (all?) cases, we're really looking for a single Whatsit here...
      list.retain(|tbox| !tbox.is_comment());
      let mut digested = List::new(list);
      digested.mode = if ismath { Some(TexMode::Math) } else { Some(TexMode::Text) };
      digested
    }
  );

  // In addition to the standard TeX Dimension, there are various LaTeX constructs
  // (particularly, the LaTeX picture environment, and the various pstricks packages)
  // that take a different sort of length.  They differ in two ways.
  //   (1) They do not accept a comma as decimal separator
  //      (they generally use it to separate coordinates), and
  //   (2) They accept a plain float which is scaled against a Dimension register.
  //      Actually, there are two subcases:
  //     (a) picture accepts a float, which is scaled against \unitlength
  //     (b) pstricks accepts a float, and optionally a unit,
  //        If the unit is omitted, it is relative to \psxunit or \psyunit.
  // How to capture these ?
  //DefParameterType!(Length, sub {
  ////   my($gullet,$unit)=@_;

  // CommaList expects something like {balancedstuff,...}
  DefParameterType!(CommaList, sub[__inner, _extra] {
      // my ($gullet, $type) = @_;
      // my $typedef = $type &&
      //       LaTeXML::Package::parseParameters(ToString($type), "CommaList")->[0];
      // my @items = ();
      // if ($gullet->ifNext(T_BEGIN)) {
      //   $gullet->readToken;
      //   my @tokens = ();
      //   my $comma  = T_OTHER(',');
      //   while (my $token = $gullet->readToken) {
      //     my $cc = $token->getCatcode;
      //     if ($cc == CC_END) {
      //       push(@items, Tokens(@tokens));
      //       last; }
      //     elsif ($token->equals($comma)) {
      //       push(@items, Tokens(@tokens)); @tokens = (); }
      //     elsif ($cc == CC_BEGIN) {
      //       push(@tokens, $token, $gullet->readBalanced->unlist, T_END); }
      //     else {
      //       push(@tokens, $token); } }
      //   if ($typedef) {
      //     @items = map { [$typedef->reparseArgument($gullet, $_)]->[0] } @items; } }
      // else {
      //   # If no brace, just read one item or token, but still make Array!
      //   push(@items, ($typedef ? $typedef->readArguments($gullet, "CommaList")
      //       : ($gullet->readToken))); }
      // LaTeXML::Core::Array->new(open => T_BEGIN, close => T_END, type => $typedef,
      //   values => [@items]); });
      unimplemented!();
      Ok(Tokens!())
  });

  // Support for Key / Value arguments.
  // The very basic form is
  //   RequiredKeyVals: $keyset
  //   OptionalKeyVals: $keyset
  // to parse Key-Value pairs from a given keyset (see the 'keyval' package
  // documentation for more information). These types of KeyVal
  // parameters will return a LaTeXML::Core::KeyVals object, which can then be
  // used to access the values of the individual items.
  // The difference between the two forms is that RequiredKeyVals expects a set of
  // key-value pairs wrapped in T_BEGIN T_END, where as OptionalKeyVals optionally
  // expects a set of KeyValue pairs wrapped in T_OTHER('[') T_OTHER(']')
  //
  // Several extension of the keyval package exist, the most common one we support
  // is the xkeyval package. This introduces further variations on the keyval
  // arguments parsing, in particular it allows to read keys from more than one
  // keyset at once. These can be specified by giving comma-seperated values in
  // the keyset argument. By default, a key will only be set in the **first**
  // keyset it occurs in. By using
  //   RequiredKeyVals+: $keysets
  //   OptionalKeyVals+: $keysets
  // the key will be set in all keysets instead.
  //
  // All keys to be parsed with these arguments should be declared using
  // DefKeyVal in LaTeXML::Package. By default, an error is thrown if an unknown
  // key is encountered. To surpress this behaviour, and instead store all
  // undefined keys, use
  //   RequiredKeyVals*: $keysets
  //   OptionalKeyVals*: $keysets
  // instead. The '*' and '+' modifiers can be combined by using:
  //   RequiredKeyVals*+: $keysets
  //   OptionalKeyVals*+: $keysets
  //
  // Furthermore, the xkeyval package supports giving prefixes to keys,
  //   RequiredKeyVals[*][+]: $prefix|$keysets
  //   OptionalKeyVals[*][+]: $prefix|$keysets
  //
  // Finally, it is possible to specify specific keys to skip when digesting the
  // object. This can be achieved using comma-seperated key values in
  //   RequiredKeyVals[*][+]: $prefix|$keysets|$skip
  //   OptionalKeyVals[*][+]: $prefix|$keysets|$skip

  pub fn required_key_vals(
    star:bool, plus:bool, keysets: Option<&Parameters>) -> Result<KeyVals> {
    if gullet::if_next(&T_BEGIN!())? {
      keyvals_aux( Some(T_END!()), KVSpec {
        star, plus,
        keysets: vec![keysets.cloned()],
        ..KVSpec::default()
      })
    } else {
      Error!("Expected","{", "Missing keyval arguments");
      Ok(KeyVals::default())
    }
  }

  DefParameterType!(RequiredKeyVals, sub[inner, _extra] {
      required_key_vals(false, false, inner)
    },
    reversion => sub[arg, _inner, _extra] {
      Ok(Tokens!(T_BEGIN!(), Tokens::new(arg).revert(), T_END!()))
    });
  DefParameterType!(RequiredKeyValsStar, sub[inner, _extra] {
      required_key_vals(true, false, inner)
    },
    reversion => sub[arg, _inner, _extra] {
      Ok(Tokens!(T_BEGIN!(), Tokens::new(arg).revert(), T_END!()))
    });
  DefParameterType!(RequiredKeyValsPlus, sub[inner, _extra] {
      required_key_vals(false, true, inner)
    },
    reversion => sub[arg, _inner, _extra] {
      Ok(Tokens!(T_BEGIN!(), Tokens::new(arg).revert(), T_END!()))
    });
  DefParameterType!(RequiredKeyValsStarPlus, sub[inner, _extra] {
      required_key_vals(true, true, inner)
    }, reversion => sub[arg, _inner, _extra] {
      Ok(Tokens!(T_BEGIN!(), Tokens::new(arg).revert(), T_END!()))
    });

  pub fn optional_key_vals(
    star: bool,
    plus: bool,
    keysets: Option<&Parameters>,

  ) -> Result<Option<KeyVals>> {
    if gullet::if_next(&T_OTHER!("["))? {
      let kvs: KeyVals = keyvals_aux(
        Some(T_OTHER!("]")),
        KVSpec {
          star,
          plus,
          keysets: vec![keysets.cloned()], // TODO: Revisit carefully
          ..KVSpec::default()
        },
          )?;
      Ok(Some(kvs))
    } else {
      Ok(None)
    }
  }

  DefParameterType!(OptionalKeyVals, sub[inner, _extra] {
    optional_key_vals(false, false, inner)
  }, optional=>true,
  reversion => sub[arg, _inner, _extra] {
    Ok(Tokens!(T_OTHER!("["), Tokens::new(arg).revert(), T_OTHER!("]")))
  });
  DefParameterType!(OptionalKeyValsStar, sub[inner, _extra] {
    optional_key_vals(true, false, inner)
  }, optional=>true,
  reversion => sub[arg, _inner, _extra] {
    Ok(Tokens!(T_OTHER!("["), Tokens::new(arg).revert(), T_OTHER!("]")))
  });
  DefParameterType!(OptionalKeyValsPlus, sub[inner, _extra] {
    optional_key_vals(false, true, inner)
  }, optional=>true,
  reversion => sub[arg, _inner, _extra] {
    Ok(Tokens!(T_OTHER!("["), Tokens::new(arg).revert(), T_OTHER!("]")))
  });
  DefParameterType!(OptionalKeyValsPlusStar, sub[inner, _extra] {
    optional_key_vals(true, true, inner)
  }, optional=>true,
  reversion => sub[arg, _inner, _extra] {
    Ok(Tokens!(T_OTHER!("["), Tokens::new(arg).revert(), T_OTHER!("]")))
  });

  // Not sure that this is the most elegant solution, but...
  // What I'd really like are some sort of parameter modifiers, mathstyle, font... until...?
  DefParameterType!(DisplayStyle, sub[_inner, _extra] { gullet::read_arg() },
    before_digest => {
      stomach_mut!().bgroup();
      MergeFont!(mathstyle => "display");
    },
    after_digest => { stomach_mut!().egroup()?; },
    reversion => sub[arg, _inner, _extra] {
      Ok(Tokens!(T_BEGIN!(), Tokens::new(arg).revert(), T_END!()))
    });
  // TODO: Add when needed
  // DefParameterType!(TextStyle, sub[inner, _extra] {
  //     $_[0]->readArg; },
  //   beforeDigest => sub {
  //     $_[0]->bgroup;
  //     MergeFont(mathstyle => 'text'); },
  //   afterDigest => sub {
  //     $_[0]->egroup; },
  //   reversion => sub { (T_BEGIN, Revert($_[0]), T_END); });
  // DefParameterType!(ScriptStyle, sub[inner, _extra] {
  //     $_[0]->readArg; },
  //   beforeDigest => sub {
  //     $_[0]->bgroup;
  //     MergeFont(mathstyle => 'script'); },
  //   afterDigest => sub {
  //     $_[0]->egroup; },
  //   reversion => sub { (T_BEGIN, Revert($_[0]), T_END); });
  // DefParameterType!(ScriptscriptStyle, sub[inner, _extra] {
  //     $_[0]->readArg; },
  //   beforeDigest => sub {
  //     $_[0]->bgroup;
  //     MergeFont(mathstyle => 'scriptscript'); },
  //   afterDigest => sub {
  //     $_[0]->egroup; },
  //   reversion => sub { (T_BEGIN, Revert($_[0]), T_END); });
  // # Perverse naming convention: not script style, but in the style of a script relative to
  // current.
  DefParameterType!(InScriptStyle, sub[_inner, _extra] {
      gullet::read_arg() },
    before_digest => {
      stomach_mut!().bgroup();
      MergeFont!(scripted => true);
    },
    after_digest => { stomach_mut!().egroup()?; },
    reversion => sub[arg, _inner, _extra] {
        Ok(Tokens!(T_BEGIN!(), Tokens::new(arg).revert(), T_END!()))
    });
  // # NOTE: the various parameter features don't combine easily!!
  // # I need a ScriptStyleUntil for \root!!!
  // # I also need to redo fractions using these new types....
  DefParameterType!(OptionalInScriptStyle, sub[_inner, _extra] {
      gullet::read_optional(None)
    },
    before_digest => {
      stomach_mut!().bgroup();
      MergeFont!(scripted => true);
    },
    after_digest => {
      stomach_mut!().egroup()?;
    },
    optional => true,
    reversion => sub[arg,_inner,_extra] {
      if arg.is_empty() { Ok(Tokens!()) }
      else {
        let mut tks = vec![T_OTHER!("[")];
        tks.extend(arg.into_iter().map(|t| t.revert()));
        tks.push(T_OTHER!("]"));
        Ok(Tokens::new(tks))
      }
    });
  DefParameterType!(InFractionStyle, sub[_inner, _extra] {
      gullet::read_arg()
    },
    before_digest => {
      stomach_mut!().bgroup();
      MergeFont!(fraction => true);
    },
    after_digest => {
      stomach_mut!().egroup()?;
    },
    reversion => sub[arg,_inner,_extra] {
      let mut reverted = vec![T_BEGIN!()];
      reverted.extend(arg.into_iter().map(Token::revert));
      reverted.push(T_END!());
      Ok(Tokens::new(reverted))
    });

  //**********************************************************************
  // LaTeX has a very particular notion of "Undefined",
  // so let's get that squared away at the outset; it's useful for TeX, too!
  // Naturally, it uses \csname to check, which ends up DEFINING the possibly undefined macro as
  // \relax
  DefMacro!("\\@ifundefined{}{}{}", sub[(name, if_token, else_token)] {
    let cs = T_CS!(s!("\\{}", Expand!(name).to_string()));
    if IsDefined!(&cs) {
      Ok(else_token)
    } else {
      state_mut!().let_i(&cs, &T_RELAX!(), None); // Yuck, but traditional!
      Ok(if_token)
    }
  });
});
