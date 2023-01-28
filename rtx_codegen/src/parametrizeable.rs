use proc_macro::TokenStream;
use quote::{format_ident, quote};
use rtx_core::common::def_parser::parse_prototype;
use syn::{DeriveInput, Lit, Meta};

/// For now this prototype compilation technique is tied tightly to the `TypedMacroWO!` macro from rtx_package
/// until we can figure out how to improve the code organization.
pub fn compile_prototype_for_typed_macro(input: DeriveInput) -> TokenStream {
  let prototype: String = match input.attrs[0].parse_meta().unwrap() {
    Meta::NameValue(v) => match v.lit {
      Lit::Str(v) => v.value(),
      _ => panic!("only accepts #[prototype = \"value\"] attribute syntax, mandatory double-quotes (Lit)"),
    },
    _ => panic!("only accepts #[prototype = \"value\"] attribute syntax, mandatory double-quotes (parse_meta)"),
  };

  if prototype.is_empty() {
    quote!(()).into()
  } else {
    match parse_prototype(&prototype, None) {
      Ok((cs, params_opt)) => {
        let csname = cs.get_cs_name();
        let proto_types: Vec<_> = params_opt
          .unwrap_or_default()
          .get_parameters()
          .iter()
          .map(|p| format_ident!("{}", p.name))
          .collect();

        quote!(
          macro_rules! this_prototype {
          (sub [ $gullet:ident, ( $($var:ident),+ ), $inner_state:ident ] $body:block $($input:tt)*) => {
            TypedMacroWO!(T_CS(#csname) #(#proto_types)*, sub [ $gullet, ( $($var),+ ), $inner_state ] $body $($input)*)
          }
        }
        )
        .into()
      },
      Err(e) => panic!("Failed to compile binding prototype: {prototype}\n Reason: {e}"),
    }
  }
}


pub fn compile_prototype(input: DeriveInput) -> TokenStream {
  let prototype: String = match input.attrs[0].parse_meta().unwrap() {
    Meta::NameValue(v) => match v.lit {
      Lit::Str(v) => v.value(),
      _ => panic!("only accepts #[prototype = \"value\"] attribute syntax, mandatory double-quotes (Lit)"),
    },
    _ => panic!("only accepts #[prototype = \"value\"] attribute syntax, mandatory double-quotes (parse_meta)"),
  };
  if prototype.is_empty() {
    panic!("Must never call on empty prototype?! input was {prototype}");
  } else {
    match parse_prototype(&prototype, None) {
      Ok((cs, params_opt)) => {
        match params_opt {
          Some(params) => quote!(
            macro_rules! this_cs_and_parameters {
              () => { (#cs, Some(#params)) };
            }).into(),
          None => quote!(
            macro_rules! this_cs_and_parameters {
              () => { (#cs, None) };
            }).into()
        }
      },
      Err(e) => panic!("Failed to compile binding prototype: {prototype}\n Reason: {e}"),
    }
  }

}