//! Configurable native-stack growth guard for deeply-recursive digestion.
//!
//! Some inputs recurse through the engine far deeper than a normal document:
//! gullet macro expansion (a number-argument macro whose argument is read by
//! expanding the next number-argument macro — xint's `\XINT_…` chains nest tens
//! of thousands deep), the document tree walk, and the math-tree walk. Left
//! unguarded these overflow the (large but finite) conversion-thread stack and
//! **abort the process** (SIGABRT) — whereas Perl degrades gracefully via its
//! `$MAXSTACK` guard. Each such site therefore grows the native stack on demand
//! with [`stacker::maybe_grow`].
//!
//! This module is the single home for the two parameters those calls share, so
//! they are tuned in **one** place and are **configurable at runtime** rather
//! than hardcoded:
//! - **red zone** — grow once fewer than this many bytes of stack remain.
//! - **segment**  — the size of each freshly-allocated stack chunk.
//!
//! Resolution precedence (highest first): an explicit [`set_red_zone_bytes`] /
//! [`set_segment_bytes`] (e.g. from a future `--stack-…` CLI flag) → the env
//! var ([`ENV_RED_ZONE`] / [`ENV_SEGMENT`], a plain byte count) → the compiled
//! default. Call [`maybe_grow`] at every deeply-recursive site instead of
//! `stacker::maybe_grow` directly.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Default red zone: grow when within this many bytes of the stack end.
/// 256 KiB leaves ample margin above any single recursion frame.
pub const DEFAULT_RED_ZONE_BYTES: usize = 256 * 1024;

/// Default growth segment: bytes of fresh stack allocated per growth step.
/// 8 MiB amortizes the allocation across many recursion levels.
pub const DEFAULT_SEGMENT_BYTES: usize = 8 * 1024 * 1024;

/// Env override for the red zone — a plain byte count (e.g. `262144`).
pub const ENV_RED_ZONE: &str = "LATEXML_STACK_RED_ZONE_BYTES";

/// Env override for the growth segment — a plain byte count (e.g. `8388608`).
pub const ENV_SEGMENT: &str = "LATEXML_STACK_SEGMENT_BYTES";

// 0 is the "unresolved" sentinel: the value is resolved from env-or-default on
// first read and cached. `set_*` stores a non-zero override that wins thereafter.
static RED_ZONE: AtomicUsize = AtomicUsize::new(0);
static SEGMENT: AtomicUsize = AtomicUsize::new(0);

fn resolve(slot: &AtomicUsize, env_key: &str, default: usize) -> usize {
  match slot.load(Ordering::Relaxed) {
    0 => {
      let value = std::env::var(env_key)
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|&v| v != 0)
        .unwrap_or(default);
      // Benign race: concurrent first-readers resolve to the identical value.
      slot.store(value, Ordering::Relaxed);
      value
    },
    value => value,
  }
}

/// Bytes of remaining stack below which [`maybe_grow`] allocates a new segment.
#[inline]
pub fn red_zone_bytes() -> usize { resolve(&RED_ZONE, ENV_RED_ZONE, DEFAULT_RED_ZONE_BYTES) }

/// Size, in bytes, of each freshly-allocated stack segment.
#[inline]
pub fn segment_bytes() -> usize { resolve(&SEGMENT, ENV_SEGMENT, DEFAULT_SEGMENT_BYTES) }

/// Override the red zone (e.g. from a CLI flag). Set before any conversion;
/// takes precedence over the env var and the default.
pub fn set_red_zone_bytes(bytes: usize) { RED_ZONE.store(bytes.max(1), Ordering::Relaxed); }

/// Override the growth segment (e.g. from a CLI flag). Set before any
/// conversion; takes precedence over the env var and the default.
pub fn set_segment_bytes(bytes: usize) { SEGMENT.store(bytes.max(1), Ordering::Relaxed); }

/// Grow the native call stack on demand, then run `f`. The single wrapper every
/// deeply-recursive site should call so the guard parameters live in one place.
/// Transparent: it only provides more stack when near the limit; it never
/// changes results.
#[inline]
pub fn maybe_grow<R>(f: impl FnOnce() -> R) -> R {
  stacker::maybe_grow(red_zone_bytes(), segment_bytes(), f)
}
