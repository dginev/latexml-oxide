#![recursion_limit = "100"]
extern crate proc_macro; // workaround until proc_macro becomes available normally

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod constructable;
mod modelable;
mod parametrizeable;
mod tokenizeable;
mod testable;

#[proc_macro_derive(CompileReplacement, attributes(replacement))]
pub fn derive_compile_replacement(input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as DeriveInput);
  constructable::compile_replacement(item)
}

#[proc_macro_derive(CompileExpansion, attributes(expansion))]
pub fn derive_compile_expansion(input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as DeriveInput);
  tokenizeable::compile_expansion(item)
}

#[proc_macro_derive(LoadModel, attributes(name))]
pub fn derive_load_model(input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as DeriveInput);
  match modelable::load_model(item) {
    Ok(loaded_model) => loaded_model,
    Err(e) => panic!("Failed to load model: {e:?}"),
  }
}

#[proc_macro_derive(CompileTokens, attributes(literal))]
pub fn derive_compile_tokenize(input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as DeriveInput);
  tokenizeable::compile_tokenize(item)
}

#[proc_macro_derive(CompileTokensInternal, attributes(literal))]
pub fn derive_compile_tokenize_internal(input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as DeriveInput);
  tokenizeable::compile_tokenize_internal(item)
}

#[proc_macro_derive(CompilePrototypeFor, attributes(prototype, inner))]
pub fn derive_compile_prototype_for(input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as DeriveInput);
  parametrizeable::compile_prototype_for(item)
}

#[proc_macro_derive(CompilePrototype, attributes(prototype))]
pub fn derive_compile_prototype(input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as DeriveInput);
  parametrizeable::compile_prototype(item)
}

static mut CONTEXT_DEPTH: u32 = 0;
// Update: still good to track the rust GH issue, but we have already found a solution,
//         just one that the Rust team would certainly frown upon.
//         In essence the `BoundState` proc derive uses a mutable singleton depth meter
//         which gets switched up/down via our custom `start_state_frame!`/`end_state_frame!` macro
// switches         this effectively allows us to do context-sensitive macro definition of `state!`,
//         binding it locally to `outer_state!` in the initial context, and to `inner_state!` in all
// others.
//
// May be good to track: https://github.com/rust-lang/rust/issues/54727
// to see if it becomes possible one day to use this type of technique,
// which would allow declarations such as:
//     #[bound_state(outer)]
//     pub fn load_definitions($state: &mut State, mut outer_stomach: Option<&mut Stomach>) ->
// Result<()> {
//
//     and
//
//     #[bound_state(inner)]
//     | ... | { before digest closure here...}
//
//    making it possible to use the simple DefMacro("\\a","\\b") form in any context, while
// auto-binding the nearest state

#[proc_macro_derive(BoundState)]
pub fn bound_state(_input: TokenStream) -> TokenStream {
  let state_declaration = if unsafe { CONTEXT_DEPTH == 0 } {
    quote!(
      #[allow(unused_macros)]
      macro_rules! state {
        () => {
          outer_state!()
        };
      }
      #[allow(unused_macros)]
      macro_rules! stomach {
        () => {
          outer_stomach!()
        };
      }
    )
  } else {
    quote!(
      #[allow(unused_macros)]
      macro_rules! state {
        () => {
          inner_state!()
        };
      }
      #[allow(unused_macros)]
      macro_rules! stomach {
        () => {
          inner_stomach!()
        };
      }
    )
  };
  state_declaration.into()
}

#[proc_macro_derive(StartStateFrame)]
pub fn start_state_frame(_input: TokenStream) -> TokenStream {
  unsafe { CONTEXT_DEPTH += 1 };
  TokenStream::new()
}

#[proc_macro_derive(EndStateFrame)]
pub fn end_state_frame(_input: TokenStream) -> TokenStream {
  unsafe { CONTEXT_DEPTH -= 1 };
  TokenStream::new()
}

#[proc_macro_derive(GlobTeXTests, attributes(directory))]
pub fn derive_tex_tests(input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as DeriveInput);
  testable::compile_tests_at(item)
}
