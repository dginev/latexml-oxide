//! Helper macros for quicker and more idiomatic construction and access to data structures.

/// A flexary macro for constructing `HashMap<&'static str, &'static str>` maps
#[macro_export]
macro_rules! static_map {
  ($( $key:expr => $val:expr ),*) => {{
    let mut map : HashMap<&'static str, &'static str> = HashMap::default();
    $( map.insert($key, $val); )*
    map
  }}
}

/// A flexary macro for constructing `HashMap<String, T>` maps, where `T` is generic.
#[macro_export]
macro_rules! map {
  ($( $key:expr => $val:expr ),*) => {{
    let mut map = HashMap::default();
    $( map.insert($key.to_string(), $val); )*
    map
  }}
}

/// A flexary macro for constructing `SymHashMap<Stored>` maps
#[macro_export]
macro_rules! stored_map {
  ($( $key:expr => $val:expr ),*) => {{
    let mut map : $crate::common::arena::SymHashMap<$crate::common::store::Stored> =
      $crate::common::arena::SymHashMap::default();
    $( map.insert($key, $val.into()); )*
    map
  }}
}

/// A flexary macro for constructing `SymHashMap<T>` maps
#[macro_export]
macro_rules! sym_map {
  ($( $key:expr => $val:expr ),*) => {{
    let mut map = $crate::common::arena::SymHashMap::default();
    $( map.insert($key, $val); )*
    map
  }}
}

/// A flexary macro for constructing `HashMap<String, T>` maps
#[macro_export]
macro_rules! string_keys_map {
  ($( $key:expr => $val:expr ),*) => {{
    let mut map = HashMap::default();
    $( map.insert($key.to_string(), $val.into()); )*
    map
  }}
}

/// A flexary macro for constructing `HashMap<String, String>` maps
#[macro_export]
macro_rules! string_map {
  ($( $key:expr => $val:expr ),*) => {{
    let mut map = HashMap::default();
    $( map.insert($key.to_string(), $val.to_string()); )*
    map
  }}
}

/// A flexary macro for constructing `HashMap<K, V>` maps, where `K` and `V` are both generic
/// (inferred at time of use)
#[macro_export]
macro_rules! raw_map {
  ($( $key:expr => $val:expr ),* $(,)?) => {{
    #[allow(unused_mut)]
    let mut map = HashMap::default();
    $( map.insert($key, $val); )*
    map
  }}
}

/// A flexary macro for constructing `HashMap<char, T>` maps, where `T` is generic
#[macro_export]
macro_rules! raw_char_map {
  ($( $key:literal => $val:expr ),*) => {{
    let mut map : HashMap<char,_> = HashMap::default();
    $( map.insert($key, $val); )*
    map
  }}
}

/// The `s!` macro is a briefer alias for `format!`
#[macro_export]
macro_rules! s {
  ($($arg : tt )*) => {format!($($arg)*)};
}

/// The `some!` macro transforms data in type `S` to `Option<Into<T>>` (always wrapping with `Some`)
#[macro_export]
macro_rules! some {
  ($arg:expr) => {
    Some($arg.into())
  };
}

/// A variant on `vec!` where each argument receives an additional `.into()` call
/// best used with an outer context that explicitly provides the expected type, such as
/// ```
/// # use std::rc::Rc;
/// # use latexml_core::common::store::Stored;
/// # use latexml_core::mixvec;
/// let stored_vec : Vec<Stored> = mixvec!(1, true, "string");
/// ```
#[macro_export]
macro_rules! mixvec {
  ($( $val:expr ),*) => {{
    vec![ $($val.into()),*]
  }}
}

/// A variant on `mixvec!` where each argument receives an additional `.into()` call
/// best used with an outer context that explicitly provides the expected type, such as
/// ```
/// # use std::rc::Rc;
/// # use latexml_core::common::store::Stored;
/// # use latexml_core::mixrc;
/// let stored_vec : Rc<[Stored]> = mixrc!(1, true, "string");
/// ```
#[macro_export]
macro_rules! mixrc {
  ($( $val:expr ),*) => {{
    Rc::new([ $($val.into()),*])
  }}
}

/// Instantiates a `Font`, using the `Font` fields as keys, and calling `.into()` for each value.
/// The specification can be partial - missing fields are taken via the `Default` trait.
#[macro_export]
macro_rules! fontmap {
  ($($key:ident => $value:expr),*) => (
    Font { $($key: Some($value.into()),)* .. Font::default() })
}

/// Simple generic helper for `HashSet<K,V>` creation
/// Source: <https://riptutorial.com/rust/example/4149/create-a-hashset-macro>
#[macro_export]
macro_rules! set {
    ( $( $x:expr ),* ) => {  // Match zero or more comma delimited items
        {
            // use rustc_hash::FxHashSet as HashSet;
            use rustc_hash::FxHashSet as HashSet;
            let mut temp_set = HashSet::default();  // Create a mutable HashSet
            $(
                temp_set.insert($x); // Insert each item matched into the HashSet
            )*
            temp_set // Return the populated HashSet
        }
    };
}
