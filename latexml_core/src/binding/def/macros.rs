// Macros requiring repetitions need to be handled outside of the main setup macro, as nested
// macros currently don't support repetition Details at:
// https://github.com/rust-lang/rust/issues/35853

/// build a Font from key=>val pairs
#[macro_export]
macro_rules! Font {
  ($($key:ident => $value:expr),*) => (
    Some(Font { $($key: $value.into_font_field(),)* .. Font::default() })
)}

/// build a FontDirective from key=>val pairs
/// (currently only FontDirective::Asset is supported in this macro)
#[macro_export]
macro_rules! FontDirective {
  ($($key:ident => $value:expr),*) => (
    Some(FontDirective::Asset(Rc::new(
      Font { $($key: $value.into_font_field(),)* .. Font::default() }
    ))))
}

/// given a struct `$name`, create a new instance of it using the given key=>val pairs
/// and complete the remaining entries via the Default instance
#[macro_export]
macro_rules! NewDefault {
  ($name:ident, $($key:ident => $value:expr),*) => ($name {
    $($key: $value,)*
    ..$name::default()
  })
}

/// Just like NewDefault, but adds a mandatory `.into_option()` to all values
#[macro_export]
macro_rules! NewDefaultV {
  ($name:ident, $($key:ident => $value:expr),*) => ($name {
    $($key: $value.into_option(),)*
    ..$name::default()
  })
}

// Useful shorthand macros, to brainstorm ergonomics ideas,
// and to aid binding development

/// Transfers a mutable pointer to a hashmap entry, or fills in with a default if missing.
///
/// Assumption: `$receiver` is HashMap<String,String>.
/// If, and only if, `$has_receiver` does not have a value at slot `$val`,
/// and `$struct_source` has a set value at `$val`,
/// then transfers (with ownership) the `$val` field of a `$struct_source` into the `$receiver`.
#[macro_export]
macro_rules! transfer_opt_default {
  ($val:ident, $struct_source:ident, $receiver:ident) => {
    if let Some(ref tval) = $struct_source.$val {
      $receiver
        .entry(stringify!($val).to_owned())
        .or_insert(tval.to_string());
    }
  };
}

// Discussion: Ideally we wouldn't need any of these closure macros, just the way latexml proper
// doesn't. In latexml, you could say:

#[macro_export]
macro_rules! before_digest {
  ($(sub)? $body:block) => {
    vec![before_digest_single!($body)]
  };
}

#[macro_export]
macro_rules! before_digest_single {
  ($body:block) => {
    Rc::new(move || $body.into_digested_result())
  };
}

#[macro_export]
macro_rules! before_digest_simple {
  ($body:block) => {
    Rc::new(move || $body.into_digested_result())
  };
}

#[macro_export]
macro_rules! tagsub {
  // 2-argument form: sub[document, node] { ... }
  ($document:ident, $node:ident, $body:block) => {
    vec![Rc::new(
      |$document: &mut Document, mut $node: &mut Node, _whatsit: Option<&Digested>| -> Result<()> {
        $body
        Ok(())
      },
    )]
  };
  // 3-argument form: sub[document, node, whatsit] { ... }
  // Matches Perl's ($document, $node, $box) signature for Tag afterClose/afterOpen
  ($document:ident, $node:ident, $whatsit:ident, $body:block) => {
    vec![Rc::new(
      |$document: &mut Document, mut $node: &mut Node, $whatsit: Option<&Digested>| -> Result<()> {
        $body
        Ok(())
      },
    )]
  };
}

#[macro_export]
macro_rules! sizersub {
  ($whatsit:ident, $body:block) => {
    Rc::new(
      |$whatsit: &Whatsit| -> Result<(Dimension, Dimension, Dimension)> {
        let macro_out = $body;
        macro_out
      },
    )
  };
}

#[macro_export]
macro_rules! rewrite_replace_sub {
  ($document:ident, $nodes:ident, $body:block) => {
  Some(Rc::new(
    |$document: &mut Document, mut $nodes: Vec<&mut Node>| -> Result<()> {
      $body
      Ok(())
    },
  ))
  };
}

#[macro_export]
macro_rules! noreplacement {
  () => {
    |doc, whatsit, props| Ok(())
  };
}

#[macro_export]
macro_rules! replacement {
  ($doc:ident, $args:ident, $props:ident, $body:block) => (
    move |$doc:&mut Document,$args: &Vec<Option<Digested>>,
      $props: &SymHashMap<Stored>| -> Result<()> {
    $body
    Ok(())
  })
}

#[macro_export]
macro_rules! construct {
  ($doc:ident, $whatsit:ident, $body:block) => {
  vec![Rc::new(
    move |$doc: &mut Document, $whatsit: &Whatsit| -> Result<()> {
      $body
      Ok(())
    }
  )]
}}

#[macro_export]
macro_rules! properties {
  (sub [$args:ident] $body:block) => {
    properties!($args, $body)
  };
  ($args:ident, $body:block) => {
    Rc::new(move |mut $args: &Vec<Option<Digested>>| -> Result<SymHashMap<Stored>> { $body })
  };
  ($(sub)? $body:block) => {
    Rc::new(
      move |_args: &Vec<Option<Digested>>| -> Result<SymHashMap<Stored>> {
        $body.into_properties_result()
      },
    )
  };
  ($value:expr) => {
    Rc::new(
      move |_args: &Vec<Option<Digested>>| -> Result<SymHashMap<Stored>> { Ok($value.clone()) },
    )
  };
}

#[macro_export]
macro_rules! after_digest {
  ($(sub)? $body:block) => {
    vec![after_digest_single!(_whatsit, $body)]
  };
  ($whatsit:ident, $body:block) => {
    vec![after_digest_single!($whatsit, $body)]
  };
}

#[macro_export]
macro_rules! after_digest_single {
  ($whatsit:ident, $body:block) => {
    Rc::new(move |$whatsit: &mut Whatsit| -> Result<Vec<Digested>> { $body.into_digested_result() })
  };
}
#[macro_export]
macro_rules! after_digest_simple {
  ($whatsit:ident, $body:block) => {
    Rc::new(move |$whatsit: &mut Whatsit| -> Result<Vec<Digested>> { $body.into_digested_result() })
  };
}

#[macro_export]
macro_rules! reader {
  ($inner:ident, $extra:ident, $body:block) => {
    Rc::new(
      |$inner: Option<&Parameters>, $extra: &[Tokens]| -> Result<ArgWrap> {
        $body.into_result_argwrap()
      },
    )
  };
}

#[macro_export]
macro_rules! predigest {
  ($arg:ident, $body:block) => {
    Some(Rc::new(
      |$arg: ArgWrap, _: &[Tokens]| -> Result<Option<Digested>> {
        $body.into_digested_option_result()
      },
    ))
  };
  ($arg:ident, $extra:ident, $body:block) => {
    Some(Rc::new(
      |$arg: ArgWrap, $extra: &[Tokens]| -> Result<Option<Digested>> {
        $body.into_digested_option_result()
      },
    ))
  };
}

/// A closure for obtaining a `RegisterValue`, usually owned by a `Register` getter.
#[macro_export]
macro_rules! getter {
  ($args: ident, $body:block) => {
    Some(Rc::new(
      move |mut $args: Vec<ArgWrap>| -> Option<RegisterValue> {
        $body.into_register_value_option()
      },
    ))
  };
}

#[macro_export]
macro_rules! setter {
  ($value:ident, $args: ident, $body:block) => {
    Some(Rc::new(
      move |$value: RegisterValue, _scope: Option<Scope>, mut $args: Vec<ArgWrap>| $body,
    ))
  };
  ($value:ident, $scope:ident, $args: ident, $body:block) => {
    Some(Rc::new(
      move |$value: RegisterValue, $scope: Option<Scope>, mut $args: Vec<ArgWrap>| $body,
    ))
  };
}

#[macro_export]
macro_rules! reversion {
  ($arg:ident, $inner:ident, $extra:ident, $body:block) => {
    Some(Rc::new(
      |mut $arg: Vec<Token>, $inner: Option<&Parameters>, $extra: &[Tokens]| -> Result<Tokens> {
        $body
      },
    ))
  };
}

#[macro_export]
macro_rules! reversion_digested {
  ($whatsit:ident, $args:ident, $body:block) => {
    Some(Reversion::Closure(Rc::new(
      move |$whatsit: &Whatsit, $args: &Vec<Option<Digested>>| -> Result<Tokens> { $body },
    )))
  };
}

// TODO: These .clone calls are silly... can we either
// 1) Document::insert_element work with a &Vec<Digested>? or
// 2) we can use mutable Whatsit properties in replacements, where we remove Vec<Digested> instances
// for cases that will be absorbed? or something else that is lighter on memory allocations?

#[macro_export]
macro_rules! prop_digested {
  ($props:ident, $key:expr) => {
    match $props.get($key) {
      Some(Stored::VecDigested(ref vd)) => vd.iter().collect::<Vec<&Digested>>(),
      Some(Stored::Digested(d)) => vec![&*d],
      Some(Stored::String(s)) => panic!(
        "prop_digested! called on a string property {:?} with value {:?}.",
        $key, s
      ),
      None => Vec::new(),
      other => {
        log::warn!(
          "Please extend the api_macros::prop_digested macro to support: {:?}",
          other
        );
        // Return empty vec instead of panicking
        Vec::new()
      },
    }
  };
}

// Discussion: It is unclear what the best authoring syntax is for our family of latexml binding
// macros. One idea is to keep them very close to the Rust internals, but we suffer from a variety
// of boilerplate, such as needing to spell out `key => Some(value.to_string())`, rather than a
// direct `key => value`.
//
// For now I am making the decision to keep writing out the verbose form,
// and will refactor at a later date, when the trade-offs become more clear. Smart use of the Cow
// struct is another idea. I will use a helper though:

#[macro_export]
macro_rules! prop_str {
  ($props:ident, $key:expr) => {
    match $props.get($key) {
      Some(&Stored::String(ref id)) => *id,
      _ => pin!(""),
    }
  };
}

#[macro_export]
macro_rules! prop_string {
  ($props:ident, $key:expr) => {
    match $props.get($key) {
      Some(&Stored::String(id)) => arena::to_string(id),
      _ => String::new(),
    }
  };
}

#[macro_export]
macro_rules! prop_whatsit {
  ($props:ident, $key:expr) => {
    match $props.get($key) {
      // Cloning here is OK now, as there is an Rc<> guard over the DigestedData
      Some(&Stored::Digested(ref rc)) => (**rc).clone(),
      _ => Digested::Whatsit(Rc::new(RefCell::new(Whatsit::default()))),
    }
  };
}

#[macro_export]
macro_rules! prop_bool {
  ($props:ident, $key:expr) => {
    match $props.get($key) {
      Some(&Stored::Bool(v)) => v,
      _ => false,
    }
  };
}

/// Convenience macro to flexibly unpack a collection of `Vec<ArgWrap>` arguments into individual
/// `Tokens` variables.
#[macro_export]
macro_rules! unref {
  ($args:ident => $var:ident) => (count_unpack_ref!(0usize, $args => $var));
  ($args:ident => $var:ident,$($tail:ident),*) => (
    count_unpack_ref!(0usize,$args => $var,$($tail),*))
}
#[macro_export]
macro_rules! count_unpack_ref {
  ($index:expr, $args:ident => $var:ident) => {
    let $var = $args[$index].as_ref().unwrap();
  };
  ($index:expr, $args:ident => $var:ident,$($tail:ident),*) => {
    count_unpack_ref!($index,$args => $var);
    count_unpack_ref!(1usize+$index, $args => $($tail),*)
  };
}

/// Try to efficiently unwrap a `Vec<T>` into a `[T;n]` for `$arg1`...`$argn`
#[macro_export]
macro_rules! unpack_opt {
  ($args:ident => $arg1:ident) => {
    let [$arg1]: [_; 1] = $args.try_into().unwrap();
  };
  ($args:ident => $arg1:ident,$arg2:ident) => {
    let [$arg1, $arg2]: [_; 2] = $args.try_into().unwrap();
  };
  ($args:ident => $arg1:ident,$arg2:ident,$arg3:ident) => {
    let [$arg1, $arg2, $arg3]: [_; 3] = $args.try_into().unwrap();
  };
  ($args:ident => $arg1:ident,$arg2:ident,$arg3:ident,$arg4:ident) => {
    let [$arg1, $arg2, $arg3, $arg4]: [_; 4] = $args.try_into().unwrap();
  };
  ($args:ident => $arg1:ident,$arg2:ident,$arg3:ident,$arg4:ident,$arg5:ident) => {
    let [$arg1, $arg2, $arg3, $arg4, $arg5]: [_; 5] = $args.try_into().unwrap();
  };
}

/// Try to efficiently unwrap a `&Vec<T>` into a `&[T;n]` for `$arg1`...`$argn`
#[macro_export]
macro_rules! unpack_opt_ref {
  ($args:ident => $arg1:ident) => {
    let [$arg1]: &[_; 1] = $args[..1].try_into().unwrap();
  };
  ($args:ident => $arg1:ident,$arg2:ident) => {
    let [$arg1, $arg2]: &[_; 2] = $args[..2].try_into().unwrap();
  };
  ($args:ident => $arg1:ident,$arg2:ident,$arg3:ident) => {
    let [$arg1, $arg2, $arg3]: &[_; 3] = $args[..3].try_into().unwrap();
  };
  ($args:ident => $arg1:ident,$arg2:ident,$arg3:ident,$arg4:ident) => {
    let [$arg1, $arg2, $arg3, $arg4]: &[_; 4] = $args[..4].try_into().unwrap();
  };
  ($args:ident => $arg1:ident,$arg2:ident,$arg3:ident,$arg4:ident,$arg5:ident) => {
    let [$arg1, $arg2, $arg3, $arg4, $arg5]: &[_; 5] = $args[..5].try_into().unwrap();
  };
}

/// Convert the number to lower case roman numerals, returning a list of LaTeXML::Core::Token
#[macro_export]
macro_rules! roman {
  ($stuff:expr) => {
    Tokens::new(ExplodeText!(roman_aux($stuff as i64)))
  };
}
/// Convert the number to upper case roman numerals, returning a list of LaTeXML::Core::Token
#[macro_export]
macro_rules! Roman {
  ($stuff:expr) => {
    Tokens::new(ExplodeText!(roman_aux($stuff as i64).to_ascii_uppercase()))
  };
}

#[macro_export]
macro_rules! requireMath {
  ($cs_name:expr) => {
    if !$crate::state::lookup_bool_sym($crate::pin!("IN_MATH")) {
      let message = s!("{} should only appear in math mode", $cs_name);
      Warn!("unexpected", "mode", message);
    }
  };
}
#[macro_export]
macro_rules! forbidMath {
  ($cs_name:expr) => {
    if $crate::state::lookup_bool_sym($crate::pin!("IN_MATH")) {
      let message = s!("{} should not appear in math mode", $cs_name);
      Warn!("unexpected", "mode", message);
    }
  };
}

#[macro_export]
macro_rules! AssignRegister {
  ($cs:literal, $value:expr) => {
    AssignRegister!($cs, $value, Vec::new())
  };
  ($cs:literal, $value:expr, $args:expr) => {
    let value_ident = { $value };
    if let Some(defn) = state::lookup_register_definition(&T_CS!($cs)) {
      (*defn).set_value(value_ident, None, $args);
    } else {
      let message = s!("The control sequence {} is not a register", $cs);
      Warn!("expected", "register", message);
    }
  };
}

#[macro_export]
macro_rules! SetCounter {
  ($ctr:expr => $value:expr) => {
    SetCounter!($ctr, $value)
  };
  ($ctr:expr, $value:expr) => {
    state::assign_register(
      &s!("\\c@{}", $ctr),
      $value.into(),
      Some(Scope::Global),
      Vec::new(),
    )?;
    after_assignment();
    def_macro(
      T_CS!(s!("\\@{}@ID", $ctr)),
      None,
      Tokens::new(Explode!($value.value_of())),
      Some(ExpandableOptions {
        scope: Some(Scope::Global),
        ..ExpandableOptions::default()
      }),
    )?;
  };
}
