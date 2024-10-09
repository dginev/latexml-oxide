//! An arena for interning strings
//!  (global, mutable, single thread only)
//!
//! Note: works under the assumption of a single threaded, short-lived process,
//! where memory starvation will not be an issue. It would need a hard reset if the same process
//! does multiple conversions, and a different implementation if one needs a thread-local arena.

use std::cell::RefCell;

use rustc_hash::FxHasher;

use std::hash::BuildHasherDefault;
use string_interner::backend::BufferBackend;
use string_interner::StringInterner;
use once_cell::sync::Lazy;

pub mod data;
pub use data::{SymStr, SymHashMap};

#[thread_local]
static ARENA: Lazy<RefCell<StringInterner<BufferBackend, BuildHasherDefault<FxHasher>>>> = Lazy::new(|| RefCell::new(
  StringInterner::with_capacity_and_hasher(32_768, BuildHasherDefault::<FxHasher>::default())));
/// the unique symbol for str value "ANY"
#[thread_local]
pub static ANY_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("ANY"));
/// the unique symbol for str value "#PCDATA"
#[thread_local]
pub static H_PCDATA_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("#PCDATA"));
/// the unique symbol for str value "#COMMENT"
#[thread_local]
pub static H_COMMENT_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("#Comment"));
/// the unique symbol for the empty str value ""
#[thread_local]
pub static EMPTY_SYM: Lazy<SymStr> = Lazy::new(|| pin_static(""));
/// the unique symbol for str value "ltx:*"
#[thread_local]
pub static LTX_STAR_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("ltx:*"));
/// the unique symbol for str value "ltx:p"
#[thread_local]
pub static LTX_P_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("ltx:p"));
/// the unique symbol for str value "\globaldefs"
#[thread_local]
pub static GLOBAL_DEFS_SYM : Lazy<SymStr> = Lazy::new(|| pin_static("\\globaldefs"));
/// the unique symbol for str value "\dont_expand"
#[thread_local]
pub static DONT_EXPAND_SYM : Lazy<SymStr> = Lazy::new(|| pin_static("\\dont_expand"));
/// the unique symbol for str value "_WildCard_"
#[thread_local]
pub static WILD_CARD_SYM : Lazy<SymStr> = Lazy::new(|| pin_static("_WildCard_"));
/// the unique symbol for str value "#ProcessingInstruction"
#[thread_local]
pub static H_PI_SYM : Lazy<SymStr> = Lazy::new(|| pin_static("#ProcessingInstruction"));
/// the unique symbol for str value "#DTD"
#[thread_local]
pub static H_DTD_SYM : Lazy<SymStr> = Lazy::new(|| pin_static("#DTD"));
/// the unique symbol for str value "#Document"
#[thread_local]
pub static H_DOC_SYM : Lazy<SymStr> = Lazy::new(|| pin_static("#Document"));
/// the unique symbol for str value "#default"
#[thread_local]
pub static H_DEFAULT_SYM : Lazy<SymStr> = Lazy::new(|| pin_static("#default"));
/// the unique symbol for str value "ltx:_Capture_"
#[thread_local]
pub static CAPTURE_SYM : Lazy<SymStr> = Lazy::new(|| pin_static("ltx:_Capture_"));
/// the unique symbol for str value "font"
#[thread_local]
pub static FONT_SYM : Lazy<SymStr> = Lazy::new(|| pin_static("font"));
/// the unique symbol for str value "xml:id"
#[thread_local]
pub static XML_ID_SYM : Lazy<SymStr> = Lazy::new(|| pin_static("xml:id"));
/// the unique symbol for str value "DTD"
#[thread_local]
pub static DTD_SYM : Lazy<SymStr> = Lazy::new(|| pin_static("DTD"));
/// the unique symbol for str value "RelaxNG"
#[thread_local]
pub static RELAXNG_SYM : Lazy<SymStr> = Lazy::new(|| pin_static("RelaxNG"));
/// the unique symbol for str value "text"
#[thread_local]
pub static TEXT_SYM : Lazy<SymStr> = Lazy::new(|| pin_static("text"));
/// the unique symbol for str value "math"
#[thread_local]
pub static MATH_SYM : Lazy<SymStr> = Lazy::new(|| pin_static("math"));

/// Assign a static str into the arena, returning a unique symbol associated with it
pub fn pin_static(text: &'static str) -> SymStr {
  ARENA.borrow_mut().get_or_intern_static(text)
}

/// Assign a string into the arena, returning a unique symbol associated with it
pub fn pin<S: AsRef<str>>(text: S) -> SymStr {
  ARENA.borrow_mut().get_or_intern(text)
}

pub fn pin_char(c: char) -> SymStr {
  let mut tmp = [0u8; 4];
  let s = c.encode_utf8(&mut tmp);
  pin(s)
}

pub fn into_pin<T:ToString>(num: T) -> SymStr {
  pin(num.to_string())
}

pub fn with<R, FnR>(sym: SymStr, caller: FnR) -> R
where FnR: FnOnce(&str) -> R {
    caller(
      ARENA
        .borrow()
        .resolve(sym)
        .expect("arena::with should only be called when the string is guaranteed to be allocated."),
    )
}

pub fn with2<R, FnR>(sym1: SymStr, sym2: SymStr, caller: FnR) -> R
where FnR: FnOnce(&str,&str) -> R {
  let arena = ARENA.borrow();
  let str1 = arena.resolve(sym1)
    .expect("arena::with should only be called when the string is guaranteed to be allocated.");
  let str2 = arena.resolve(sym2)
    .expect("arena::with should only be called when the string is guaranteed to be allocated.");
  caller(str1,str2)
}
pub fn with3<R, FnR>(sym1: SymStr, sym2: SymStr, sym3:SymStr, caller: FnR) -> R
where FnR: FnOnce(&str,&str,&str) -> R {
  let arena = ARENA.borrow();
  let str1 = arena.resolve(sym1)
    .expect("arena::with should only be called when the string is guaranteed to be allocated.");
  let str2 = arena.resolve(sym2)
    .expect("arena::with should only be called when the string is guaranteed to be allocated.");
  let str3 = arena.resolve(sym3)
    .expect("arena::with should only be called when the string is guaranteed to be allocated.");
  caller(str1,str2,str3)
}

pub fn with_many<R, FnR>(syms: &[SymStr], caller: FnR) -> R
where FnR: FnOnce(Vec<&str>) -> R {
  let arena = ARENA.borrow();
  let many = syms.iter().map(|sym| arena.resolve(*sym)
    .expect("arena::with should only be called when the string is guaranteed to be allocated.")).collect();
  caller(many)
}

pub fn to_string(sym: SymStr) -> String {
  ARENA.borrow()
    .resolve(sym)
    .expect(
      "arena::to_string should only be called when the string is guaranteed to be allocated.",
    )
    .to_owned()
}

pub fn join(syms: &[SymStr], sep:&str) -> String {
  with_many(syms, |strs| {
    strs.join(sep)
  })
}

pub fn len() -> usize {
  ARENA.borrow().len()
}