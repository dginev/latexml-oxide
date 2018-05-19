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
    Some(Font { $($key: Some($value.to_string()),)* .. Font::default() })
)}

#[macro_export]
macro_rules! NewDefault {
  ($name:ident, $($key:ident => $value:expr),*) => ($name {
    $($key: $value,)*
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
      .or_insert($struct_source.$val.clone().to_string());
  };
}
#[macro_export]
macro_rules! transfer_opt_default {
  ($val:ident, $struct_source:ident, $hash_receiver:ident) => {
    if let &Some(ref $val) = &$struct_source.$val {
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
    |$doc:&mut Document,$args: &Vec<Option<Digested>>,$props: &HashMap<String, ObjectStore>, $state: &mut State| -> Result<()> {
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
  ($stomach:ident, $args:ident, $state:ident, $body:expr) => {
    |$stomach: &mut Stomach, mut $args: Vec<Tokens>, $state: &mut State| $body
  };
}
#[macro_export]
macro_rules! primitiveproc {
  ($stomach:ident, $args:ident, $state:ident, $body:expr) => (
    |$stomach:&mut Stomach, mut $args : Vec<Tokens>, $state:&mut State| {
      $body
      Ok(Vec::new())
    }
  )
}

#[macro_export]
macro_rules! beforesub {
  ($stomach:ident, $state:ident, $body:expr) => {
    |$stomach: &mut Stomach, $state: &mut State| $body
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
macro_rules! aftersub {
  ($stomach:ident, $whatsit:ident, $state:ident, $body:expr) => {
    |$stomach: &mut Stomach, $whatsit: &mut Whatsit, $state: &mut State| $body
  };
}
#[macro_export]
macro_rules! afterproc {
  ($stomach:ident, $whatsit:ident, $state:ident, $body:expr) => (
    Rc::new(move |$stomach:&mut Stomach, $whatsit:&mut Whatsit, $state:&mut State| {
      $body
      Ok(Vec::new())
    }
  ))
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
macro_rules! v {
  ($val:expr) => {
    Some($val.to_string())
  };
}

#[macro_export]
macro_rules! prop_digested {
  ($props: ident, $key: expr) =>(
    match $props.get($key) {
        Some(ObjectStore::VecDigested(vd)) => vd.clone(),
        Some(ObjectStore::Digested(d)) => vec![(**d).clone()],
        _ => Vec::new()
    }
  )
}

#[macro_export]
macro_rules! prop_str {
  ($props: ident, $key: expr) =>(
    match $props.get($key) {
      Some(& ObjectStore::String(ref id)) => id,
      _ => ""
    }
  )
}


#[macro_export]
macro_rules! prop_string {
  ($props: ident, $key: expr) =>(
    match $props.get($key) {
      Some(& ObjectStore::String(ref id)) => id.to_string(),
      _ => String::new()
    }
  )
}

#[macro_export]
macro_rules! prop_whatsit {
  ($props: ident, $key: expr) =>(
    match $props.get($key) {
      // TODO: Cloning here ought to be terribly inefficient and should be avoided. How?
      Some(& ObjectStore::Digested(ref rc)) => (**rc).clone(),
      _ => Digested::Whatsit(Whatsit::default())
    };
  )
}

#[macro_export]
macro_rules! prop_bool {
  ($props: ident, $key: expr) =>(
    match $props.get($key) {
      Some(& ObjectStore::Bool(v)) => v,
      _ => false
    }
  )
}

#[macro_export]
macro_rules! unpack_to_string {
  ($args:ident => $var:ident) => (count_unpack_to_string!(0usize, $args => $var));
  ($args:ident => $($var:ident),*) => (count_unpack_to_string!(0usize, $args => $($var),*));
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
macro_rules! unpack {
  ($args:ident => $var:ident) => (count_unpack!(0usize, $args => $var));
  ($args:ident => $($var:ident),*) => (count_unpack!(0usize, $args => $($var),*));
}

#[macro_export]
macro_rules! count_unpack {
  ($index:expr, $args:ident => $var:ident) => (
    let $var = $args[$index].clone().unwrap_or_default();
  );
  ($index:expr, $args:ident => $var:ident,$($tail:ident),*) => {
    count_unpack!($index,$args => $var);
    count_unpack!(1usize+$index, $args => $($tail),*)
  }
}