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

/// create a PrimitiveClosure from the pieces, with a forced empty return value
#[macro_export]
macro_rules! primitiveproc {
  ($stomach:ident, $args:ident, $inner_state:ident, $body:block) => (Rc::new(
    |$stomach:&mut Stomach, mut $args : Vec<ArgWrap>, $inner_state:&mut State| {
      BindInnerState!($stomach, $inner_state);
      $body
      end_state_frame!();
      Ok(Vec::new())
    }
  ))
}

#[macro_export]
macro_rules! before_digest {
  ($(sub)? $body:block) => {
    vec![before_digest_single!(stomach, state, $body)]
  };
  ($stomach:ident, $state:ident, $body:block) => {
    vec![before_digest_single!($stomach, $state, $body)]
  };
}

#[macro_export]
macro_rules! before_digest_single {
  ($stomach:ident, $state:ident, $body:block) => {
    Rc::new(move |$stomach: &mut Stomach, $state: &mut State| {
      BindInnerState!($stomach, $state);
      let macro_out = $body;
      end_state_frame!();
      macro_out.into_digested_result()
    })
  };
}

#[macro_export]
macro_rules! before_digest_simple {
  ($stomach:ident, $state:ident, $body:block) => {
    Rc::new(move |$stomach: &mut Stomach, $state: &mut State| {
      let macro_out = $body;
      macro_out.into_digested_result()
    })
  };
}

#[macro_export]
macro_rules! tagsub {
  ($document:ident, $node:ident, $state:ident, $body:block) => {
    vec![Rc::new(
      |$document: &mut Document, mut $node: &mut Node, $state: &mut State| -> Result<()> {
        BindInnerState!($state);
        $body
        end_state_frame!();
        Ok(())
      },
    )]
  };
}

#[macro_export]
macro_rules! sizersub {
  ($whatsit:ident, $state:ident, $body:block) => {
    Rc::new(
      |$whatsit: &Whatsit, $state: &mut State| -> Result<(Dimension, Dimension, Dimension)> {
        BindInnerState!($state);
        let macro_out = $body;
        end_state_frame!();
        macro_out
      },
    )
  };
}

#[macro_export]
macro_rules! rewrite_replace_sub {
  ($document:ident, $nodes:ident, $state:ident, $body:block) => {
  Some(Rc::new(
    |$document: &mut Document, mut $nodes: Vec<&mut Node>, $state: &mut State| -> Result<()> {
      BindInnerState!($state);
      $body
      end_state_frame!();
      Ok(())
    },
  ))
  };
}

#[macro_export]
macro_rules! noreplacement {
  () => {
    |doc, whatsit, props, state| Ok(())
  };
}

#[macro_export]
macro_rules! replacement {
  ($doc:ident, $args:ident, $props:ident, $state:ident, $body:block) => (
    move |$doc:&mut Document,$args: &Vec<Option<Digested>>,
      $props: &HashMap<String, Stored>, $state: &mut State| -> Result<()> {
    BindInnerState!($state);
    $body
    end_state_frame!();
    Ok(())
  })
}

#[macro_export]
macro_rules! construct {
  ($doc:ident, $whatsit:ident, $state:ident, $body:block) => {
  vec![Rc::new(
    move |$doc: &mut Document, $whatsit: &Whatsit, $state: &mut State| -> Result<()> {
      BindInnerState!($state);
      $body
      end_state_frame!();
      Ok(())
    }
  )]
}}

#[macro_export]
macro_rules! properties {
  (sub [ $stomach:ident, $args:ident, $inner_state:ident ] $body:block) => {
    properties!($stomach, $args, $inner_state, $body)
  };
  ($stomach:ident, $args:ident, $inner_state:ident, $body:block) => {
    Rc::new(
      move |$stomach: &mut Stomach,
            mut $args: &Vec<Option<Digested>>,
            $inner_state: &mut State|
            -> Result<HashMap<String, Stored>> {
        WithInnerState!($body, $stomach, $inner_state)
      },
    )
  };
  ($(sub)? $body:block) => {
    Rc::new(
      move |stomach: &mut Stomach,
            args: &Vec<Option<Digested>>,
            state: &mut State|
            -> Result<HashMap<String, Stored>> {
        WithInnerState!($body, stomach, state).into_properties_result()
      },
    )
  };
  ($value:expr) => {
    Rc::new(
      move |_stomach: &mut Stomach,
            _args: &Vec<Option<Digested>>,
            _state: &mut State|
            -> Result<HashMap<String, Stored>> { Ok($value.clone()) },
    )
  };
}

#[macro_export]
macro_rules! after_digest {
  ($(sub)? $body:block) => {
    vec![after_digest_single!(stomach, whatsit, state, $body)]
  };
  ($stomach:ident, $whatsit:ident, $state:ident, $body:block) => {
    vec![after_digest_single!($stomach, $whatsit, $state, $body)]
  };
}

#[macro_export]
macro_rules! after_digest_single {
  ($stomach:ident, $whatsit:ident, $state:ident, $body:block) => {
    Rc::new(
      move |$stomach: &mut Stomach,
            $whatsit: &mut Whatsit,
            $state: &mut State|
            -> Result<Vec<Digested>> {
        WithInnerState!($body, $stomach, $state).into_digested_result()
      },
    )
  };
}
#[macro_export]
macro_rules! after_digest_simple {
  ($stomach:ident, $whatsit:ident, $state:ident, $body:block) => {
    Rc::new(
      move |$stomach: &mut Stomach,
            $whatsit: &mut Whatsit,
            $state: &mut State|
            -> Result<Vec<Digested>> { $body.into_digested_result() },
    )
  };
}

#[macro_export]
macro_rules! reader {
  ($gullet:ident, $inner:ident, $extra:ident, $state:ident, $body:block) => {
    Rc::new(
      |$gullet: &mut Gullet,
       $inner: Option<&Parameters>,
       $extra: &[Tokens],
       $state: &mut State|
       -> Result<ArgWrap> { WithInnerState!($body, $state).into_result_argwrap() },
    )
  };
}

#[macro_export]
macro_rules! predigest {
  ($stomach:ident, $arg:ident, $state:ident, $body:block) => {
    Some(Rc::new(
      |$stomach: &mut Stomach, $arg: ArgWrap, $state: &mut State| -> Result<Option<Digested>> {
        WithInnerState!($body, $stomach, $state).into_digested_option_result()
      },
    ))
  };
}

/// A closure for obtaining a `RegisterValue`, usually owned by a `Register` getter.
#[macro_export]
macro_rules! getter {
  ($args: ident, $state:ident, $body:block) => {
    Some(Rc::new(
      move |mut $args: Vec<ArgWrap>, $state: &mut State| -> Option<RegisterValue> {
        WithInnerState!($body, $state).into_register_value_option()
      },
    ))
  };
}

#[macro_export]
macro_rules! setter {
  ($value:ident, $args: ident, $state:ident, $body:block) => {
    Some(Rc::new(
      move |$value: RegisterValue, _scope: Option<Scope>, mut $args: Vec<ArgWrap>, $state: &mut State| {
        WithInnerState!($body, $state)
      },
    ))
  };
  ($value:ident, $scope:ident, $args: ident, $state:ident, $body:block) => {
    Some(Rc::new(
      move |$value: RegisterValue, $scope: Option<Scope>, mut $args: Vec<ArgWrap>, $state: &mut State| {
        WithInnerState!($body, $state)
      },
    ))
  };
}

#[macro_export]
macro_rules! reversion {
  ($gullet:ident, $arg:ident, $inner:ident, $extra:ident, $state:ident, $body:block) => {
    Some(Rc::new(
      |mut $arg: Vec<Token>,
       $inner: Option<&Parameters>,
       $extra: &[Tokens],
       $state: &State|
       -> Result<Tokens> {
        BindInnerState!($state);
        let macro_out = $body;
        end_state_frame!();
        macro_out
      },
    ))
  };
}

#[macro_export]
macro_rules! reversion_digested {
  ($whatsit:ident, $args:ident, $state:ident, $body:block) => {
    Some(Reversion::Closure(Rc::new(
      move |$whatsit: &Whatsit, $args: &Vec<Option<Digested>>, $state: &State| -> Result<Tokens> {
        BindInnerState!($state);
        let macro_out = $body;
        end_state_frame!();
        macro_out
      },
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
        eprintln!(
          "Please extend the api_macros::prop_digested macro to support: {:?}",
          other
        );
        unimplemented!();
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
      _ => arena::pin_static(""),
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

/// Try to efficiently unwrap a Vec<T> into a [T;n] for `$arg1`...`$argn`
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

/// Try to efficiently unwrap a &Vec<T> into a &[T;n] for `$arg1`...`$argn`
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
  ($cs_name:expr) => {{
    bind_state_mut!(st);
    requireMath!($cs_name, st)
  }};
  ($cs_name:expr, $state_arg:ident) => {
    if !$state_arg.lookup_bool("IN_MATH") {
      let message = s!("{} should only appear in math mode", $cs_name);
      Warn!("unexpected", "mode", None, message);
    }
  };
}
#[macro_export]
macro_rules! forbidMath {
  ($cs_name:expr) => {{
    bind_state_mut!(st);
    forbidMath!($cs_name, st)
  }};
  ($cs_name:expr, $state_arg:ident) => {
    if $state_arg.lookup_bool("IN_MATH") {
      let message = s!("{} should not appear in math mode", $cs_name);
      Warn!("unexpected", "mode", None, message);
    }
  };
}

#[macro_export]
macro_rules! AssignRegister {
  ($cs:literal, $value:expr) => {{
    let value_ident = { $value };
    bind_state_mut!(stmch, st);
    AssignRegister!($cs, value_ident, Vec::new(), st);
  }};
  ($cs:literal, $value:ident, $args:expr, $state_arg: ident) => {{
    if let Some(defn) = $state_arg.lookup_register_definition(&T_CS!($cs)) {
      (*defn).set_value($value, None, $args, $state_arg);
    } else {
      let message = s!("The control sequence {} is not a register", $cs);
      Warn!("expected", "register", None, message);
    }
  }};
  ($cs:literal, $value:expr, $args:expr, $state_arg: ident) => {{
    let value_ident = { $value };
    AssignRegister!($cs, value_ident, $args, $state_arg);
  }};
}

#[macro_export]
macro_rules! SetCounter {
  ($ctr:expr, $value:expr) => {
    AssignValue!(&s!("\\c@{}",$ctr), $value, Some(Scope::Global));
    DefMacro!(T_CS!(s!("\\@{}@ID",$ctr)), None, Tokens::new(Explode!($value.value_of())),
                scope => Some(Scope::Global)
    );
  };
  ($ctr:expr, $value:expr) => {
    AssignValue!(&s!("\\c@{}",$ctr), $value, Some(Scope::Global));
    AfterAssignment!();
    DefMacro!(T_CS!(s!("\\@{}@ID",$ctr)), None, Tokens::new(Explode!($value.value_of())),
                scope => Some(Scope::Global)
    );
  };
  ($ctr:expr, $value:expr, $stomach:ident, $state_arg:ident) => {
    $state_arg.assign_value(&s!("\\c@{}",$ctr), $value, Some(Scope::Global));
    $state_arg.after_assignment($stomach.get_gullet_mut());
    def_macro(T_CS!(s!("\\@{}@ID",$ctr)), None,
      Tokens::new(Explode!($value.value_of())),
      Some(ExpandableOptions{ scope: Some(Scope::Global),
         ..ExpandableOptions::default()}), $state_arg)?;
  }
}
