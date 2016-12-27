use std::rc::Rc;
use std::collections::VecDeque;
use libxml::tree::Node;

use rtx_core::Digested;
use rtx_core::state::State;
use rtx_core::token::Token;
use rtx_core::tokens::Tokens;
use rtx_core::parameter::{Parameter, Parameters};
use rtx_core::gullet::Gullet;
use rtx_core::stomach::Stomach;
use rtx_core::definition::expandable::Expandable;
use rtx_core::definition::primitive::{Primitive,PrimitiveOptions};
use rtx_core::definition::constructor::ConstructorOptions;
use rtx_core::document::Document;
use rtx_core::document::tag::TagOptions;

use package::*;
pub fn load_definitions(state: &mut State) {

  RegisterNamespace!("ltx"  , "http://dlmf.nist.gov/LaTeXML", state);
  RegisterNamespace!("svg"  , "http://www.w3.org/2000/svg", state);
  RegisterNamespace!("xlink", "http://www.w3.org/1999/xlink", state);   // Needed for SVG
  // Not directly used, but let's stake out the ground
  RegisterNamespace!("m"    , "http://www.w3.org/1998/Math/MathML", state);
  RegisterNamespace!("xhtml", "http://www.w3.org/1999/xhtml", state);

  DefMacroI!(T_CS!("\\@empty"), None,
   |_gullet, _args, state| {
     Vec::new()
   },
   state);


  //======================================================================
  // Core ID functionality.
  //======================================================================
  // DOCUMENTID is the ID of the document
  // AND prefixes IDs on all other elements.
  // if let Some(& ObjectStore::String(ref docid)) = LookupValue!("DOCUMENTID", state) {
  //   // Wrap in T_OTHER so funny chars don't screw up (no space!)
  //   DefMacroI!("\thedocument@ID", None, T_OTHER!(docid.to_string()), state);
  // } else {
  //   Let!("\thedocument@ID", "\@empty", state);
  // }
  // NewCounter!("@XMARG", "document", idprefix: "XM");

  // Optionally, add ID's to ALL nodes.
  // By default, this is OFF;
  // Set to 1 (or \usepackage[ids]{latexml}) to enable.
  // Set to 0 (or \usepackage[noids]{latexml}) to disable.

  // Tag!("ltx:*", after_open: |document, node, state| {
  //   // If GENERATE_IDS is true, we'll assign an ID to EVERY element,
  //   // EXCEPT ltx:document which only gets an id from an EXPLICIT \thedocument@id.
  //   let tag = document.get_node_qname(node);
  //   if tag != "ltx:document")
  //     && (tag != "ltx:XMWrap")    // No auto-generated id on wrap???
  //     && BoolValue!("GENERATE_IDS") {
  //       GenerateID!(document, node, state);
  //   }
  // }, state);

  //======================================================================
  Tag!("ltx:document", TagOptions{
    after_open: vec![Rc::new(|document, node, box_opt, state| {
      document.process_pending_resources(state);
    })],
    after_close: vec![Rc::new(|document, node, box_opt, state| {
      document.process_pending_resources(state);
    })],
    ..TagOptions::default()
  }, state);

  RequireResource!("LaTeXML.css", state);


  //**********************************************************************
  // CORE TeX; Built-in commands.
  //**********************************************************************

  // ======================================================================
  // Define parsers for standard parameter types.
  DefParameterType!("Plain",
    Parameter {
      reader: Rc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
        let mut value: Vec<Token> = gullet.read_arg(state);
        for inner_opt in inner {
          if let Some(inner_p) = inner_opt {
            value = inner_p.reparse_argument(gullet, value, state);
          }
        }
        value
      }),
      reversion: Some(Rc::new(|_gullet: &mut Gullet, _arg: Vec<Token>, _inner: Vec<Option<Parameters>>, _state: &mut State| -> Vec<Token> {
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
       read_tokens
      })),
      ..Parameter::default()
   }, state);

  DefParameterType!("Optional",
   Parameter {
     reader: Rc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
       // TODO: default !!!
       // let value = gullet.read_optional(state);
       // if (!$value && $default) {
       //   $value = $default; }
       // elsif ($inner) {
       //   ($value) = $inner->reparseArgument($gullet, $value); }
       // value

       gullet.read_optional(state)
     }),

     optional: true,
     reversion: Some(Rc::new(|_gullet: &mut Gullet, arg: Vec<Token>, _inner: Vec<Option<Parameters>>, _state: &mut State| -> Vec<Token> {
       // TODO : default!
       if arg.len() > 0 {
         let mut read_tokens: Vec<Token> = vec![T_OTHER!("[".to_string())];
         // TODO: ($inner ? $inner->revertArguments($arg) : Revert($arg)),
         read_tokens.push(T_OTHER!("]".to_string()));
         read_tokens
       } else {
         Vec::new()
       }
     })),
     ..Parameter::default()
   },
   state);

  // Skip any spaces, but don't contribute an argument.
  DefParameterType!("SkipSpaces",
   Parameter {
     reader: Rc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
       gullet.skip_spaces(state);
       Vec::new()
     }),
     novalue: true,
     ..Parameter::default()
   },
   state);

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

  DefParameterType!("Until",Parameter{
    reader: Rc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, until: Vec<Token>, state: &mut State| {
      gullet.read_until(until, state)
    }),
    // reversion: |arg, until| { vec![Revert!(arg), Revert!(until)] },
    ..Parameter::default()
  }, state);

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
    Parameter {
      reader: Rc::new(|gullet: &mut Gullet, _inner, extra, state:&mut State| {
        gullet.read_match(extra, state)
      }), ..Parameter::default()
    }, state);

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
    Parameter {
      reader: Rc::new(|gullet: &mut Gullet, _inner, _extra, state:&mut State| {
        gullet.read_balanced(state)
      }), ..Parameter::default()
    }, state);


  // Read a Semiverbatim argument; ie w/ most catcodes neutralized.
  DefParameterType!("Semiverbatim",
   Parameter {
     reader: Rc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| gullet.read_arg(state)),
     reversion: Some(Rc::new(|_gullet: &mut Gullet, _arg: Vec<Token>, _inner: Vec<Option<Parameters>>, _state: &mut State| -> Vec<Token> {
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
       read_tokens
     })),
     semiverbatim: true,
     ..Parameter::default()
   },
   state);

  // Read a LaTeX-style optional argument (ie. in []), but the contents read as Semiverbatim.
  DefParameterType!("OptionalSemiverbatim",
   Parameter {
     reader: Rc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| gullet.read_optional(state)),
     semiverbatim: true,
     optional: true,
     reversion: Some(Rc::new(|_gullet: &mut Gullet, arg: Vec<Token>, _inner: Vec<Option<Parameters>>, _state: &mut State| -> Vec<Token> {
       if arg.len() > 0 {
         let mut read_tokens = vec![T_OTHER!("[".to_string())];
         // TODO: add these: Revert($_[0])
         read_tokens.push(T_OTHER!("]".to_string()));
         read_tokens
       } else {
         Vec::new()
       }
     })),
     ..Parameter::default()
   },
   state);

  // Read a token as used when defining it, ie. it may be enclosed in braces.
  DefParameterType!("DefToken",
    Parameter {
      reader: Rc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
        let mut token = gullet.read_token(state);
        let begin_token = Some(T_BEGIN!());
        let space_token = T_SPACE!();

        while token == begin_token {
          let mut toks : Vec<Token> = gullet.read_balanced(state).into_iter().filter(|t| *t != space_token).collect();
          let mut new_tokens = toks.split_off(1);
          gullet.unread(toks);

          token = if new_tokens.is_empty() {
            None
          } else {
            new_tokens.pop()
          };
        }
        match token {
          Some(t) => vec![t],
          None => Vec::new()
        }
      }),
      undigested: true,
      .. Parameter::default()
    }, state);

  // Read the next token
  DefParameterType!("Token",Parameter{
    reader: Rc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
      if let Some(t) = gullet.read_token(state) {
        vec![t]
      } else {
        Vec::new()
      }
    }),
    ..Parameter::default()
  }, state);

  // Read the next token, after expanding any expandable ones.
  DefParameterType!("XToken",Parameter{
    reader: Rc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
      if let Some(t) = gullet.read_x_token(false, false, state) {
        vec![t]
      } else {
        Vec::new()
      }
    }),
    ..Parameter::default()
  }, state);

  // Read a number
  DefParameterType!("Number",Parameter{
    reader: Rc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
      gullet.read_number(state)
    }),
    ..Parameter::default()
  }, state);

  // // Read a floating point number
  // DefParameterType!("Float",Parameter{
  //   reader: Rc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
  //     gullet.read_float()
  //   }),
  //   ..Parameter::default()
  // }, state);

  // Read until the next (balanced) open brace {
  // used for the last TeX-style delimited argument
  DefParameterType!("UntilBrace", Parameter{
    reader: Rc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
      gullet.read_until_brace(state)
    }),
    ..Parameter::default()
  }, state);


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
                 LoadPool!("LaTeX", state);
                 return vec![T_CS!(ltxtrigger)];
               },
               state);
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

  DefConstructorI!(T_CS!("\\par"), None, Some(Rc::new(
    |document: &mut Document, args: &Vec<_>, props:&HashMap<String, ObjectStore>, state: &mut State| {
      let in_preamble = match props.get("inPreamble") {
        Some(& ObjectStore::Bool(v)) => v,
        _ => false
      };
      if !in_preamble {
        document.maybe_close_element("ltx:p", state);
        if let Some(c) = props.get("class") {
          let element = document.get_element();
          if let Some(node) = element {
            if document.get_node_qname(&node, state) == "ltx:para" {  // Only set on the para about to close!
              let class_str = match c {
                & ObjectStore::String(ref v) => v.to_string(),
                _ => String::new()
              };
              document.set_attribute(&node, "class", &class_str);
            }
          }
        }
        document.maybe_close_element("ltx:para", state);
     }
    })),
    ConstructorOptions {
    after_digest: vec![Rc::new(|stomach, whatsit, state| {
      let in_preamble = match LookupValue!("inPreamble", state) {
        Some(& ObjectStore::Bool(v)) => v,
        _ => false
      };
      if in_preamble {
        whatsit.set_property("inPreamble", ObjectStore::Bool(true));
      } else {
        if let Some(c) = RemoveValue!("next_para_class", state) {
          whatsit.set_property("class", c);
        }
        // Digest!(Tokens!(
        //     T_CS("\\LTX@vadjust@afterpar"),
        //     T_CS("\\LTX@clear@vadjust@afterpar")
        // ));
      }
      Vec::new()
    })],
    properties: skippable_props,
    alias: Some("\\par\n".to_string()),
    ..ConstructorOptions::default()
  }, state);

  // OTOH, sometimes \par is just a minimalistic "start a new line"
  // This should be closer for those cases.
  DefConstructorI!(T_CS!("\\inner@par"), None, Some(Rc::new(|document, args, props, state| {
      // if document.maybe_close_element("ltx:p") { }
      // else if document.canContain(document.get_node(), "ltx:break") {
      //   document.insertElement("ltx:break");
      // }
    })),
    ConstructorOptions::default(), state
  );

// Tag("ltx:para", autoClose => 1, autoOpen => 1);

  fn do_def(globally: bool, expanded: bool, stomach: &mut Stomach,  args: Vec<Tokens>, state: &mut State) -> Vec<Digested> {
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
    for arg in args.into_iter() {
      token_args.extend(arg.unlist().into_iter());
    }
    let cs = token_args.pop_front().unwrap();
    // is there a more idiomatic way to downgrade a VecDeque into a Vec?
    let def_body = token_args.into_iter().collect::<Vec<Token>>();
    let params = None;
    let body = Rc::new(move |gullet:&mut Gullet, args:Vec<Tokens>, state:&mut State| def_body.clone());
    println_stderr!("Installing definition for cs: {:?}", cs);
    state.install_definition(ObjectStore::Expandable(Rc::new(
      Expandable{cs: cs, paramlist: params, expansion: body,
        ..Expandable::default()
      })),
      scope);
    // AfterAssignment!(state);
    Vec::new()
  }


  DefPrimitiveI!("\\def SkipSpaces Token UntilBrace {}", |stomach, args, state| {
      do_def(false, false, stomach, args, state)
    },
    PrimitiveOptions {
      locked: true,
      ..PrimitiveOptions::default()
    }, state);
  DefPrimitiveI!("\\gdef SkipSpaces Token UntilBrace {}", |stomach, args, state| {
      do_def(true, false, stomach, args, state)
    },
    PrimitiveOptions {
      locked: true,
      ..PrimitiveOptions::default()
    }, state);
  DefPrimitiveI!("\\edef SkipSpaces Token UntilBrace {}", |stomach, args, state| {
      do_def(false, true, stomach, args, state)
    },
    PrimitiveOptions {
      locked: true,
      ..PrimitiveOptions::default()
    }, state);
  DefPrimitiveI!("\\xdef SkipSpaces Token UntilBrace {}", |stomach, args, state| {
      do_def(true, true, stomach, args, state)
    },
    PrimitiveOptions {
      locked: true,
      ..PrimitiveOptions::default()
    }, state);

    Tag!("ltx:para", TagOptions{auto_close: true, auto_open: true, ..TagOptions::default()}, state);

    let trim_node_whitespace_closure = Rc::new(|document: &mut Document, node: Node, box_opt: Option<Digested>, state: &mut State| {
      document.trim_node_whitespace(node, state);
    });
    Tag!("ltx:p", TagOptions{auto_close: true, auto_open: true, after_close: vec![trim_node_whitespace_closure], ..TagOptions::default()}, state);

}
