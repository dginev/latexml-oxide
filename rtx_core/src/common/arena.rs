//! An arena for interning strings
//!  (global, mutable, single thread only)
//!
//! Note: works under the assumption of a single threaded, short-lived process,
//! where memory starvation will not be an issue. It would need a hard reset if the same process
//! does multiple conversions, and a different implementation if one needs a thread-local arena.

use std::thread_local;
use std::sync::RwLock;
// use std::borrow::Cow;
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;
use string_interner::backend::BufferBackend;
use string_interner::symbol::SymbolU32;
use string_interner::StringInterner;

thread_local! {
  static T: RwLock<StringInterner<BufferBackend, BuildHasherDefault<FxHasher>>> =
    RwLock::new(StringInterner::with_capacity_and_hasher(10_000, BuildHasherDefault::<FxHasher>::default()));
  /// the unique symbol for str value "ANY"
  pub static ANY_SYM: SymbolU32 = pin_static("ANY");
  /// the unique symbol for str value "#PCDATA"
  pub static H_PCDATA_SYM: SymbolU32 = pin_static("#PCDATA");
  /// the unique symbol for str value "#COMMENT"
  pub static H_COMMENT_SYM: SymbolU32 = pin_static("#Comment");
  /// the unique symbol for the empty str value ""
  pub static EMPTY_SYM: SymbolU32 = pin_static("");
  /// the unique symbol for str value "ltx:*"
  pub static LTX_STAR_SYM: SymbolU32 = pin_static("ltx:*");
  /// the unique symbol for str value "ltx:p"
  pub static LTX_P_SYM: SymbolU32 = pin_static("ltx:p");
  /// the unique symbol for str value "\globaldefs"
  pub static GLOBAL_DEFS_SYM : SymbolU32 = pin_static("\\globaldefs");
  /// the unique symbol for str value "_WildCard_"
  pub static WILD_CARD_SYM : SymbolU32 = pin_static("_WildCard_");
  /// the unique symbol for str value "#ProcessingInstruction"
  pub static H_PI_SYM : SymbolU32 = pin_static("#ProcessingInstruction");
  /// the unique symbol for str value "#DTD"
  pub static DTD_SYM : SymbolU32 = pin_static("#DTD");
  /// the unique symbol for str value "#Document"
  pub static H_DOC_SYM : SymbolU32 = pin_static("#Document");
  /// the unique symbol for str value "ltx:_Capture_"
  pub static CAPTURE_SYM : SymbolU32 = pin_static("ltx:_Capture_");
  /// the unique symbol for str value "font"
  pub static FONT_SYM : SymbolU32 = pin_static("font");
}

/// Assign a static str into the arena, returning a unique symbol associated with it
pub fn pin_static(text: &'static str) -> SymbolU32 {
  T.with(|arena| arena.write().unwrap().get_or_intern_static(text) )
}

/// Assign a string into the arena, returning a unique symbol associated with it
pub fn pin<S: AsRef<str>>(text: S) -> SymbolU32 {
  T.with(|arena| arena.write().unwrap().get_or_intern(text) )
}

pub fn with<R, FnR>(sym: SymbolU32, caller: FnR) -> R
where FnR: FnOnce(&str) -> R {
  T.with(|arena|
    caller(arena.read().unwrap().resolve(sym)
    .expect("arena.resolve should only be called when the string is guaranteed to be allocated.")))
}

/// Attempt to resolve a string associated with a unique symbol from this arena, None if missing
pub fn try_with<R, FnR>(sym: SymbolU32, caller: FnR) -> R
where FnR: FnOnce(Option<&str>) -> R {
  T.with(|arena|
    caller(arena.read().unwrap().resolve(sym)
    ))
}

// pub fn cowned(sym: SymbolU32) -> Cow<'static, str> {
//   T.with(|arena|
//     Cow::Owned(String::from(arena.resolve(sym).expect("arena.resolve should only be called when the string is guaranteed to be allocated.")))
//   )
// }

// pub fn owned(sym: SymbolU32) -> String {
//   T.with(|arena|
//     String::from(arena.resolve(sym).expect("arena.resolve should only be called when the string is guaranteed to be allocated."))
//   )
// }
