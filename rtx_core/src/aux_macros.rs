#[macro_export]
macro_rules! println_stderr(
    ($($arg:tt)*) => ({
      use std::io::Write;
      match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
        Ok(_) => {},
        Err(x) => panic!("Unable to write to stderr: {}", x),
      }
    })
);

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