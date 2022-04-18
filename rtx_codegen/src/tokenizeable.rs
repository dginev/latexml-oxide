use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Lit, Meta};

use rtx_core::mouth;
use rtx_core::tokens::Tokens;

// Very similar tokenization procedures, enabled at compile-time
// definitely possible to clean this up further...

pub fn compile_expansion(input: DeriveInput) -> TokenStream {
  let expansion: String = match input.attrs[0].parse_meta().unwrap() {
    Meta::NameValue(v) => match v.lit {
      Lit::Str(v) => v.value(),
      _ => panic!("only accepts #[name = \"filename\"] attribute syntax, mandatory double-quotes (Lit)"),
    },
    _ => panic!("only accepts #[name = \"filename\"] attribute syntax, mandatory double-quotes (parse_meta)"),
  };
  let compiled_expansion = if expansion.is_empty() {
    quote!(None)
  } else {
    let performed_expansion = mouth::tokenize_internal(&expansion, None);
    if performed_expansion.is_empty() {
      quote!(None)
    } else {
      // println!("expanded into: {:?} tokens: {:?}", performed_expansion.len(),
      // performed_expansion);
      //
      // TODO: Should "substitute_parameters" be specially performed for runtime-read expansions (via RawTeX?), e.g. when
      // reading external style files? should that even be allowed? We can easily pre-compile all of texlive
      // (or the ~200 supported sty and cls files in the ecosystem) once
      // and have all expansions handled by this code snippet. Hmmm... arguable benefit at this early stage, maybe something beyond 1.0
      quote!(
        Some(ExpansionBody::Tokens(#performed_expansion))
      )
    }
  };
  // We have to jump an extra hoop, since we are forcing the struct-derive
  // mechanism. Once the new procedural macro scheme lands, this begs to be
  // refactored.
  quote!(
    macro_rules! this_expansion {
      () => {#compiled_expansion}
    }
  )
  .into()
}

pub fn compile_tokenize(input: DeriveInput) -> TokenStream {
  let literal: String = match input.attrs[0].parse_meta().unwrap() {
    Meta::NameValue(v) => match v.lit {
      Lit::Str(v) => v.value(),
      _ => panic!("only accepts #[literal = \"value\"] attribute syntax, mandatory double-quotes (Lit)"),
    },
    _ => panic!("only accepts #[literal = \"value\"] attribute syntax, mandatory double-quotes (parse_meta)"),
  };

  let tokenized = if literal.is_empty() {
    Tokens::default()
  } else {
    mouth::tokenize(&literal, None)
  };

  quote!(
    macro_rules! these_tokens {
      () => {#tokenized}
    }
  )
  .into()
}

pub fn compile_tokenize_internal(input: DeriveInput) -> TokenStream {
  let literal: String = match input.attrs[0].parse_meta().unwrap() {
    Meta::NameValue(v) => match v.lit {
      Lit::Str(v) => v.value(),
      _ => panic!("only accepts #[literal = \"value\"] attribute syntax, mandatory double-quotes (Lit)"),
    },
    _ => panic!("only accepts #[literal = \"value\"] attribute syntax, mandatory double-quotes (parse_meta)"),
  };

  let tokenized = if literal.is_empty() {
    Tokens::default()
  } else {
    mouth::tokenize_internal(&literal, None)
  };

  quote!(
    macro_rules! these_internal_tokens {
      () => {#tokenized}
    }
  )
  .into()
}
