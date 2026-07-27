//! Test-only helpers for the two process-global resources this crate's
//! unit tests touch: the **environment** and **temporary directories**.
//!
//! `cargo test` runs every test in one binary on many threads, so both
//! are shared mutable state. Two concrete hazards this module closes:
//!
//! * **Env-var interleaving.** `graphics::tests` points `PATH` at a fake
//!   converter, and `graphics_cache::tests` sets
//!   `LATEXML_GRAPHICS_CACHE_DIR`. Rust's own `ENV_LOCK` makes the
//!   individual `set_var`/`var` calls safe against each other, but it does
//!   nothing about *ordering*: without a lock spanning the whole
//!   set → use → restore window, which test observes which value is
//!   scheduler-dependent. [`EnvGuard`] holds [`env_lock`] for that whole
//!   window, so the interleaving is gone rather than merely unlikely.
//!
//! * **Leaked temp dirs.** A fixed-name dir under `/tmp` collides between
//!   two concurrent `cargo test` runs, and a dir cleaned up only on the
//!   success path accumulates forever once a test starts failing.
//!   [`TempDir`] gives each call a unique name and removes it in `Drop`,
//!   which runs on the panic path too.
//!
//! Residual, deliberately not papered over: a C library linked into the
//! test binary (libxml2, kpathsea) can call `getenv` concurrently with a
//! `setenv` here, which no Rust-side lock can serialise. That is why the
//! graphics cache is steered by an explicit
//! [`CachePolicy`](crate::graphics_cache::CachePolicy) argument rather
//! than by an env var — correctness must not rest on this lock.

use std::{
  path::{Path, PathBuf},
  sync::{Mutex, MutexGuard, OnceLock},
};

/// The lock serialising every environment mutation in this test binary.
///
/// Anything that calls `set_var`/`remove_var` must hold it — normally by
/// going through [`EnvGuard`].
pub(crate) fn env_lock() -> &'static Mutex<()> {
  static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
  LOCK.get_or_init(|| Mutex::new(()))
}

/// Sets environment variables for the duration of a test and restores
/// their previous values on drop — holding [`env_lock`] throughout, so no
/// other env-mutating test can observe or clobber the window.
///
/// Acquire once and call [`set`](EnvGuard::set) for each variable: the
/// lock is **not** reentrant, so two live guards on one thread would
/// deadlock.
pub(crate) struct EnvGuard {
  /// Held for the guard's lifetime; released in `Drop` after the restore.
  _lock: MutexGuard<'static, ()>,
  /// `(key, previous value)`, restored in reverse order.
  saved: Vec<(String, Option<String>)>,
}

impl EnvGuard {
  /// Take the env lock. Recovers from a poisoned mutex: a panicking test
  /// elsewhere must not cascade into unrelated failures here, and the
  /// guarded data is `()`, so there is no broken invariant to inherit.
  pub(crate) fn acquire() -> Self {
    let lock = env_lock().lock().unwrap_or_else(|e| e.into_inner());
    Self { _lock: lock, saved: Vec::new() }
  }

  /// Set `key` to `value`, remembering the prior value for restoration.
  pub(crate) fn set(&mut self, key: &str, value: &str) {
    self.saved.push((key.to_string(), std::env::var(key).ok()));
    // SAFETY: `set_var` is `unsafe` in edition 2024 because a concurrent
    // env read/write from another thread is a data race. We hold
    // `env_lock` for the guard's whole lifetime, so no other env-mutating
    // test in this binary runs concurrently, and Rust's internal
    // `ENV_LOCK` covers `std::env::var` readers.
    unsafe { std::env::set_var(key, value) };
  }
}

impl Drop for EnvGuard {
  fn drop(&mut self) {
    // Reverse order, so repeated sets of one key unwind to the original.
    for (key, old) in self.saved.drain(..).rev() {
      match old {
        // SAFETY: restoring a value this guard saved, still under
        // `env_lock`; same rationale as `set`.
        Some(v) => unsafe { std::env::set_var(&key, v) },
        // SAFETY: removing a var this guard introduced; same rationale.
        None => unsafe { std::env::remove_var(&key) },
      }
    }
  }
}

/// A uniquely-named temporary directory that deletes itself on drop.
///
/// Unlike a fixed `env::temp_dir().join("some_test")`, the name embeds
/// the pid and a monotonic counter, so concurrent `cargo test` runs (and
/// concurrent tests within one run) never share a directory. `Drop` runs
/// on the unwind path, so a failing assertion cleans up too — the reason
/// `/tmp` used to fill with `latexml_graphics_dedupe_*` leftovers.
pub(crate) struct TempDir {
  path: PathBuf,
}

impl TempDir {
  /// Create `<tmp>/latexml_post_<label>_<pid>_<n>`, failing loudly — a
  /// test that cannot get its fixture directory must not continue.
  pub(crate) fn new(label: &str) -> Self {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path =
      std::env::temp_dir().join(format!("latexml_post_{label}_{}_{n}", std::process::id()));
    // A leftover from a recycled pid would otherwise leak stale files
    // into this run's fixture.
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path)
      .unwrap_or_else(|e| panic!("could not create temp dir {}: {e}", path.display()));
    Self { path }
  }

  pub(crate) fn path(&self) -> &Path { &self.path }

  /// Path to `name` inside this directory.
  pub(crate) fn join(&self, name: &str) -> PathBuf { self.path.join(name) }
}

impl Drop for TempDir {
  fn drop(&mut self) {
    // Best-effort: a failure to clean up must never mask the test's own
    // outcome (and would double-panic during unwind).
    let _ = std::fs::remove_dir_all(&self.path);
  }
}
