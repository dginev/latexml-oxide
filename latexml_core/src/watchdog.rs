//! Wall-clock watchdog that forcibly aborts the process after a deadline.
//!
//! The existing `stomach::check_timeout()` is a cooperative mechanism — it only
//! fires when the digestion loop polls it. That leaves tight native loops
//! (Marpa precompute / parse, libxml2 post-processing, FFI calls into libxslt,
//! ...) completely unguarded: a 60-second timeout can easily turn into 10
//! minutes if control never returns to the digestion loop.
//!
//! This module provides a main-level `Watchdog` that spawns a dedicated thread
//! at construction, wakes after the specified number of seconds, and — if the
//! watchdog has not yet been cancelled — prints a message and calls
//! `std::process::abort()`. That guarantees the process dies within
//! `timeout + poll_interval` of the configured deadline, regardless of what
//! the main thread is doing.
//!
//! # Design notes
//!
//! - Uses `Arc<AtomicBool>` for cancellation. Polling every 100 ms keeps the cancellation latency
//!   low without burning CPU.
//! - `Drop` on the `Watchdog` handle cancels the watchdog thread, so RAII usage (`let _wd =
//!   Watchdog::new(secs)`) is sufficient.
//! - We use `std::process::abort()` rather than `panic!` because panic may unwind or be caught by a
//!   surrounding `catch_unwind`, which would defeat the safety guarantee. `abort()` delivers
//!   `SIGABRT` and always terminates the process.
//! - The existing cooperative `stomach::check_timeout()` path is retained: on most conversions it
//!   fires before the hard abort, giving callers a nice `Err(Fatal)` with proper error propagation.
//!   The watchdog is a safety net for the pathological cases where cooperative polling doesn't
//!   happen.
//!
//! # Resource limits
//!
//! [`Watchdog::with_limits`] guards **both** a wall-clock deadline and a
//! resident-memory ceiling — the two defenses any executable that converts
//! arbitrary input needs. It is the shared guard reused by both
//! `cortex_worker` (in-process, one paper per process) and the
//! `latexml_oxide --server` LSP (run inside each forked body child, which
//! self-terminates on breach so the parent reaps it via pipe EOF). The exit
//! codes are distinct so a supervising parent can tell them apart:
//! `124` = wall-clock timeout, `137` = memory ceiling.
//!
//! # Portability
//!
//! The wall-clock guard is portable (`std::thread` + `Instant`). The RAM guard
//! samples RSS from `/proc/self/status` and is therefore **Linux-only** today;
//! on other platforms [`process_rss_kb`] returns `None` and the memory ceiling
//! is silently inactive (the time guard still works). LONG-TERM the guards
//! should be fully portable — macOS via `task_info(TASK_BASIC_INFO)` and
//! Windows via `GetProcessMemoryInfo` — so every supported OS gets the same
//! reliable time + RAM defenses. Tracked as a portability follow-up.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

/// Current resident set size of this process in KiB, or `None` if it can't be
/// determined (non-Linux, or `/proc` unavailable). Reads `VmRSS` from
/// `/proc/self/status`. Cheap enough to poll a few times a second.
pub fn process_rss_kb() -> Option<u64> {
  let status = std::fs::read_to_string("/proc/self/status").ok()?;
  for line in status.lines() {
    if let Some(rest) = line.strip_prefix("VmRSS:") {
      return rest.split_whitespace().next()?.parse::<u64>().ok();
    }
  }
  None
}

/// Exit code used when the wall-clock deadline is exceeded (standard `timeout`).
pub const EXIT_TIMEOUT: i32 = 124;
/// Exit code used when the memory ceiling is exceeded (128 + SIGKILL).
pub const EXIT_OOM: i32 = 137;

/// Handle to a watchdog thread. Cancels on drop.
///
/// `Watchdog::new(0)` is a no-op — produces a handle that does nothing. This
/// lets call-sites set a watchdog conditionally without special-casing the
/// "no timeout" branch.
/// Optional hook to run from the watchdog thread immediately before
/// `exit(124)`. Used by `cortex_worker --standalone` to write a
/// structured `Status:conversion:3` placeholder to `--output` so the
/// timeout produces a usable failure artifact instead of a missing
/// file. Set once at startup via `set_pre_exit_hook`; the hook is
/// invoked exactly once. Zero overhead on the happy path — only the
/// watchdog firing reads it.
type PreExitHook = Box<dyn FnOnce() + Send + 'static>;

static PRE_EXIT_HOOK: std::sync::OnceLock<std::sync::Mutex<Option<PreExitHook>>> =
  std::sync::OnceLock::new();

pub fn set_pre_exit_hook(hook: PreExitHook) {
  let cell = PRE_EXIT_HOOK.get_or_init(|| std::sync::Mutex::new(None));
  if let Ok(mut guard) = cell.lock() {
    *guard = Some(hook);
  }
}

fn run_pre_exit_hook() {
  if let Some(cell) = PRE_EXIT_HOOK.get() {
    if let Ok(mut guard) = cell.lock() {
      if let Some(hook) = guard.take() {
        hook();
      }
    }
  }
}

pub struct Watchdog {
  cancelled: Arc<AtomicBool>,
}

impl Watchdog {
  /// Create a wall-clock-only watchdog. `timeout_secs = 0` disables it.
  /// Equivalent to [`Watchdog::with_limits(timeout_secs, 0)`].
  pub fn new(timeout_secs: u64) -> Self { Self::with_limits(timeout_secs, 0) }

  /// Create a watchdog guarding a wall-clock deadline **and** a resident-memory
  /// ceiling. `timeout_secs = 0` disables the time guard; `max_rss_kb = 0`
  /// disables the memory guard. With both `0` this is a no-op handle.
  ///
  /// The thread polls `cancelled`, the deadline, and RSS every `poll_interval`.
  /// On a time breach it exits [`EXIT_TIMEOUT`]; on a memory breach,
  /// [`EXIT_OOM`]. The memory guard is inactive where [`process_rss_kb`]
  /// returns `None` (non-Linux); see the module portability note.
  pub fn with_limits(timeout_secs: u64, max_rss_kb: u64) -> Self {
    let cancelled = Arc::new(AtomicBool::new(false));
    if timeout_secs > 0 || max_rss_kb > 0 {
      let c = cancelled.clone();
      thread::Builder::new()
        .name("latexml-watchdog".to_string())
        .spawn(move || Self::run(c, timeout_secs, max_rss_kb))
        .expect("watchdog thread spawn failed");
    }
    Self { cancelled }
  }

  fn run(cancelled: Arc<AtomicBool>, timeout_secs: u64, max_rss_kb: u64) {
    let deadline = (timeout_secs > 0).then(|| Instant::now() + Duration::from_secs(timeout_secs));
    let poll_interval = Duration::from_millis(100);
    loop {
      if cancelled.load(Ordering::Relaxed) {
        return; // cancelled: graceful exit.
      }
      if let Some(deadline) = deadline {
        if Instant::now() >= deadline {
          if cancelled.load(Ordering::Relaxed) {
            return;
          }
          eprintln!(
            "Fatal:timeout:wallclock latexml-oxide: main-level wall-clock timeout after {timeout_secs}s — exiting process"
          );
          // Run the optional pre-exit hook (e.g. cortex_worker writing a
          // structured Status:conversion:3 placeholder to its --output path)
          // BEFORE exiting. The hook is invoked at most once per process.
          run_pre_exit_hook();
          // `std::process::exit(124)` instead of `abort()`: the watchdog must
          // terminate the whole process (the worker thread is presumed wedged
          // in a tight loop that won't observe a cooperative cancel), but
          // `abort()` produces a "Aborted (core dumped)" SIGABRT trace from
          // the shell. `exit(124)` (standard timeout exit code) runs atexit
          // handlers, flushes stderr, and leaves a clean exit signal the
          // parent harness can interpret as "paper timed out" without
          // conflating it with a Rust panic / memory corruption. Witnesses:
          // 2602.11915, 2604.11500, 2604.13944, hep-ph9205242, q-alg9604005,
          // q-alg9605003, q-alg9605028 — the 7 "Aborted" rows in the
          // 2026-05-13 588-paper sweep.
          std::process::exit(EXIT_TIMEOUT);
        }
      }
      if max_rss_kb > 0 {
        if let Some(rss) = process_rss_kb() {
          if rss > max_rss_kb {
            if cancelled.load(Ordering::Relaxed) {
              return;
            }
            eprintln!(
              "Fatal:oom:rss latexml-oxide: resident memory {}MB exceeded the {}MB ceiling — exiting process",
              rss / 1024,
              max_rss_kb / 1024
            );
            run_pre_exit_hook();
            std::process::exit(EXIT_OOM);
          }
        }
      }
      thread::sleep(poll_interval);
    }
  }

  /// Explicitly cancel the watchdog. Idempotent.
  pub fn cancel(&self) { self.cancelled.store(true, Ordering::Relaxed); }
}

impl Drop for Watchdog {
  fn drop(&mut self) { self.cancel(); }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn watchdog_zero_timeout_is_noop() {
    // timeout_secs=0 should NOT spawn a thread and NOT abort.
    let w = Watchdog::new(0);
    assert!(
      !w.cancelled.load(Ordering::Relaxed),
      "initial cancelled state is false"
    );
    // Dropping is safe — there's no live thread to interact with.
    drop(w);
  }

  #[test]
  fn watchdog_cancel_is_idempotent() {
    let w = Watchdog::new(60);
    w.cancel();
    assert!(w.cancelled.load(Ordering::Relaxed));
    // Calling again is a no-op.
    w.cancel();
    assert!(w.cancelled.load(Ordering::Relaxed));
  }

  #[test]
  fn watchdog_drop_cancels() {
    let cancelled_ref = {
      let w = Watchdog::new(60);
      // Grab a reference to the atomic so we can inspect post-drop.
      w.cancelled.clone()
    }; // w dropped here
    assert!(
      cancelled_ref.load(Ordering::Relaxed),
      "drop should set cancelled=true"
    );
  }

  #[test]
  fn watchdog_explicit_cancel_before_drop() {
    // Pre-drop cancellation is also reflected on the clone.
    let w = Watchdog::new(60);
    let cancelled_ref = w.cancelled.clone();
    w.cancel();
    assert!(cancelled_ref.load(Ordering::Relaxed));
    // Explicit drop after cancel remains idempotent.
    drop(w);
    assert!(cancelled_ref.load(Ordering::Relaxed));
  }

  #[test]
  fn watchdog_long_timeout_doesnt_fire_quickly() {
    // 60-second timeout shouldn't fire during a 50 ms sleep.
    let _w = Watchdog::new(60);
    thread::sleep(Duration::from_millis(50));
    // If the watchdog had fired, we'd be dead. We made it here → fine.
  }
}
