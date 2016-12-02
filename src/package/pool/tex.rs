use std::sync::Arc;
use rtx_core::state::State;
use rtx_core::token::*;
use rtx_core::parameter::{Parameter, Parameters};
use rtx_core::gullet::Gullet;
use rtx_core::definition::constructor::ConstructorOptions;

use package::*;
pub fn load_definitions(state: &mut State) {
  //**********************************************************************
  // CORE TeX; Built-in commands.
  //**********************************************************************

  // ======================================================================
  // Define parsers for standard parameter types.
  DefParameterType!("Plain",
    Parameter {
      reader: Arc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
        let mut value: Vec<Token> = gullet.read_arg(state);
        for inner_opt in inner {
          if let Some(inner_p) = inner_opt {
            value = inner_p.reparse_argument(gullet, value, state);
          }
        }
        value
      }),
      reversion: Some(Arc::new(|_gullet: &mut Gullet, _arg: Vec<Token>, _inner: Vec<Option<Parameters>>, _state: &mut State| -> Vec<Token> {
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
     reader: Arc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
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
     reversion: Some(Arc::new(|_gullet: &mut Gullet, arg: Vec<Token>, _inner: Vec<Option<Parameters>>, _state: &mut State| -> Vec<Token> {
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

  // Read a Semiverbatim argument; ie w/ most catcodes neutralized.
  DefParameterType!("Semiverbatim",
   Parameter {
     reader: Arc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| gullet.read_arg(state)),
     reversion: Some(Arc::new(|_gullet: &mut Gullet, _arg: Vec<Token>, _inner: Vec<Option<Parameters>>, _state: &mut State| -> Vec<Token> {
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
     reader: Arc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| gullet.read_optional(state)),
     semiverbatim: true,
     optional: true,
     reversion: Some(Arc::new(|_gullet: &mut Gullet, arg: Vec<Token>, _inner: Vec<Option<Parameters>>, _state: &mut State| -> Vec<Token> {
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

  // Skip any spaces, but don't contribute an argument.
  DefParameterType!("SkipSpaces",
   Parameter {
     reader: Arc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
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
  //   reader: Arc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
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

  // DefParameterType!("Until",Parameter{
  //   reader: Arc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, until: Vec<Token>, state: &mut State| {
  //     gullet.read_until(until)
  //   }),
  //   // reversion: |arg, until| { vec![Revert!(arg), Revert!(until)] },
  //   ..Parameter::default()
  // }, state);

  // DefParameterType!("Skip1Space",Parameter{
  //   reader: Arc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
  //     gullet.skip_one_space();
  //     vec![]
  //   }),
  //   novalue: true,
  //   ..Parameter::default()
  // }, state);

  // // Read the next token
  // DefParameterType!("Token",Parameter{
  //   reader: Arc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
  //     gullet.read_token()
  //   }),
  //   ..Parameter::default()
  // }, state);

  // // Read the next token, after expanding any expandable ones.
  // DefParameterType!("XToken",Parameter{
  //   reader: Arc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
  //     gullet.read_x_token()
  //   }),
  //   ..Parameter::default()
  // }, state);

  // // Read a number
  // DefParameterType!("Number",Parameter{
  //   reader: Arc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
  //     gullet.read_number()
  //   }),
  //   ..Parameter::default()
  // }, state);

  // // Read a floating point number
  // DefParameterType!("Float",Parameter{
  //   reader: Arc::new(|gullet: &mut Gullet, inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
  //     gullet.read_float()
  //   }),
  //   ..Parameter::default()
  // }, state);

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

  DefConstructorI!(T_CS!("\\par"), None, Some(Arc::new(
    |document: &mut Document, args: &Vec<_>, props:&HashMap<String, ObjectStore>, state: &mut State| {
      let in_preamble = match props.get("inPreamble") {
        Some(& ObjectStore::Bool(v)) => v,
        _ => false
      };
      if !in_preamble {
        // document.maybe_close_element("ltx:p");
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
        // document.maybe_close_element("ltx:para");
     }
    })),
    ConstructorOptions {
    after_digest: vec![Arc::new(|stomach, whatsit, state| {
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
  DefConstructorI!(T_CS!("\\inner@par"), None, Some(Arc::new(|document, args, props, state| {
      // if document.maybe_close_element("ltx:p") { }
      // else if document.canContain(document.get_node(), "ltx:break") {
      //   document.insertElement("ltx:break");
      // }
    })),
    ConstructorOptions::default(), state
  );

// Tag("ltx:para", autoClose => 1, autoOpen => 1);


}
