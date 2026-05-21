use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

use latexml_core::mouth;
use latexml_core::tokens::Tokens;

// Very similar tokenization procedures, enabled at compile-time
// definitely possible to clean this up further...

pub fn compile_expansion(input: DeriveInput) -> TokenStream {
  let expansion = crate::attr_name_value_str(&input.attrs[0], "name");
  let compiled_expansion = if expansion.is_empty() {
    quote!(None)
  } else {
    let performed_expansion = mouth::tokenize_internal(&expansion);
    if performed_expansion.is_empty() {
      quote!(None)
    } else {
      // rescan for match tokens and unwrap dont_expand...
      let expansion = performed_expansion.pack_parameters().unwrap();
      // println!("expanded into: {:?} tokens: {:?}", performed_expansion.len(),
      // performed_expansion);
      //
      // TODO: Should "substitute_parameters" be specially performed for runtime-read expansions
      // (via RawTeX?), e.g. when reading external style files? should that even be allowed?
      // We can easily pre-compile all of texlive (or the ~200 supported sty and cls files in
      // the ecosystem) once and have all expansions handled by this code snippet. Hmmm...
      // arguable benefit at this early stage, maybe something beyond 1.0
      quote!(
        Some(ExpansionBody::Tokens(#expansion))
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
  let literal = crate::attr_name_value_str(&input.attrs[0], "literal");

  let tokenized = if literal.is_empty() {
    Tokens::default()
  } else {
    mouth::tokenize(&literal)
  };
  quote!(
    macro_rules! these_tokens {
      () => {#tokenized}
    }
  )
  .into()
}

pub fn compile_tokenize_internal(input: DeriveInput) -> TokenStream {
  let literal = crate::attr_name_value_str(&input.attrs[0], "literal");

  let tokenized = if literal.is_empty() {
    Tokens::default()
  } else {
    mouth::tokenize_internal(&literal)
  };

  quote!(
    macro_rules! these_internal_tokens {
      () => {#tokenized}
    }
  )
  .into()
}
