use package::*;
pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  RegisterNamespace!("ltx"  , "http://dlmf.nist.gov/LaTeXML");
  RegisterNamespace!("svg"  , "http://www.w3.org/2000/svg");
  RegisterNamespace!("xlink", "http://www.w3.org/1999/xlink");   // Needed for SVG
  // Not directly used, but let's stake out the ground
  RegisterNamespace!("m"    , "http://www.w3.org/1998/Math/MathML");
  RegisterNamespace!("xhtml", "http://www.w3.org/1999/xhtml");

  DefMacroT!(T_CS!("\\@empty"), None, None);


  //======================================================================
  // Core ID functionality.
  //======================================================================
  // DOCUMENTID is the ID of the document
  // AND prefixes IDs on all other elements.
  if !state.documentid.is_empty() {
    let docid = state.documentid.clone();
    // Wrap in T_OTHER so funny chars don't screw up (no space!)
    DefMacroT!(T_CS!("\\thedocument@ID"), None, T_OTHER!(docid));
  } else {
    Let!("\\thedocument@ID", "\\@empty");
  }
  // TODO:
  // NewCounter!("@XMARG", "document", idprefix: "XM");

  // Optionally, add ID's to ALL nodes.
  // By default, this is OFF;
  // Set to 1 (or \usepackage[ids]{latexml}) to enable.
  // Set to 0 (or \usepackage[noids]{latexml}) to disable.

  Tag!("ltx:*", after_open => sub!(|document, node, box_opt, state| {
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
    after_open => sub!(|document, node, box_opt, state| {
      document.process_pending_resources(state);
    }),
    after_close => sub!(|document, node, box_opt, state| {
      document.process_pending_resources(state);
    })
  );

  RequireResource!("LaTeXML.css");


  //**********************************************************************
  // CORE TeX; Built-in commands.
  //**********************************************************************

  // ======================================================================
  // Define parsers for standard parameter types.
  DefParameterType!("Plain",
    reader => Rc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
      let mut value: Vec<Token> = try!(gullet.read_arg(state));
      for inner_opt in inner {
        if let Some(inner_p) = inner_opt {
          value = inner_p.reparse_argument(gullet, value, state);
        }
      }
      Ok(value)
    }),
    reversion => Some(Rc::new(|_gullet: &mut Gullet, _arg: Vec<Token>, _inner: Vec<Option<Parameters>>, _state: &mut State| {
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
     Ok(read_tokens)
    }))
  );

  DefParameterType!("Optional",
    reader => Rc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
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
    reversion => Some(Rc::new(|_gullet: &mut Gullet, arg: Vec<Token>, _inner: Vec<Option<Parameters>>, _state: &mut State| {
      // TODO : default!
      if arg.len() > 0 {
        let mut read_tokens: Vec<Token> = vec![T_OTHER!("[".to_string())];
        // TODO: ($inner ? $inner->revertArguments($arg) : Revert($arg)),
        read_tokens.push(T_OTHER!("]".to_string()));
        Ok(read_tokens)
      } else {
        Ok(Vec::new())
      }
    }))
  );

  // Skip any spaces, but don't contribute an argument.
  DefParameterType!("SkipSpaces",
    reader => Rc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
      gullet.skip_spaces(state);
      Ok(Vec::new())
    }),
    novalue => true
  );

  // // This is a peculiar type of argument of the form
  // //   <general text> = <filler>{<balanced text><right brace>
  // // however, <filler> does get expanded while searching for the initial {
  // // which IS required in contrast to a general argument; ie a single token is not correct.
  // DefParameterType!("GeneralText",Parameter{
  //   reader: Rc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
  //     let open = gullet.read_x_token();
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
    reader => Rc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, until: Vec<Token>, state: &mut State| {
      gullet.read_until(until, state)
    })
    // reversion: |arg, until| { vec![Revert!(arg), Revert!(until)] },
  );

  // DefParameterType!("Skip1Space",Parameter{
  //   reader: Rc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
  //     gullet.skip_one_space();
  //     vec![]
  //   }),
  //   novalue: true,
  //   ..Parameter::default()
  // }, state);

  // Read a matching keyword, eg. Match:=
  DefParameterType!("Match",
    reader => Rc::new(|gullet: &mut Gullet, _inner, extra, state:&mut State| {
      gullet.read_match(extra, state)
    })
  );

  // Read a keyword; eg. Keyword:to
  // (like Match, but ignores catcodes)
  // DefParameterType!("Keyword",
  //   Parameter {
  //     reader: Rc::new(|gullet: &mut Gullet, _inner, _extra, state:&mut State| {
  //       gullet.read_keyword(state);
  //     }), ..Parameter::default()
  //   }, state);

  // Read balanced material (?)
  DefParameterType!("Balanced",
    reader => Rc::new(|gullet: &mut Gullet, _inner, _extra, state:&mut State| {
      gullet.read_balanced(state)
    })
  );


  // Read a Semiverbatim argument; ie w/ most catcodes neutralized.
  DefParameterType!("Semiverbatim",
    reader => Rc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| gullet.read_arg(state)),
    reversion => Some(Rc::new(|_gullet: &mut Gullet, _arg: Vec<Token>, _inner: Vec<Option<Parameters>>, _state: &mut State| {
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
      Ok(read_tokens)
    })),
    semiverbatim => true);

  // Read a LaTeX-style optional argument (ie. in []), but the contents read as Semiverbatim.
  DefParameterType!("OptionalSemiverbatim",
    reader => Rc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| gullet.read_optional(state)),
    semiverbatim => true,
    optional => true,
    reversion => Some(Rc::new(|_gullet: &mut Gullet, arg: Vec<Token>, _inner: Vec<Option<Parameters>>, _state: &mut State| {
     if arg.len() > 0 {
       let mut read_tokens = vec![T_OTHER!("[".to_string())];
       // TODO: add these: Revert($_[0])
       read_tokens.push(T_OTHER!("]".to_string()));
       Ok(read_tokens)
     } else {
       Ok(Vec::new())
     }
    }))
  );

  // Read a token as used when defining it, ie. it may be enclosed in braces.
  DefParameterType!("DefToken",
    reader => Rc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
      let mut token = gullet.read_token(state);
      let begin_token = Some(T_BEGIN!());
      let space_token = T_SPACE!();

      while token == begin_token {
        let mut toks : Vec<Token> = try!(gullet.read_balanced(state)).into_iter().filter(|t| *t != space_token).collect();
        let mut new_tokens = toks.split_off(1);
        gullet.unread(toks);

        token = if new_tokens.is_empty() {
          None
        } else {
          new_tokens.pop()
        };
      }
      match token {
        Some(t) => Ok(vec![t]),
        None => Ok(Vec::new())
      }
    }),
    undigested => true
  );

  // Read the next token
  DefParameterType!("Token",
    reader => Rc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
      if let Some(t) = gullet.read_token(state) {
        Ok(vec![t])
      } else {
        Ok(Vec::new())
      }
    })
  );

  // Read the next token, after expanding any expandable ones.
  DefParameterType!("XToken",
    reader => Rc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
      if let Some(t) = try!(gullet.read_x_token(false, false, state)) {
        Ok(vec![t])
      } else {
        Ok(Vec::new())
      }
    })
  );

  // Read a number
  DefParameterType!("Number",
    reader => Rc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
      gullet.read_number(state)
    })
  );

  // // Read a floating point number
  // DefParameterType!("Float",Parameter{
  //   reader: Rc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
  //     gullet.read_float()
  //   }),
  //   ..Parameter::default()
  // }, state);

  // Read until the next (balanced) open brace {
  // used for the last TeX-style delimited argument
  DefParameterType!("UntilBrace",
    reader => Rc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
      gullet.read_until_brace(state)
    })
  );


  // No, \documentclass isn't really a primitive -- It's not even TeX!
  // But we define a number of stubs here that will automatically load
  // the LaTeX pool (or AmSTeX.pool) (which will presumably redefine them), and then
  // stuff the token back to be reexecuted.
  for ltxtrigger in ["\\documentclass",
                     "\\newcommand",
                     "\\renewcommand",
                     "\\newenvironment",
                     "\\renewenvironment",
                     "\\NeedsTeXFormat",
                     "\\ProvidesPackage",
                     "\\RequirePackage",
                     "\\ProvidesFile",
                     "\\makeatletter",
                     "\\makeatother",
                     "\\typeout",
                     "\\begin",
                     "\\listfiles"]
                      .into_iter()
                      .map(|s| s.to_string()) {

    DefMacroI!(T_CS!(ltxtrigger),
               None,
               move |_gullet, _args, state| {
                 try!(input_definitions("LaTeX".to_string(),
                  InputDefinitionOptions {
                    extension: Some("pool"),
                    ..InputDefinitionOptions::default()
                  }, state));
                 return Ok(vec![T_CS!(ltxtrigger)]);
               });
  }

  //----------------------------------------------------------------------
  // These determine whether the _next_ paragraph gets indented!
  // thus it needs \par to check whether such indentation has been set.
  // DefPrimitiveI!("\indent",   None, |state| AssignValue(next_para_class => 'ltx_indent'); });
  // DefPrimitiveI!("\noindent", None, || AssignValue(next_para_class => 'ltx_noindent'); });

  // <ltx:para> represents a Logical Paragraph, whereas <ltx:p> is a `physical paragraph'.
  // A para can contain both p and displayed equations and such.

  // Remember; \par _closes_, not opens, paragraphs!
  // Here, we want to close both an open p and para (if either are open).
  let mut skippable_props = HashMap::new();
  skippable_props.insert("alignmentSkippable".to_string(), ObjectStore::Bool(true));

  DefConstructorI!(T_CS!("\\par"), None,
    |document: &mut Document, args: &Vec<_>, props:&HashMap<String, ObjectStore>, state: &mut State| {
      let in_preamble = match props.get("inPreamble") {
        Some(& ObjectStore::Bool(v)) => v,
        _ => false
      };
      if !in_preamble {
        document.maybe_close_element("ltx:p", state);
        if let Some(c) = props.get("class") {
          let element = document.get_element();
          if let Some(mut node) = element {
            if document.get_node_qname(&node, state) == "ltx:para" {  // Only set on the para about to close!
              let class_str = match *c {
                ObjectStore::String(ref v) => v.to_string(),
                _ => String::new()
              };
              document.set_attribute(&mut node, "class", &class_str);
            }
          }
        }
        document.maybe_close_element("ltx:para", state);
     }
    },
    after_digest => sub!(|stomach, whatsit, state| {
      let in_preamble = state.lookup_bool("inPreamble");
      if in_preamble {
        whatsit.set_property("inPreamble", ObjectStore::Bool(true));
      } else {
        if let Some(c) = state.remove_value("next_para_class") {
          whatsit.set_property("class", c);
        }
        // Digest!(Tokens!(
        //     T_CS("\\LTX@vadjust@afterpar"),
        //     T_CS("\\LTX@clear@vadjust@afterpar")
        // ));
      }
      Ok(Vec::new())
    }),
    properties => skippable_props,
    alias => Some("\\par\n".to_string())
  );

  // OTOH, sometimes \par is just a minimalistic "start a new line"
  // This should be closer for those cases.
  DefConstructorI!(T_CS!("\\inner@par"), None, |document, args, props, state| {
      if document.maybe_close_element("ltx:p", state).is_some() { }
      else if document.can_contain(document.get_node(), "ltx:break", state) {
        document.insert_element("ltx:break", Vec::new(), None, state);
      }
  });

  fn do_def(globally: bool, expanded: bool, stomach: &mut Stomach,  args: Vec<Tokens>, state: &mut State) -> Result<Vec<Digested>> {
    // params = parseDefParameters(cs, params);
    if expanded {
      state.noexpand_the = true;
      // body = Expand!(body);
    }

    let scope = if globally {
      Some(Scope::Global)
    } else {
      None
    };
    // switch args from a Vec<Tokens> into a Vec<Token>
    let mut token_args : VecDeque<Token> = VecDeque::new();
    for arg in args {
      token_args.extend(arg.unlist().into_iter());
    }
    let cs = token_args.pop_front().unwrap();
    // is there a more idiomatic way to downgrade a VecDeque into a Vec?
    let def_body = token_args.into_iter().collect::<Vec<Token>>();
    let params = None;
    let body = Rc::new(move |gullet:&mut Gullet, args:Vec<Tokens>, state:&mut State| Ok(def_body.clone()));
    info!("Installing definition for cs: {:?}", cs);
    state.install_definition(ObjectStore::Expandable(Rc::new(
      Expandable{cs: cs, paramlist: params, expansion: body,
        ..Expandable::default()
      })),
      scope);
    // AfterAssignment!(state);
    Ok(Vec::new())
  }


  DefPrimitiveI!("\\def SkipSpaces Token UntilBrace {}", |stomach, args, state| {
      do_def(false, false, stomach, args, state)
    },
    locked => true
  );
  DefPrimitiveI!("\\gdef SkipSpaces Token UntilBrace {}", |stomach, args, state| {
      do_def(true, false, stomach, args, state)
    },
    locked => true
  );
  DefPrimitiveI!("\\edef SkipSpaces Token UntilBrace {}", |stomach, args, state| {
      do_def(false, true, stomach, args, state)
    },
      locked => true
  );
  DefPrimitiveI!("\\xdef SkipSpaces Token UntilBrace {}", |stomach, args, state| {
      do_def(true, true, stomach, args, state)
    },
    locked => true
  );

  Tag!("ltx:para", auto_close => true, auto_open => true);

  let trim_node_whitespace_closure = Rc::new(|document: &mut Document, node: Node, box_opt: Option<Digested>, state: &mut State| {
    document.trim_node_whitespace(node, state);
  });
  Tag!("ltx:p", auto_close => true, auto_open => true, after_close => vec![trim_node_whitespace_closure]);

  // TODO: Move to the right place in the pool definitions (maybe split out individual sub-pools by chapter?)
  DefMacroT!(T_CS!("\\space"), None, T_SPACE!());

  Ok(())
}
