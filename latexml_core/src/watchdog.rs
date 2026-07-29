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
//! [`Watchdog::with_limits`](crate::watchdog::Watchdog::with_limits) guards
//! **both** a wall-clock deadline and a
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
//! The wall-clock guard is portable (`std::thread` + `Instant`).
//!
//! **Portable:** [`total_memory_bytes`](crate::watchdog::total_memory_bytes) and [`available_disk_bytes`](crate::watchdog::available_disk_bytes) answer on
//! Linux, macOS and Windows — `sysconf(_SC_PHYS_PAGES)`/`statvfs` are POSIX, and
//! Windows uses `GlobalMemoryStatusEx`/`GetDiskFreeSpaceExW`. They back the
//! machine-derived default ceiling ([`default_ceiling_mib`](crate::watchdog::default_ceiling_mib)) and the
//! spill-headroom check, so those behave identically on every supported OS.
//!
//! **Still Linux-only:** [`process_rss_kb`](crate::watchdog::process_rss_kb) samples `/proc/self/status`, so the
//! *enforcement* half of the RAM guard is inactive elsewhere — the ceiling is
//! computed correctly but never checked, and only the time guard bites. Closing
//! this needs macOS `task_info(TASK_BASIC_INFO)` and Windows
//! `GetProcessMemoryInfo`; both are reachable through the crates this module
//! already links, so it is a bounded follow-up rather than a new dependency
//! question.

use std::{
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
  thread,
  time::{Duration, Instant},
};

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

/// Total physical RAM on this machine in bytes, or `None` if it cannot be
/// determined.
///
/// Portable by construction: `sysconf(_SC_PHYS_PAGES) * sysconf(_SC_PAGE_SIZE)`
/// is POSIX and answers on both Linux and macOS, and Windows has
/// `GlobalMemoryStatusEx`. No new dependency — `libc` and `windows-sys` are
/// already in the tree.
///
/// Deliberately NOT `/proc/meminfo`: that would repeat the Linux-only mistake
/// this module already carries in [`process_rss_kb`], which returns `None`
/// everywhere else and so silently deactivates the memory ceiling on
/// macOS/Windows.
pub fn total_memory_bytes() -> Option<u64> {
  #[cfg(unix)]
  {
    // SAFETY: `sysconf` is a pure query with no pointer arguments; a negative
    // return means "unavailable", which we map to `None` rather than trusting.
    let pages = unsafe { libc::sysconf(libc::_SC_PHYS_PAGES) };
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if pages > 0 && page_size > 0 {
      return (pages as u64).checked_mul(page_size as u64);
    }
    None
  }
  #[cfg(windows)]
  {
    use windows_sys::Win32::System::SystemInformation::{
      GLOBAL_MEMORY_STATUS_EX, GlobalMemoryStatusEx,
    };
    let mut status: GLOBAL_MEMORY_STATUS_EX = unsafe { std::mem::zeroed() };
    status.dwLength = std::mem::size_of::<GLOBAL_MEMORY_STATUS_EX>() as u32;
    // SAFETY: `status` is a correctly sized, zeroed struct with `dwLength` set,
    // exactly as the API requires.
    if unsafe { GlobalMemoryStatusEx(&mut status) } != 0 {
      return Some(status.ullTotalPhys);
    }
    None
  }
  #[cfg(not(any(unix, windows)))]
  {
    None
  }
}

/// The default per-conversion memory ceiling in MiB, derived from the machine.
///
/// `min(64 GiB, 90 % of physical RAM)`, falling back to
/// [`FALLBACK_CEILING_MIB`] when the machine cannot be probed.
///
/// The previous default was a flat 6144 MiB regardless of hardware, which is
/// simultaneously absurd on a 256 GB host (a conversion that would comfortably
/// fit is refused) and over-generous on an 8 GB laptop (the box starts swapping
/// before the guard ever fires). Both halves of the rule matter:
///
/// * **90 % of RAM** leaves the OS and everything else on the machine a share.
///   A ceiling at 100 % is a promise the kernel will not keep.
/// * **the 64 GiB cap** is not about this machine but about the *others*: in a
///   parallel fleet the aggregate is `N_processes x ceiling`, so an uncapped
///   fraction-of-RAM rule on a big host would let a busy fleet OOM it. The
///   `cortex_worker` fleet overrides this with its own per-child ceiling
///   anyway; the cap keeps the single-process default from being reckless.
pub fn default_ceiling_mib() -> u64 {
  const MIB: u64 = 1024 * 1024;
  match total_memory_bytes() {
    Some(total) => {
      let ninety_percent = total / MIB * 9 / 10;
      ninety_percent.clamp(1, MAX_DEFAULT_CEILING_MIB)
    },
    None => FALLBACK_CEILING_MIB,
  }
}

/// Upper bound on the machine-derived default ceiling (64 GiB in MiB).
pub const MAX_DEFAULT_CEILING_MIB: u64 = 64 * 1024;

/// Ceiling used when the machine's RAM cannot be probed — the historical flat
/// default, kept so an unprobeable platform behaves exactly as before rather
/// than losing its guard entirely.
pub const FALLBACK_CEILING_MIB: u64 = 6144;

/// Free space in bytes on the filesystem holding `path`, or `None` if it cannot
/// be determined. Used to check headroom before spilling intermediates to disk.
pub fn available_disk_bytes(path: &std::path::Path) -> Option<u64> {
  #[cfg(unix)]
  {
    use std::os::unix::ffi::OsStrExt;
    let c_path = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    // SAFETY: zeroed `statvfs` is a valid initial state; `c_path` is a
    // NUL-terminated string that outlives the call.
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) } != 0 {
      return None;
    }
    // `f_bavail` is blocks available to unprivileged users — the honest figure,
    // as `f_bfree` includes the root-reserved slice we cannot use.
    (stat.f_bavail as u64).checked_mul(stat.f_frsize as u64)
  }
  #[cfg(windows)]
  {
    use std::os::windows::ffi::OsStrExt;

    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide.push(0);
    let mut free_to_caller: u64 = 0;
    // SAFETY: `wide` is NUL-terminated and outlives the call; the two unused
    // out-parameters are passed as null, which the API permits.
    let ok = unsafe {
      GetDiskFreeSpaceExW(
        wide.as_ptr(),
        &mut free_to_caller,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
      )
    };
    (ok != 0).then_some(free_to_caller)
  }
  #[cfg(not(any(unix, windows)))]
  {
    let _ = path;
    None
  }
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
  if let Some(cell) = PRE_EXIT_HOOK.get()
    && let Ok(mut guard) = cell.lock()
    && let Some(hook) = guard.take()
  {
    hook();
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
      if let Some(deadline) = deadline
        && Instant::now() >= deadline
      {
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
      if max_rss_kb > 0
        && let Some(rss) = process_rss_kb()
        && rss > max_rss_kb
      {
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

  /// The ceiling is derived from the machine, so the machine must be probeable
  /// on every platform we ship. A `None` here means the default silently falls
  /// back to a flat number and the ceiling stops tracking the host — the exact
  /// failure `process_rss_kb` already has on non-Linux.
  #[test]
  #[cfg(any(unix, windows))]
  fn total_memory_is_probeable_and_plausible() {
    let total = total_memory_bytes().expect("physical RAM must be probeable on unix/windows");
    // No real machine we support has under 256 MiB, and none has over 100 TiB;
    // a value outside that says the units are wrong, not that the host is odd.
    assert!(
      total > 256 * 1024 * 1024,
      "implausibly small total RAM ({total} bytes) — check the unit conversion"
    );
    assert!(
      total < 100 * 1024 * 1024 * 1024 * 1024,
      "implausibly large total RAM ({total} bytes) — check the unit conversion"
    );
  }

  /// `min(64 GiB, 90 % of RAM)`, and never zero — a zero ceiling would mean
  /// "no limit" downstream (`resolve_rss_cap`), inverting the intent.
  #[test]
  fn default_ceiling_respects_both_halves_of_the_rule() {
    let ceiling = default_ceiling_mib();
    assert!(
      ceiling > 0,
      "a derived ceiling of 0 would read as 'unlimited'"
    );
    assert!(
      ceiling <= MAX_DEFAULT_CEILING_MIB,
      "derived {ceiling} MiB exceeds the {MAX_DEFAULT_CEILING_MIB} MiB cap; on a \
       large host an uncapped fraction-of-RAM default would let a parallel fleet \
       (N_processes x ceiling) OOM the machine"
    );
    if let Some(total) = total_memory_bytes() {
      let ninety = total / (1024 * 1024) * 9 / 10;
      assert_eq!(
        ceiling,
        ninety.clamp(1, MAX_DEFAULT_CEILING_MIB),
        "ceiling should be min(cap, 90 % of RAM)"
      );
      assert!(
        ceiling <= total / (1024 * 1024),
        "the ceiling must leave the OS a share of physical RAM"
      );
    } else {
      assert_eq!(ceiling, FALLBACK_CEILING_MIB);
    }
  }

  /// Free-disk probing backs the spill-headroom check; if it cannot answer we
  /// would have to spill blind.
  #[test]
  #[cfg(any(unix, windows))]
  fn available_disk_is_probeable() {
    let free = available_disk_bytes(std::path::Path::new("."))
      .expect("free space must be probeable on unix/windows");
    assert!(
      free > 0,
      "reported zero free space on the working directory"
    );
  }
}
