#![recursion_limit = "100"]
extern crate proc_macro; // workaround until proc_macro becomes available normally

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod constructable;
mod modelable;
mod parametrizeable;
mod testable;
mod tokenizeable;

/// Extract the string value of a `#[name = "..."]` attribute (syn 2 idiom).
///
/// Replaces the syn-1 `attr.parse_meta()` + `Meta::NameValue { lit: Lit::Str(_), .. }`
/// pattern, which was removed in syn 2 (the NameValue arm now stores `value: Expr`
/// instead of `lit: Lit`).
pub(crate) fn attr_name_value_str(attr: &syn::Attribute, expected: &str) -> String {
  if let syn::Meta::NameValue(nv) = &attr.meta {
    if let syn::Expr::Lit(syn::ExprLit {
      lit: syn::Lit::Str(s),
      ..
    }) = &nv.value
    {
      return s.value();
    }
  }
  panic!("only accepts #[{expected} = \"value\"] attribute syntax, mandatory double-quotes")
}

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

#[proc_macro_derive(GlobTeXTests, attributes(directory))]
pub fn derive_tex_tests(input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as DeriveInput);
  testable::compile_tests_at(item)
}
