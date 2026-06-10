//! Windowed cycle-detection infinite-loop guard.
//!
//! A general defense layer that complements the coarse size/count limits
//! (`Gullet::token_limit` / `pushback_limit`) and the outermost RSS soft cap
//! (`stomach::check_timeout`). Where those catch a runaway only after it has
//! consumed millions of tokens or gigabytes of RSS, this guard spots the
//! *structure* of a loop directly: a short window of items that repeats many
//! times back-to-back.
//!
//! Two instances run at different kernel levels (the project's layered-guard
//! philosophy): one over the gullet's expansion stream (token fingerprints),
//! one over the stomach's accumulated digest list (box fingerprints). Either
//! can terminate a runaway with a clean `Fatal` long before the RSS cap.
//!
//! Algorithm (per the design directive): record a stream of `u64`
//! fingerprints in a fixed ring buffer. Periodically check whether the most
//! recent items are periodic with some period `W` in `1..=MAX_WINDOW`,
//! repeated at least `REPEAT` times. The check is pure periodicity over the
//! last `W*REPEAT` items (`item[i] == item[i-W]`), so it is **phase/offset
//! independent** — the cycle need not align to any buffer boundary. The
//! smallest matching period is reported.
//!
//! Cost: detection is throttled to once per `CHECK_EVERY` pushes and is
//! `O(MAX_WINDOW^2 * REPEAT)` per check (~5.4k u64 compares for the defaults),
//! i.e. a few amortized compares per push. Callers further gate activation on
//! an already-high item count so normal conversions pay nothing.

/// Largest cycle period (in items) we look for.
pub const MAX_WINDOW: usize = 10;
/// How many consecutive repetitions of a window constitute "infinite".
pub const REPEAT: usize = 100;
/// Ring-buffer capacity: enough to hold `MAX_WINDOW` repeated `REPEAT` times.
const CAP: usize = MAX_WINDOW * REPEAT;
/// Run the (cheap but non-trivial) periodicity scan only this often.
const CHECK_EVERY: usize = 256;

/// A windowed cycle detector over a stream of `u64` fingerprints.
pub struct CycleGuard {
  buf:   Box<[u64; CAP]>,
  /// next write position (ring)
  head:  usize,
  /// number of valid entries (saturates at CAP)
  len:   usize,
  /// throttle counter
  since: usize,
}

impl Default for CycleGuard {
  fn default() -> Self { Self::new() }
}

impl std::fmt::Debug for CycleGuard {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // The ring buffer is large and uninteresting; show only the state.
    f.debug_struct("CycleGuard")
      .field("len", &self.len)
      .field("head", &self.head)
      .finish()
  }
}

impl CycleGuard {
  pub fn new() -> Self {
    CycleGuard { buf: Box::new([0u64; CAP]), head: 0, len: 0, since: 0 }
  }

  /// Drop all recorded history (call at the start of each conversion).
  pub fn reset(&mut self) {
    self.head = 0;
    self.len = 0;
    self.since = 0;
  }

  /// Record one fingerprint. Returns `Some(period)` if the recent stream is a
  /// window of `period` items repeated at least [`REPEAT`] times.
  #[inline]
  pub fn push(&mut self, fp: u64) -> Option<usize> {
    self.buf[self.head] = fp;
    self.head = if self.head + 1 == CAP { 0 } else { self.head + 1 };
    if self.len < CAP {
      self.len += 1;
    }
    self.since += 1;
    if self.since >= CHECK_EVERY {
      self.since = 0;
      return self.detect();
    }
    None
  }

  /// The `k`-th most recently pushed item (`k = 0` is newest).
  #[inline]
  fn at_from_end(&self, k: usize) -> u64 {
    // head points one past the newest, modulo CAP.
    let idx = (self.head + CAP - 1 - k) % CAP;
    self.buf[idx]
  }

  fn detect(&self) -> Option<usize> {
    // Smallest period first, so a 2-cycle reports period 2, not 4/6/…
    for w in 1..=MAX_WINDOW {
      let need = w * REPEAT;
      if self.len < need {
        // Larger windows need even more history; the loop is monotone in `w`.
        break;
      }
      // The last `need` items are periodic with period `w` iff
      // item[i] == item[i-w] for every i in the most-recent `need - w`.
      let mut periodic = true;
      for k in 0..(need - w) {
        if self.at_from_end(k) != self.at_from_end(k + w) {
          periodic = false;
          break;
        }
      }
      if periodic {
        // UNIFORM-run suppression (PR #249 review P1-3): a window whose
        // items are all IDENTICAL describes a long run of one repeated
        // fingerprint — textually legitimate input (a verbatim `====…`
        // separator row, dot leaders, repeated rule glyphs), not a loop.
        // Real macro loops re-read ≥2 distinct tokens per cycle
        // (`\def\x{a\x}` → a,\x,a,\x…), and the pure single-token
        // self-expansion `\def\x{\x}` is caught by the Expandable
        // self-recursion error before any guard. Note a uniform stream
        // matches EVERY w, so rejecting uniform windows here rejects the
        // whole stream (each larger window is uniform too) — by design.
        let first = self.at_from_end(0);
        let has_distinct = (1..w).any(|k| self.at_from_end(k) != first);
        if has_distinct {
          return Some(w);
        }
      }
    }
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn run(stream: &[u64]) -> Option<usize> {
    let mut g = CycleGuard::new();
    let mut hit = None;
    for &x in stream {
      if let Some(p) = g.push(x) {
        hit = Some(p);
        break;
      }
    }
    hit
  }

  #[test]
  fn uniform_run_does_not_fire() {
    // A long run of IDENTICAL fingerprints is textually legitimate input
    // (a verbatim `====…` separator row, a dot-leader, a repeated rule
    // glyph) — NOT a loop. Real macro loops re-read at least two distinct
    // tokens (`\def\x{a\x}` → a, \x, a, \x …), and a pure single-token
    // self-expansion is caught by the Expandable self-recursion error
    // before any guard. PR #249 review P1-3.
    let s: Vec<u64> = std::iter::repeat(7).take(2000).collect();
    assert_eq!(run(&s), None);
  }

  #[test]
  fn alternating_pair_still_fires() {
    // `\def\x{a\x}` shape: two distinct fingerprints alternating — the
    // canonical real loop must still be detected (as period 2).
    let s: Vec<u64> = (0..900).map(|i| (i % 2) as u64).collect();
    assert_eq!(run(&s), Some(2));
  }

  #[test]
  fn detects_period_three() {
    // Stream must outlast the first throttled scan that sees >= 3*REPEAT items.
    let s: Vec<u64> = (0..900).map(|i| (i % 3) as u64).collect();
    assert_eq!(run(&s), Some(3));
  }

  #[test]
  fn detects_period_ten() {
    let s: Vec<u64> = (0..2000).map(|i| (i % 10) as u64).collect();
    assert_eq!(run(&s), Some(10));
  }

  #[test]
  fn ignores_non_cycle() {
    // Strictly increasing — never periodic.
    let s: Vec<u64> = (0..5000).collect();
    assert_eq!(run(&s), None);
  }

  #[test]
  fn ignores_short_repeat() {
    // A window repeated only 50 times (< REPEAT) must NOT fire.
    let s: Vec<u64> = (0..50 * 4).map(|i| (i % 4) as u64).collect();
    assert_eq!(run(&s), None);
  }

  #[test]
  fn ignores_period_over_max_window() {
    // Period 11 (> MAX_WINDOW) repeated many times must NOT fire.
    let s: Vec<u64> = (0..11 * 300).map(|i| (i % 11) as u64).collect();
    assert_eq!(run(&s), None);
  }

  #[test]
  fn reset_clears_history() {
    let mut g = CycleGuard::new();
    for _ in 0..150 {
      g.push(1);
    }
    g.reset();
    // After reset, a fresh non-cyclic stream must not immediately fire.
    let mut hit = None;
    for i in 0..50 {
      if let Some(p) = g.push(i) {
        hit = Some(p);
      }
    }
    assert_eq!(hit, None);
  }
}
