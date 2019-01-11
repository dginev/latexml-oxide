#![feature(proc_macro_hygiene)]
#![feature(proc_macro_quote)]
#![recursion_limit = "100"]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rtx_core;
extern crate proc_macro;
#[macro_use]
extern crate quote;

use crate::util::{get_option, get_options_from_input};
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod ast_builder;
mod constructable;
mod modelable;
mod util;

#[proc_macro_derive(CompileReplacement, attributes(compile_replacement_options))]
pub fn derive_compile_replacement(input: TokenStream) -> TokenStream {
  match parse_macro_input(&input.to_string()) {
    Ok(item) => match constructable::compile_replacement(item).to_string().parse() {
      Ok(parsed) => parsed,
      Err(e) => panic!("Failed to compile replacement: {:?}", e),
    },
    Err(e) => panic!("Failed to parse macro input: {:?}", e),
  }
}

#[proc_macro_derive(CompileExpansion, attributes(compile_expansion_options))]
pub fn derive_compile_expansion(input: TokenStream) -> TokenStream {
  match parse_macro_input(&input.to_string()) {
    Ok(item) => match constructable::compile_expansion(item).to_string().parse() {
      Ok(parsed) => parsed,
      Err(e) => panic!("Failed to compile expansion: {:?}", e),
    },
    Err(e) => panic!("Failed to parse macro input: {:?}", e),
  }
}

#[proc_macro_derive(LoadModel, attributes(load_model_options))]
pub fn derive_load_model(input: TokenStream) -> TokenStream {
  match parse_macro_input(&input.to_string()) {
    Ok(item) => match modelable::load_model(item) {
      Ok(loaded_model) => match loaded_model.to_string().parse() {
        Ok(parsed) => parsed,
        Err(e) => panic!("Failed to load model: {:?}", e),
      },
      Err(e) => panic!("Failed to load model: {:?}", e),
    },
    Err(e) => panic!("Failed to parse macro input: {:?}", e),
  }
}

#[proc_macro_derive(LoadIndirectModel, attributes(load_indirect_model_options))]
pub fn derive_load_indirect_model(input: TokenStream) -> TokenStream {
  match parse_macro_input(&input.to_string()) {
    Ok(item) => match modelable::load_indirect_model(item).to_string().parse() {
      Ok(parsed) => parsed,
      Err(e) => panic!("Failed to load indirect model: {:?}", e),
    },
    Err(e) => panic!("Failed to parse macro input: {:?}", e),
  }
}

// May be good to track: https://github.com/rust-lang/rust/issues/54727
// to see if it becomes possible one day to use this type of technique,
// which would allow declarations such as:
//     #[bound_state(outer)]
//     pub fn load_definitions($state: &mut State, mut outer_stomach: Option<&mut Stomach>) -> Result<()> {
//
//     and
//
//     #[bound_state(inner)]
//     | ... | { before digest closure here...}
//
//    making it possible to use the simple DefMacro("\\a","\\b") form in any context, while auto-binding the nearest state

fn bug() -> ! { panic!("bug") }

#[proc_macro_derive(BoundState, attributes(bind_options))]
pub fn bound_state(input: TokenStream) -> TokenStream {
  match parse_macro_input(&input.to_string()) {
    Ok(item) => {
      let options = get_options_from_input("bound_options", &item.attrs, bug);
      let location_opt = options.as_ref().map(|o| get_option(&o, "location", bug));
      let location = match location_opt {
        Some(n) => n,
        None => "outer",
      };

      let state_declaration = if location == "outer" {
        quote!(
          macro_rules! state {
            () => {
              outer_state!()
            };
          }
        )
      } else if location == "inner" {
        quote!(
          macro_rules! state {
            () => {
              inner_state!()
            };
          }
        )
      } else {
        panic!("Unsupported bound state location: {:?}", location);
      };

      state_declaration.to_string().parse().unwrap()
    },
    Err(e) => panic!("Binding state failed: {:?}", e),
  }
}
