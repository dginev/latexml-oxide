//! Eager → `--streaming` restart policy shared by the CLI and `cortex_worker`
//! (batches 56dw/56dz): when to stream up front (`resolve_streaming`, the
//! source-size projection), where an eligible eager run stops to rerun
//! (`restart_watermark`, nine tenths of the memory fuse), whether a finished
//! eager attempt asks for the rerun (`restart_budget`), and the memory release
//! the rerun needs first (`release_before_rerun`).

use std::{fs::File, io::Read, path::Path};

use latexml_core::stomach::{self, RestartSignal};

/// The RSS at which an eligible eager run stops to rerun under `--streaming`:
/// nine tenths of the cooperative fuse in force for this thread. The stop
/// exists for clean reporting (an Info instead of a Fatal, no salvage of a
/// partial document) more than for time: sweep #104 put it at two thirds and
/// five documents whose eager peak sat between the watermark and the fuse paid
/// a needless rerun (+330 s of sweep wall, no verdict changed), while the
/// witnesses that fuse anyway saved little (pgf-PeriodicTable's manual is
/// bounded by its ~380 s streaming rerun, not by where the eager attempt
/// stops). `None` when memory limiting is disabled (`--max-memory=0`): the
/// single knob governs every ceiling.
pub fn restart_watermark() -> Option<u64> { stomach::resolve_rss_cap().map(|cap| cap * 9 / 10) }

/// After an eager attempt on this thread: the fragment budget to rerun under
/// when digestion stopped for memory — at the watermark or at the fuse. Takes
/// the signal, so a later paper on the same thread starts clean. `None` = keep
/// the eager result.
pub fn restart_budget(max_memory_mib: u64, source: &str) -> Option<usize> {
  stomach::take_streaming_restart_signal()?;
  resolve_streaming(Some(true), max_memory_mib, source)
}

/// Give the fused attempt's memory back before the rerun's first fuse check:
/// its boxes still sit in the stomach's lists and its freed pages in mimalloc's
/// heap, so the rerun read the OLD peak and fused again (807 MB against an
/// 805 MB cap half a second in). Returns the resident size after the release
/// and the fuse, both in MB; the caller reruns only when the first is below
/// the second.
pub fn release_before_rerun() -> (u64, u64) {
  drop(stomach::salvage_pending_box_lists(false));
  unsafe {
    libmimalloc_sys::mi_collect(true);
  }
  let rss_mb = latexml_core::watchdog::process_rss_kb().unwrap_or(0) / 1024;
  let cap_mb = stomach::resolve_rss_cap().unwrap_or(0) / (1024 * 1024);
  (rss_mb, cap_mb)
}

/// Test seam: pretend digestion stopped for memory on this thread.
pub fn note_restart_signal(signal: RestartSignal) {
  stomach::note_streaming_restart_signal(signal);
}

pub fn resolve_streaming(
  requested: Option<bool>,
  max_memory_mib: u64,
  source: &str,
) -> Option<usize> {
  const PEAK_BYTES_PER_SOURCE_BYTE: u64 = 1900; // ~1.84 GB/MB, measured
  const BYTES_PER_BOX: u64 = 2416; // stomach::BYTES_PER_LIGHT_BOX's basis
  // `--max-memory=0` disables the death ceiling, but the machine is still
  // finite and such a document may still need to spill. Judge — and size
  // fragments — against the ceiling we WOULD have derived. Without this the
  // arithmetic degenerated: `projected > 0` always fired, and
  // `0 / 8 / 2416 -> max(1)` gave a ONE-BOX budget, i.e. a yield after every
  // box.
  let yardstick_mib = if max_memory_mib == 0 {
    latexml_core::watchdog::default_ceiling_mib()
  } else {
    max_memory_mib
  };
  // Compare against the cooperative FUSE, not the ceiling: death happens at
  // 75% of the ceiling, so comparing against the ceiling left a band
  // (0.75-1.0x) that was judged "fits", took the eager path, and was then
  // killed by the fuse — 8.1-10.8 MB of source on a 16 GB laptop.
  let fuse_mib = stomach::soft_cap_from_ceiling(yardstick_mib) / (1024 * 1024);
  let auto = || {
    let projected_mib =
      projected_source_bytes(source).saturating_mul(PEAK_BYTES_PER_SOURCE_BYTE) / (1024 * 1024);
    (projected_mib > fuse_mib).then_some(())
  };
  match requested {
    // Explicit opt-out: never stream, not even when projected to die.
    Some(false) => return None,
    Some(true) => {},
    None if auto().is_none() => return None,
    None => {},
  }
  // The eighth is MEASURED, not guessed, and shrinking it buys nothing: on the
  // 19.8 MB witness at an 8192 MiB ceiling, divisors 8/16/32 peak at
  // 4747/4719/4714 MB with an invariant ramp (3788/3784/3787 MB at fragment 2)
  // and byte-identical output. Peak there is a STARTUP TRANSIENT that this knob
  // does not size — see task #158.
  let budget_boxes = (yardstick_mib.saturating_mul(1024 * 1024) / 8 / BYTES_PER_BOX) as usize;
  Some(budget_boxes.max(1))
}

/// The byte size the memory projection must reason from: the DOCUMENT, not the
/// main file.
///
/// A 2 KB `index.tex` that `\input`s a thousand chapters is a half-gigabyte
/// document, but `metadata(main).len()` projects it at 2 KB — "fits easily" —
/// and the eager path then dies on it. When the main file actually names an
/// inclusion command, sum the source tree (`.tex`/`.ltx`/`.bbl`) instead.
///
/// Gated on the command being present so a SELF-CONTAINED paper sitting in a
/// directory of unused alternates (a common arXiv bundle shape) still projects
/// as itself and keeps the eager path.
///
/// Known limitation: an inclusion assembled by macro expansion
/// (`\myinput{ch1}`) names no literal command and is not detected; such a
/// document needs an explicit `--streaming`.
pub fn projected_source_bytes(source: &str) -> u64 {
  /// Enough of the main file to see its inclusion commands without reading a
  /// 131 MB self-contained source in full (whose own size already dominates).
  const SCAN_BYTES: u64 = 4 * 1024 * 1024;
  /// Backstop against a pathological tree (a home directory as source dir).
  const WALK_ENTRIES: usize = 50_000;
  const INCLUSION_COMMANDS: [&str; 6] = [
    "\\input",
    "\\include",
    "\\import",
    "\\subimport",
    "\\subfile",
    "\\includeonly",
  ];

  let own = std::fs::metadata(source).map(|m| m.len()).unwrap_or(0);
  let Some(dir) = Path::new(source).parent() else {
    return own;
  };
  let mut head = Vec::new();
  if File::open(source)
    .map(|f| f.take(SCAN_BYTES).read_to_end(&mut head))
    .is_err()
  {
    return own;
  }
  let head = String::from_utf8_lossy(&head);
  if !INCLUSION_COMMANDS.iter().any(|cmd| head.contains(cmd)) {
    return own;
  }
  let mut total = 0u64;
  let mut seen = 0usize;
  let mut stack = vec![dir.to_path_buf()];
  while let Some(d) = stack.pop() {
    let Ok(entries) = std::fs::read_dir(&d) else {
      continue;
    };
    for entry in entries.flatten() {
      seen += 1;
      if seen > WALK_ENTRIES {
        return total.max(own);
      }
      match entry.file_type() {
        Ok(t) if t.is_dir() => stack.push(entry.path()),
        Ok(t) if t.is_file() => {
          let path = entry.path();
          if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("tex" | "ltx" | "bbl")
          ) {
            total = total.saturating_add(entry.metadata().map(|m| m.len()).unwrap_or(0));
          }
        },
        _ => {},
      }
    }
  }
  total.max(own)
}
