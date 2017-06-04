use package::*;
use rtx_core::document::tag::TagConstructionClosure;

pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

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
                 Ok(vec![T_CS!(ltxtrigger)])
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

  DefConstructorI!(T_CS!("\\par"), None, replacement!(document, args, props, state, {
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
    }),
    after_digest => sub!(|stomach, whatsit, state| {
      let in_preamble = state.lookup_bool("inPreamble");
      if in_preamble {
        whatsit.set_property("inPreamble", ObjectStore::Bool(true));
      } else if let Some(c) = state.remove_value("next_para_class") {
          whatsit.set_property("class", c);
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
  DefConstructorI!(T_CS!("\\inner@par"), None, replacement!(document, args, props, state, {
    if try!(document.maybe_close_element("ltx:p", state)).is_some() { }
    else if document.can_contain(document.get_node(), "ltx:break", state) {
      try!(document.insert_element("ltx:break", Vec::new(), None, state));
    }
  }));

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
    let cs = match token_args.pop_front() {
      Some(cs) => cs,
      None => fatal!(Macro, Expected, "Bad definition macro - no arguments, when some were expected.".to_owned())
    };
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

  let trim_node_whitespace_closure  : Vec<TagConstructionClosure> = tagsub!(document, node, state, {
    document.trim_node_whitespace(node);
  });
  Tag!("ltx:p", auto_close => true, auto_open => true, after_close => trim_node_whitespace_closure);

  // TODO: Move to the right place in the pool definitions (maybe split out individual sub-pools by chapter?)
  DefMacroT!(T_CS!("\\space"), None, T_SPACE!());

  Ok(())
}
