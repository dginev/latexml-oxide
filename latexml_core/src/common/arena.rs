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

/// Cumulative `pin` call count — incremented on every call even when
/// the string is deduplicated (same call pressure regardless).
/// A runaway Mouth loop (observed in 0906.1883: 163M unique anonymous
/// mouth-source strings) exceeds the `string-interner` BufferBackend's
/// u32 byte-offset range (~4GB), at which point SymStr values wrap and
/// resolve returns garbage. 50M pins is a conservative threshold:
/// normal documents are well under 1M. Panic at 50M surfaces the loop
/// site cleanly before the arena silently overflows.
#[thread_local]
static PIN_CALLS: std::cell::Cell<usize> = std::cell::Cell::new(0);

/// Assign a string into the arena, returning a unique symbol.
pub fn pin<S: AsRef<str>>(text: S) -> SymStr {
  let count = PIN_CALLS.get().wrapping_add(1);
  PIN_CALLS.set(count);
  if count == 50_000_000 {
    // Only fire once (on exact equality) to avoid re-entering panic
    // machinery that itself might pin strings for its own formatting.
    panic!(
      "arena::pin invoked 50,000,000 times — arena is near u32 offset \
       overflow. A runaway loop is creating unique strings. See \
       SYNC_STATUS 0906.1883 for context (163M Mouth::default() loop)."
    );
  }
  with_arena_mut(|arena| arena.get_or_intern(text))
}

/// ASCII char-pin cache: every unique ASCII byte resolves to a single
/// SymStr for the lifetime of the thread (arena is append-only, syms
/// never change). Cache entries use `u32::MAX` as the "not yet pinned"
/// sentinel — all valid interner offsets are strictly below that.
/// Called from `lookup_catcode` / `assign_catcode` on every token,
/// so the RefCell + hashmap overhead on `pin` is a measurable cost
/// (1.4% Ir per callgrind on siunitx-heavy fixtures). The fast path
/// avoids `with_arena_mut` entirely for the common ASCII case.
#[thread_local]
static ASCII_CHAR_SYM: [std::cell::Cell<u32>; 128] = [const { std::cell::Cell::new(u32::MAX) }; 128];

pub fn pin_char(c: char) -> SymStr {
  use string_interner::Symbol;
  let code = c as u32;
  if code < 128 {
    let cached = ASCII_CHAR_SYM[code as usize].get();
    if cached != u32::MAX {
      // SAFETY: cached was produced by a prior successful `pin` below, so
      // the SymStr is valid for this arena.
      return SymStr::try_from_usize(cached as usize)
        .expect("invalid cached ASCII SymStr");
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
