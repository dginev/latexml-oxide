#[macro_export]
macro_rules! static_map {
  ($( $key:expr => $val:expr ),*) => {{
    let mut map : HashMap<&'static str, &'static str> = ::std::collections::HashMap::new();
    $( map.insert($key, $val); )*
    map
  }}
}

#[macro_export]
macro_rules! map {
  ($( $key:expr => $val:expr ),*) => {{
    let mut map = ::std::collections::HashMap::new();
    $( map.insert($key.to_string(), $val); )*
    map
  }}
}

#[macro_export]
macro_rules! stored_map {
  ($( $key:expr => $val:expr ),*) => {{
    let mut map : ::std::collections::HashMap<String,Stored> = ::std::collections::HashMap::new();
    $( map.insert($key.to_string(), $val.into()); )*
    map
  }}
}

#[macro_export]
macro_rules! string_map {
  ($( $key:expr => $val:expr ),*) => {{
    let mut map = ::std::collections::HashMap::new();
    $( map.insert($key.to_string(), $val.to_string()); )*
    map
  }}
}

#[macro_export]
macro_rules! raw_map {
  ($( $key:expr => $val:expr ),*) => {{
    let mut map = ::std::collections::HashMap::new();
    $( map.insert($key, $val); )*
    map
  }}
}

#[macro_export]
macro_rules! raw_char_map {
  ($( $key:literal => $val:expr ),*) => {{
    let mut map : HashMap<char,_> = ::std::collections::HashMap::new();
    $( map.insert($key, $val); )*
    map
  }}
}

#[macro_export]
macro_rules! s {
  ($($arg : tt )*) => (format!($($arg)*))
}

#[macro_export]
macro_rules! mixvec {
  ($( $val:expr ),*) => {{
    vec![ $($val.into()),*]
  }}
}

#[macro_export]
macro_rules! fontmap {
  ($($key:ident => $value:expr),*) => (
    Font { $($key: Some($value.into()),)* .. Font::default() })
}

// Simple helper for hashset creation
// Source: https://riptutorial.com/rust/example/4149/create-a-hashset-macro
#[macro_export]
macro_rules! set {
    ( $( $x:expr ),* ) => {  // Match zero or more comma delimited items
        {
            use std::collections::HashSet;
            let mut temp_set = HashSet::new();  // Create a mutable HashSet
            $(
                temp_set.insert($x); // Insert each item matched into the HashSet
            )*
            temp_set // Return the populated HashSet
        }
    };
}
