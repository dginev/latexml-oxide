use latexml_core::common::arena;
use latexml_core::common::def_parser::parse_prototype;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

/// For now this prototype compilation technique is tied tightly to the `TypedMacroWO!` macro from
/// latexml_package until we can figure out how to improve the code organization.
pub fn compile_prototype_for(input: DeriveInput) -> TokenStream {
  let prototype = crate::attr_name_value_str(&input.attrs[0], "prototype");
  let inner = crate::attr_name_value_str(&input.attrs[1], "macro");

  if prototype.is_empty() {
    quote!(()).into()
  } else {
    match parse_prototype(&prototype, false) {
      Ok((cs, params_opt)) => {
        let csname = cs.with_cs_name(ToString::to_string);
        let proto_types: Vec<_> = if let Some(ref params) = params_opt {
          // if there is an *inner* parameter, as with {Number}
          // the name we want to pass in is the inner one. Otherwise the main one.
          params
            .get_parameters()
            .iter()
            .filter(|p| !arena::with(p.name, |name| name.starts_with("Skip")))
            .map(|p| {
              if let Some(ref inner_p) = p.inner {
                if let Some(first_inner) = inner_p.get_parameters().first() {
                  arena::with(first_inner.name, |name| format_ident!("{name}"))
                } else {
                  arena::with(p.name, |name| format_ident!("{name}"))
                }
              } else {
                arena::with(p.name, |name| format_ident!("{name}"))
              }
            })
            .collect()
        } else {
          Vec::new()
        };
        let quoted_params = if let Some(params) = params_opt {
          quote!(Some(#params.init()?))
        } else {
          quote!(None)
        };
        let inneri = format_ident!("{}", inner);
        quote!(
          macro_rules! this_prototype {
          (sub [( $($var:ident),* )]
            $body:block $($input:tt)*) => {
            let these_parameters = #quoted_params;
            #inneri!(
              #csname, these_parameters, sub [( $($var),* ):(#(#proto_types),*)] $body $($input)*)
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
  let prototype = crate::attr_name_value_str(&input.attrs[0], "prototype");
  if prototype.is_empty() {
    panic!("Must never call on empty prototype?! input was {prototype}");
  } else {
    match parse_prototype(&prototype, false) {
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
