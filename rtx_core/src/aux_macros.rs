#[macro_export]
macro_rules! map {
  ($( $key:expr => $val:expr ),*) => {{
    let mut map = ::std::collections::HashMap::new();
    $( map.insert($key, $val); )*
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