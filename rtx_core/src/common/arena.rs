//! An arena for interning strings
//!  (global, mutable, single thread only)
//!
//! Note: works under the assumption of a single threaded, short-lived process,
//! where memory starvation will not be an issue. It would need a hard reset if the same process
//! does multiple conversions, and a different implementation if one needs a thread-local arena.

use once_cell::sync::Lazy;
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;
use string_interner::backend::StringBackend;
use string_interner::symbol::SymbolU32;
use string_interner::StringInterner;

static mut T: Lazy<StringInterner<StringBackend, BuildHasherDefault<FxHasher>>> =
  Lazy::new(|| StringInterner::with_capacity_and_hasher(10_000, BuildHasherDefault::<FxHasher>::default()));

/// Assign a string into the arena, returning a unique symbol associated with it
pub fn pin<S: AsRef<str>>(text: S) -> SymbolU32 { unsafe { T.get_or_intern(text) } }

/// Resolve the data associated with a unique symbol from this arena, asserting it's already present
pub fn resolve(sym: SymbolU32) -> &'static str {
  unsafe { T.resolve(sym) }
    .expect("arena::fetch should only be called when the string is guaranteed to be allocated.")
}

/// Attempt to resolve a string associated with a unique symbol from this arena, None if missing
pub fn try_resolve(sym: SymbolU32) -> Option<&'static str> { unsafe { T.resolve(sym) } }

/// Pin-and-resolve a string into the arena, obtaining a `&'static str` reference to it
/// (useful for str lifetime collisions)
pub fn as_static<S: AsRef<str>>(raw: S) -> &'static str { resolve(pin(raw)) }

/// the unique symbol for str value "ANY"
pub static ANY_SYM: Lazy<SymbolU32> = Lazy::new(|| pin("ANY"));
/// the unique symbol for str value "#PCDATA"
pub static PCDATA_SYM: Lazy<SymbolU32> = Lazy::new(|| pin("#PCDATA"));
/// the unique symbol for the empty str value ""
pub static EMPTY_SYM: Lazy<SymbolU32> = Lazy::new(|| pin(""));
/// the unique symbol for str value "ltx:*"
pub static LTX_STAR_SYM: Lazy<SymbolU32> = Lazy::new(|| pin("ltx:*"));
/// the unique symbol for str value "ltx:p"
pub static LTX_P_SYM: Lazy<SymbolU32> = Lazy::new(|| pin("ltx:p"));