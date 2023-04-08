use string_interner::{StringInterner};
use string_interner::symbol::SymbolU32;
use string_interner::backend::StringBackend;
use once_cell::sync::Lazy;
// use std::sync::Mutex;

static mut T : Lazy<StringInterner<StringBackend>> = Lazy::new(|| {
  let mut interner = StringInterner::with_capacity(10_000);
  interner.extend([
    " ", "!", "\"", "#", "$", "%", "&", "\"", "(", ")", "*", "+", ",", "-", ".", "/",
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ":", ";", "<", "=", ">", "?", "@",
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z", "[", "\\", "]", "^", "_", "`", "a", "b", "c", "d", "e",
    "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v",
    "w", "x", "y", "z", "{", "|", "}"].iter());
  interner
});

pub fn pin<S:AsRef<str>>(text:S) -> SymbolU32 {
  unsafe { T.get_or_intern(text) }
}

pub fn resolve(sym: SymbolU32) -> &'static str {
  unsafe { T.resolve(sym) }
    .expect("arena::fetch should only be called when the string is guaranteed to be allocated.")
}