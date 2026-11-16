//! An arena for interning strings
//!  (global, mutable, single thread only)
//!
//! Note: works under the assumption of a single threaded, short-lived process,
//! where memory starvation will not be an issue. It would need a hard reset if the same process
//! does multiple conversions, and a different implementation if one needs a thread-local arena.
//!
//! ## Borrow safety
//!
//! A lend ([`with`] and its siblings) holds a shared `RefCell` borrow of the arena for the closure's duration, so the
//! buffer it lends from cannot move under it. A string interned while a lend is live cannot grow that buffer: it goes
//! to the overflow page, its own `Rc<str>` under a symbol at or above `OVERFLOW_BASE`. One string has one symbol across
//! both tiers until a [`reset`]: every intern probes the buffer first, then the page, and only a string neither holds
//! is added (to the buffer outside a lend, to the page under one).

use std::{
  cell::{Cell, RefCell},
  hash::BuildHasherDefault,
  ops::Deref,
  rc::Rc,
};

use once_cell::sync::Lazy;
use rustc_hash::{FxHashMap, FxHasher};
use string_interner::{StringInterner, Symbol, backend::BufferBackend};

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
  RefCell::new(new_interner())
});

fn new_interner() -> Interner {
  StringInterner::with_capacity_and_hasher(131_072, BuildHasherDefault::<FxHasher>::default())
}

/// The first overflow symbol: the arena's own symbols are byte offsets into its buffer, kept below 2 GiB.
const OVERFLOW_BASE: usize = 1 << 31;

/// The strings interned while a lend was live, which the arena's buffer could not take without moving.
#[derive(Default)]
struct Overflow {
  texts:   Vec<Rc<str>>,
  symbols: FxHashMap<Rc<str>, SymStr>,
  /// New strings since [`take_overflow_interns`] last read the count.
  fresh:   usize,
}

#[thread_local]
static OVERFLOW: Lazy<RefCell<Overflow>> = Lazy::new(Default::default);

/// The overflow page's symbol for `text`, if the page holds it.
#[inline(never)]
fn page_symbol(text: &str) -> Option<SymStr> { OVERFLOW.borrow().symbols.get(text).copied() }

/// A symbol of the arena's buffer, refusing one past the 2 GiB the overflow symbols start at. Lookups pass theirs
/// through too: a string interned past 2 GiB stays in the buffer after the panic is caught, never to read as an
/// overflow symbol.
#[inline]
fn in_buffer(sym: SymStr) -> SymStr {
  assert!(
    sym.to_usize() < OVERFLOW_BASE,
    "the string arena's buffer reached 2 GiB, where the overflow symbols start"
  );
  sym
}

#[inline]
fn intern(text: &str) -> SymStr {
  if let Ok(mut arena) = ARENA.try_borrow_mut() {
    if let Some(sym) = arena.get(text) {
      return in_buffer(sym);
    }
    // A string first interned under a lend keeps its overflow symbol.
    return page_symbol(text).unwrap_or_else(|| in_buffer(arena.get_or_intern(text)));
  }
  intern_cold(text)
}

/// [`intern`] while a lend is live: a string either tier holds keeps its symbol (the buffer probed first, as most
/// interns find their string); a new one goes to the overflow page.
#[cold]
#[inline(never)]
fn intern_cold(text: &str) -> SymStr {
  if let Some(sym) = ARENA.borrow().get(text) {
    return in_buffer(sym);
  }
  let mut overflow = OVERFLOW.borrow_mut();
  if let Some(&sym) = overflow.symbols.get(text) {
    return sym;
  }
  let sym = SymStr::try_from_usize(OVERFLOW_BASE + overflow.texts.len())
    .expect("the arena's overflow page is full");
  let text: Rc<str> = Rc::from(text);
  overflow.texts.push(Rc::clone(&text));
  overflow.symbols.insert(text, sym);
  overflow.fresh += 1;
  sym
}

/// A symbol's text, from the arena's buffer (under the caller's borrow) or the overflow page (its own `Rc`).
enum Text<'a> {
  Buffer(&'a str),
  Page(Rc<str>),
}

impl Deref for Text<'_> {
  type Target = str;
  #[inline]
  fn deref(&self) -> &str {
    match self {
      Text::Buffer(text) => text,
      Text::Page(text) => text,
    }
  }
}

#[inline]
fn text(arena: &Interner, sym: SymStr) -> Text<'_> {
  if is_paged(sym) {
    Text::Page(page_text(sym))
  } else {
    Text::Buffer(buffer_text(arena, sym))
  }
}

#[inline]
fn is_paged(sym: SymStr) -> bool { sym.to_usize() >= OVERFLOW_BASE }

/// The text of a symbol of the buffer (the callers' test of [`is_paged`] lets the compiler drop this one's).
#[inline]
fn buffer_text(arena: &Interner, sym: SymStr) -> &str {
  assert!(
    !is_paged(sym),
    "an overflow symbol resolved in the arena's buffer"
  );
  // SAFETY: `resolve_unchecked` skips the bounds and UTF-8 checks. Every symbol below `OVERFLOW_BASE` came from
  // the buffer's `get_or_intern` of a `&str`, and the buffer is append-only between `reset`s, so the symbol names a
  // whole UTF-8 string in it. Callgrind measured the validating `resolve` at ~3% of total Ir (`str::from_utf8`).
  unsafe { arena.resolve_unchecked(sym) }
}

#[cold]
#[inline(never)]
fn page_text(sym: SymStr) -> Rc<str> {
  Rc::clone(&OVERFLOW.borrow().texts[sym.to_usize() - OVERFLOW_BASE])
}

/// Assign a static str into the arena, returning a unique symbol.
pub fn pin_static(text: &'static str) -> SymStr { intern(text) }

/// The [`pin!`] call sites that cached a symbol on this thread: [`reset`]
/// replaces the arena and clears each, so a cached symbol never indexes the
/// replaced arena. A site registers on its first (cold) pin, so the hot path
/// stays one thread-local load.
#[thread_local]
static PINNED_SITES: RefCell<Vec<&'static std::thread::LocalKey<Cell<Option<SymStr>>>>> =
  RefCell::new(Vec::new());

/// A [`pin!`] call site's first pin after a [`reset`] (the cold path): pin the
/// literal, cache it in the site and register the site for the next reset.
#[doc(hidden)]
#[cold]
#[inline(never)]
pub fn pin_site(
  site: &'static std::thread::LocalKey<Cell<Option<SymStr>>>,
  text: &'static str,
) -> SymStr {
  let sym = pin_static(text);
  site.with(|c| c.set(Some(sym)));
  PINNED_SITES.borrow_mut().push(site);
  sym
}

/// Call-site-cached interning for string literals — the first call on
/// a thread (after an arena [`reset`]) pins the literal via `pin_static`,
/// later calls return the cached `SymStr` directly (a thread-local load,
/// no arena access). Use this from hot state-key lookup sites so you can keep
/// writing string literals at the call site and still skip the per-call
/// `pin()` hash probe:
///
/// ```ignore
/// if state::lookup_bool_sym(pin!("groupNonBoxing")) { ... }
/// ```
///
/// Each call site gets its own thread-local cache (no dedicated pub-static
/// constant per key), so there is no ergonomic cost beyond typing the macro
/// name; a site registers in `PINNED_SITES` on its first pin so that
/// [`reset`] can clear it.
///
/// Note: the macro `pin!` and the runtime-string function
/// `arena::pin(s)` share a name but occupy different namespaces in
/// Rust — `pin!(…)` is the macro, `pin(…)` is the function — so both
/// remain callable.
#[macro_export]
macro_rules! pin {
  ($s:literal) => {{
    std::thread_local! {
      static CACHED: std::cell::Cell<Option<$crate::common::arena::SymStr>> =
        const { std::cell::Cell::new(None) };
    }
    match CACHED.with(std::cell::Cell::get) {
      Some(sym) => sym,
      None => $crate::common::arena::pin_site(&CACHED, $s),
    }
  }};
}

/// Assign a string into the arena, returning a unique symbol.
///
/// No overflow guard below 2 GiB: the main-level wall-clock watchdog (watchdog.rs) catches genuinely runaway loops
/// long before any real-world workload approaches it. Earlier versions had both a call-count and a distinct-symbol
/// sentinel; the call-count one false-fired on dedup-heavy hot loops, and the distinct-symbol one added a per-call
/// `arena.len()` read on a hot path (~350k calls per doc). Neither cost was paying for itself.
pub fn pin<S: AsRef<str>>(text: S) -> SymStr { intern(text.as_ref()) }

/// Probe-only lookup: the symbol for `text` if it was ever interned, without
/// interning it. For existence-style checks (e.g. `install_definition`'s
/// `"{cs}:locked"` gate) a `None` here proves the key cannot be bound in any
/// state table — and skipping the intern avoids permanently growing the arena
/// with one probe-key twin per defined control sequence (2026-08-23 audit R6).
pub fn get<S: AsRef<str>>(text: S) -> Option<SymStr> {
  let text = text.as_ref();
  let found = ARENA.borrow().get(text).map(in_buffer);
  found.or_else(|| page_symbol(text))
}

/// The buffer [`get_suffixed`] and [`pin_qname`] spell their key in. Freed by [`reset`], as a `#[thread_local]` has no
/// destructor.
#[thread_local]
static SUFFIXED_KEY: RefCell<String> = RefCell::new(String::new());

/// [`get`] of `name` with `suffix` appended, without formatting a string: every definition asks this of its side keys
/// (`"{cs}:locked"`, `"{cs}:error@stub"`), and almost none of them exists.
pub fn get_suffixed(name: &str, suffix: &str) -> Option<SymStr> {
  let mut key = SUFFIXED_KEY.borrow_mut();
  key.clear();
  key.push_str(name);
  key.push_str(suffix);
  get(key.as_str())
}

/// [`pin`] of `prefix:local`, spelled in a reused buffer rather than a new `String`: `get_node_qname` asks it of every
/// element and attribute it names.
pub fn pin_qname(prefix: &str, local: &str) -> SymStr {
  let mut key = SUFFIXED_KEY.borrow_mut();
  key.clear();
  key.push_str(prefix);
  key.push(':');
  key.push_str(local);
  pin(key.as_str())
}

/// ASCII char-pin cache: every unique ASCII byte resolves to a single
/// SymStr for the lifetime of the thread's arena (append-only between
/// [`reset`]s, which clears the cache with it). Cache entries use `u32::MAX` as the "not yet pinned"
/// sentinel — every symbol value, the overflow page's included, is strictly below that.
/// Called from `lookup_catcode` / `assign_catcode` on every token,
/// so the RefCell + hashmap overhead on `pin` is a measurable cost
/// (1.4% Ir per callgrind on siunitx-heavy fixtures). The fast path
/// avoids the arena entirely for the common ASCII case.
#[thread_local]
static ASCII_CHAR_SYM: [Cell<u32>; 128] = [const { Cell::new(u32::MAX) }; 128];

pub fn pin_char(c: char) -> SymStr {
  let code = c as u32;
  if code < 128 {
    let cached = ASCII_CHAR_SYM[code as usize].get();
    if cached != u32::MAX {
      // (a prior `pin` below cached it, and `reset` clears the cache with the arena)
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

/// Resolve a symbol and call the closure with a `&str` reference. The closure may intern (`pin()`, `pin_char()`, …):
/// the text is lent under a shared borrow of the arena, so a string new during the lend goes to the overflow page
/// instead of growing (and moving) the buffer the text lives in (string-interner 0.20 `buffer.rs:144`).
/// An overflow symbol's lend holds the borrow too, so every lend is one where a [`reset`] panics.
#[inline]
pub fn with<R, FnR>(sym: SymStr, caller: FnR) -> R
where FnR: FnOnce(&str) -> R {
  let arena = ARENA.borrow();
  // (one branch, not `text()`'s two: the hottest lend)
  let page;
  let text = if is_paged(sym) {
    page = page_text(sym);
    &*page
  } else {
    buffer_text(&arena, sym)
  };
  caller(text)
}

/// The first `N` bytes of `sym`'s text, if it has as many, for a test by name on a hot path.
#[inline]
pub fn first_bytes<const N: usize>(sym: SymStr) -> Option<[u8; N]> {
  with(sym, |text| text.as_bytes().first_chunk().copied())
}

pub fn with2<R, FnR>(sym1: SymStr, sym2: SymStr, caller: FnR) -> R
where FnR: FnOnce(&str, &str) -> R {
  let arena = ARENA.borrow();
  caller(&text(&arena, sym1), &text(&arena, sym2))
}

pub fn with3<R, FnR>(sym1: SymStr, sym2: SymStr, sym3: SymStr, caller: FnR) -> R
where FnR: FnOnce(&str, &str, &str) -> R {
  let arena = ARENA.borrow();
  caller(
    &text(&arena, sym1),
    &text(&arena, sym2),
    &text(&arena, sym3),
  )
}

pub fn with_many<R, FnR>(syms: &[SymStr], caller: FnR) -> R
where FnR: FnOnce(Vec<&str>) -> R {
  let arena = ARENA.borrow();
  let texts: Vec<Text> = syms.iter().map(|sym| text(&arena, *sym)).collect();
  caller(texts.iter().map(|text| &**text).collect())
}

pub fn to_string(sym: SymStr) -> String { with(sym, str::to_owned) }

pub fn join(syms: &[SymStr], sep: &str) -> String { with_many(syms, |strs| strs.join(sep)) }

pub fn len() -> usize { ARENA.borrow().len() + OVERFLOW.borrow().texts.len() }

/// The number of strings the overflow page took (interned while a lend was live) since the last call: every conversion
/// entry point takes it at its start, and the conversion's end reports it as a Debug line.
pub fn take_overflow_interns() -> usize { std::mem::take(&mut OVERFLOW.borrow_mut().fresh) }

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
  // A lend holds a shared borrow until its closure returns, so a reset from inside one finds the arena borrowed.
  *ARENA
    .try_borrow_mut()
    .expect("arena::reset inside a lend (`arena::with`): the lent text would dangle") =
    new_interner();
  *OVERFLOW.borrow_mut() = Overflow::default();
  // The cached symbols index the replaced arena: a character or literal first
  // pinned mid-document would otherwise resolve to another string in the next
  // conversion on this thread.
  for cached in &ASCII_CHAR_SYM {
    cached.set(u32::MAX);
  }
  for site in PINNED_SITES.borrow_mut().drain(..) {
    site.with(|c| c.set(None));
  }
  *SUFFIXED_KEY.borrow_mut() = String::new();
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
pub(crate) fn force_init() { Lazy::force(&ARENA); }

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn pin_literal_cache_follows_an_arena_reset() {
    fn cached() -> SymStr { pin!("a literal pinned at a call site") }
    assert_eq!(to_string(cached()), "a literal pinned at a call site");
    reset();
    // A string interned first in the new arena takes the slot the literal had.
    pin("another string");
    assert_eq!(to_string(cached()), "a literal pinned at a call site");
  }

  #[test]
  fn pin_char_cache_follows_an_arena_reset() {
    assert_eq!(to_string(pin_char('~')), "~");
    reset();
    // A string interned first in the new arena takes the slot `~` had.
    pin("not a tilde");
    assert_eq!(to_string(pin_char('~')), "~");
  }

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

  /// A closure lent a text may intern: 3000 new strings must not move the lent text (the use-after-free of the former
  /// re-entrant `&mut` path, which grew the buffer under the lend; Miri flags it at the first new string).
  #[test]
  fn lend_survives_interning_many_new_strings() {
    // (counts are absolute: a test thread that converted, or ran another test, before must not shift them)
    reset();
    let lent = pin("arena_lend_text");
    let read = with(lent, |text| {
      for i in 0..3000 {
        pin(format!("arena_lend_new_{i}"));
      }
      text.to_owned()
    });
    assert_eq!(read, "arena_lend_text");
    for i in [0, 1499, 2999] {
      let sym = get(format!("arena_lend_new_{i}")).expect("interned under the lend");
      assert_eq!(to_string(sym), format!("arena_lend_new_{i}"));
    }
    assert_eq!(take_overflow_interns(), 3000);
    assert_eq!(take_overflow_interns(), 0, "the count is read once");
  }

  /// The same with more new bytes than the buffer's whole capacity (131,072 × 13 bytes, string-interner 0.20
  /// `buffer.rs:161-169`), which would move it natively, not only under Miri, had they gone into it: the lent text
  /// still reads, from where it was. Guards the current arena only (the old one is gone; red under Miri on it: the
  /// test above). A native test: its 1.7 MB take Miri 2 min under Stacked and 13 under Tree Borrows.
  #[test]
  #[cfg_attr(miri, ignore)]
  fn lend_survives_more_new_bytes_than_the_buffer_holds() {
    reset();
    let lent = pin("arena_move_text");
    let filler = "x".repeat(1 << 15);
    let count = 131_072 * 13 / filler.len() + 2;
    let (read, unmoved) = with(lent, |text| {
      let at = text.as_ptr();
      for i in 0..count {
        pin(format!("{i}{filler}"));
      }
      (text.to_owned(), with(lent, |again| again.as_ptr()) == at)
    });
    assert_eq!(read, "arena_move_text");
    assert!(unmoved, "the buffer moved under the lend");
    assert_eq!(to_string(pin(format!("{}{filler}", count - 1))).len(), filler.len() + 2);
    assert_eq!(take_overflow_interns(), count);
  }

  /// Nested lends, and a pin of a string the buffer already holds, which keeps its buffer symbol.
  #[test]
  fn nested_lends_and_a_pin_of_an_existing_string() {
    let outer = pin("arena_nest_outer");
    let inner = pin("arena_nest_inner");
    let joined = with(outer, |o| {
      with2(inner, outer, |i, o2| {
        assert_eq!(
          pin("arena_nest_inner"),
          inner,
          "an existing string keeps its symbol under a lend"
        );
        format!("{o}/{i}/{o2}/{}", to_string(inner))
      })
    });
    assert_eq!(
      joined,
      "arena_nest_outer/arena_nest_inner/arena_nest_outer/arena_nest_inner"
    );
    assert_eq!(take_overflow_interns(), 0, "no string was new");
  }

  /// `get` under a lend sees both tiers and interns nothing.
  #[test]
  fn get_under_a_lend() {
    let lent = pin("arena_get_lent");
    let before = len();
    let (buffered, paged, absent) = with(lent, |_| {
      let paged = pin("arena_get_paged");
      (
        get("arena_get_lent"),
        (paged, get("arena_get_paged")),
        get("arena_get_absent"),
      )
    });
    assert_eq!(buffered, Some(lent));
    assert_eq!(paged.1, Some(paged.0));
    assert_eq!(absent, None);
    assert_eq!(len(), before + 1);
  }

  /// A string first interned under a lend keeps its overflow symbol when pinned again outside any lend, and its text
  /// reads the same through every entry point.
  #[test]
  fn an_overflow_string_dedups_across_tiers() {
    reset();
    let lent = pin("arena_dedup_lent");
    let paged = with(lent, |_| pin("arena_dedup_paged"));
    assert_eq!(
      pin("arena_dedup_paged"),
      paged,
      "the later pin outside the lend finds the overflow string"
    );
    assert_eq!(pin_static("arena_dedup_paged"), paged);
    assert_eq!(get("arena_dedup_paged"), Some(paged));
    assert_eq!(to_string(paged), "arena_dedup_paged");
    assert_eq!(first_bytes::<3>(paged), Some(*b"are"));
    assert_eq!(
      with3(lent, paged, lent, |a, b, c| format!("{a}{b}{c}")),
      "arena_dedup_lentarena_dedup_pagedarena_dedup_lent"
    );
    assert_eq!(
      join(&[paged, lent], ","),
      "arena_dedup_paged,arena_dedup_lent"
    );
    // Outside a lend with the overflow page in use, a new string still goes to the buffer, also one of the paged one's
    // length and last byte.
    let buffered = pin("arena_dedup_after");
    assert!(!is_paged(buffered));
    assert_eq!(to_string(buffered), "arena_dedup_after");
    let twin = pin("arena_dedup_pagdd");
    assert!(!is_paged(twin));
    assert_eq!(get("arena_dedup_pagdd"), Some(twin));
    assert_eq!(to_string(twin), "arena_dedup_pagdd");
    assert_eq!(take_overflow_interns(), 1);
  }

  /// `reset` empties both tiers; the overflow page's symbols start over.
  #[test]
  fn reset_clears_the_overflow_page() {
    reset();
    let lent = pin("arena_reset_lent");
    let paged = with(lent, |_| pin("arena_reset_paged"));
    assert_eq!(len(), 2);
    reset();
    assert_eq!(len(), 0);
    assert_eq!(get("arena_reset_paged"), None);
    assert_eq!(take_overflow_interns(), 0);
    let lent = pin("arena_reset_lent2");
    assert_eq!(
      with(lent, |_| pin("arena_reset_paged2")),
      paged,
      "the first overflow symbol again"
    );
    assert_eq!(to_string(paged), "arena_reset_paged2");
  }

  /// `reset` inside a lend would leave the lent text dangling: it refuses.
  #[test]
  #[should_panic(expected = "arena::reset inside a lend")]
  fn reset_inside_a_lend_panics() {
    let lent = pin("arena_reset_inside");
    with(lent, |_| reset());
  }

  /// So does `reset` inside the lend of an overflow symbol, whose text is its own `Rc`: every lend borrows the arena.
  #[test]
  #[should_panic(expected = "arena::reset inside a lend")]
  fn reset_inside_an_overflow_lend_panics() {
    let lent = pin("arena_reset_inside_page");
    let paged = with(lent, |_| pin("arena_reset_inside_paged"));
    with(paged, |_| reset());
  }
}
