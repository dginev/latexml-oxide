use std::sync::Arc;
use state::State;
use core::token::*;
use core::definition::ExpansionClosure;
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
    let load_pool_closure : ExpansionClosure = Arc::new(Box::new( move |state| {
      latex::load_definitions(state); vec![T_CS(ltxtrigger.clone())] } ));
    let expansion = Vec::new();
    
    DefMacroI!(trigger_saved, expansion, load_pool_closure, state);
  }
}