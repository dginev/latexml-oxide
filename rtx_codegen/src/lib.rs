#![recursion_limit="100"]
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

#[proc_macro_derive(CompileReplacement)]
pub fn derive_compile_replacement(input: TokenStream) -> TokenStream {
  let item = parse_macro_input(&input.to_string()).unwrap();
  constructable::compile_replacement(item).to_string().parse().unwrap()
}

#[proc_macro_derive(LoadModel)]
pub fn derive_load_model(input: TokenStream) -> TokenStream {
  let item = parse_macro_input(&input.to_string()).unwrap();
  modelable::load_model(item).to_string().parse().unwrap()
}

#[proc_macro_derive(LoadIndirectModel)]
pub fn derive_load_indirect_model(input: TokenStream) -> TokenStream {
  let item = parse_macro_input(&input.to_string()).unwrap();
  modelable::load_indirect_model(item).to_string().parse().unwrap()
}