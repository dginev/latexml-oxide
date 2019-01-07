#[macro_export]
macro_rules! map {
  ($( $key:expr => $val:expr ),*) => {{
    let mut map = ::std::collections::HashMap::new();
    $( map.insert($key.to_string(), $val); )*
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
