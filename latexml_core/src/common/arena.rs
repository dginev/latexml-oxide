//! An arena for interning strings
//!  (global, mutable, single thread only)
//!
//! Note: works under the assumption of a single threaded, short-lived process,
//! where memory starvation will not be an issue. It would need a hard reset if the same process
//! does multiple conversions, and a different implementation if one needs a thread-local arena.
//!
//! ## Borrow safety
//!
//! All arena access goes through `with_arena_mut()`, which acquires a mutable
//! borrow from `RefCell` on first call and caches the raw pointer in a
//! thread-local `Cell`. Re-entrant calls (e.g., `pin()` inside a `with()`
//! closure) reuse the cached pointer without touching `RefCell`, eliminating
//! borrow conflicts entirely. A RAII guard clears the pointer on scope exit
//! (including panics), so the invariant holds through unwinds.

use std::cell::{Cell, RefCell};

use rustc_hash::FxHasher;

use once_cell::sync::Lazy;
use std::hash::BuildHasherDefault;
use string_interner::StringInterner;
use string_interner::backend::BufferBackend;

pub mod data;
pub use data::{SymHashMap, SymStr};

type Interner = StringInterner<BufferBackend, BuildHasherDefault<FxHasher>>;

#[thread_local]
static ARENA: Lazy<RefCell<Interner>> =
  Lazy::new(|| {
    RefCell::new(StringInterner::with_capacity_and_hasher(
      32_768,
      BuildHasherDefault::<FxHasher>::default(),
    ))
  });

/// Cached raw pointer to the arena's inner interner, valid only while the
/// outermost `with_arena_mut` holds its `RefMut` guard. Null when idle.
#[thread_local]
static ACTIVE: Cell<*mut Interner> = Cell::new(std::ptr::null_mut());

/// RAII guard that clears `ACTIVE` on drop (including during unwinds).
/// Declared AFTER the `RefMut` guard so it drops FIRST (Rust drops in
/// reverse declaration order), ensuring no window where ACTIVE is stale.
struct ArenaCleanup;
impl Drop for ArenaCleanup {
  fn drop(&mut self) {
    ACTIVE.set(std::ptr::null_mut());
  }
}

/// Execute `f` with mutable access to the interner.
/// First (outermost) call acquires `borrow_mut()` from `RefCell` and caches
/// the raw pointer. Re-entrant calls reuse the cached pointer — no RefCell
/// interaction, so no borrow conflicts.
///
/// # Safety
/// Sound because: (1) `#[thread_local]` guarantees single-thread access,
/// (2) `ArenaCleanup` clears the pointer before the `RefMut` drops,
/// (3) re-entrant access is strictly nested (same stack, same thread).
#[inline]
fn with_arena_mut<R>(f: impl FnOnce(&mut Interner) -> R) -> R {
  let ptr = ACTIVE.get();
  if !ptr.is_null() {
    // Re-entrant call — reuse existing mutable borrow.
    // SAFETY: ptr was set by the outermost call on this thread, which still
    // holds the RefMut guard. We are nested on the same stack.
    f(unsafe { &mut *ptr })
  } else {
    // Outermost call — acquire mutable borrow from RefCell.
    let mut guard = ARENA.borrow_mut();
    let ptr = &mut *guard as *mut Interner;
    ACTIVE.set(ptr);
    let _cleanup = ArenaCleanup; // drops BEFORE guard (reverse order)
    f(&mut guard)
  }
}

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
pub static GLOBAL_DEFS_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("\\globaldefs"));
/// the unique symbol for str value "\dont_expand"
#[thread_local]
pub static DONT_EXPAND_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("\\dont_expand"));
/// the unique symbol for str value "_WildCard_"
#[thread_local]
pub static WILD_CARD_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("_WildCard_"));
/// the unique symbol for str value "#ProcessingInstruction"
#[thread_local]
pub static H_PI_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("#ProcessingInstruction"));
/// the unique symbol for str value "#DTD"
#[thread_local]
pub static H_DTD_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("#DTD"));
/// the unique symbol for str value "#Document"
#[thread_local]
pub static H_DOC_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("#Document"));
/// the unique symbol for str value "#default"
#[thread_local]
pub static H_DEFAULT_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("#default"));
/// the unique symbol for str value "ltx:_Capture_"
#[thread_local]
pub static CAPTURE_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("ltx:_Capture_"));
/// the unique symbol for str value "font"
#[thread_local]
pub static FONT_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("font"));
/// the unique symbol for str value "xml:id"
#[thread_local]
pub static XML_ID_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("xml:id"));
/// the unique symbol for str value "RelaxNG"
#[thread_local]
pub static RELAXNG_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("RelaxNG"));
/// the unique symbol for str value "text"
#[thread_local]
pub static TEXT_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("text"));
/// the unique symbol for str value "math"
#[thread_local]
pub static MATH_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("math"));
/// Pre-pinned symbols for hot state lookup keys (used by lookup_* helpers
/// and top callers to skip the per-call `pin(key)` hash lookup).
#[thread_local]
pub static IN_MATH_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("IN_MATH"));
#[thread_local]
pub static MODE_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("MODE"));
#[thread_local]
pub static BOUND_MODE_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("BOUND_MODE"));
#[thread_local]
pub static IN_PREAMBLE_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("inPreamble"));
#[thread_local]
pub static INTERPRETING_DEFINITIONS_SYM: Lazy<SymStr> =
  Lazy::new(|| pin_static("INTERPRETING_DEFINITIONS"));
#[thread_local]
pub static XGLOBAL_AT_SYM: Lazy<SymStr> = Lazy::new(|| pin_static("xglobal@"));

/// Assign a static str into the arena, returning a unique symbol.
pub fn pin_static(text: &'static str) -> SymStr {
  with_arena_mut(|arena| arena.get_or_intern_static(text))
}

/// Assign a string into the arena, returning a unique symbol.
pub fn pin<S: AsRef<str>>(text: S) -> SymStr {
  with_arena_mut(|arena| arena.get_or_intern(text))
}

pub fn pin_char(c: char) -> SymStr {
  let mut tmp = [0u8; 4];
  let s = c.encode_utf8(&mut tmp);
  pin(s)
}

/// Resolve a symbol and call the closure with a `&str` reference.
/// The closure may safely call `pin()` or any other arena function —
/// re-entrant access reuses the cached borrow.
pub fn with<R, FnR>(sym: SymStr, caller: FnR) -> R
where FnR: FnOnce(&str) -> R {
  with_arena_mut(|arena| {
    let s = arena
      .resolve(sym)
      .expect("arena::with: symbol not found in arena");
    caller(s)
  })
}

pub fn with2<R, FnR>(sym1: SymStr, sym2: SymStr, caller: FnR) -> R
where FnR: FnOnce(&str, &str) -> R {
  with_arena_mut(|arena| {
    let s1 = arena
      .resolve(sym1)
      .expect("arena::with2: symbol not found in arena");
    let s2 = arena
      .resolve(sym2)
      .expect("arena::with2: symbol not found in arena");
    caller(s1, s2)
  })
}

pub fn with3<R, FnR>(sym1: SymStr, sym2: SymStr, sym3: SymStr, caller: FnR) -> R
where FnR: FnOnce(&str, &str, &str) -> R {
  with_arena_mut(|arena| {
    let s1 = arena
      .resolve(sym1)
      .expect("arena::with3: symbol not found in arena");
    let s2 = arena
      .resolve(sym2)
      .expect("arena::with3: symbol not found in arena");
    let s3 = arena
      .resolve(sym3)
      .expect("arena::with3: symbol not found in arena");
    caller(s1, s2, s3)
  })
}

pub fn with_many<R, FnR>(syms: &[SymStr], caller: FnR) -> R
where FnR: FnOnce(Vec<&str>) -> R {
  with_arena_mut(|arena| {
    let many = syms
      .iter()
      .map(|sym| {
        arena
          .resolve(*sym)
          .expect("arena::with_many: symbol not found in arena")
      })
      .collect();
    caller(many)
  })
}

pub fn to_string(sym: SymStr) -> String {
  with_arena_mut(|arena| {
    arena
      .resolve(sym)
      .expect("arena::to_string: symbol not found in arena")
      .to_owned()
  })
}

pub fn join(syms: &[SymStr], sep: &str) -> String { with_many(syms, |strs| strs.join(sep)) }

pub fn len() -> usize { with_arena_mut(|arena| arena.len()) }
