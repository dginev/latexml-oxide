//! Compile-time embedded TeXLive kernel dumps, indexed by TeXLive year.
//!
//! `latexml_engine/build.rs` scans `resources/dumps/` for versioned dumps
//! (`plain.YYYY.dump.txt`, `latex.YYYY.dump.txt`, `texlive.YYYY.version`),
//! **gzips each `*.dump.txt`** into `$OUT_DIR` (DEP-12, 2026-05-18), and
//! emits `embedded_dumps_manifest.rs` listing every bundled year as
//! `&'static [u8]` gzip blobs. This module `include!`s that manifest,
//! exposing year-aware accessors that decompress on demand.
//!
//! Why compress: ~4.7× gzip ratio on the record-shaped dump text
//! (3.8 MB → 805 KB per year). Both years combined drop from
//! ~7.6 MB to ~870 KB in `.rodata` — a ~6.7 MB binary-size win for a
//! one-time ~6.5 ms decompression cost at engine bootstrap (latex)
//! plus ~0.1 ms (plain), measured at 560 MB/s on this machine.
//!
//! Two-layer cache:
//!
//! 1. **Per-thread** `thread_local!(RefCell<HashMap<(year, kind), &'static str>>)`
//!    populated via `Box::leak`. Avoids re-decompressing or even
//!    re-reading the disk file when the same `embedded_*_dump` is
//!    called twice (e.g. probe-then-load: `*_embedded_available()`
//!    followed by the actual loader). Matches the project's
//!    thread-local-state idiom — no `Mutex` on the hot path.
//! 2. **Cross-process** disk cache at `$TMPDIR/latexml-oxide-dumps-<hash>/`.
//!    First process per machine-boot decompresses and atomically writes
//!    the plain-text dump; every subsequent process reads the file
//!    directly (~1 ms on tmpfs) instead of gunzipping again.
//!
//! Skipped entirely under `LATEXML_NO_EMBEDDED_DUMP=1` (no embedded
//! fallback fires) or when an ambient-year disk dump is found earlier
//! in the resolution chain.
//!
//! Selection at runtime:
//!
//! 1. If [`crate::dump_paths::detect_ambient_texlive_year`] returns a
//!    year that is bundled, use that exact year.
//! 2. Otherwise (no ambient TeXLive, or ambient year not bundled), fall
//!    back to the most-recent bundled year.
//! 3. If nothing is bundled at all, return `None`.
//!
//! Opt out of the embedded fallback altogether with
//! `LATEXML_NO_EMBEDDED_DUMP=1` — useful when iterating locally and you
//! want the binary to surface "no dump available" instead of silently
//! using a stale embedded snapshot.

use std::cell::RefCell;
use std::io::{Read, Write};
use std::path::PathBuf;

use once_cell::sync::Lazy;
use rustc_hash::FxHashMap as HashMap;

include!(concat!(env!("OUT_DIR"), "/embedded_dumps_manifest.rs"));

static NO_EMBEDDED: Lazy<bool> =
  Lazy::new(|| std::env::var_os("LATEXML_NO_EMBEDDED_DUMP").is_some());

thread_local! {
  /// In-thread dedupe so the same `(year, kind)` is decompressed (or
  /// disk-read) at most once per thread. Keyed by `(year, kind)` where
  /// `kind` is the static string `"plain"` or `"latex"`. Matches the
  /// project's thread-local-state convention (see CLAUDE.md "State is
  /// a thread-local, global, mutable singleton").
  static IN_THREAD_CACHE: RefCell<HashMap<(u32, &'static str), &'static str>> =
    RefCell::new(HashMap::default());
}

/// Per-machine cache dir for decompressed embedded dumps, keyed by the
/// build-time content hash. Under the system temp dir
/// (`std::env::temp_dir()` — cross-platform: `$TMPDIR`/`/tmp` on Unix,
/// `$TMPDIR` on macOS, `%TEMP%`/`%TMP%` on Windows). Properties:
/// * Always writable (no `$HOME` / CI / read-only-mount issues),
/// * Usually tmpfs on Linux — RAM-speed reads, faster than gunzip,
/// * Cleared at the OS's discretion (reboot on Linux/macOS, possibly
///   never on Windows) — natural cache invalidation when stale, with
///   the per-build content hash giving stable cross-process reuse for
///   the same binary version.
static CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| {
  std::env::temp_dir().join(format!("latexml-oxide-dumps-{}", EMBEDDED_DUMPS_CONTENT_HASH))
});

/// Pick the embedded entry that best matches `prefer` (typically the
/// ambient TL year). Returns `(entry, exact_match)`.
pub(crate) fn select_embedded(prefer: Option<u32>) -> Option<(&'static EmbeddedDumpYear, bool)> {
  if *NO_EMBEDDED {
    return None;
  }
  let entries = non_empty_entries();
  if entries.is_empty() {
    return None;
  }
  if let Some(year) = prefer {
    if let Some(e) = entries.iter().find(|e| e.year == year) {
      return Some((e, true));
    }
  }
  // EMBEDDED_DUMPS is sorted descending by build.rs, so first non-empty is
  // the most-recent year.
  entries.first().copied().map(|e| (e, false))
}

fn non_empty_entries() -> Vec<&'static EmbeddedDumpYear> {
  EMBEDDED_DUMPS
    .iter()
    .filter(|e| !e.plain_gz.is_empty() && !e.latex_gz.is_empty())
    .collect()
}

/// Resolve the cached-decompressed dump for `(year, kind)`. Tiered lookup:
///
/// 0. **In-process cache** — `Mutex<HashMap>` keyed by `(year, kind)`.
///    Avoids re-allocating on repeated calls within one process (e.g.
///    `*_embedded_available()` probe followed by the actual loader).
/// 1. **Disk cache** at `$TMPDIR/latexml-oxide-dumps-<hash>/<kind>.<year>.dump.txt`
///    (set up on a prior invocation in this boot cycle). Plain
///    `read_to_string` — typically <1 ms on tmpfs.
/// 2. **Decompress + persist**: gunzip the bundled blob (~6.5 ms for
///    latex, ~0.1 ms for plain), then atomically write the result to
///    the cache dir so the next process can take path #1. Best-effort
///    write — disk-full / EROFS does not fail the conversion.
///
/// Returns `None` for empty embedded blobs (build.rs stub when no
/// dumps were available at compile time) or for unrecoverable gunzip
/// failures.
fn decompressed_dump(year: u32, kind: &'static str, gz: &[u8]) -> Option<&'static str> {
  if gz.is_empty() {
    return None;
  }

  // Tier 0: per-thread cache.
  if let Some(cached) = IN_THREAD_CACHE.with(|c| c.borrow().get(&(year, kind)).copied()) {
    return Some(cached);
  }

  let leaked: &'static str = {
    let cache_path = CACHE_DIR.join(format!("{kind}.{year}.dump.txt"));

    // Tier 1: try disk cache.
    if let Ok(disk_text) = std::fs::read_to_string(&cache_path) {
      log::debug!(
        "[embedded_dumps] {kind} TL{year} loaded from disk cache {}",
        cache_path.display()
      );
      Box::leak(disk_text.into_boxed_str())
    } else {
      // Tier 2: gunzip the bundled blob.
      let mut decoder = flate2::read::GzDecoder::new(gz);
      let mut buf = String::with_capacity(gz.len() * 5); // ~4.7× ratio + slack
      if decoder.read_to_string(&mut buf).is_err() {
        log::warn!(
          "[embedded_dumps] gunzip failed for {kind} TL{year} ({} bytes); \
           falling back to no embedded dump",
          gz.len()
        );
        return None;
      }

      // Tier 3: best-effort atomic persist for the next process.
      // Atomic = write-to-sibling-temp + rename; on POSIX rename is atomic
      // within a filesystem, so concurrent first-time writers can't tear.
      if let Err(e) = write_cache_atomic(&cache_path, &buf) {
        log::debug!(
          "[embedded_dumps] disk cache write skipped ({}): {} — falling back to in-process only",
          cache_path.display(),
          e
        );
      }

      Box::leak(buf.into_boxed_str())
    }
  };

  // Publish into the per-thread cache so subsequent calls on this
  // thread (probe-then-load) skip even the disk read.
  IN_THREAD_CACHE.with(|c| c.borrow_mut().insert((year, kind), leaked));

  Some(leaked)
}

/// Best-effort atomic write of the decompressed dump to the cache
/// dir. Returns errors silently — the caller continues with the
/// in-memory copy regardless. Race-safe across platforms: two
/// parallel first-time writers each write to a unique
/// pid-suffixed tempfile and then `std::fs::rename` into place.
///
/// `std::fs::rename` is atomic for files within one filesystem on
/// Unix (POSIX rename(2)) and on Windows (MoveFileExW with
/// REPLACE_EXISTING). The second rename simply overwrites the first
/// — file content is identical so either outcome is correct.
fn write_cache_atomic(target: &std::path::Path, content: &str) -> std::io::Result<()> {
  std::fs::create_dir_all(&*CACHE_DIR)?;
  let temp_path = target.with_file_name(format!(
    "{}.tmp.{}",
    target
      .file_name()
      .and_then(|s| s.to_str())
      .unwrap_or("dump"),
    std::process::id()
  ));
  {
    let mut fh = std::fs::File::create(&temp_path)?;
    fh.write_all(content.as_bytes())?;
    fh.sync_data().ok(); // best-effort durability; not critical
  }
  std::fs::rename(&temp_path, target)
}

/// Bundled `plain.YYYY.dump.txt` content for the year that best matches
/// `prefer`. `None` when nothing is bundled or `LATEXML_NO_EMBEDDED_DUMP`
/// is set. First call per machine-boot decompresses + caches to
/// `/tmp`; subsequent calls in this AND future processes load the
/// decompressed file directly from the disk cache.
pub fn embedded_plain_dump(prefer: Option<u32>) -> Option<&'static str> {
  let (entry, _) = select_embedded(prefer)?;
  decompressed_dump(entry.year, "plain", entry.plain_gz)
}

/// Bundled `latex.YYYY.dump.txt` content for the year that best matches
/// `prefer`. See [`embedded_plain_dump`] for cache semantics. First
/// call per machine-boot pays ~6.5 ms gunzip + ~5 ms write; every
/// subsequent process across the boot reads the cached file in ~1 ms.
pub fn embedded_latex_dump(prefer: Option<u32>) -> Option<&'static str> {
  let (entry, _) = select_embedded(prefer)?;
  decompressed_dump(entry.year, "latex", entry.latex_gz)
}

/// First line of the bundled `texlive.YYYY.version` stamp for the chosen
/// year (used by the staleness check). `None` if no embedded dump applies.
pub fn embedded_texlive_version_first_line(prefer: Option<u32>) -> Option<&'static str> {
  let (entry, _) = select_embedded(prefer)?;
  entry.stamp.lines().next()
}

/// Year-tag for the chosen embedded dump (used in log messages).
/// Returns `None` if no embedded dump applies.
pub fn embedded_year(prefer: Option<u32>) -> Option<u32> {
  let (entry, _) = select_embedded(prefer)?;
  Some(entry.year)
}

/// Whether at least one embedded plain+latex pair is bundled and the
/// opt-out env var isn't set.
pub fn plain_embedded_available() -> bool {
  embedded_plain_dump(None).is_some()
}

pub fn latex_embedded_available() -> bool {
  embedded_latex_dump(None).is_some()
}
