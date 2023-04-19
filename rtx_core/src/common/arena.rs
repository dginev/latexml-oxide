//! An arena for interning strings
//!  (global, mutable, single thread only)
//!
//! Note: works under the assumption of a single threaded, short-lived process,
//! where memory starvation will not be an issue. It would need a hard reset if the same process
//! does multiple conversions, and a different implementation if one needs a thread-local arena.

use std::cell::RefCell;
// use std::borrow::Cow;
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;
use string_interner::backend::BufferBackend;
use string_interner::symbol::SymbolU32;
use string_interner::StringInterner;

thread_local! {
  static T: RefCell<StringInterner<BufferBackend, BuildHasherDefault<FxHasher>>> = RefCell::new(
    StringInterner::with_capacity_and_hasher(32_768, BuildHasherDefault::<FxHasher>::default()));
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
  pub static H_DTD_SYM : SymbolU32 = pin_static("#DTD");
  /// the unique symbol for str value "#Document"
  pub static H_DOC_SYM : SymbolU32 = pin_static("#Document");
  /// the unique symbol for str value "#default"
  pub static H_DEFAULT_SYM : SymbolU32 = pin_static("#default");
  /// the unique symbol for str value "ltx:_Capture_"
  pub static CAPTURE_SYM : SymbolU32 = pin_static("ltx:_Capture_");
  /// the unique symbol for str value "font"
  pub static FONT_SYM : SymbolU32 = pin_static("font");
  /// the unique symbol for str value "xml:id"
  pub static XML_ID_SYM : SymbolU32 = pin_static("xml:id");
  /// the unique symbol for str value "DTD"
  pub static DTD_SYM : SymbolU32 = pin_static("DTD");
  /// the unique symbol for str value "RelaxNG"
  pub static RELAXNG_SYM : SymbolU32 = pin_static("RelaxNG");
}

/// Assign a static str into the arena, returning a unique symbol associated with it
pub fn pin_static(text: &'static str) -> SymbolU32 {
  T.with(|arena| arena.borrow_mut().get_or_intern_static(text))
}

/// Assign a string into the arena, returning a unique symbol associated with it
pub fn pin<S: AsRef<str>>(text: S) -> SymbolU32 {
  T.with(|arena| arena.borrow_mut().get_or_intern(text))
}

pub fn pin_char(c: char) -> SymbolU32 {
  let mut tmp = [0u8; 3];
  let s = c.encode_utf8(&mut tmp);
  pin(s)
}

pub fn with<R, FnR>(sym: SymbolU32, caller: FnR) -> R
where FnR: FnOnce(&str) -> R {
  T.with(|arena| {
    caller(
      arena
        .borrow()
        .resolve(sym)
        .expect("arena::with should only be called when the string is guaranteed to be allocated."),
    )
  })
}

pub fn to_string(sym: SymbolU32) -> String {
  T.with(|arena| {
    arena
      .borrow()
      .resolve(sym)
      .expect(
        "arena::to_string should only be called when the string is guaranteed to be allocated.",
      )
      .to_owned()
  })
}

// TODO: Is this needed? The tighter call would guarantee the T lock is released early.
pub fn chars(sym: SymbolU32) -> Vec<char> {
  T.with(|arena| {
    arena
      .borrow()
      .resolve(sym)
      .expect("arena::chars should only be called when the string is guaranteed to be allocated.")
      .chars()
      .collect()
  })
}
