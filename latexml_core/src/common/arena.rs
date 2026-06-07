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
static ARENA: Lazy<RefCell<Interner>> = Lazy::new(|| {
  // 131,072 = 2^17 — sized to absorb the latex.dump (109,863 entries)
  // plus a typical conversion's content (~15k more), so the hot path
  // hits no reallocation. Profiled 2026-05-12: a representative
  // `\usepackage{glossaries}` + math conversion ends at ~125,628
  // strings allocated.
  //
  // Cost: ~2-3 MB extra startup memory vs the prior 32,768. Worth it
  // when running batched conversions over a corpus (one fewer
  // BufferBackend Vec realloc + HashMap rehash per process startup);
  // negligible on a single short-lived run.
  RefCell::new(StringInterner::with_capacity_and_hasher(
    131_072,
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
  fn drop(&mut self) { ACTIVE.set(std::ptr::null_mut()); }
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

/// Call-site-cached interning for string literals — the first call on
/// a thread pins the literal via `pin_static`, later calls return the
/// cached `SymStr` directly (thread-local `OnceCell` load, no arena
/// access). Use this from hot state-key lookup sites so you can keep
/// writing string literals at the call site and still skip the per-call
/// `pin()` hash probe:
///
/// ```ignore
/// if state::lookup_bool_sym(pin!("groupNonBoxing")) { ... }
/// ```
///
/// Each call site gets its own thread-local cache (no global registry,
/// no dedicated pub-static constant per key), so there is no ergonomic
/// cost beyond typing the macro name.
///
/// Note: the macro `pin!` and the runtime-string function
/// `arena::pin(s)` share a name but occupy different namespaces in
/// Rust — `pin!(…)` is the macro, `pin(…)` is the function — so both
/// remain callable.
#[macro_export]
macro_rules! pin {
  ($s:literal) => {{
    std::thread_local! {
      static CACHED: std::cell::OnceCell<$crate::common::arena::SymStr>
        = const { std::cell::OnceCell::new() };
    }
    CACHED.with(|c| *c.get_or_init(|| $crate::common::arena::pin_static($s)))
  }};
}

/// Assign a string into the arena, returning a unique symbol.
///
/// No overflow guard: the main-level wall-clock watchdog (watchdog.rs)
/// catches genuinely runaway loops that would eventually saturate the
/// BufferBackend's u32 byte-offset range (~4.29 GB) long before any
/// real-world workload approaches it. Earlier versions had both a
/// call-count and a distinct-symbol sentinel; the call-count one
/// false-fired on dedup-heavy hot loops, and the distinct-symbol one
/// added a per-call `arena.len()` read on a hot path (~350k calls
/// per doc). Neither cost was paying for itself.
pub fn pin<S: AsRef<str>>(text: S) -> SymStr { with_arena_mut(|arena| arena.get_or_intern(text)) }

/// ASCII char-pin cache: every unique ASCII byte resolves to a single
/// SymStr for the lifetime of the thread (arena is append-only, syms
/// never change). Cache entries use `u32::MAX` as the "not yet pinned"
/// sentinel — all valid interner offsets are strictly below that.
/// Called from `lookup_catcode` / `assign_catcode` on every token,
/// so the RefCell + hashmap overhead on `pin` is a measurable cost
/// (1.4% Ir per callgrind on siunitx-heavy fixtures). The fast path
/// avoids `with_arena_mut` entirely for the common ASCII case.
#[thread_local]
static ASCII_CHAR_SYM: [std::cell::Cell<u32>; 128] =
  [const { std::cell::Cell::new(u32::MAX) }; 128];

pub fn pin_char(c: char) -> SymStr {
  use string_interner::Symbol;
  let code = c as u32;
  if code < 128 {
    let cached = ASCII_CHAR_SYM[code as usize].get();
    if cached != u32::MAX {
      // SAFETY: cached was produced by a prior successful `pin` below, so
      // the SymStr is valid for this arena.
      return SymStr::try_from_usize(cached as usize).expect("invalid cached ASCII SymStr");
    }
  }
  let sym = {
    let mut tmp = [0u8; 4];
    let s = c.encode_utf8(&mut tmp);
    pin(s)
  };
  if code < 128 {
    ASCII_CHAR_SYM[code as usize].set(sym.to_usize() as u32);
  }
  sym
}

/// Resolve a symbol and call the closure with a `&str` reference.
/// The closure may safely call `pin()` or any other arena function —
/// re-entrant access reuses the cached borrow.
///
/// # Safety
///
/// Uses `resolve_unchecked` → `from_utf8_unchecked`. Sound because
/// every path into the arena (`pin_static(&'static str)`,
/// `pin<S: AsRef<str>>(s)`, `pin_char(c: char)`) can only produce a
/// SymStr from content that was already valid UTF-8. The interner's
/// buffer is append-only by design: once a byte range is associated
/// with a symbol it is never mutated. Callgrind showed the default
/// validating `resolve` was ~3% of total Ir via `str::from_utf8`.
pub fn with<R, FnR>(sym: SymStr, caller: FnR) -> R
where FnR: FnOnce(&str) -> R {
  with_arena_mut(|arena| {
    // SAFETY: all input strings were valid UTF-8 at intern time (see
    // docstring above); every SymStr in this codebase originates
    // from a successful `get_or_intern(_static|_char)` call on a
    // valid `&str`, so the symbol always corresponds to a valid
    // byte range in the interner's buffer.
    let s = unsafe { arena.resolve_unchecked(sym) };
    caller(s)
  })
}

pub fn with2<R, FnR>(sym1: SymStr, sym2: SymStr, caller: FnR) -> R
where FnR: FnOnce(&str, &str) -> R {
  with_arena_mut(|arena| {
    // SAFETY: same invariant as `arena::with` — every SymStr here was
    // returned by a successful intern of a valid &str.
    let s1 = unsafe { arena.resolve_unchecked(sym1) };
    let s2 = unsafe { arena.resolve_unchecked(sym2) };
    caller(s1, s2)
  })
}

pub fn with3<R, FnR>(sym1: SymStr, sym2: SymStr, sym3: SymStr, caller: FnR) -> R
where FnR: FnOnce(&str, &str, &str) -> R {
  with_arena_mut(|arena| {
    // SAFETY: see `arena::with`.
    let s1 = unsafe { arena.resolve_unchecked(sym1) };
    let s2 = unsafe { arena.resolve_unchecked(sym2) };
    let s3 = unsafe { arena.resolve_unchecked(sym3) };
    caller(s1, s2, s3)
  })
}

pub fn with_many<R, FnR>(syms: &[SymStr], caller: FnR) -> R
where FnR: FnOnce(Vec<&str>) -> R {
  with_arena_mut(|arena| {
    // SAFETY: see `arena::with`.
    let many = syms
      .iter()
      .map(|sym| unsafe { arena.resolve_unchecked(*sym) })
      .collect();
    caller(many)
  })
}

pub fn to_string(sym: SymStr) -> String {
  with_arena_mut(|arena| {
    // SAFETY: see `arena::with`.
    unsafe { arena.resolve_unchecked(sym) }.to_owned()
  })
}

pub fn join(syms: &[SymStr], sep: &str) -> String { with_many(syms, |strs| strs.join(sep)) }

pub fn len() -> usize { with_arena_mut(|arena| arena.len()) }

/// Free every interned string on this thread, returning the arena to a
/// fresh, empty state.
///
/// **Danger:** this invalidates *every* outstanding [`SymStr`] on the
/// thread — they become dangling indices that may resolve to unrelated
/// strings after re-interning. It is only sound when **nothing on the
/// thread will read a pre-reset `SymStr` again**: i.e. between fully
/// independent conversions in a reused process (the test harness, where
/// each test has already serialized its output to owned `String`s and
/// the thread is about to exit or be re-initialized) or a future daemon
/// that re-initializes the engine afterward. The single-conversion
/// `latexml_oxide` binary never calls this — it exits instead.
///
/// Needed because the engine's roots are `#[thread_local]` *attribute*
/// statics, which (unlike the `thread_local!` macro) do **not** run
/// destructors on thread exit. Without an explicit reset, every reused
/// thread leaks its interner (~tens of MB for a full document). See
/// `latexml_core::reset_thread_engine`.
pub fn reset() {
  with_arena_mut(|arena| {
    *arena = StringInterner::with_capacity_and_hasher(
      131_072,
      BuildHasherDefault::<FxHasher>::default(),
    );
  });
}

/// Eagerly initialize this thread's `#[thread_local]` `ARENA` Lazy.
///
/// `ARENA` is the *leaf* of the engine's thread-local dependency graph:
/// every other root's `Lazy` initializer interns symbols via [`pin`], so it
/// reaches into `ARENA`. Calling this at conversion entry (see
/// `Core::new`), before any other root is touched, guarantees those later
/// initializers find a fully-constructed `ARENA` instead of triggering its
/// initialization *re-entrantly from within their own*. That re-entrant
/// cross-`#[thread_local]` initialization is benign on Linux/ELF TLS but is
/// the documented macOS hazard (rust-lang/rust#29594) behind the macOS
/// worker-thread memory corruption in issue #217. `ARENA`'s own initializer
/// touches no other thread-local, so forcing it first is always safe.
/// No behavioral change on Linux.
pub fn force_init() { Lazy::force(&ARENA); }

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn pin_dedups_equal_strings() {
    let a = pin("arena_test_foo");
    let b = pin("arena_test_foo");
    assert_eq!(a, b, "equal strings must return the same SymStr");
  }

  #[test]
  fn pin_distinguishes_different_strings() {
    let a = pin("arena_test_bar");
    let b = pin("arena_test_baz");
    assert_ne!(a, b);
  }

  #[test]
  fn pin_static_matches_pin() {
    let a = pin_static("arena_test_qux");
    let b = pin("arena_test_qux");
    assert_eq!(a, b, "pin_static and pin should intern to the same SymStr");
  }

  #[test]
  fn to_string_roundtrips() {
    let sym = pin("arena_test_quux");
    assert_eq!(to_string(sym), "arena_test_quux");
  }

  #[test]
  fn with_borrows_without_allocating() {
    let sym = pin("arena_test_corge");
    let len = with(sym, |s| s.len());
    assert_eq!(len, "arena_test_corge".len());
  }

  #[test]
  fn with_predicate_returns_bool() {
    let sym = pin("arena_test_predicate");
    let starts_with = with(sym, |s| s.starts_with("arena"));
    assert!(starts_with);
  }

  #[test]
  fn reset_empties_interner_and_stays_usable() {
    // Each `#[test]` runs on its own thread, so this thread's arena
    // starts empty and the reset is isolated from sibling tests.
    let _ = pin("arena_reset_alpha");
    let _ = pin("arena_reset_beta");
    assert!(len() >= 2, "expected the two pins to be interned");
    reset();
    assert_eq!(len(), 0, "reset must return the interner to empty");
    // Interning still works after a reset (fresh backend installed).
    let s = pin("arena_reset_gamma");
    assert_eq!(to_string(s), "arena_reset_gamma");
  }

  #[test]
  fn pin_char_ascii_roundtrips() {
    let sym = pin_char('a');
    assert_eq!(to_string(sym), "a");
    let sym2 = pin_char('a');
    assert_eq!(sym, sym2, "ASCII char pin is cached");
  }

  #[test]
  fn pin_char_distinct_chars_distinct_syms() {
    assert_ne!(pin_char('a'), pin_char('b'));
    assert_ne!(pin_char('0'), pin_char('1'));
  }

  #[test]
  fn pin_char_unicode_roundtrips() {
    // Non-ASCII chars go through the general arena path.
    let sym = pin_char('π');
    assert_eq!(to_string(sym), "π");
  }

  #[test]
  fn join_concatenates_with_separator() {
    let a = pin("arena_test_alpha");
    let b = pin("arena_test_beta");
    let c = pin("arena_test_gamma");
    let out = join(&[a, b, c], ",");
    assert_eq!(out, "arena_test_alpha,arena_test_beta,arena_test_gamma");
  }

  #[test]
  fn pin_macro_caches_per_site() {
    // The `pin!` macro returns a cached SymStr per call site. Two
    // call sites with identical strings cache independently but
    // intern to the same underlying symbol.
    let a = pin!("arena_test_literal");
    let b = pin!("arena_test_literal");
    assert_eq!(a, b, "same literal at different call sites → same SymStr");
  }
}
