use rtx_package::*;

#[macro_export]
macro_rules! TypedMacro {
  // closure
  ($cs:literal $($var:ident : $ptype:ident),+ => sub [ $gullet:ident, $inner_state:ident ] $body:block $($input:tt)*) => {{

    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    let mut parameters = Vec::new();
    $(
    parameters.push(
        Parameter {
          name: stringify!($ptype).to_string(),
          ..Parameter::default()
        }
        .init($inner_state)?,
      );
    )+
    let expansion_closure: Option<ExpansionBody> = Some(ExpansionBody::Closure(Arc::new(
      move |$gullet: &mut Gullet, mut args: Vec<ArgWrap>, $inner_state:&mut State| {
        $(
          let $var: $ptype = match args.remove(0).try_into() {
            Ok(v) => v,
            Err(e) => {
              use rtx_core::Error;
              Error!("expected", "argument", $gullet, None, e);
              $ptype::default()
            }
          };
        )+
        WithInnerState!($body, $inner_state).into_tokens_result()
      }
    )));
    let params = if parameters.is_empty() {
      None
    } else {
      Some(Parameters::new(parameters))
    };
    defi_macro!(T_CS!($cs), params, expansion_closure, Some(options));
  }};
}

LoadDefinitions!(state, {

  TypedMacro!("\\sampler" number:Number, token:Token, dimension:Dimension => sub[gullet,state] {
    dbg!(dbg!(number).value_of());
    dbg!(token);
    dbg!(dbg!(dimension).value_of());
    Tokens!()
  });

});
