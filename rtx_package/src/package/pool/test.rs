use crate::package::*;

#[macro_export]
macro_rules! TypedMacro {
  // closure
  ($cs:literal [ $($var:ident : $ptype:ident),+ ] => sub [ $gullet:ident, $inner_state:ident ] $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    let mut parameters = Vec::new();
    $(
    parameters.push(
        Parameter {
          name: stringify!($ptype).to_string(),
          ..Parameter::default()
        }
        .init(state)?,
      );
    )+
    // let (cs, params) = parse_prototype!($proto);
    let expansion_closure: Option<ExpansionBody> = Some(ExpansionBody::Closure(Arc::new(
      move |$gullet, mut args, $inner_state| {
        $(
          let $var: $ptype = args.remove(0).into();
        )+
        WithInnerState!($body, $inner_state).into_tokens_result()
      }
    )));
    // defi_macro!(cs, params, expansion_closure, Some(options));
  }};
}

// LoadDefinitions!(state, {

//   TypedMacro!("\\sampler"[number:OptionalNumber, token:OptionalToken, dimension:OptionalDimension] => sub[gullet,state] {
//     dbg!(dbg!(number).unwrap().value_of());
//     dbg!(token);
//     dbg!(dbg!(dimension).unwrap().value_of());
//     Ok(Vec::new())
//   });

// });
