//! Thread-spawn helper that degrades instead of panicking.
//!
//! A bare `std::thread::spawn` (and `Builder::spawn(..).expect(..)`) panics when
//! the OS refuses a new thread — `EAGAIN`/`WouldBlock` under fleet thread or
//! memory exhaustion. On the cortex fleet those panics surfaced as
//! `imageprocessing:worker_panicked` (a per-paper conversion aborted) and, at
//! the top-level and kpathsea-prewarm spawns, as whole-document failures. For
//! best-effort background work — a stderr drain, the kpathsea prewarm — a
//! missing thread is harmless, so the spawn should degrade to `None`.

use std::thread::{Builder, JoinHandle};

/// Spawn `f` on a named thread, returning `None` (instead of panicking) if the
/// OS refuses the thread. Use only where a missing thread is safe: the work is
/// best-effort and its absence changes no output. Load-bearing threads must
/// keep propagating the error.
pub fn try_spawn_or_degrade<F, T>(name: &str, f: F) -> Option<JoinHandle<T>>
where
  F: FnOnce() -> T + Send + 'static,
  T: Send + 'static,
{
  Builder::new().name(name.to_owned()).spawn(f).ok()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn spawns_and_returns_a_handle() {
    let h = try_spawn_or_degrade("test-worker", || 21 * 2);
    assert_eq!(
      h.expect("a thread spawns under normal limits")
        .join()
        .unwrap(),
      42
    );
  }
}
