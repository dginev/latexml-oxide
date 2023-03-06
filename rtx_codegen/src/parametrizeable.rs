use proc_macro::TokenStream;
use quote::{format_ident, quote};
use rtx_core::common::def_parser::parse_prototype;
use syn::{DeriveInput, Lit, Meta};

/// For now this prototype compilation technique is tied tightly to the `TypedMacroWO!` macro from rtx_package
/// until we can figure out how to improve the code organization.
pub fn compile_prototype_for(input: DeriveInput) -> TokenStream {
  let prototype: String = match input.attrs[0].parse_meta().unwrap() {
    Meta::NameValue(v) => match v.lit {
      Lit::Str(v) => v.value(),
      _ => panic!("only accepts #[prototype = \"value\"] attribute syntax, mandatory double-quotes (Lit)"),
    },
    _ => panic!("only accepts #[prototype = \"value\"] attribute syntax, mandatory double-quotes (parse_meta)"),
  };
  let inner: String = match input.attrs[1].parse_meta().unwrap() {
    Meta::NameValue(v) => match v.lit {
      Lit::Str(v) => v.value(),
      _ => panic!("only accepts #[macro = \"TypedMacro\"] attribute syntax, mandatory double-quotes (Lit)"),
    },
    _ => panic!("only accepts #[macro = \"TypedMacro\"] attribute syntax, mandatory double-quotes (parse_meta)"),
  };

  if prototype.is_empty() {
    quote!(()).into()
  } else {
    match parse_prototype(&prototype, None) {
      Ok((cs, params_opt)) => {
        let csname = cs.get_cs_name();
        let proto_types: Vec<_> = if let Some(ref params) = params_opt {
          // if there is an *inner* parameter, as with {Number}
          // the name we want to pass in is the inner one. Otherwise the main one.
          params
            .get_parameters()
            .iter()
            .filter(|p| !p.name.starts_with("Skip"))
            .map(|p| {
              if let Some(ref inner_p) = p.inner {
                if let Some(first_inner) = inner_p.get_parameters().first() {
                  format_ident!("{}", first_inner.name)
                } else {
                  format_ident!("{}", p.name)
                }
              } else {
                format_ident!("{}", p.name)
              }
            })
            .collect()
        } else {
          Vec::new()
        };
        let quoted_params = if let Some(params) = params_opt {
          quote!(Some(#params.init(outer_state!())?))
        } else {
          quote!(None)
        };
        let inneri = format_ident!("{}", inner);
        quote!(
          macro_rules! this_prototype {
          (sub [ $gullet:ident, ( $($var:ident),* ), $inner_state:ident ] $body:block $($input:tt)*) => {
            let these_parameters = #quoted_params;
            #inneri!(#csname, these_parameters, sub [ $gullet, ( $($var),* ):(#(#proto_types),*), $inner_state ] $body $($input)*)
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
      Ok((cs, params_opt)) => match params_opt {
        Some(params) => quote!(
          macro_rules! this_cs_and_parameters {
              () => { (#cs, Some(#params)) };
            }
        )
        .into(),
        None => quote!(
          macro_rules! this_cs_and_parameters {
              () => { (#cs, None) };
            }
        )
        .into(),
      },
      Err(e) => panic!("Failed to compile binding prototype: {prototype}\n Reason: {e}"),
    }
  }
}
