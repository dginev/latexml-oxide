#![recursion_limit="100"]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use] extern crate quote;
extern crate proc_macro;
extern crate syn;

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate rtx_core;
extern crate regex;

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod ast_builder;
mod util;
mod constructable;
mod modelable;

#[proc_macro_derive(CompileReplacement,attributes(compile_replacement_options))]
pub fn derive_compile_replacement(input: TokenStream) -> TokenStream {
  match parse_macro_input(&input.to_string()) {
    Ok(item) => match constructable::compile_replacement(item).to_string().parse() {
      Ok(parsed) => parsed,
      Err(e) => panic!("Failed to compile replacement: {:?}", e)
    },
    Err(e) => panic!("Failed to parse macro input: {:?}", e)
  }
}

#[proc_macro_derive(CompileExpansion,attributes(compile_expansion_options))]
pub fn derive_compile_expansion(input: TokenStream) -> TokenStream {
  match parse_macro_input(&input.to_string()) {
    Ok(item) => match constructable::compile_expansion(item).to_string().parse() {
      Ok(parsed) => parsed,
      Err(e) => panic!("Failed to compile expansion: {:?}", e)
    },
    Err(e) => panic!("Failed to parse macro input: {:?}", e)
  }
}

#[proc_macro_derive(LoadModel,attributes(load_model_options))]
pub fn derive_load_model(input: TokenStream) -> TokenStream {
  match parse_macro_input(&input.to_string()) {
    Ok(item) => match modelable::load_model(item) {
      Ok(loaded_model) => match loaded_model.to_string().parse() {
        Ok(parsed) => parsed,
        Err(e) => panic!("Failed to load model: {:?}", e)
      },
      Err(e) => panic!("Failed to load model: {:?}", e)
  	},
    Err(e) => panic!("Failed to parse macro input: {:?}", e)
  }
}

#[proc_macro_derive(LoadIndirectModel,attributes(load_indirect_model_options))]
pub fn derive_load_indirect_model(input: TokenStream) -> TokenStream {
  match parse_macro_input(&input.to_string()) {
    Ok(item) => match modelable::load_indirect_model(item).to_string().parse() {
      Ok(parsed) => parsed,
      Err(e) => panic!("Failed to load indirect model: {:?}", e)
    },
    Err(e) => panic!("Failed to parse macro input: {:?}", e)
  }
}
