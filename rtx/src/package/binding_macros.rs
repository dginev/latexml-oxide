/// Tokenize($string); Tokenizes the string using the standard cattable, returning a
/// LaTeXML::Core::Tokens
#[macro_export]
macro_rules! Tokenize {
  ($string:expr) => {
    mouth::tokenize($string, None)
  };
  ($string:expr, $state:ident) => {
    mouth::tokenize($string, Some($state))
  };
}

/// TokenizeInternal($string); Tokenizes the string using the internal cattable, returning a
/// LaTeXML::Core::Tokens
#[macro_export]
macro_rules! TokenizeInternal {
  ($string:expr) => {
    mouth::tokenize_internal($string, None)
  };
  ($string:expr, $state:ident) => {
    mouth::tokenize_internal($string, Some($state))
  };
}

// Macros requiring repetitions need to be handled outside of the main setup macro, as nested
// macros currently don't support repetition Details at: https://github.com/rust-lang/rust/issues/35853
#[macro_export]
macro_rules! Font {
  ($($key:ident => $value:expr),*) => (
    Some(Font { $($key: Some($value.to_string().into()),)* .. Font::default() })
)}

#[macro_export]
macro_rules! NewDefault {
  ($name:ident, $($key:ident => $value:expr),*) => ($name {
    $($key: $value,)*
    ..$name::default()
  })
}

#[macro_export]
macro_rules! NewDefaultV {
  ($name:ident, $($key:ident => $value:expr),*) => ($name {
    $($key: $value.into_option(),)*
    ..$name::default()
  })
}

// Useful shorthand macros, to brainstorm ergonomics ideas,
// and to aid binding development

#[macro_export]
macro_rules! transfer_default {
  ($val:ident, $struct_source:ident, $hash_receiver:ident) => {
    $hash_receiver
      .entry(stringify!($val).to_owned())
      .or_insert($struct_source.$val.to_string());
  };
}
#[macro_export]
macro_rules! transfer_opt_default {
  ($val:ident, $struct_source:ident, $hash_receiver:ident) => {
    if let Some(ref $val) = $struct_source.$val {
      $hash_receiver
        .entry(stringify!($val).to_owned())
        .or_insert($val.to_owned());
    }
  };
}

#[macro_export]
macro_rules! sub {
  ($body:expr) => {
    vec![Rc::new($body)]
  };
}

#[macro_export]
macro_rules! tagsub {
  ($document:ident, $node:ident, $state:ident, $body:expr) => {
    vec![Rc::new(
      |$document: &mut Document, mut $node: &mut Node, $state: &mut State| -> Result<()> {
        $body;
        Ok(())
      },
    )]
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
  ($doc:ident, $args:ident, $props:ident, $state:ident, $body:expr) => (
    |$doc:&mut Document,$args: &Vec<Option<Digested>>,$props: &HashMap<String, Stored>, $state: &mut State| -> Result<()> {
    $body
    Ok(())
  })
}

#[macro_export]
macro_rules! noprimitive {
  () => {
    |stomach: &mut Stomach, args: Vec<Tokens>, state: &mut State| Ok(Vec::new())
  };
}

#[macro_export]
macro_rules! primitivesub {
  ($stomach:ident, $args:ident, $state:ident, $body:block) => {
    |$stomach: &mut Stomach, mut $args: Vec<Tokens>, $state: &mut State| $body
  };
}
#[macro_export]
macro_rules! primitiveproc {
  ($stomach:ident, $args:ident, $state:ident, $body:block) => (
    |$stomach:&mut Stomach, mut $args : Vec<Tokens>, $state:&mut State| {
      $body
      Ok(Vec::new())
    }
  )
}

#[macro_export]
macro_rules! beforesub {
  ($stomach:ident, $state:ident, $body:block) => {
    vec![Rc::new(|$stomach: &mut Stomach, $state: &mut State| $body)]
  };
}
#[macro_export]
macro_rules! beforeproc {
  // just as beforesub! but with a default return value
  ($stomach:ident, $state:ident, $body:expr) => {
    Rc::new(move |$stomach: &mut Stomach, $state: &mut State| {
      $body;
      Ok(Vec::new())
    })
  };
}

#[macro_export]
macro_rules! properties {
  (sub [ $stomach:ident, $args:ident, $inner_state:ident ] $body:block) => {
    Rc::new(
      move |$stomach: &mut Stomach,
            mut $args: &Vec<Option<Digested>>,
            $inner_state: &mut State|
            -> Result<HashMap<String, Stored>> { $body },
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
macro_rules! aftersub {
  ($stomach:ident, $whatsit:ident, $state:ident, $body:expr) => {
    vec![Rc::new(
      move |$stomach: &mut Stomach,
            $whatsit: &mut Whatsit,
            $state: &mut State|
            -> Result<Vec<Digested>> { $body },
    )]
  };
}

#[macro_export]
macro_rules! afterproc {
  ($stomach:ident, $whatsit:ident, $state:ident, $body:expr) => (
    vec![Rc::new(move |$stomach:&mut Stomach, $whatsit:&mut Whatsit, $state:&mut State| -> Result<Vec<Digested>> {
      $body
      Ok(Vec::new())
    }
  )])
}

#[macro_export]
macro_rules! reader {
  ($gullet:ident, $inner:ident, $extra:ident, $state:ident, $body:block) => {
    Rc::new(
      |$gullet: &mut Gullet,
       $inner: Vec<Option<Parameters>>,
       $extra: Vec<ParameterExtra>,
       $state: &mut State|
       -> Result<Tokens> { $body },
    )
  };
}

#[macro_export]
macro_rules! reversion {
  ($gullet:ident, $arg:ident, $inner:ident, $state:ident, $body:block) => {
    Some(Rc::new(
      |$gullet: &mut Gullet,
       mut $arg: Vec<Token>,
       $inner: Vec<ParameterExtra>,
       $state: &mut State|
       -> Result<Tokens> { $body },
    ))
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
macro_rules! prop_digested {
  ($props:ident, $key:expr) => {
    match $props.get($key) {
      Some(Stored::VecDigested(vd)) => vd.clone(),
      Some(Stored::Digested(d)) => vec![(**d).clone()],
      Some(Stored::String(s)) => vec![s.into()],
      _ => Vec::new(),
    }
  };
}

#[macro_export]
macro_rules! prop_str {
  ($props:ident, $key:expr) => {
    match $props.get($key) {
      Some(&Stored::String(ref id)) => id,
      _ => "",
    }
  };
}

#[macro_export]
macro_rules! prop_string {
  ($props:ident, $key:expr) => {
    match $props.get($key) {
      Some(&Stored::String(ref id)) => id.to_string(),
      _ => String::new(),
    }
  };
}

#[macro_export]
macro_rules! prop_whatsit {
  ($props:ident, $key:expr) => {
    match $props.get($key) {
      // TODO: Cloning here ought to be terribly inefficient and should be avoided. How?
      Some(&Stored::Digested(ref rc)) => (**rc).clone(),
      _ => Digested::Whatsit(Rc::new(RefCell::new(Whatsit::default()))),
    };
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

#[macro_export]
macro_rules! unpack_to_string {
  ($args:ident => $var:ident) => (count_unpack_to_string!(0usize, $args => $var));
  ($args:ident => $($var:ident),*) => (count_unpack_to_string!(0usize, $args => $($var),*));
}

#[macro_export]
macro_rules! unpack_to_token {
  ($args:ident => $var:ident) => (count_unpack_to_token!(0usize, $args => $var));
  ($args:ident => $($var:ident),*) => (count_unpack_to_token!(0usize, $args => $($var),*));
}

#[macro_export]
macro_rules! count_unpack_to_string {
  ($index:expr, $args:ident => $var:ident) => (
    let $var = $args[$index].clone().unwrap_or_default().to_string();
  );
  ($index:expr, $args:ident => $var:ident,$($tail:ident),*) => {
    count_unpack_to_string!($index,$args => $var);
    count_unpack_to_string!(1usize+$index, $args => $($tail),*)
  }
}

#[macro_export]
macro_rules! count_unpack_to_token {
  ($index:expr, $args:ident => $var:ident) => (
    let tmp_tks : Tokens = $args[$index].clone().unwrap_or_default();
    let $var : Token = tmp_tks.into();
  );
  ($index:expr, $args:ident => $var:ident,$($tail:ident),*) => {
    count_unpack_to_token!($index,$args => $var);
    count_unpack_to_token!(1usize+$index, $args => $($tail),*)
  }
}

#[macro_export]
macro_rules! unpack {
  ($args:ident => $var:ident) => (count_unpack!(0usize, $args => $var));
  ($args:ident => $($var:ident),*) => (count_unpack!(0usize, $args => $($var),*));
}

#[macro_export]
macro_rules! count_unpack {
  ($index:expr, $args:ident => $var:ident) => (
    let mut $var = $args[$index].clone().unwrap_or_default();
  );
  ($index:expr, $args:ident => $var:ident,$($tail:ident),*) => {
    count_unpack!($index,$args => $var);
    count_unpack!(1usize+$index, $args => $($tail),*)
  }
}
