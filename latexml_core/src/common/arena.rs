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

/// Assign a static str into the arena, returning a unique symbol.
pub fn pin_static(text: &'static str) -> SymStr {
  with_arena_mut(|arena| arena.get_or_intern_static(text))
}

/// Call-site-cached interning for string literals — the first call on a
/// thread pins the literal via `pin_static`, later calls return the
/// cached `SymStr` directly (thread-local `OnceCell` load, no arena
/// access). Use this from hot state-key lookup sites so you can keep
/// writing string literals at the call site and still skip the per-call
/// `pin()` hash probe:
///
/// ```ignore
/// if state::lookup_bool_sym(pin_literal!("groupNonBoxing")) { ... }
/// ```
///
/// Each call site gets its own thread-local cache (no global registry,
/// no dedicated pub-static constant per key), so there is no ergonomic
/// cost beyond typing the macro name.
#[macro_export]
macro_rules! pin_literal {
  ($s:literal) => {{
    std::thread_local! {
      static CACHED: std::cell::OnceCell<$crate::common::arena::SymStr>
        = const { std::cell::OnceCell::new() };
    }
    CACHED.with(|c| *c.get_or_init(|| $crate::common::arena::pin_static($s)))
  }};
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
