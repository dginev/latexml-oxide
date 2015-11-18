use std::sync::Arc;
use state::State;
use core::token::*;
use core::definition::expandable::ExpansionClosure;
use core::parameter::{Parameter, Parameters};
use core::gullet::Gullet;
use core::package::*;
use core::package::pool::latex;

pub fn load_definitions(state : &mut State) {
  // No, \documentclass isn't really a primitive -- It's not even TeX!
  // But we define a number of stubs here that will automatically load
  // the LaTeX pool (or AmSTeX.pool) (which will presumably redefine them), and then
  // stuff the token back to be reexecuted.
  for ltxtrigger in ["\\documentclass", "\\newcommand", "\\renewcommand", "\\newenvironment", "\\renewenvironment",
    "NeedsTeXFormat", "\\ProvidesPackage", "\\RequirePackage", "\\ProvidesFile",
    "makeatletter", "\\makeatother", "\\typeout", "\\begin", "\\listfiles"].into_iter().map(|s| s.to_string()) {
    
    let trigger_saved = ltxtrigger.clone();
    let load_pool_closure : ExpansionClosure = Arc::new(Box::new( move 
      |_gullet, _args, state| {
        latex::load_definitions(state);
        return vec![T_CS(ltxtrigger.clone())];
      }));
    let expansion = None;
    
    DefMacroI!(T_CS(trigger_saved.to_string()), expansion, load_pool_closure, state);
  }

  //======================================================================
  // Define parsers for standard parameter types.
  DefParameterType("Plain".to_string(), Parameter {
    reader: Arc::new(Box::new(|gullet : &mut Gullet, inner : Vec<Option<Parameters>>, state : &mut State| {
      let mut value : Vec<Token> = gullet.read_arg(state);
      for inner_opt in inner.into_iter() {
        match inner_opt {
          Some(inner_p) => {
            value = inner_p.reparse_argument(gullet, value, state);
          },
          _ => {}
        };
      }
      value
    })),
    reversion: Some(Arc::new(Box::new(|_gullet : &mut Gullet, _arg : Vec<Token>, _inner : Vec<Option<Parameters>>, _state : &mut State| -> Vec<Token> {
      // let mut reverted_inner;
      let mut read_tokens : Vec<Token> = vec![T_BEGIN()];
      // for inner_opt in inner.into_iter() {
      //   reverted_inner = match inner_opt {
      //     Some(inner_p) => inner_p.revert_arguments(arg, state),
      //     None => Revert(arg)
      //   };
      // }
      // TODO : push reverted_inner to the read_tokens
      read_tokens.push(T_END());
      read_tokens
    }))),
    ..Parameter::default()}, state);

  DefParameterType("Optional".to_string(), Parameter {
    reader: Arc::new(Box::new(|gullet : &mut Gullet, _inner : Vec<Option<Parameters>>, state : &mut State| {
      // TODO: default !!!
      let value = gullet.read_optional(state);
      // if (!$value && $default) {
      //   $value = $default; }
      // elsif ($inner) {
      //   ($value) = $inner->reparseArgument($gullet, $value); }
      value })),
    
    optional: true,
    reversion: Some(Arc::new(Box::new(|_gullet : &mut Gullet, arg : Vec<Token>, _inner : Vec<Option<Parameters>>, _state : &mut State| -> Vec<Token> {
      // TODO : default!
      if arg.len() > 0 {
        let mut read_tokens : Vec<Token> = vec![T_OTHER("[".to_string())];
        // TODO: ($inner ? $inner->revertArguments($arg) : Revert($arg)),
        read_tokens.push(T_OTHER("]".to_string()));
        read_tokens
      }
      else { Vec::new() }
    }))),
    ..Parameter::default()}, state);



  // Read a Semiverbatim argument; ie w/ most catcodes neutralized.
  DefParameterType("Semiverbatim".to_string(), Parameter {
    reader: Arc::new(Box::new(|gullet : &mut Gullet, _inner : Vec<Option<Parameters>>, state : &mut State| {
      gullet.read_arg(state)
    })),
    reversion: Some(Arc::new(Box::new(|_gullet : &mut Gullet, _arg : Vec<Token>, _inner : Vec<Option<Parameters>>, _state : &mut State| -> Vec<Token> {
      // let mut reverted_inner;
      let mut read_tokens : Vec<Token> = vec![T_BEGIN()];
      // for inner_opt in inner.into_iter() {
      //   reverted_inner = match inner_opt {
      //     Some(inner_p) => inner_p.revert_arguments(arg, state),
      //     None => Revert(arg)
      //   };
      // }
      // TODO : push reverted_inner to the read_tokens
      read_tokens.push(T_END());
      read_tokens
    }))),
    semiverbatim: true,
    ..Parameter::default()}, state);

  // Read a LaTeX-style optional argument (ie. in []), but the contents read as Semiverbatim.
  DefParameterType("OptionalSemiverbatim".to_string(), Parameter {
    reader: Arc::new(Box::new(|gullet : &mut Gullet, _inner : Vec<Option<Parameters>>, state : &mut State| {
      gullet.read_optional(state)
    })),
    semiverbatim : true,
    optional : true,
    reversion : Some(Arc::new(Box::new(|_gullet : &mut Gullet, arg : Vec<Token>, _inner : Vec<Option<Parameters>>, _state : &mut State| -> Vec<Token> {
      if arg.len() > 0 {
        let mut read_tokens = vec![T_OTHER("[".to_string())];
        // TODO: add these: Revert($_[0])
        read_tokens.push(T_OTHER("]".to_string()));
        read_tokens
      } else {
        Vec::new()
      }
    }))),
    ..Parameter::default()}, state);

  // Skip any spaces, but don't contribute an argument.
  DefParameterType("SkipSpaces".to_string() , Parameter {
    reader: Arc::new(Box::new(|gullet : &mut Gullet, _inner : Vec<Option<Parameters>>, state : &mut State| {
      gullet.skip_spaces(state);
      Vec::new()
    })),
    novalue : true,
    ..Parameter::default()}, state);    

}