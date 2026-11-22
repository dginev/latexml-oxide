//! Handing freed heap memory back to the OS on long runs.
//!
//! libxml2/libxslt allocate on glibc's heap, and glibc keeps freed mid-heap pages mapped: a book-scale core
//! pass or split render cycles a DOM per fragment or page through it, so RSS creeps into the memory fuse
//! unless the free pages are trimmed (the 131 MB witness: core RSS 22→36 GB, post +152 KB/page). A trim
//! walks the whole heap and madvises EVERY free chunk again, so a trim per fixed page count costs
//! pages × heap: 203 K madvise calls at 8,094 pages on Nasser's document (#777 FAQ, 130 of 194 min in sys
//! under WSL 1). [`sample`] trims only when the resident set grew by [`step_kb`] since the last trim, which
//! bounds the trims by the run's memory growth instead of its length.

use std::cell::Cell;

/// The page cadence of [`sample`] in the render loops: one `/proc/self/status` read per this many pages.
pub const SAMPLE_PAGES: usize = 64;

/// Release the free pages of both heaps: glibc's (`malloc_trim`) and mimalloc's (`mi_collect`).
pub fn release_free_heap() {
  #[cfg(target_os = "linux")]
  // SAFETY: `malloc_trim(0)` only advises glibc to release unused heap pages; no Rust refs are affected.
  unsafe {
    libc::malloc_trim(0);
  }
  #[cfg(not(feature = "dhat-heap"))]
  // SAFETY: `mi_collect(true)` only purges unused mimalloc pages; live allocations remain valid.
  unsafe {
    libmimalloc_sys::mi_collect(true);
  }
}

thread_local! {
  /// RSS (KiB) right after the last trim, lowered when the resident set falls below it.
  static TRIMMED_AT_KB: Cell<u64> = const { Cell::new(0) };
}

/// The resident growth that warrants a trim: 1/64 of the memory cap (128 MB under `--max-memory=8192`),
/// within 32-256 MB. Per sample this replaces up to step/page-cost of retained free pages (~800 pages at
/// the 152 KB/page growth) with one `/proc/self/status` read; the fuse sits at 75 % of the cap, so the
/// step is under 2 % of the fuse's headroom.
pub fn step_kb() -> u64 {
  const MIB: u64 = 1024;
  let cap_kb = latexml_core::stomach::resolve_rss_cap().map_or(u64::MAX, |cap| cap / 1024);
  (cap_kb / 64).clamp(32 * MIB, 256 * MIB)
}

/// Sample the resident set and trim when it grew by [`step_kb`] since the last trim. Returns whether it
/// trimmed. Callers sample at a page or fragment cadence; the first sample of a thread trims.
pub fn sample() -> bool {
  let Some(rss_kb) = latexml_core::watchdog::process_rss_kb() else {
    // No RSS reading (an unsupported OS): nothing to gate on, and nothing to trim off Linux.
    return false;
  };
  let trimmed_at = TRIMMED_AT_KB.get();
  if rss_kb < trimmed_at {
    TRIMMED_AT_KB.set(rss_kb);
    return false;
  }
  if rss_kb - trimmed_at < step_kb() {
    return false;
  }
  release_free_heap();
  TRIMMED_AT_KB.set(latexml_core::watchdog::process_rss_kb().unwrap_or(rss_kb));
  true
}
